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
- `GET /chat` redirects unauthenticated users back to `/`.
- `GET /chat` shows a basic authenticated placeholder after login.

Temporary login credentials:

| Username | Password |
| --- | --- |
| `daniel` | `change-me-daniel` |
| `friend` | `change-me-friend` |

These credentials are temporary scaffolding and must be replaced with Argon2id password hashes before real use.

Planned behavior:

- Two fixed users.
- Passwords verified using Argon2id hashes.
- Local session cookies after login.
- WebSocket broadcasts new messages to both connected users.
- Message history stored locally.
- Server binds to the MacBook Tailscale IP:

  ```text
  100.124.77.92:8787
  ```

## Security Notes

- Do not store plaintext passwords.
- Do not expose the Rust app directly to the public internet.
- Tailscale protects network transport between devices, but the app should still implement real authentication.
- Login should be rate-limited before use outside local testing.
- Messages should be encrypted at rest after the basic message flow works.
- If browser HTTPS warnings become a problem, use Tailscale HTTPS certificates or a local reverse proxy.

## Next Step

Build the smallest Rust server that can:

1. Replace temporary plaintext credentials with Argon2id password hashes.
2. Keep a local authenticated session.
3. Serve a basic chat page.
4. Send and receive messages over WebSocket.
5. Store and load message history locally.
