# Tro

Tro is a Vietnamese-first desktop tutor for university students. Press a global shortcut, ask in Vietnamese or Vietnamese-English, and Tro can use an on-demand screenshot to explain the visible task, draw click-through guidance, preview dictation, or run a tightly guarded computer-use session.

The MVP targets macOS 14.2+ and Windows 11 x64. It uses Tauri 2, React, Rust, and an Axum provider broker. Screenshots, audio, transcripts, and dictated text are memory-only by default. Provider keys remain server-side.

## Development

Requirements: Node.js 22+, pnpm 10.23, Rust 1.95, and Docker for API integration tests.

1. Create a Google Cloud OAuth client with application type **Desktop app**, configure the consent screen/test users, then copy `.env.example` to `.env` and set `GOOGLE_OAUTH_CLIENT_ID`, the optional `GOOGLE_OAUTH_CLIENT_SECRET`, and the other server secrets. Google login uses a random IPv4 loopback callback and needs no fixed redirect URI in the desktop app.
2. Run `just bootstrap`.
3. Run `docker compose up -d postgres` and `sqlx migrate run --source services/api/migrations`.
4. Run `just dev` for the desktop app or `cargo run -p api` for the API.

Use `just check`, `just test`, `just build`, and `just eval-offline` before a pull request. Native input smoke tests are ignored unless run explicitly on a supervised fixture window.

## Safety boundary

Tro cannot automate passwords, OTP/MFA, payments, banking, government, medical or legal actions, permission/security changes, elevated applications, or proctored assessments. Submit, send, upload, delete, download, settings, unknown fields, and external navigation always require a one-action confirmation.

See `docs/architecture.md`, `docs/privacy.md`, `docs/native-spike.md`, and `docs/manual-qa.md`.
