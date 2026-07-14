# minos

I built this because I was sick of WordPress. I wanted something that is actually fast and secure.

This is a custom CMS designed for performance. It uses a Rust backend with Axum and SQLx for type-safe reliability, and a React frontend with Tailwind for a clean, responsive UI.

No bloated plugins. No security nightmares. Just raw speed and control.

You get a full admin dashboard to manage tutorials, pages, and posts. It runs on SQLite so deployment is simple and backups are just file copies.

If you care about performance and want a CMS that doesn't get hacked every other week, this is for you.

## Tech Stack
*   **Backend:** Rust, Axum, SQLx
*   **Frontend:** React 18, Vite, TailwindCSS
*   **Database:** SQLite

## Quick Start
1.  Clone the repo.
2.  `cd backend` and `cargo run`.
3.  `npm install` and `npm run dev`.
4.  Go to `localhost:5173`.

Login with the credentials in your `.env` file.

## Deployment behind a reverse proxy

When the backend runs behind a reverse proxy (nginx, Caddy, the bundled
Docker Compose setup, ...), you **must** set `TRUST_PROXY_IP_HEADERS=true`.
Otherwise the backend strips the `X-Forwarded-*` headers and sees every
visitor with the proxy's IP address — which means IP-based rate limits
(login throttling, guest comment limits) apply globally to all visitors
instead of per client.

Only set this when the proxy is trusted and overwrites the forwarding
headers itself; with a direct internet-facing deployment leave it `false`
so clients cannot spoof their IP. See `.env.example` for the related
`ENABLE_HSTS` and `AUTH_COOKIE_SECURE` settings.
