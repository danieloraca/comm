use std::{net::IpAddr, process::Stdio};

use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, net::lookup_host, process::Command, time};

const FETCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(4);
const MAX_HTML_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LinkPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub image_url: Option<String>,
}

pub fn first_url(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .map(trim_url_token)
        .find(|token| parse_http_url(token).is_some())
        .map(str::to_owned)
}

pub async fn fetch(url: &str) -> Option<LinkPreview> {
    let parsed = parse_http_url(url)?;

    if let Some(preview) = fetch_youtube_oembed(&parsed).await {
        return Some(preview);
    }

    let fetch_url = metadata_fetch_url(&parsed);
    let fetch_url = parse_http_url(&fetch_url)?;
    ensure_public_host(&fetch_url).await.ok()?;

    let html = fetch_metadata_html(&fetch_url).await?;
    let title = meta_content(&html, "property", "og:title")
        .or_else(|| meta_content(&html, "name", "twitter:title"))
        .or_else(|| title_text(&html));
    let description = meta_content(&html, "property", "og:description")
        .or_else(|| meta_content(&html, "name", "description"))
        .or_else(|| meta_content(&html, "name", "twitter:description"));
    let site_name =
        meta_content(&html, "property", "og:site_name").or_else(|| Some(parsed.host.clone()));
    let image_url = meta_content(&html, "property", "og:image")
        .or_else(|| meta_content(&html, "property", "og:image:url"))
        .or_else(|| meta_content(&html, "name", "twitter:image"))
        .and_then(|url| normalize_preview_url(&url, &fetch_url));

    if title.is_none() && description.is_none() {
        return None;
    }

    Some(LinkPreview {
        url: parsed.url,
        title,
        description,
        site_name,
        image_url,
    })
}

async fn fetch_metadata_html(fetch_url: &ParsedUrl) -> Option<String> {
    if let Some(html) = curl_fetch(&fetch_url.url, MAX_HTML_BYTES).await {
        return Some(html);
    }

    let fallback = www_variant(fetch_url)?;
    ensure_public_host(&fallback).await.ok()?;
    curl_fetch(&fallback.url, MAX_HTML_BYTES).await
}

async fn fetch_youtube_oembed(parsed: &ParsedUrl) -> Option<LinkPreview> {
    if !is_youtube_host(&parsed.host) {
        return None;
    }

    let endpoint = format!(
        "https://www.youtube.com/oembed?url={}&format=json",
        percent_encode(&parsed.url)
    );
    let endpoint = parse_http_url(&endpoint)?;
    ensure_public_host(&endpoint).await.ok()?;

    let json = curl_fetch(&endpoint.url, 16 * 1024).await?;
    let value: serde_json::Value = serde_json::from_str(&json).ok()?;
    let title = value
        .get("title")
        .and_then(|value| value.as_str())
        .and_then(clean_text);
    let site_name = value
        .get("provider_name")
        .and_then(|value| value.as_str())
        .and_then(clean_text)
        .or_else(|| Some("YouTube".to_owned()));
    let image_url = value
        .get("thumbnail_url")
        .and_then(|value| value.as_str())
        .filter(|url| parse_http_url(url).is_some())
        .map(str::to_owned);

    Some(LinkPreview {
        url: parsed.url.clone(),
        title,
        description: None,
        site_name,
        image_url,
    })
}

async fn curl_fetch(url: &str, max_bytes: usize) -> Option<String> {
    let mut child = Command::new("curl")
        .arg("--silent")
        .arg("--show-error")
        .arg("--fail")
        .arg("--max-time")
        .arg("4")
        .arg("--proto")
        .arg("=http,https")
        .arg("--range")
        .arg(format!("0-{}", max_bytes - 1))
        .arg("--user-agent")
        .arg("CommLinkPreview/1.0")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let bytes = time::timeout(FETCH_TIMEOUT, async {
        let mut bytes = Vec::with_capacity(max_bytes.min(8192));
        let mut buffer = [0; 8192];

        while bytes.len() < max_bytes {
            let count = stdout.read(&mut buffer).await.ok()?;

            if count == 0 {
                break;
            }

            let remaining = max_bytes - bytes.len();
            bytes.extend_from_slice(&buffer[..count.min(remaining)]);
        }

        Some(bytes)
    })
    .await
    .ok()??;

    let _ = child.kill().await;
    let _ = child.wait().await;

    if bytes.is_empty() {
        return None;
    }

    Some(String::from_utf8_lossy(&bytes).into_owned())
}

#[derive(Debug)]
struct ParsedUrl {
    url: String,
    host: String,
    port: u16,
    path: String,
}

fn parse_http_url(value: &str) -> Option<ParsedUrl> {
    let (scheme, rest) = value.split_once("://")?;
    let port = match scheme {
        "http" => 80,
        "https" => 443,
        _ => return None,
    };

    let authority = rest.split(['/', '?', '#']).next()?.trim();
    let path = rest
        .strip_prefix(authority)
        .unwrap_or_default()
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .to_owned();
    if authority.is_empty() || authority.contains('@') {
        return None;
    }

    let (host, port) = if authority.starts_with('[') {
        let end = authority.find(']')?;
        let host = authority[1..end].to_owned();
        let port = authority[end + 1..]
            .strip_prefix(':')
            .and_then(|value| value.parse().ok())
            .unwrap_or(port);
        (host, port)
    } else if let Some((host, explicit_port)) = authority.rsplit_once(':') {
        if explicit_port
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            (host.to_owned(), explicit_port.parse().ok()?)
        } else {
            (authority.to_owned(), port)
        }
    } else {
        (authority.to_owned(), port)
    };

    if host.is_empty() || host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
        return None;
    }

    Some(ParsedUrl {
        url: value.to_owned(),
        host,
        port,
        path,
    })
}

fn metadata_fetch_url(parsed: &ParsedUrl) -> String {
    if parsed.host.eq_ignore_ascii_case("youtu.be") {
        let id = parsed
            .path
            .trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or("");

        if !id.is_empty() {
            return format!("https://www.youtube.com/watch?v={id}");
        }
    }

    parsed.url.clone()
}

fn www_variant(parsed: &ParsedUrl) -> Option<ParsedUrl> {
    if parsed.host.starts_with("www.") || parsed.host.parse::<IpAddr>().is_ok() {
        return None;
    }

    parse_http_url(&parsed.url.replacen("://", "://www.", 1))
}

fn normalize_preview_url(value: &str, base: &ParsedUrl) -> Option<String> {
    if parse_http_url(value).is_some() {
        return Some(value.to_owned());
    }

    if value.starts_with("//") {
        return Some(format!("https:{value}")).filter(|url| parse_http_url(url).is_some());
    }

    if value.starts_with('/') {
        return Some(format!("https://{}{}", base.host, value))
            .filter(|url| parse_http_url(url).is_some());
    }

    None
}

fn is_youtube_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("youtu.be")
        || host.eq_ignore_ascii_case("youtube.com")
        || host.to_ascii_lowercase().ends_with(".youtube.com")
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }

    encoded
}

async fn ensure_public_host(parsed: &ParsedUrl) -> Result<(), ()> {
    if let Ok(ip) = parsed.host.parse::<IpAddr>() {
        return public_ip(ip).then_some(()).ok_or(());
    }

    let addrs = lookup_host((parsed.host.as_str(), parsed.port))
        .await
        .map_err(|_| ())?
        .collect::<Vec<_>>();

    if addrs.is_empty() || addrs.iter().any(|addr| !public_ip(addr.ip())) {
        return Err(());
    }

    Ok(())
}

fn public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_documentation()
                || ip.is_unspecified())
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local())
        }
    }
}

fn meta_content(html: &str, attr: &str, attr_value: &str) -> Option<String> {
    html.match_indices("<meta")
        .filter_map(|(index, _)| html[index..].split_once('>').map(|(tag, _)| tag))
        .filter(|tag| attr_equals(tag, attr, attr_value))
        .find_map(|tag| attr_value_from_tag(tag, "content").and_then(|value| clean_text(&value)))
}

fn attr_equals(tag: &str, name: &str, expected: &str) -> bool {
    attr_value_from_tag(tag, name)
        .map(|value| value.eq_ignore_ascii_case(expected))
        .unwrap_or(false)
}

fn attr_value_from_tag(tag: &str, name: &str) -> Option<String> {
    let lower_tag = tag.to_ascii_lowercase();
    let pattern = format!("{name}=");
    let start = lower_tag.find(&pattern)? + pattern.len();
    let value = &tag[start..];
    let quote = value.chars().next()?;

    if quote == '"' || quote == '\'' {
        let end = value[1..].find(quote)? + 1;
        Some(value[1..end].to_owned())
    } else {
        Some(
            value
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('/')
                .to_owned(),
        )
    }
}

fn title_text(html: &str) -> Option<String> {
    let lower_html = html.to_ascii_lowercase();
    let start = lower_html.find("<title")?;
    let content_start = lower_html[start..].find('>')? + start + 1;
    let content_end = lower_html[content_start..].find("</title>")? + content_start;
    clean_text(&html[content_start..content_end])
}

fn clean_text(value: &str) -> Option<String> {
    let value = decode_basic_entities(value)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let value = value.trim();

    if value.is_empty() {
        None
    } else {
        Some(value.chars().take(220).collect())
    }
}

fn decode_basic_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn trim_url_token(value: &str) -> &str {
    value.trim_matches(|character: char| {
        matches!(
            character,
            '"' | '\''
                | '<'
                | '>'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | ','
                | '.'
                | '!'
                | '?'
                | ';'
                | ':'
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        first_url, is_youtube_host, meta_content, metadata_fetch_url, normalize_preview_url,
        parse_http_url, percent_encode, title_text, www_variant,
    };

    #[test]
    fn first_url_trims_common_message_punctuation() {
        assert_eq!(
            first_url("watch this (https://youtu.be/example).").as_deref(),
            Some("https://youtu.be/example")
        );
    }

    #[test]
    fn parse_http_url_rejects_non_http_urls() {
        assert!(parse_http_url("file:///tmp/test").is_none());
    }

    #[test]
    fn meta_content_reads_open_graph_title() {
        let html = r#"<meta property="og:title" content="A &amp; B">"#;
        assert_eq!(
            meta_content(html, "property", "og:title").as_deref(),
            Some("A & B")
        );
    }

    #[test]
    fn meta_content_reads_open_graph_image() {
        let html = r#"<meta property="og:image" content="https://example.com/thumb.jpg">"#;
        assert_eq!(
            meta_content(html, "property", "og:image").as_deref(),
            Some("https://example.com/thumb.jpg")
        );
    }

    #[test]
    fn title_text_reads_document_title() {
        let html = "<html><head><title> Example page </title></head></html>";
        assert_eq!(title_text(html).as_deref(), Some("Example page"));
    }

    #[test]
    fn metadata_fetch_url_rewrites_short_youtube_links() {
        let parsed = parse_http_url("https://youtu.be/abc123?si=test").unwrap();
        assert_eq!(
            metadata_fetch_url(&parsed),
            "https://www.youtube.com/watch?v=abc123"
        );
    }

    #[test]
    fn youtube_hosts_are_detected() {
        assert!(is_youtube_host("www.youtube.com"));
        assert!(is_youtube_host("m.youtube.com"));
        assert!(is_youtube_host("youtu.be"));
        assert!(!is_youtube_host("notyoutube.com"));
    }

    #[test]
    fn percent_encode_encodes_url_query_delimiters() {
        assert_eq!(
            percent_encode("https://www.youtube.com/watch?v=abc&x=1"),
            "https%3A%2F%2Fwww.youtube.com%2Fwatch%3Fv%3Dabc%26x%3D1"
        );
    }

    #[test]
    fn www_variant_adds_www_to_plain_host() {
        let parsed = parse_http_url("https://example.com/a").unwrap();
        assert_eq!(
            www_variant(&parsed).map(|parsed| parsed.url),
            Some("https://www.example.com/a".to_owned())
        );
    }

    #[test]
    fn normalize_preview_url_accepts_protocol_relative_images() {
        let parsed = parse_http_url("https://example.com/a").unwrap();
        assert_eq!(
            normalize_preview_url("//cdn.example.com/image.jpg", &parsed),
            Some("https://cdn.example.com/image.jpg".to_owned())
        );
    }
}
