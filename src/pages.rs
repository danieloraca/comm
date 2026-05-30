use axum::{extract::Query, response::Html};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginPageQuery {
    error: Option<String>,
}

pub async fn login_page(Query(query): Query<LoginPageQuery>) -> Html<String> {
    let error = match query.error.as_deref() {
        Some("rate_limited") => {
            r#"<p class="error" role="alert">Too many failed attempts. Try again shortly.</p>"#
        }
        Some(_) => r#"<p class="error" role="alert">Invalid username or password.</p>"#,
        None => "",
    };

    Html(LOGIN_PAGE.replace("{{error}}", error))
}

pub fn chat_page(username: &str) -> Html<String> {
    Html(CHAT_PAGE.replace("{{username}}", username))
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
      width: min(100%, 360px);
      display: grid;
      gap: 18px;
    }

    h1 {
      margin: 0;
      font-size: 1.45rem;
      line-height: 1.2;
      font-weight: 700;
    }

    form {
      display: grid;
      gap: 14px;
      padding: 20px;
      border: 1px solid #d5dde2;
      border-radius: 8px;
      background: #ffffff;
      box-shadow: 0 12px 30px rgba(25, 32, 36, 0.08);
    }

    label {
      display: grid;
      gap: 6px;
      font-size: 0.9rem;
      font-weight: 650;
    }

    input {
      width: 100%;
      min-height: 44px;
      padding: 10px 12px;
      border: 1px solid #bac5cc;
      border-radius: 6px;
      font: inherit;
      background: #ffffff;
      color: #192024;
    }

    input:focus {
      outline: 3px solid #9ac2ff;
      outline-offset: 1px;
      border-color: #336fb2;
    }

    button {
      min-height: 44px;
      border: 0;
      border-radius: 6px;
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
        border-color: #32434d;
        background: #182229;
        box-shadow: none;
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
  </style>
</head>
<body>
  <main>
    <h1>Comm</h1>
    <form method="post" action="/login">
      {{error}}
      <label>
        Username
        <input name="username" autocomplete="username" required>
      </label>
      <label>
        Password
        <input name="password" type="password" autocomplete="current-password" required>
      </label>
      <button type="submit">Log in</button>
    </form>
  </main>
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

    .read-status {
      color: var(--muted);
      font-size: 0.72rem;
      font-weight: 700;
    }

    .read-status[hidden] {
      display: none;
    }

    .message-bubble {
      max-width: 100%;
      padding: 8px 10px;
      border-radius: 7px;
      background: var(--message-bg);
      white-space: pre-wrap;
      overflow-wrap: anywhere;
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
      grid-template-columns: 1fr auto;
      gap: 10px;
    }

    .emoji-bar {
      display: flex;
      grid-column: 1 / -1;
      gap: 6px;
      overflow-x: auto;
      padding-bottom: 2px;
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
      :root:not([data-theme="light"]):not([data-theme="dim"]) {
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
      --message-bg: #3a444d;
      --own-message-bg: #325875;
      --hover-bg: #3a444d;
      --danger: #ffd1d1;
      --danger-bg: #4a2b2f;
      --privacy-bg: #20252a;
    }

    @media (max-width: 520px) {
      body {
        padding: 0;
      }

      main {
        height: 100vh;
        height: 100dvh;
        min-height: 0;
        border-radius: 0;
        border-width: 0;
      }

      form {
        grid-template-columns: minmax(0, 1fr) auto;
      }

      header {
        align-items: start;
      }
    }
  </style>
</head>
<body>
  <main>
    <header>
      <div class="title">
        <h1>Comm</h1>
        <p class="status" id="status">Connecting as {{username}}</p>
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
    <section class="messages" id="messages" aria-live="polite"></section>
    <p class="typing" id="typing" aria-live="polite"></p>
    <form id="chat-form">
      <div class="emoji-bar" aria-label="Emoji shortcuts">
        <button class="emoji-button" type="button" data-emoji="🙂" title="Smile">🙂</button>
        <button class="emoji-button" type="button" data-emoji="❤️" title="Heart">❤️</button>
        <button class="emoji-button" type="button" data-emoji="🤗" title="Hug">🤗</button>
        <button class="emoji-button" type="button" data-emoji="💛" title="Yellow heart">💛</button>
        <button class="emoji-button" type="button" data-emoji="😂" title="Laugh">😂</button>
      </div>
      <div class="input-wrap">
        <div class="emoji-suggestions" id="emoji-suggestions" role="listbox" hidden></div>
        <textarea id="message" name="message" autocomplete="off" maxlength="2000" rows="1" required></textarea>
      </div>
      <button type="submit">Send</button>
    </form>
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
    const typingEl = document.querySelector("#typing");
    const form = document.querySelector("#chat-form");
    const input = document.querySelector("#message");
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
      { code: "heart", emoji: "❤️" },
      { code: "heart-yellow", emoji: "💛" },
      { code: "hug", emoji: "🤗" },
      { code: "lol", emoji: "😂" },
      { code: "smile", emoji: "🙂" },
      { code: "yellow-heart", emoji: "💛" },
    ];
    let typingTimeoutId = null;
    let typingSent = false;
    let activeEmojiMatch = null;
    let selectedEmojiIndex = 0;
    let privacyTapCount = 0;
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
      statusEl.textContent = `Connected as ${currentUser}`;
      resizeComposer();
      input.focus();
    });

    socket.addEventListener("close", () => {
      statusEl.textContent = "Disconnected";
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
    });

    form.addEventListener("submit", (event) => {
      event.preventDefault();
      const body = expandEmojiShortcodes(input.value.trim());

      if (!body || socket.readyState !== WebSocket.OPEN) {
        return;
      }

      socket.send(JSON.stringify({ type: "message", body }));
      input.value = "";
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
      if (event.key === "Escape") {
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

      if (message.from === currentUser) {
        const readStatus = document.createElement("span");
        readStatus.className = "read-status";
        readStatus.textContent = "Read";
        readStatus.hidden = !message.read_at;
        meta.append(readStatus);
      }

      const body = document.createElement("div");
      body.className = "message-bubble";
      body.textContent = message.body;

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
        if (event.target.closest(".message-menu")) {
          return;
        }

        const willOpen = menu.hidden;
        closeMessageMenus();
        menu.hidden = !willOpen;
      });

      item.append(meta, body, menu);
      messagesEl.append(item);
      messagesEl.scrollTop = messagesEl.scrollHeight;

      if (message.from !== currentUser) {
        readObserver.observe(item);
        markMessageRead(item);
      }
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

      readStatus.hidden = false;
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
