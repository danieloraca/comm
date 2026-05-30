# Private Two-Party Chat

This project is a small Rust chat app for two endpoints. The server runs on a trusted host, stores message history locally, and lets both parties send and receive messages after logging in.

## Goal

- Run the server on the trusted host machine.
- Allow exactly two users to log in with usernames and passwords.
- Let both users exchange messages in real time.
- Store message history only on the trusted host.
- Store passwords as hashes, not plaintext.
- Store messages encrypted at rest.
- Avoid exposing the app directly to the public internet.

## Connection Approach

The first networking approach is Tailscale.

Tailscale creates a private network between trusted devices. This means the Rust app can listen on the trusted host and be reached from another trusted device without opening a public router port.

Example device layout:

| Device                 | Tailscale IP | OS |
|------------------------| --- | --- |
| `chat-server`          | `<server-tailscale-ip>` | macOS/Linux |
| `chat-client`          | `<client-tailscale-ip>` | iOS/Windows/macOS/Linux |

## Completed Connectivity Test

1. Install and connect Tailscale on the server device.
2. Install and connect Tailscale on the client device.
3. Confirmed both devices appeared in:

   ```bash
   tailscale status
   ```

4. Start a temporary HTTP server on the server device:

   ```bash
   python3 -m http.server 8787 --bind <server-tailscale-ip>
   ```

5. Open this URL from the client device:

   ```text
   http://<server-tailscale-ip>:8787
   ```

6. Confirm that the client device can reach the temporary server.

This proves that another device can reach a service running on the server through Tailscale.

## Planned First Rust Prototype

The first app version is intentionally small:

```text
GET  /          -> login page or chat UI
POST /login     -> username/password login
GET  /history   -> message history for authenticated users
GET  /ws        -> authenticated WebSocket connection
```

Current behavior:

- `GET /` serves a login page.
- `POST /login` validates a username and password.
- Successful login sets an in-memory session cookie and redirects to `/chat`.
- Failed login redirects to `/?error=1`.
- Five failed login attempts for the same username trigger a 60-second cooldown.
- `GET /chat` redirects unauthenticated users back to `/`.
- `GET /chat` shows the authenticated chat UI after login.
- `POST /logout` removes the in-memory session and expires the session cookie.
- `GET /ws` accepts authenticated WebSocket connections.
- The chat header includes a Settings menu with an optional best-effort logout when the tab closes.
- Privacy Mode can lock the chat when the tab or browser window loses focus and require the current user's password to reveal it.
- Messages sent over WebSocket are broadcast to all connected authenticated clients.
- Messages are stored locally in SQLite.
- New WebSocket clients receive recent message history after connecting.
- Message bodies are encrypted at rest before being written to SQLite.
- Messages can be deleted for one user or soft-deleted for everyone by their sender.
- Typing indicators are sent as transient WebSocket events and are not stored.
- Online status is based on active WebSocket connections and is not stored.

The app does not store passwords in Rust code. It reads Argon2id password hashes from `users.toml`, which is ignored by git.

To create a password hash:

```bash
cargo run --bin hash_password
```

Then put the generated hash into `users.toml`:

```toml
[[user]]
username = "alice"
password_hash = "$argon2id$..."

[[user]]
username = "bob"
password_hash = "$argon2id$..."
```

`users.example.toml` documents the file shape without storing real hashes.

Planned behavior:

- Two fixed users.
- Passwords verified using Argon2id hashes loaded from `users.toml`.
- Local session cookies after login.
- WebSocket broadcasts new messages to both connected users.
- Message history stored locally in `comm.sqlite3` by default.
- Message encryption key stored locally in `message.key` by default.
- `Delete for me` stores a row in `hidden_messages` and only affects that user's history/view.
- `Delete for everyone` sets `messages.deleted_at`, is only allowed for the sender, and removes the message for all clients.
- The green online dot turns on when the other user has at least one active chat tab connected.
- Presence changes are sent as transient WebSocket events. There is no polling and no database row for online status.
- Server binds to the configured Tailscale IP:

  ```text
  <server-tailscale-ip>:8787
  ```

## Security Notes

- Do not store plaintext passwords.
- Do not expose the Rust app directly to the public internet.
- Tailscale protects network transport between devices, but the app should still implement real authentication.
- Login attempts are rate-limited in memory; this resets when the server restarts.
- Privacy Mode hides message content in the browser UI and requires password verification before revealing it again, but it does not remove the authenticated session.
- Browser close-tab logout uses `sendBeacon` when available. It is useful for convenience, but it should not be treated as a guaranteed security boundary.
- Message bodies are encrypted with a local key before being stored in SQLite.
- If someone copies only `comm.sqlite3`, they should not be able to read message bodies.
- If someone copies both `comm.sqlite3` and `message.key`, they can decrypt message bodies.
- If browser HTTPS warnings become a problem, use Tailscale HTTPS certificates or a local reverse proxy.

## Next Step

Build the smallest Rust server that can:

1. Improve operational hardening before real use.
