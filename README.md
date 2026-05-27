# Private Two-Party Chat

This project is intended to become a very small Rust chat app for two users. The server runs on main user's laptop, stores the message history locally, and lets both parties send and receive messages after logging in.

## Goal

- Run the server on main user's MacBook.
- Allow exactly two users to log in with usernames and passwords.
- Let both users exchange messages in real time.
- Store message history only on main user's computer.
- Store passwords as hashes, not plaintext.
- Store messages encrypted at rest in a later step.
- Avoid exposing the app directly to the public internet.

## Connection Approach

The first networking approach is Tailscale.

Tailscale creates a private network between trusted devices. This means the Rust app can listen on main user's laptop and be reached from another trusted device without opening a public router port.

Current tested devices:

| Device                 | Tailscale IP | OS |
|------------------------| --- | --- |
| `mainuser-macbook-pro` | `100.124.77.92` | macOS |
| `iphone-15`            | `100.107.209.117` | iOS |

## Completed Connectivity Test

1. Installed and connected Tailscale on main user's MacBook.
2. Installed and connected Tailscale on the iPhone.
3. Confirmed both devices appeared in:

   ```bash
   tailscale status
   ```

4. Started a temporary HTTP server on the MacBook:

   ```bash
   python3 -m http.server 8787 --bind 100.124.77.92
   ```

5. Opened this URL on the iPhone:

   ```text
   http://100.124.77.92:8787
   ```

6. Confirmed the iPhone could see the `/Users/mainuser/User/comm` directory listing.

This proves that another device can reach a service running on the laptop through Tailscale.

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
- Messages sent over WebSocket are broadcast to all connected authenticated clients.
- Messages are stored locally in SQLite.
- New WebSocket clients receive recent message history after connecting.
- Message bodies are encrypted at rest before being written to SQLite.
- Messages can be soft-deleted; deleted messages are removed from live clients and excluded from future history.

Local test login credentials:

| Username | Password |
|----------|----------|
| `u1`     | `u1p`    |
| `u2`     | `u2p`    |

The app does not store these passwords in Rust code. It reads Argon2id password hashes from `users.toml`, which is ignored by git.

To create a password hash:

```bash
cargo run --bin hash_password
```

Then put the generated hash into `users.toml`:

```toml
[[user]]
username = "u1"
password_hash = "$argon2id$..."

[[user]]
username = "u2"
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
- Deleted messages keep a row with `deleted_at` set, but are no longer returned to clients.
- Server binds to the MacBook Tailscale IP:

  ```text
  100.124.77.92:8787
  ```

## Security Notes

- Do not store plaintext passwords.
- Do not expose the Rust app directly to the public internet.
- Tailscale protects network transport between devices, but the app should still implement real authentication.
- Login attempts are rate-limited in memory; this resets when the server restarts.
- Message bodies are encrypted with a local key before being stored in SQLite.
- If someone copies only `comm.sqlite3`, they should not be able to read message bodies.
- If someone copies both `comm.sqlite3` and `message.key`, they can decrypt message bodies.
- If browser HTTPS warnings become a problem, use Tailscale HTTPS certificates or a local reverse proxy.

## Next Step

Build the smallest Rust server that can:

1. Improve operational hardening before real use.
