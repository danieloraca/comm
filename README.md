# Private Two-Party Chat

Small Rust chat app for two users. The server runs on a trusted host, stores message history locally, and exposes the chat only to trusted devices, usually through Tailscale.

## Goals

- Two authenticated users.
- Real-time messaging over WebSockets.
- Local SQLite message history on the trusted host.
- Argon2id password hashes, never plaintext passwords.
- Encrypted message bodies at rest.
- No direct public internet exposure.

## Network Model

Use Tailscale so clients can reach the server through a private network address without opening a public router port.

Example layout:

| Device | Address | Role |
| --- | --- | --- |
| `chat-server` | `<server-tailscale-ip>:8787` | Runs the Rust server and stores data |
| `chat-client` | `<client-tailscale-ip>` | Opens the chat in a browser |

Connectivity check:

```bash
tailscale status
python3 -m http.server 8787 --bind <server-tailscale-ip>
```

Then open this from the client device:

```text
http://<server-tailscale-ip>:8787
```

## Configuration

The app reads local files and bind settings from environment variables.

| Variable | Default | Purpose |
| --- | --- | --- |
| `COMM_BIND_ADDR` | `127.0.0.1:8787` | Address and port the server listens on |
| `COMM_USERS_FILE` | `users.toml` | Usernames and Argon2id password hashes |
| `COMM_DATABASE_FILE` | `comm.sqlite3` | SQLite database path |
| `COMM_MESSAGE_KEY_FILE` | `message.key` | Local encryption key path |
| `COMM_PRESENCE_SOUND` | unset | Optional sound file played with `afplay` when a user comes online/offline |

For Tailscale access, run with a Tailscale bind address:

```bash
COMM_BIND_ADDR=<server-tailscale-ip>:8787 cargo run --bin comm
```

On macOS, choose a presence sound with:

```bash
COMM_PRESENCE_SOUND=/System/Library/Sounds/Glass.aiff cargo run --bin comm
```

If `COMM_PRESENCE_SOUND` is unset, the app falls back to the terminal bell.

## Users And Passwords

Create password hashes with:

```bash
cargo run --bin hash_password
```

Put the generated hashes in `users.toml`:

```toml
[[user]]
username = "alice"
password_hash = "$argon2id$..."

[[user]]
username = "bob"
password_hash = "$argon2id$..."
```

To change a password, generate a new hash, replace that user's `password_hash`, and restart the server. Existing password hashes cannot be reversed.

## Current Features

- `GET /` serves the login page.
- `POST /login` validates credentials, creates an in-memory session, and redirects to `/chat`.
- `GET /chat` serves the authenticated chat UI.
- `POST /logout` removes the current session and expires the session cookie.
- `POST /verify-password` checks the current user's password for Privacy Mode unlock.
- `GET /ws` opens the authenticated WebSocket connection.
- New WebSocket clients receive recent message history.
- Messages are encrypted before being written to SQLite.
- `Delete for me` hides a message only for the requester.
- `Delete for everyone` is allowed only for the sender and soft-deletes the message for both users.
- Typing indicators and online presence are transient WebSocket events and are not stored.
- The online dot is based on active WebSocket connections.
- Read receipts are stored locally and are sent only after an incoming message is visible while the chat is unlocked.
- Emoji toolbar and shortcodes are supported for common reactions such as `:smile`, `:heart`, `:hug`, `:lol`, `:punch`, `:face-punch`, `:kiss`, `:smirk`, `:eyeroll`, `:cry`, `:angry`, `:fire`, `:yes`, `:no`, `:eyes`, `:facepalm`, `:shrug`, `:middle-finger`, `:finger`, and `:fu`.
- The message composer supports multiline input with Shift+Enter; Enter sends.
- Per-user appearance preferences are stored in the browser, including `System`, `Light`, `Dark`, `Dim`, and `Red` themes plus text size controls from 90% to 130%.

## Privacy Mode

Privacy Mode is available from the chat Settings menu.

When enabled:

- Losing tab or browser focus covers the chat with a blank privacy screen.
- The privacy screen shows no text or password field at first.
- Three clicks or taps on the blank screen reveal the password prompt.
- The current user's password must be verified by `POST /verify-password` before the chat is shown again.
- Failed unlock attempts reuse the same in-memory rate limiting as login attempts.

Privacy Mode hides the browser UI; it does not destroy the authenticated session. Use the separate `Log out when this tab closes` setting if you also want best-effort session removal on close.

## Security Notes

- Keep `users.toml`, `comm.sqlite3`, and `message.key` private.
- `users.toml`, `comm.sqlite3`, and `message.key` should not be committed.
- If someone copies only `comm.sqlite3`, message bodies should not be readable.
- If someone copies both `comm.sqlite3` and `message.key`, they can decrypt message bodies.
- Login and Privacy Mode password failures are rate-limited in memory; the limit resets when the server restarts.
- Close-tab logout uses browser `sendBeacon` when available and `fetch(... keepalive: true)` as a fallback. This is convenient but not guaranteed by browsers.
- Tailscale protects network transport between devices, but the app still requires its own authentication.
- If browser HTTPS warnings become a problem, use Tailscale HTTPS certificates or a local reverse proxy.

## Development Checks

```bash
cargo fmt
cargo build
cargo test
cargo build --release
```
