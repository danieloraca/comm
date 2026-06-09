use axum::{extract::Query, response::Html};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginPageQuery {
    error: Option<String>,
    username: Option<String>,
}

pub async fn login_page(Query(query): Query<LoginPageQuery>) -> Html<String> {
    let username = query.username.unwrap_or_default();
    let password_step = !username.is_empty();
    let error = match query.error.as_deref() {
        Some("rate_limited") => {
            r#"<p class="error" role="alert">Too many failed attempts. Try again shortly.</p>"#
        }
        Some(_) => r#"<p class="error" role="alert">Try again in a moment.</p>"#,
        None => "",
    };

    let hidden_username = if password_step {
        format!(
            r#"<input type="hidden" name="username" value="{}">"#,
            escape_html_attribute(&username)
        )
    } else {
        String::new()
    };

    Html(
        LOGIN_PAGE
            .replace(
                "{{form_action}}",
                if password_step {
                    "/login"
                } else {
                    "/start-login"
                },
            )
            .replace(
                "{{input_name}}",
                if password_step { "password" } else { "q" },
            )
            .replace(
                "{{input_type}}",
                if password_step { "password" } else { "search" },
            )
            .replace(
                "{{input_autocomplete}}",
                if password_step {
                    "current-password"
                } else {
                    "off"
                },
            )
            .replace(
                "{{input_label}}",
                if password_step { "Password" } else { "Search" },
            )
            .replace(
                "{{input_placeholder}}",
                if password_step { "Search" } else { "Search" },
            )
            .replace("{{hidden_username}}", &hidden_username)
            .replace("{{error}}", error)
            .replace(
                "{{delay_on_error}}",
                if password_step && query.error.as_deref() == Some("1") {
                    "true"
                } else {
                    "false"
                },
            ),
    )
}

pub fn chat_page(username: &str) -> Html<String> {
    Html(CHAT_PAGE.replace("{{username}}", username))
}

fn escape_html_attribute(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const LOGIN_PAGE: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Comm Login</title>
  <style>
    :root {
      color-scheme: light dark;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      background: #f5f7f8;
      color: #192024;
    }

    * {
      box-sizing: border-box;
    }

    body {
      min-height: 100vh;
      margin: 0;
      display: grid;
      place-items: center;
      padding: 24px;
      background: #eef2f4;
    }

    main {
      width: min(100%, 560px);
      display: grid;
      gap: 26px;
      justify-items: center;
    }

    .wordmark {
      margin: 0;
      font-size: clamp(2.8rem, 12vw, 5.2rem);
      font-weight: 700;
      letter-spacing: 0;
      line-height: 1;
    }

    .wordmark span:nth-child(1),
    .wordmark span:nth-child(4) {
      color: #2f80ed;
    }

    .wordmark span:nth-child(2),
    .wordmark span:nth-child(6) {
      color: #d94841;
    }

    .wordmark span:nth-child(3) {
      color: #f2b705;
    }

    .wordmark span:nth-child(5) {
      color: #219653;
    }

    form {
      width: 100%;
      display: grid;
      gap: 14px;
      justify-items: center;
    }

    .search-row {
      width: 100%;
      display: grid;
      grid-template-columns: minmax(0, 1fr) auto;
      gap: 10px;
    }

    input {
      width: 100%;
      min-height: 48px;
      padding: 11px 16px;
      border: 1px solid #d5dde2;
      border-radius: 999px;
      font: inherit;
      background: #ffffff;
      color: #192024;
      box-shadow: 0 8px 22px rgba(25, 32, 36, 0.08);
    }

    input:focus {
      outline: 3px solid #9ac2ff;
      outline-offset: 1px;
      border-color: #336fb2;
    }

    button {
      min-height: 48px;
      padding: 0 18px;
      border: 0;
      border-radius: 999px;
      background: #1d5f8f;
      color: #ffffff;
      font: inherit;
      font-weight: 700;
      cursor: pointer;
    }

    button:focus {
      outline: 3px solid #9ac2ff;
      outline-offset: 2px;
    }

    .error {
      margin: 0;
      padding: 10px 12px;
      border: 1px solid #d97979;
      border-radius: 6px;
      background: #fff0f0;
      color: #8a1f1f;
      font-size: 0.9rem;
      font-weight: 650;
    }

    @media (prefers-color-scheme: dark) {
      :root {
        background: #11181c;
        color: #edf3f7;
      }

      body {
        background: #11181c;
      }

      form {
        background: transparent;
      }

      input {
        border-color: #48616f;
        background: #11181c;
        color: #edf3f7;
      }

      button {
        background: #4b9ad3;
        color: #071014;
      }

      .error {
        border-color: #8b4c4c;
        background: #311d1d;
        color: #ffd4d4;
      }
    }

    @media (max-width: 520px) {
      body {
        place-items: start center;
        padding-top: 35vh;
      }
    }
  </style>
</head>
<body>
  <main>
    <h1 class="wordmark" aria-label="Search"><span>S</span><span>e</span><span>a</span><span>r</span><span>c</span><span>h</span></h1>
    <form id="login-form" method="post" action="{{form_action}}" data-delay-on-error="{{delay_on_error}}">
      <div class="search-row">
        <input id="search-input" name="{{input_name}}" type="{{input_type}}" autocomplete="{{input_autocomplete}}" placeholder="{{input_placeholder}}" aria-label="{{input_label}}" required autofocus>
        <button id="search-button" type="submit">Search</button>
      </div>
      {{hidden_username}}
      {{error}}
    </form>
  </main>
  <script>
    const form = document.querySelector('#login-form');
    const input = document.querySelector('#search-input');
    const button = document.querySelector('#search-button');

    if (form.dataset.delayOnError === "true") {
      input.disabled = true;
      button.disabled = true;

      window.setTimeout(() => {
        input.disabled = false;
        button.disabled = false;
        input.focus();
        input.select();
      }, 3000);
    } else {
      input.focus();
    }
  </script>
</body>
</html>
"#;

const CHAT_PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Comm Chat</title>
  <style>
    :root {
      color-scheme: light dark;
      font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      --page-bg: #eef2f4;
      --app-bg: #ffffff;
      --surface-bg: #f8fafb;
      --elevated-bg: #ffffff;
      --text: #192024;
      --muted: #53636d;
      --border: #d5dde2;
      --control-border: #bac5cc;
      --menu-border: #c7d1d8;
      --shadow: 0 12px 30px rgba(25, 32, 36, 0.08);
      --menu-shadow: 0 8px 18px rgba(25, 32, 36, 0.14);
      --accent: #1d5f8f;
      --accent-text: #ffffff;
      --accent-soft: #9ac2ff;
      --logo-color: #1d5f8f;
      --presence-off: #7f8d96;
      --presence-on: #24b45a;
      --message-bg: #e6eef4;
      --own-message-bg: #d6e9f7;
      --hover-bg: #edf3f7;
      --danger: #8a1f1f;
      --danger-bg: #fff0f0;
      --privacy-bg: #eef2f4;
      background: var(--page-bg);
      color: var(--text);
    }

    * {
      box-sizing: border-box;
    }

    html,
    body {
      height: 100%;
    }

    body {
      height: 100vh;
      height: 100dvh;
      margin: 0;
      padding: 18px;
      overflow: hidden;
      background: var(--page-bg);
    }

    main {
      width: min(100%, 760px);
      height: calc(100vh - 36px);
      height: calc(100dvh - 36px);
      min-height: 0;
      margin: 0 auto;
      position: relative;
      display: grid;
      grid-template-rows: auto minmax(0, 1fr) auto auto;
      gap: 14px;
      padding: 20px;
      border: 1px solid var(--border);
      border-radius: 8px;
      background: var(--app-bg);
      box-shadow: var(--shadow);
    }

    h1 {
      margin: 0;
      font-size: 1.35rem;
      line-height: 1.2;
    }

    .brand-mark {
      width: 76px;
      height: 48px;
      color: var(--logo-color);
    }

    .brand-bird {
      display: block;
      width: 100%;
      height: 100%;
      filter: drop-shadow(0 4px 10px rgba(0, 0, 0, 0.18));
    }

    .brand-bird-body {
      fill: none;
      stroke: currentColor;
      stroke-linecap: round;
      stroke-linejoin: round;
      stroke-width: 5;
    }

    .brand-bird-wing {
      fill: none;
      stroke: currentColor;
      stroke-linecap: round;
      stroke-linejoin: round;
      stroke-width: 4.5;
    }

    .brand-bird-eye {
      fill: currentColor;
    }

    .brand-bird-beak {
      fill: #f0b84b;
    }

    p {
      margin: 0;
      color: var(--muted);
    }

    header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
      position: relative;
    }

    .title {
      display: grid;
      gap: 2px;
    }

    .status {
      font-size: 0.86rem;
    }

    .status[hidden] {
      display: none;
    }

    .presence {
      display: inline-flex;
      align-items: center;
      gap: 6px;
      font-size: 0.82rem;
      color: var(--muted);
    }

    .presence-dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: var(--presence-off);
    }

    .presence-dot.online {
      background: var(--presence-on);
    }

    .typing {
      min-height: 18px;
      font-size: 0.84rem;
      color: var(--muted);
    }

    .typing:empty {
      display: none;
    }

    .logout-form {
      display: block;
    }

    .header-actions {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .settings-button {
      min-height: 36px;
      padding: 0 12px;
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--control-border);
    }

    .logout-button {
      min-height: 36px;
      padding: 0 12px;
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--control-border);
    }

    .settings-panel {
      position: absolute;
      z-index: 4;
      top: calc(100% + 8px);
      right: 0;
      width: min(280px, 100%);
      display: grid;
      gap: 12px;
      padding: 12px;
      border: 1px solid var(--menu-border);
      border-radius: 6px;
      background: var(--elevated-bg);
      box-shadow: var(--menu-shadow);
    }

    .settings-panel[hidden] {
      display: none;
    }

    .settings-title {
      margin: 0;
      color: var(--text);
      font-size: 0.9rem;
      font-weight: 700;
    }

    .setting-row {
      display: flex;
      align-items: start;
      gap: 10px;
      color: var(--text);
      font-size: 0.88rem;
      line-height: 1.35;
    }

    .setting-row input {
      flex: 0 0 auto;
      width: 18px;
      height: 18px;
      margin-top: 1px;
      accent-color: var(--accent);
    }

    .setting-select {
      flex: 1 1 auto;
      min-width: 0;
      width: 100%;
      min-height: 38px;
      padding: 6px 10px;
      border: 1px solid var(--control-border);
      border-radius: 6px;
      background: var(--elevated-bg);
      color: var(--text);
      font: inherit;
    }

    .font-size-control {
      display: grid;
      grid-template-columns: 1fr auto;
      align-items: center;
      gap: 10px;
      color: var(--text);
      font-size: 0.88rem;
      line-height: 1.35;
    }

    .font-size-buttons {
      display: flex;
      align-items: center;
      gap: 6px;
    }

    .font-size-button {
      width: 36px;
      min-height: 36px;
      padding: 0;
      border: 1px solid var(--control-border);
      background: transparent;
      color: var(--accent);
      font-size: 1rem;
    }

    .font-size-button:disabled {
      color: var(--muted);
      cursor: default;
      opacity: 0.6;
    }

    .font-size-value {
      min-width: 44px;
      color: var(--muted);
      text-align: center;
      font-size: 0.82rem;
      font-weight: 700;
    }

    .settings-section {
      display: grid;
      gap: 8px;
    }

    .settings-section + .settings-section {
      padding-top: 10px;
      border-top: 1px solid var(--border);
    }

    .settings-heading {
      margin: 0;
      color: var(--muted);
      font-size: 0.76rem;
      font-weight: 800;
      letter-spacing: 0;
      text-transform: uppercase;
    }

    .privacy-screen {
      position: absolute;
      z-index: 10;
      inset: 0;
      display: grid;
      place-items: center;
      padding: 24px;
      border-radius: inherit;
      background: var(--privacy-bg);
    }

    .privacy-screen[hidden] {
      display: none;
    }

    .photo-viewer {
      position: fixed;
      z-index: 20;
      inset: 0;
      display: grid;
      place-items: center;
      padding: 14px;
      background: #000000;
      cursor: zoom-out;
    }

    .photo-viewer[hidden] {
      display: none;
    }

    .photo-viewer img {
      max-width: 100%;
      max-height: 100%;
      object-fit: contain;
    }

    .privacy-content {
      width: min(100%, 320px);
      display: grid;
      gap: 12px;
      text-align: center;
    }

    .privacy-content[hidden] {
      display: none;
    }

    .privacy-content h2 {
      margin: 0;
      color: var(--text);
      font-size: 1.35rem;
      line-height: 1.2;
    }

    .privacy-content p {
      color: var(--muted);
      font-size: 0.92rem;
    }

    .privacy-form {
      display: grid;
      grid-template-columns: 1fr;
      gap: 10px;
    }

    .privacy-form input {
      width: 100%;
      min-height: 44px;
      padding: 10px 12px;
      border: 1px solid var(--control-border);
      border-radius: 6px;
      font: inherit;
      background: var(--elevated-bg);
      color: var(--text);
    }

    .privacy-error {
      min-height: 18px;
      color: var(--danger);
      font-size: 0.86rem;
      font-weight: 650;
    }

    .privacy-error:empty {
      display: none;
    }

    .messages {
      min-height: 0;
      overflow: auto;
      overscroll-behavior: contain;
      -webkit-overflow-scrolling: touch;
      display: flex;
      flex-direction: column;
      gap: 10px;
      padding: 12px;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--surface-bg);
    }

    .message {
      position: relative;
      width: fit-content;
      max-width: min(82%, 520px);
      display: grid;
      gap: 3px;
      justify-items: start;
      align-self: flex-start;
      overflow-wrap: anywhere;
      cursor: pointer;
    }

    .messages.activity-mode .message {
      display: none;
    }

    .activity-log-view {
      min-height: 100%;
      display: grid;
      grid-template-rows: auto minmax(0, 1fr);
      gap: 10px;
      color: var(--text);
    }

    .activity-log-view[hidden] {
      display: none;
    }

    .activity-log-hint {
      color: var(--muted);
      font-size: 0.78rem;
      font-weight: 700;
      text-transform: uppercase;
    }

    .activity-log-lines {
      margin: 0;
      overflow: auto;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
      color: var(--text);
      font: 0.84rem/1.45 ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace;
    }

    .message.own {
      justify-items: end;
      align-self: flex-end;
    }

    .message-meta {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 0 2px;
      color: var(--muted);
      font-size: 0.72rem;
    }

    .message-meta strong {
      font-size: 0.78rem;
    }

    .message-bubble {
      position: relative;
      max-width: 100%;
      padding: 8px 10px;
      border-radius: 7px;
      background: var(--message-bg);
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }

    .message-bubble.emoji-only {
      padding: 9px 12px;
      font-size: 2.35rem;
      line-height: 1;
    }

    .message-bubble a {
      color: var(--accent);
      text-decoration: underline;
      text-decoration-thickness: 1px;
      text-underline-offset: 2px;
    }

    .message-photo {
      display: block;
      width: min(100%, 320px);
      max-height: 360px;
      object-fit: contain;
      border-radius: 6px;
      cursor: zoom-in;
    }

    .message-photo + .message-text {
      margin-top: 6px;
    }

    .read-status {
      position: absolute;
      top: -4px;
      right: -4px;
      width: 9px;
      height: 9px;
      border: 2px solid var(--surface-bg);
      border-radius: 50%;
      background: var(--presence-off);
      box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.12);
    }

    .read-status.is-read {
      background: var(--presence-on);
    }

    .message.own .message-bubble {
      background: var(--own-message-bg);
    }

    .message time {
      color: inherit;
    }

    .message-menu {
      position: absolute;
      z-index: 2;
      top: calc(100% + 4px);
      left: 0;
      min-width: 178px;
      padding: 4px;
      border: 1px solid var(--menu-border);
      border-radius: 6px;
      background: var(--elevated-bg);
      box-shadow: var(--menu-shadow);
    }

    .message.own .message-menu {
      right: 0;
      left: auto;
    }

    .message-menu[hidden] {
      display: none;
    }

    .message-action {
      width: 100%;
      min-height: 34px;
      padding: 0 10px;
      border: 0;
      border-radius: 4px;
      background: transparent;
      color: var(--text);
      text-align: left;
      font-size: 0.88rem;
    }

    .message-action:hover,
    .message-action:focus {
      background: var(--hover-bg);
    }

    .message-action.danger {
      color: var(--danger);
    }

    .message-action.danger:hover,
    .message-action.danger:focus {
      background: var(--danger-bg);
    }

    form {
      display: grid;
      grid-template-columns: auto 1fr auto;
      gap: 10px;
    }

    .emoji-bar {
      display: flex;
      grid-column: 1 / -1;
      gap: 6px;
      overflow-x: auto;
      padding-bottom: 2px;
    }

    .attachment-draft {
      grid-column: 1 / -1;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 8px;
      padding: 7px 9px;
      border: 1px solid var(--border);
      border-radius: 6px;
      background: var(--surface-bg);
      color: var(--muted);
      font-size: 0.82rem;
    }

    .attachment-draft[hidden],
    .photo-input {
      display: none;
    }

    .upload-button {
      width: 44px;
      min-height: 44px;
      padding: 0;
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--control-border);
      font-size: 1.15rem;
    }

    .clear-attachment-button {
      min-height: 28px;
      padding: 0 8px;
      background: transparent;
      color: var(--accent);
      border: 1px solid var(--control-border);
      font-size: 0.78rem;
    }

    textarea {
      min-width: 0;
      min-height: 44px;
      max-height: 140px;
      padding: 10px 12px;
      border: 1px solid var(--control-border);
      border-radius: 6px;
      font: inherit;
      background: var(--elevated-bg);
      color: var(--text);
      resize: none;
      overflow-y: auto;
    }

    button {
      min-height: 44px;
      padding: 0 16px;
      border: 0;
      border-radius: 6px;
      background: var(--accent);
      color: var(--accent-text);
      font: inherit;
      font-weight: 700;
      cursor: pointer;
    }

    .emoji-button {
      width: 44px;
      min-height: 44px;
      padding: 0;
      border: 1px solid var(--control-border);
      background: var(--elevated-bg);
      color: var(--text);
      font-size: 1.15rem;
    }

    .input-wrap {
      position: relative;
      min-width: 0;
    }

    .input-wrap textarea {
      width: 100%;
    }

    .emoji-suggestions {
      position: absolute;
      z-index: 3;
      right: 0;
      bottom: calc(100% + 6px);
      min-width: 180px;
      padding: 4px;
      border: 1px solid var(--menu-border);
      border-radius: 6px;
      background: var(--elevated-bg);
      box-shadow: var(--menu-shadow);
    }

    .emoji-suggestions[hidden] {
      display: none;
    }

    .emoji-suggestion {
      width: 100%;
      min-height: 34px;
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 0 10px;
      border: 0;
      border-radius: 4px;
      background: transparent;
      color: var(--text);
      text-align: left;
      font-size: 0.9rem;
    }

    .emoji-suggestion[aria-selected="true"],
    .emoji-suggestion:hover,
    .emoji-suggestion:focus {
      background: var(--hover-bg);
    }

    textarea:focus,
    .setting-select:focus,
    .privacy-form input:focus,
    button:focus {
      outline: 3px solid var(--accent-soft);
      outline-offset: 1px;
    }

    @media (prefers-color-scheme: dark) {
      :root:not([data-theme="light"]):not([data-theme="dim"]):not([data-theme="red"]) {
        color-scheme: dark;
        --page-bg: #11181c;
        --app-bg: #182229;
        --surface-bg: #11181c;
        --elevated-bg: #182229;
        --text: #edf3f7;
        --muted: #afbdc5;
        --border: #32434d;
        --control-border: #48616f;
        --menu-border: #48616f;
        --shadow: none;
        --menu-shadow: none;
        --accent: #4b9ad3;
        --accent-text: #071014;
        --logo-color: #f7fbff;
        --message-bg: #253641;
        --own-message-bg: #234a66;
        --hover-bg: #253641;
        --danger: #ffd4d4;
        --danger-bg: #311d1d;
        --privacy-bg: #11181c;
      }
    }

    :root[data-theme="light"] {
      color-scheme: light;
    }

    :root[data-theme="dark"] {
      color-scheme: dark;
      --page-bg: #11181c;
      --app-bg: #182229;
      --surface-bg: #11181c;
      --elevated-bg: #182229;
      --text: #edf3f7;
      --muted: #afbdc5;
      --border: #32434d;
      --control-border: #48616f;
      --menu-border: #48616f;
      --shadow: none;
      --menu-shadow: none;
      --accent: #4b9ad3;
      --accent-text: #071014;
      --logo-color: #f7fbff;
      --message-bg: #253641;
      --own-message-bg: #234a66;
      --hover-bg: #253641;
      --danger: #ffd4d4;
      --danger-bg: #311d1d;
      --privacy-bg: #11181c;
    }

    :root[data-theme="dim"] {
      color-scheme: dark;
      --page-bg: #20252a;
      --app-bg: #2a3036;
      --surface-bg: #22282e;
      --elevated-bg: #303740;
      --text: #eef2f5;
      --muted: #b7c0c7;
      --border: #46515b;
      --control-border: #596672;
      --menu-border: #596672;
      --shadow: none;
      --menu-shadow: 0 8px 18px rgba(0, 0, 0, 0.22);
      --accent: #7fb5de;
      --accent-text: #10171d;
      --logo-color: #f7fbff;
      --message-bg: #3a444d;
      --own-message-bg: #325875;
      --hover-bg: #3a444d;
      --danger: #ffd1d1;
      --danger-bg: #4a2b2f;
      --privacy-bg: #20252a;
    }

    :root[data-theme="red"] {
      color-scheme: dark;
      --page-bg: #2a0508;
      --app-bg: #3b090d;
      --surface-bg: #240407;
      --elevated-bg: #4b0d13;
      --text: #fff5f5;
      --muted: #ffc0c0;
      --border: #8a1f29;
      --control-border: #b8323f;
      --menu-border: #b8323f;
      --shadow: 0 12px 30px rgba(72, 0, 8, 0.28);
      --menu-shadow: 0 8px 18px rgba(0, 0, 0, 0.32);
      --accent: #ff3347;
      --accent-text: #ffffff;
      --accent-soft: #ff8793;
      --logo-color: #f7fbff;
      --presence-off: #b77a80;
      --presence-on: #51d27a;
      --message-bg: #66151e;
      --own-message-bg: #d7192f;
      --hover-bg: #6f1721;
      --danger: #ffe0e0;
      --danger-bg: #7a101d;
      --privacy-bg: #2a0508;
    }

    @media (max-width: 520px) {
      body {
        padding: 0;
      }

      main {
        height: 100vh;
        height: 100dvh;
        min-height: 0;
        gap: 8px;
        padding: 8px;
        border-radius: 0;
        border-width: 0;
      }

      header {
        align-items: start;
        gap: 8px;
      }

      .brand-mark {
        width: 58px;
        height: 37px;
      }

      .status {
        font-size: 0.78rem;
      }

      .presence {
        font-size: 0.76rem;
      }

      .header-actions {
        gap: 6px;
      }

      .settings-button,
      .logout-button {
        min-height: 34px;
        padding: 0 9px;
        font-size: 0.86rem;
      }

      .messages {
        gap: 6px;
        padding: 6px 4px;
        border: 0;
        border-radius: 0;
      }

      .message {
        max-width: 88%;
        gap: 2px;
      }

      .message-meta {
        gap: 6px;
        font-size: 0.66rem;
      }

      .message-meta strong {
        font-size: 0.7rem;
      }

      .message-bubble {
        padding: 6px 8px;
        border-radius: 10px;
      }

      .message-bubble.emoji-only {
        padding: 7px 10px;
        font-size: 2.2rem;
      }

      .typing {
        min-height: 14px;
        font-size: 0.76rem;
      }

      form {
        grid-template-columns: auto minmax(0, 1fr) auto;
        gap: 6px;
      }

      .emoji-bar {
        display: none;
      }

      textarea {
        min-height: 40px;
        padding: 8px 10px;
        border-radius: 18px;
      }

      .upload-button {
        width: 40px;
        min-height: 40px;
        border-radius: 18px;
      }

      button {
        min-height: 40px;
        padding: 0 12px;
        border-radius: 18px;
      }
    }
  </style>
</head>
<body>
  <main>
    <header>
      <div class="title">
        <h1 class="brand-mark" aria-label="Comm">
          <svg class="brand-bird" viewBox="0 0 116 72" aria-hidden="true" focusable="false">
            <path class="brand-bird-wing" d="M11 41c17-16 36-22 58-17"/>
            <path class="brand-bird-wing" d="M27 53c15-12 32-17 51-14"/>
            <path class="brand-bird-body" d="M24 42c13 2 25 7 37 15 10-1 19-5 26-12 7-7 9-15 4-20-7-7-20-2-31 10"/>
            <path class="brand-bird-body" d="M61 35c8-15 19-24 34-27-1 13-7 24-18 33"/>
            <path class="brand-bird-beak" d="M91 27l18 7-18 7 4-7Z"/>
            <circle class="brand-bird-eye" cx="86" cy="25" r="2.2"/>
          </svg>
        </h1>
        <p class="status" id="status" hidden></p>
        <p class="presence">
          <span class="presence-dot" id="presence-dot"></span>
          <span id="presence-label">No one else online</span>
        </p>
      </div>
      <div class="header-actions">
        <button class="settings-button" id="settings-button" type="button" aria-expanded="false" aria-controls="settings-panel">Settings</button>
        <form class="logout-form" method="post" action="/logout">
          <button class="logout-button" type="submit">Log out</button>
        </form>
      </div>
      <div class="settings-panel" id="settings-panel" hidden>
        <h2 class="settings-title">Settings</h2>
        <div class="settings-section">
          <h3 class="settings-heading">Appearance</h3>
          <label class="setting-row">
            <span>Theme</span>
            <select class="setting-select" id="theme-select">
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="dim">Dim</option>
              <option value="red">Red</option>
            </select>
          </label>
          <div class="font-size-control">
            <span>Text size</span>
            <div class="font-size-buttons" aria-label="Text size controls">
              <button class="font-size-button" id="font-size-decrease" type="button" title="Decrease text size">-</button>
              <span class="font-size-value" id="font-size-value">100%</span>
              <button class="font-size-button" id="font-size-increase" type="button" title="Increase text size">+</button>
            </div>
          </div>
        </div>
        <div class="settings-section">
          <h3 class="settings-heading">Privacy Mode</h3>
          <label class="setting-row">
            <input id="privacy-mode" type="checkbox">
            <span>Require password when this tab loses focus</span>
          </label>
        </div>
        <div class="settings-section">
          <h3 class="settings-heading">Session</h3>
          <label class="setting-row">
            <input id="logout-on-close" type="checkbox">
            <span>Log out when this tab closes</span>
          </label>
        </div>
      </div>
    </header>
    <section class="messages" id="messages" aria-live="polite">
      <div class="activity-log-view" id="activity-log-view" hidden>
        <div class="activity-log-hint">Activity logs - press q to return</div>
        <pre class="activity-log-lines" id="activity-log-lines"></pre>
      </div>
    </section>
    <p class="typing" id="typing" aria-live="polite"></p>
    <form id="chat-form">
      <div class="emoji-bar" aria-label="Emoji shortcuts">
        <button class="emoji-button" type="button" data-emoji="🙂" title="Smile">🙂</button>
        <button class="emoji-button" type="button" data-emoji="❤️" title="Heart">❤️</button>
        <button class="emoji-button" type="button" data-emoji="💛" title="Yellow heart">💛</button>
        <button class="emoji-button" type="button" data-emoji="🤗" title="Hug">🤗</button>
        <button class="emoji-button" type="button" data-emoji="😘" title="Kiss">😘</button>
        <button class="emoji-button" type="button" data-emoji="😂" title="Laugh">😂</button>
        <button class="emoji-button" type="button" data-emoji="😏" title="Smirk">😏</button>
        <button class="emoji-button" type="button" data-emoji="🙄" title="Eye roll">🙄</button>
        <button class="emoji-button" type="button" data-emoji="🤦" title="Facepalm">🤦</button>
        <button class="emoji-button" type="button" data-emoji="🤷" title="Shrug">🤷</button>
        <button class="emoji-button" type="button" data-emoji="😭" title="Cry">😭</button>
        <button class="emoji-button" type="button" data-emoji="😡" title="Angry">😡</button>
        <button class="emoji-button" type="button" data-emoji="👊" title="Punch">👊</button>
        <button class="emoji-button" type="button" data-emoji="🖕" title="Middle finger">🖕</button>
        <button class="emoji-button" type="button" data-emoji="✅" title="Yes">✅</button>
        <button class="emoji-button" type="button" data-emoji="❌" title="No">❌</button>
        <button class="emoji-button" type="button" data-emoji="👀" title="Eyes">👀</button>
        <button class="emoji-button" type="button" data-emoji="🔥" title="Fire">🔥</button>
      </div>
      <div class="attachment-draft" id="attachment-draft" hidden>
        <span id="attachment-draft-label"></span>
        <button class="clear-attachment-button" id="clear-attachment" type="button">Remove</button>
      </div>
      <input class="photo-input" id="photo-input" type="file" accept="image/jpeg,image/png,image/webp,image/gif,image/heic,image/heif">
      <button class="upload-button" id="upload-button" type="button" title="Add photo">+</button>
      <div class="input-wrap">
        <div class="emoji-suggestions" id="emoji-suggestions" role="listbox" hidden></div>
        <textarea id="message" name="message" autocomplete="off" maxlength="2000" rows="1"></textarea>
      </div>
      <button type="submit">Send</button>
    </form>
    <section class="photo-viewer" id="photo-viewer" hidden>
      <img id="photo-viewer-image" alt="">
    </section>
    <section class="privacy-screen" id="privacy-screen" hidden aria-live="polite">
      <div class="privacy-content" id="privacy-content" hidden>
        <h2>Chat locked</h2>
        <p>Enter your password to reveal this chat.</p>
        <form class="privacy-form" id="privacy-form">
          <input id="privacy-password" name="password" type="password" autocomplete="current-password" required>
          <p class="privacy-error" id="privacy-error" role="alert"></p>
          <button id="privacy-reveal" type="submit">Reveal chat</button>
        </form>
      </div>
    </section>
  </main>
  <script>
    const currentUser = "{{username}}";
    const statusEl = document.querySelector("#status");
    const presenceDotEl = document.querySelector("#presence-dot");
    const presenceLabelEl = document.querySelector("#presence-label");
    const messagesEl = document.querySelector("#messages");
    const activityLogView = document.querySelector("#activity-log-view");
    const activityLogLines = document.querySelector("#activity-log-lines");
    const photoViewer = document.querySelector("#photo-viewer");
    const photoViewerImage = document.querySelector("#photo-viewer-image");
    const typingEl = document.querySelector("#typing");
    const form = document.querySelector("#chat-form");
    const input = document.querySelector("#message");
    const photoInput = document.querySelector("#photo-input");
    const uploadButton = document.querySelector("#upload-button");
    const attachmentDraft = document.querySelector("#attachment-draft");
    const attachmentDraftLabel = document.querySelector("#attachment-draft-label");
    const clearAttachmentButton = document.querySelector("#clear-attachment");
    const settingsButton = document.querySelector("#settings-button");
    const settingsPanel = document.querySelector("#settings-panel");
    const themeSelect = document.querySelector("#theme-select");
    const fontSizeDecreaseButton = document.querySelector("#font-size-decrease");
    const fontSizeIncreaseButton = document.querySelector("#font-size-increase");
    const fontSizeValue = document.querySelector("#font-size-value");
    const privacyModeInput = document.querySelector("#privacy-mode");
    const privacyScreen = document.querySelector("#privacy-screen");
    const privacyContent = document.querySelector("#privacy-content");
    const privacyForm = document.querySelector("#privacy-form");
    const privacyPasswordInput = document.querySelector("#privacy-password");
    const privacyErrorEl = document.querySelector("#privacy-error");
    const logoutOnCloseInput = document.querySelector("#logout-on-close");
    const emojiSuggestionsEl = document.querySelector("#emoji-suggestions");
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const socket = new WebSocket(`${protocol}//${window.location.host}/ws`);
    const themeKey = `comm.theme.${currentUser}`;
    const fontSizeKey = `comm.fontSize.${currentUser}`;
    const privacyModeKey = `comm.privacyMode.${currentUser}`;
    const logoutOnCloseKey = `comm.logoutOnClose.${currentUser}`;
    const fontSizeMin = 90;
    const fontSizeMax = 130;
    const fontSizeStep = 10;
    const emojiShortcodes = [
      { code: "angry", emoji: "😡" },
      { code: "cry", emoji: "😭" },
      { code: "eyeroll", emoji: "🙄" },
      { code: "eyes", emoji: "👀" },
      { code: "face-punch", emoji: "👊" },
      { code: "facepalm", emoji: "🤦" },
      { code: "finger", emoji: "🖕" },
      { code: "fire", emoji: "🔥" },
      { code: "fu", emoji: "🖕" },
      { code: "heart", emoji: "❤️" },
      { code: "heart-yellow", emoji: "💛" },
      { code: "hug", emoji: "🤗" },
      { code: "kiss", emoji: "😘" },
      { code: "lol", emoji: "😂" },
      { code: "middle-finger", emoji: "🖕" },
      { code: "no", emoji: "❌" },
      { code: "punch", emoji: "👊" },
      { code: "shrug", emoji: "🤷" },
      { code: "smile", emoji: "🙂" },
      { code: "smirk", emoji: "😏" },
      { code: "yes", emoji: "✅" },
      { code: "yellow-heart", emoji: "💛" },
    ];
    let typingTimeoutId = null;
    let typingSent = false;
    let activeEmojiMatch = null;
    let selectedEmojiIndex = 0;
    let privacyTapCount = 0;
    let activityLogMode = false;
    let pendingAttachment = null;
    const onlineUsers = new Set();
    const readMessageIds = new Set();
    const readObserver = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          markMessageRead(entry.target);
        }
      });
    }, { root: messagesEl, threshold: 0.6 });

    themeSelect.value = localStorage.getItem(themeKey) || "system";
    applyTheme(themeSelect.value);
    applyFontSize(readFontSize());
    privacyModeInput.checked = localStorage.getItem(privacyModeKey) === "true";
    logoutOnCloseInput.checked = localStorage.getItem(logoutOnCloseKey) === "true";

    socket.addEventListener("open", () => {
      statusEl.hidden = true;
      statusEl.textContent = "";
      resizeComposer();
      input.focus();
    });

    socket.addEventListener("close", () => {
      statusEl.textContent = "Disconnected";
      statusEl.hidden = false;
      input.disabled = true;
      form.querySelector("button").disabled = true;
    });

    socket.addEventListener("message", (event) => {
      const serverEvent = JSON.parse(event.data);

      if (serverEvent.type === "message") {
        appendMessage(serverEvent);
      }

      if (serverEvent.type === "delete_for_me" || serverEvent.type === "delete_for_everyone") {
        removeMessage(serverEvent.id);
      }

      if (serverEvent.type === "typing") {
        typingEl.textContent = serverEvent.is_typing
          ? `${serverEvent.from} is typing...`
          : "";
      }

      if (serverEvent.type === "presence") {
        updatePresence(serverEvent.username, serverEvent.online);
      }

      if (serverEvent.type === "read") {
        markMessageReadInUi(serverEvent.id, serverEvent.read_at);
      }

      if (serverEvent.type === "activity_logs") {
        showActivityLogs(serverEvent.logs || []);
      }
    });

    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const rawBody = input.value.trim();
      const body = expandEmojiShortcodes(rawBody);

      if ((!body && !pendingAttachment) || socket.readyState !== WebSocket.OPEN) {
        return;
      }

      if (rawBody === "/activityLogs") {
        requestActivityLogs();
        input.value = "";
        resizeComposer();
        sendTyping(false);
        return;
      }

      socket.send(JSON.stringify({
        type: "message",
        body,
        attachment_id: pendingAttachment?.id ?? null,
      }));
      input.value = "";
      clearPendingAttachment();
      resizeComposer();
      sendTyping(false);
    });

    input.addEventListener("input", () => {
      resizeComposer();

      if (!input.value.trim()) {
        closeEmojiSuggestions();
        sendTyping(false);
        return;
      }

      updateEmojiSuggestions();
      sendTyping(true);
      window.clearTimeout(typingTimeoutId);
      typingTimeoutId = window.setTimeout(() => sendTyping(false), 1500);
    });

    input.addEventListener("blur", () => sendTyping(false));

    input.addEventListener("click", updateEmojiSuggestions);

    uploadButton.addEventListener("click", () => {
      photoInput.click();
    });

    clearAttachmentButton.addEventListener("click", clearPendingAttachment);

    photoInput.addEventListener("change", async () => {
      const file = photoInput.files?.[0];

      if (!file) {
        return;
      }

      await uploadPhoto(file);
      photoInput.value = "";
    });

    settingsButton.addEventListener("click", () => {
      const willOpen = settingsPanel.hidden;
      settingsPanel.hidden = !willOpen;
      settingsButton.setAttribute("aria-expanded", String(willOpen));
    });

    logoutOnCloseInput.addEventListener("change", () => {
      localStorage.setItem(logoutOnCloseKey, String(logoutOnCloseInput.checked));
    });

    themeSelect.addEventListener("change", () => {
      localStorage.setItem(themeKey, themeSelect.value);
      applyTheme(themeSelect.value);
    });

    fontSizeDecreaseButton.addEventListener("click", () => {
      setFontSize(readFontSize() - fontSizeStep);
    });

    fontSizeIncreaseButton.addEventListener("click", () => {
      setFontSize(readFontSize() + fontSizeStep);
    });

    privacyModeInput.addEventListener("change", () => {
      localStorage.setItem(privacyModeKey, String(privacyModeInput.checked));
    });

    privacyScreen.addEventListener("click", (event) => {
      if (privacyContent.hidden) {
        privacyTapCount += 1;

        if (privacyTapCount >= 3) {
          showPrivacyPrompt();
        }

        return;
      }

      if (!event.target.closest(".privacy-content")) {
        privacyPasswordInput.focus();
      }
    });

    privacyForm.addEventListener("submit", verifyPrivacyPassword);

    photoViewer.addEventListener("click", closePhotoViewer);

    input.addEventListener("keydown", (event) => {
      if (event.key === "Enter" && !event.shiftKey && emojiSuggestionsEl.hidden) {
        event.preventDefault();
        form.requestSubmit();
        return;
      }

      if (emojiSuggestionsEl.hidden) {
        return;
      }

      if (event.key === "ArrowDown") {
        event.preventDefault();
        moveEmojiSelection(1);
      }

      if (event.key === "ArrowUp") {
        event.preventDefault();
        moveEmojiSelection(-1);
      }

      if (event.key === "Enter") {
        event.preventDefault();
        applySelectedEmoji();
      }

      if (event.key === "Escape") {
        event.preventDefault();
        closeEmojiSuggestions();
      }
    });

    document.querySelectorAll("[data-emoji]").forEach((button) => {
      button.addEventListener("click", () => {
        insertAtCursor(button.dataset.emoji);
        resizeComposer();
        sendTyping(true);
        window.clearTimeout(typingTimeoutId);
        typingTimeoutId = window.setTimeout(() => sendTyping(false), 1500);
        closeEmojiSuggestions();
      });
    });

    window.addEventListener("beforeunload", () => {
      if (socket.readyState === WebSocket.OPEN && typingSent) {
        socket.send(JSON.stringify({ type: "typing", is_typing: false }));
      }

      if (logoutOnCloseInput.checked) {
        logoutFromClosingTab();
      }
    });

    window.addEventListener("pageshow", (event) => {
      if (event.persisted) {
        window.location.reload();
      }
    });

    window.addEventListener("blur", () => {
      if (privacyModeInput.checked) {
        lockPrivacyScreen();
      }
    });

    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "hidden" && privacyModeInput.checked) {
        lockPrivacyScreen();
      }

      if (
        document.visibilityState === "visible"
        && !privacyScreen.hidden
        && !privacyContent.hidden
      ) {
        privacyPasswordInput.focus();
      }

      scanReadableMessages();
    });

    document.addEventListener("click", (event) => {
      if (!event.target.closest("header")) {
        closeSettings();
      }

      if (!event.target.closest(".message")) {
        closeMessageMenus();
      }

      if (!event.target.closest(".input-wrap")) {
        closeEmojiSuggestions();
      }
    });

    document.addEventListener("keydown", (event) => {
      if (
        activityLogMode
        && (event.key === "q" || event.key === "Escape")
        && !event.metaKey
        && !event.ctrlKey
        && !event.altKey
      ) {
        event.preventDefault();
        exitActivityLogs();
        return;
      }

      if (event.key === "Escape") {
        closePhotoViewer();
        closeSettings();
        closeMessageMenus();
        closeEmojiSuggestions();
      }
    });

    function appendMessage(message) {
      if (document.querySelector(`[data-message-id="${message.id}"]`)) {
        return;
      }

      const item = document.createElement("article");
      item.className = message.from === currentUser ? "message own" : "message";
      item.dataset.messageId = message.id;

      const meta = document.createElement("div");
      meta.className = "message-meta";

      const from = document.createElement("strong");
      from.textContent = message.from;

      const sentAt = document.createElement("time");
      const date = new Date(message.created_at);
      sentAt.dateTime = message.created_at;
      sentAt.textContent = Number.isNaN(date.getTime())
        ? message.created_at
        : date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });

      meta.append(from, sentAt);

      const body = document.createElement("div");
      body.className = "message-bubble";
      if (message.attachment) {
        const image = document.createElement("img");
        image.className = "message-photo";
        image.src = `/attachments/${message.attachment.id}`;
        image.alt = message.attachment.original_name || "Attached photo";
        image.loading = "lazy";
        image.addEventListener("click", (event) => {
          event.stopPropagation();
          openPhotoViewer(image.src, image.alt);
        });
        body.append(image);
      }

      if (message.body) {
        const text = document.createElement("div");
        text.className = "message-text";
        renderMessageBody(text, message.body);
        body.append(text);
      }

      body.classList.toggle(
        "emoji-only",
        !message.attachment && isSingleEmojiMessage(message.body)
      );

      if (message.from === currentUser) {
        const readStatus = document.createElement("span");
        readStatus.className = "read-status";
        readStatus.setAttribute("aria-label", message.read_at ? "Read" : "Unread");
        if (message.read_at) {
          readStatus.classList.add("is-read");
          readStatus.title = `Read ${new Date(message.read_at).toLocaleString()}`;
        }
        body.append(readStatus);
      }

      const menu = document.createElement("div");
      menu.className = "message-menu";
      menu.hidden = true;

      const deleteForMeButton = document.createElement("button");
      deleteForMeButton.className = "message-action";
      deleteForMeButton.type = "button";
      deleteForMeButton.textContent = "Delete for me";
      deleteForMeButton.addEventListener("click", (event) => {
        event.stopPropagation();

        if (socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify({ type: "delete_for_me", id: message.id }));
        }

        closeMessageMenus();
      });

      menu.append(deleteForMeButton);

      if (message.from === currentUser) {
        const deleteForEveryoneButton = document.createElement("button");
        deleteForEveryoneButton.className = "message-action danger";
        deleteForEveryoneButton.type = "button";
        deleteForEveryoneButton.textContent = "Delete for everyone";
        deleteForEveryoneButton.addEventListener("click", (event) => {
          event.stopPropagation();

          if (socket.readyState === WebSocket.OPEN) {
            socket.send(JSON.stringify({ type: "delete_for_everyone", id: message.id }));
          }

          closeMessageMenus();
        });

        menu.append(deleteForEveryoneButton);
      }

      item.addEventListener("click", (event) => {
        if (event.target.closest(".message-menu, a")) {
          return;
        }

        const willOpen = menu.hidden;
        closeMessageMenus();
        menu.hidden = !willOpen;
      });

      item.append(meta, body, menu);
      messagesEl.append(item);
      scrollMessagesToBottom();

      if (message.from !== currentUser) {
        readObserver.observe(item);
        markMessageRead(item);
      }
    }

    function requestActivityLogs() {
      messagesEl.classList.add("activity-mode");
      activityLogView.hidden = false;
      activityLogMode = true;
      typingEl.hidden = true;
      activityLogLines.textContent = "Loading activity logs...";
      closeMessageMenus();
      closeEmojiSuggestions();
      socket.send(JSON.stringify({ type: "activity_logs" }));
    }

    function showActivityLogs(logs) {
      messagesEl.classList.add("activity-mode");
      activityLogView.hidden = false;
      activityLogMode = true;
      typingEl.hidden = true;

      activityLogLines.textContent = logs.length > 0
        ? logs.map(formatActivityLog).join("\n")
        : "No activity logs yet.";
      messagesEl.scrollTop = 0;
    }

    function exitActivityLogs() {
      activityLogMode = false;
      activityLogView.hidden = true;
      messagesEl.classList.remove("activity-mode");
      typingEl.hidden = false;
      scrollMessagesToBottom();
      input.focus();
      scanReadableMessages();
    }

    function scrollMessagesToBottom() {
      requestAnimationFrame(() => {
        messagesEl.scrollTop = messagesEl.scrollHeight;
      });
    }

    async function uploadPhoto(file) {
      if (!file.type.startsWith("image/")) {
        attachmentDraft.hidden = false;
        attachmentDraftLabel.textContent = "Choose an image file.";
        return;
      }

      if (file.size > 20 * 1024 * 1024) {
        attachmentDraft.hidden = false;
        attachmentDraftLabel.textContent = "Photo is too large. Max size is 20 MB.";
        return;
      }

      uploadButton.disabled = true;
      attachmentDraft.hidden = false;
      attachmentDraftLabel.textContent = "Encrypting photo...";

      const formData = new FormData();
      formData.append("photo", file);

      let response;
      try {
        response = await fetch("/attachments", {
          method: "POST",
          body: formData,
          credentials: "same-origin",
        });
      } catch {
        attachmentDraftLabel.textContent = "Photo upload failed. Check the connection.";
        uploadButton.disabled = false;
        return;
      }

      uploadButton.disabled = false;

      if (!response.ok) {
        attachmentDraftLabel.textContent = "Photo upload failed.";
        return;
      }

      pendingAttachment = await response.json();
      renderPendingAttachment();
      input.focus();
    }

    function renderPendingAttachment() {
      if (!pendingAttachment) {
        attachmentDraft.hidden = true;
        attachmentDraftLabel.textContent = "";
        return;
      }

      attachmentDraft.hidden = false;
      attachmentDraftLabel.textContent = pendingAttachment.original_name
        ? `Photo ready: ${pendingAttachment.original_name}`
        : "Photo ready";
    }

    function clearPendingAttachment() {
      pendingAttachment = null;
      renderPendingAttachment();
    }

    function openPhotoViewer(src, alt) {
      closeMessageMenus();
      closeEmojiSuggestions();
      photoViewerImage.src = src;
      photoViewerImage.alt = alt;
      photoViewer.hidden = false;
    }

    function closePhotoViewer() {
      if (photoViewer.hidden) {
        return;
      }

      photoViewer.hidden = true;
      photoViewerImage.removeAttribute("src");
      photoViewerImage.alt = "";
    }

    function formatActivityLog(log) {
      return `${log.occurred_at} user ${log.action}: ${log.username}`;
    }

    function isSingleEmojiMessage(value) {
      const text = value.trim();

      if (!text) {
        return false;
      }

      const segments = typeof Intl !== "undefined" && Intl.Segmenter
        ? [...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(text)].map((segment) => segment.segment)
        : Array.from(text.replace(/\uFE0F/g, ""));

      return segments.length === 1 && /(\p{Extended_Pictographic}|\u2705|\u274C)/u.test(segments[0]);
    }

    function renderMessageBody(container, value) {
      container.replaceChildren();
      const urlPattern = /\bhttps?:\/\/[^\s<>"']+/gi;
      let lastIndex = 0;

      for (const match of value.matchAll(urlPattern)) {
        const rawUrl = trimTrailingUrlPunctuation(match[0]);
        const start = match.index;
        const end = start + rawUrl.length;

        if (start > lastIndex) {
          container.append(document.createTextNode(value.slice(lastIndex, start)));
        }

        const link = document.createElement("a");
        link.href = rawUrl;
        link.textContent = rawUrl;
        link.target = "_blank";
        link.rel = "noopener noreferrer";
        container.append(link);

        lastIndex = end;
      }

      if (lastIndex < value.length) {
        container.append(document.createTextNode(value.slice(lastIndex)));
      }
    }

    function trimTrailingUrlPunctuation(value) {
      return value.replace(/[),.!?;:]+$/u, "");
    }

    function removeMessage(id) {
      const item = document.querySelector(`[data-message-id="${id}"]`);

      if (item) {
        readObserver.unobserve(item);
        readMessageIds.delete(Number(id));
        item.remove();
      }
    }

    function markMessageRead(item) {
      if (
        item.classList.contains("own")
        || !privacyScreen.hidden
        || document.visibilityState !== "visible"
        || socket.readyState !== WebSocket.OPEN
        || !messageIsVisible(item)
      ) {
        return;
      }

      const id = Number(item.dataset.messageId);
      if (!Number.isFinite(id) || readMessageIds.has(id)) {
        return;
      }

      readMessageIds.add(id);
      readObserver.unobserve(item);
      socket.send(JSON.stringify({ type: "read", id }));
    }

    function messageIsVisible(item) {
      const itemRect = item.getBoundingClientRect();
      const messagesRect = messagesEl.getBoundingClientRect();
      return itemRect.bottom > messagesRect.top && itemRect.top < messagesRect.bottom;
    }

    function scanReadableMessages() {
      if (!privacyScreen.hidden || document.visibilityState !== "visible") {
        return;
      }

      document.querySelectorAll(".message:not(.own)").forEach(markMessageRead);
    }

    function markMessageReadInUi(id, readAt) {
      const item = document.querySelector(`[data-message-id="${id}"]`);
      const readStatus = item?.querySelector(".read-status");

      if (!readStatus) {
        return;
      }

      readStatus.classList.add("is-read");
      readStatus.setAttribute("aria-label", "Read");
      if (readAt) {
        readStatus.title = `Read ${new Date(readAt).toLocaleString()}`;
      }
    }

    function closeMessageMenus() {
      document.querySelectorAll(".message-menu").forEach((menu) => {
        menu.hidden = true;
      });
    }

    function closeSettings() {
      settingsPanel.hidden = true;
      settingsButton.setAttribute("aria-expanded", "false");
    }

    function applyTheme(theme) {
      if (theme === "system") {
        document.documentElement.removeAttribute("data-theme");
        return;
      }

      document.documentElement.dataset.theme = theme;
    }

    function readFontSize() {
      const stored = Number(localStorage.getItem(fontSizeKey));
      return clampFontSize(Number.isFinite(stored) ? stored : 100);
    }

    function setFontSize(value) {
      const nextValue = clampFontSize(value);
      localStorage.setItem(fontSizeKey, String(nextValue));
      applyFontSize(nextValue);
    }

    function applyFontSize(value) {
      const nextValue = clampFontSize(value);

      if (nextValue === 100) {
        document.documentElement.style.fontSize = "";
      } else {
        document.documentElement.style.fontSize = `${nextValue}%`;
      }

      fontSizeValue.textContent = `${nextValue}%`;
      fontSizeDecreaseButton.disabled = nextValue <= fontSizeMin;
      fontSizeIncreaseButton.disabled = nextValue >= fontSizeMax;
      resizeComposer();
    }

    function clampFontSize(value) {
      return Math.min(fontSizeMax, Math.max(fontSizeMin, value));
    }

    function lockPrivacyScreen() {
      closeSettings();
      closeMessageMenus();
      closeEmojiSuggestions();
      sendTyping(false);
      privacyForm.reset();
      privacyErrorEl.textContent = "";
      privacyContent.hidden = true;
      privacyTapCount = 0;
      privacyScreen.hidden = false;
    }

    function unlockPrivacyScreen() {
      privacyScreen.hidden = true;
      privacyContent.hidden = true;
      input.focus();
      scanReadableMessages();
    }

    function showPrivacyPrompt() {
      privacyTapCount = 0;
      privacyContent.hidden = false;
      privacyPasswordInput.focus();
    }

    async function verifyPrivacyPassword(event) {
      event.preventDefault();
      privacyErrorEl.textContent = "";

      let response;
      try {
        response = await fetch("/verify-password", {
          method: "POST",
          body: new URLSearchParams(new FormData(privacyForm)),
          credentials: "same-origin",
        });
      } catch {
        privacyErrorEl.textContent = "Could not verify password. Check the connection.";
        return;
      }

      if (response.status === 204) {
        privacyForm.reset();
        unlockPrivacyScreen();
        return;
      }

      if (response.status === 429) {
        privacyErrorEl.textContent = "Too many failed attempts. Try again shortly.";
        return;
      }

      privacyErrorEl.textContent = "Incorrect password.";
      privacyPasswordInput.select();
    }

    function logoutFromClosingTab() {
      if (navigator.sendBeacon) {
        navigator.sendBeacon("/logout", new Blob([], { type: "application/x-www-form-urlencoded" }));
        return;
      }

      fetch("/logout", {
        method: "POST",
        credentials: "same-origin",
        keepalive: true,
      });
    }

    function sendTyping(isTyping) {
      window.clearTimeout(typingTimeoutId);

      if (typingSent === isTyping || socket.readyState !== WebSocket.OPEN) {
        return;
      }

      typingSent = isTyping;
      socket.send(JSON.stringify({ type: "typing", is_typing: isTyping }));
    }

    function updatePresence(username, online) {
      if (username === currentUser) {
        return;
      }

      if (online) {
        onlineUsers.add(username);
      } else {
        onlineUsers.delete(username);
      }

      const others = [...onlineUsers].sort();
      presenceDotEl.classList.toggle("online", others.length > 0);
      presenceLabelEl.textContent = others.length > 0
        ? `${others.join(", ")} online`
        : "No one else online";
    }

    function insertAtCursor(value) {
      const start = input.selectionStart ?? input.value.length;
      const end = input.selectionEnd ?? input.value.length;
      input.value = `${input.value.slice(0, start)}${value}${input.value.slice(end)}`;
      const nextPosition = start + value.length;
      input.focus();
      input.setSelectionRange(nextPosition, nextPosition);
      resizeComposer();
    }

    function updateEmojiSuggestions() {
      const match = activeEmojiShortcode();

      if (!match) {
        closeEmojiSuggestions();
        return;
      }

      const matches = emojiShortcodes.filter((entry) => entry.code.startsWith(match.query));

      if (matches.length === 0) {
        closeEmojiSuggestions();
        return;
      }

      const previousQuery = activeEmojiMatch?.query;
      activeEmojiMatch = { ...match, matches };
      if (previousQuery !== match.query) {
        selectedEmojiIndex = 0;
      }
      selectedEmojiIndex = Math.min(selectedEmojiIndex, matches.length - 1);
      emojiSuggestionsEl.replaceChildren(
        ...matches.map((entry, index) => {
          const option = document.createElement("button");
          option.className = "emoji-suggestion";
          option.type = "button";
          option.role = "option";
          option.ariaSelected = index === selectedEmojiIndex ? "true" : "false";
          option.textContent = `${entry.emoji} :${entry.code}`;
          option.addEventListener("mousedown", (event) => event.preventDefault());
          option.addEventListener("click", () => {
            selectedEmojiIndex = index;
            applySelectedEmoji();
          });
          return option;
        })
      );
      emojiSuggestionsEl.hidden = false;
    }

    function activeEmojiShortcode() {
      const cursor = input.selectionStart ?? input.value.length;
      const prefix = input.value.slice(0, cursor);
      const match = prefix.match(/(^|\s):([a-z]{1,20})$/i);

      if (!match) {
        return null;
      }

      return {
        start: cursor - match[2].length - 1,
        end: cursor,
        query: match[2].toLowerCase(),
      };
    }

    function moveEmojiSelection(direction) {
      if (!activeEmojiMatch) {
        return;
      }

      const count = activeEmojiMatch.matches.length;
      selectedEmojiIndex = (selectedEmojiIndex + direction + count) % count;
      updateEmojiSuggestions();
    }

    function applySelectedEmoji() {
      if (!activeEmojiMatch) {
        return;
      }

      const entry = activeEmojiMatch.matches[selectedEmojiIndex];
      input.value =
        input.value.slice(0, activeEmojiMatch.start) +
        entry.emoji +
        input.value.slice(activeEmojiMatch.end);
      const nextPosition = activeEmojiMatch.start + entry.emoji.length;
      input.focus();
      input.setSelectionRange(nextPosition, nextPosition);
      resizeComposer();
      closeEmojiSuggestions();
    }

    function closeEmojiSuggestions() {
      emojiSuggestionsEl.hidden = true;
      emojiSuggestionsEl.replaceChildren();
      activeEmojiMatch = null;
      selectedEmojiIndex = 0;
    }

    function expandEmojiShortcodes(value) {
      return emojiShortcodes.reduce(
        (result, entry) => result.replaceAll(`:${entry.code}:`, entry.emoji).replaceAll(`:${entry.code}`, entry.emoji),
        value
      );
    }

    function resizeComposer() {
      input.style.height = "auto";
      input.style.height = `${Math.min(input.scrollHeight, 140)}px`;
    }
  </script>
</body>
</html>
"##;
