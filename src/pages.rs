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
      background: #f5f7f8;
      color: #192024;
    }

    * {
      box-sizing: border-box;
    }

    body {
      min-height: 100vh;
      margin: 0;
      padding: 18px;
      background: #eef2f4;
    }

    main {
      width: min(100%, 760px);
      min-height: calc(100vh - 36px);
      margin: 0 auto;
      display: grid;
      grid-template-rows: auto 1fr auto;
      gap: 14px;
      padding: 20px;
      border: 1px solid #d5dde2;
      border-radius: 8px;
      background: #ffffff;
      box-shadow: 0 12px 30px rgba(25, 32, 36, 0.08);
    }

    h1 {
      margin: 0;
      font-size: 1.35rem;
      line-height: 1.2;
    }

    p {
      margin: 0;
      color: #53636d;
    }

    header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
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
      color: #53636d;
    }

    .presence-dot {
      width: 8px;
      height: 8px;
      border-radius: 999px;
      background: #7f8d96;
    }

    .presence-dot.online {
      background: #24b45a;
    }

    .typing {
      min-height: 18px;
      font-size: 0.84rem;
      color: #53636d;
    }

    .logout-form {
      display: block;
    }

    .logout-button {
      min-height: 36px;
      padding: 0 12px;
      background: transparent;
      color: #1d5f8f;
      border: 1px solid #9fb5c3;
    }

    .messages {
      min-height: 280px;
      overflow: auto;
      display: flex;
      flex-direction: column;
      gap: 10px;
      padding: 12px;
      border: 1px solid #d5dde2;
      border-radius: 6px;
      background: #f8fafb;
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
      color: #53636d;
      font-size: 0.72rem;
    }

    .message-meta strong {
      font-size: 0.78rem;
    }

    .message-bubble {
      max-width: 100%;
      padding: 8px 10px;
      border-radius: 7px;
      background: #e6eef4;
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }

    .message.own .message-bubble {
      background: #d6e9f7;
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
      border: 1px solid #c7d1d8;
      border-radius: 6px;
      background: #ffffff;
      box-shadow: 0 8px 18px rgba(25, 32, 36, 0.14);
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
      color: #192024;
      text-align: left;
      font-size: 0.88rem;
    }

    .message-action:hover,
    .message-action:focus {
      background: #edf3f7;
    }

    .message-action.danger {
      color: #8a1f1f;
    }

    .message-action.danger:hover,
    .message-action.danger:focus {
      background: #fff0f0;
    }

    form {
      display: grid;
      grid-template-columns: auto 1fr auto;
      gap: 10px;
    }

    .emoji-bar {
      display: flex;
      gap: 6px;
    }

    textarea {
      min-width: 0;
      min-height: 44px;
      max-height: 140px;
      padding: 10px 12px;
      border: 1px solid #bac5cc;
      border-radius: 6px;
      font: inherit;
      background: #ffffff;
      color: #192024;
      resize: none;
      overflow-y: auto;
    }

    button {
      min-height: 44px;
      padding: 0 16px;
      border: 0;
      border-radius: 6px;
      background: #1d5f8f;
      color: #ffffff;
      font: inherit;
      font-weight: 700;
      cursor: pointer;
    }

    .emoji-button {
      width: 44px;
      min-height: 44px;
      padding: 0;
      border: 1px solid #bac5cc;
      background: #ffffff;
      color: #192024;
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
      border: 1px solid #c7d1d8;
      border-radius: 6px;
      background: #ffffff;
      box-shadow: 0 8px 18px rgba(25, 32, 36, 0.14);
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
      color: #192024;
      text-align: left;
      font-size: 0.9rem;
    }

    .emoji-suggestion[aria-selected="true"],
    .emoji-suggestion:hover,
    .emoji-suggestion:focus {
      background: #edf3f7;
    }

    textarea:focus,
    button:focus {
      outline: 3px solid #9ac2ff;
      outline-offset: 1px;
    }

    @media (prefers-color-scheme: dark) {
      :root,
      body {
        background: #11181c;
        color: #edf3f7;
      }

      main {
        border-color: #32434d;
        background: #182229;
        box-shadow: none;
      }

      p {
        color: #afbdc5;
      }

      .typing {
        color: #afbdc5;
      }

      .presence {
        color: #afbdc5;
      }

      .messages {
        border-color: #32434d;
        background: #11181c;
      }

      .message-bubble {
        background: #253641;
      }

      .message.own {
        background: transparent;
      }

      .message-meta,
      .message time {
        color: #afbdc5;
      }

      .message.own .message-bubble {
        background: #234a66;
      }

      .message-menu {
        border-color: #48616f;
        background: #182229;
        box-shadow: none;
      }

      .message-action {
        color: #edf3f7;
      }

      .message-action:hover,
      .message-action:focus {
        background: #253641;
      }

      .message-action.danger {
        color: #ffd4d4;
      }

      .message-action.danger:hover,
      .message-action.danger:focus {
        background: #311d1d;
      }

      textarea {
        border-color: #48616f;
        background: #11181c;
        color: #edf3f7;
      }

      button {
        background: #4b9ad3;
        color: #071014;
      }

      .emoji-button {
        border-color: #48616f;
        background: #11181c;
        color: #edf3f7;
      }

      .emoji-suggestions {
        border-color: #48616f;
        background: #182229;
        box-shadow: none;
      }

      .emoji-suggestion {
        color: #edf3f7;
      }

      .emoji-suggestion[aria-selected="true"],
      .emoji-suggestion:hover,
      .emoji-suggestion:focus {
        background: #253641;
      }

      .logout-button {
        background: transparent;
        color: #8bc7f0;
        border-color: #48616f;
      }
    }

    @media (max-width: 520px) {
      body {
        padding: 0;
      }

      main {
        min-height: 100vh;
        border-radius: 0;
        border-width: 0;
      }

      form {
        grid-template-columns: 1fr;
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
      <form class="logout-form" method="post" action="/logout">
        <button class="logout-button" type="submit">Log out</button>
      </form>
    </header>
    <section class="messages" id="messages" aria-live="polite"></section>
    <p class="typing" id="typing" aria-live="polite"></p>
    <form id="chat-form">
      <div class="emoji-bar" aria-label="Emoji shortcuts">
        <button class="emoji-button" type="button" data-emoji="🙂" title="Smile">🙂</button>
        <button class="emoji-button" type="button" data-emoji="❤️" title="Heart">❤️</button>
      </div>
      <div class="input-wrap">
        <div class="emoji-suggestions" id="emoji-suggestions" role="listbox" hidden></div>
        <textarea id="message" name="message" autocomplete="off" maxlength="2000" rows="1" required></textarea>
      </div>
      <button type="submit">Send</button>
    </form>
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
    const emojiSuggestionsEl = document.querySelector("#emoji-suggestions");
    const protocol = window.location.protocol === "https:" ? "wss:" : "ws:";
    const socket = new WebSocket(`${protocol}//${window.location.host}/ws`);
    const emojiShortcodes = [
      { code: "heart", emoji: "❤️" },
      { code: "smile", emoji: "🙂" },
    ];
    let typingTimeoutId = null;
    let typingSent = false;
    let activeEmojiMatch = null;
    let selectedEmojiIndex = 0;
    const onlineUsers = new Set();

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
    });

    document.addEventListener("click", (event) => {
      if (!event.target.closest(".message")) {
        closeMessageMenus();
      }

      if (!event.target.closest(".input-wrap")) {
        closeEmojiSuggestions();
      }
    });

    document.addEventListener("keydown", (event) => {
      if (event.key === "Escape") {
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
    }

    function removeMessage(id) {
      const item = document.querySelector(`[data-message-id="${id}"]`);

      if (item) {
        item.remove();
      }
    }

    function closeMessageMenus() {
      document.querySelectorAll(".message-menu").forEach((menu) => {
        menu.hidden = true;
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
