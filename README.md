# Tro

Tro is a Vietnamese-first desktop tutor for university students. Press a global shortcut, ask in Vietnamese or Vietnamese-English, and Tro can use an on-demand screenshot to explain the visible task, draw click-through guidance, preview dictation, or run a tightly guarded computer-use session.

The MVP targets macOS 14.2+ and Windows 11 x64. It uses Tauri 2, React, Rust, and an Axum provider broker. Screenshots, audio, transcripts, and dictated text are memory-only by default. Provider keys remain server-side.

## Development

Requirements: Node.js 22+, pnpm 10.23, Rust 1.95, and Docker for API integration tests.

1. Create a Google Cloud OAuth client with application type **Desktop app**, configure the consent screen/test users, then copy `.env.example` to `.env` and set `GOOGLE_OAUTH_CLIENT_ID`, the optional `GOOGLE_OAUTH_CLIENT_SECRET`, and the other server secrets. Google login uses a random IPv4 loopback callback and needs no fixed redirect URI in the desktop app.
2. Run `just bootstrap`.
3. Run `docker compose up -d postgres` and `sqlx migrate run --source services/api/migrations`.
4. Run `just dev` for the desktop app or `cargo run -p api` for the API.

Computer use is gated twice: set `AGENT_ENABLED=true` and `RELIABLE_COMPUTER_USE_ENABLED=true`, then select exactly one server-side adapter with `COMPUTER_PROVIDER=openai_responses`, `openrouter_chat`, or `scale_cua`. Provider keys remain in the API environment; the desktop stores only its revocable device token and optional local app approvals. There is no automatic provider fallback, so a screenshot is never silently resent to another recipient.

The initial production-quality candidate is direct OpenAI Responses with `gpt-5.6`. OpenRouter is an explicit experiment route. ScaleCUA is the self-hosted model/checkpoint and vLLM is the recommended Linux/NVIDIA server for its research canary; MLX-VLM is a Mac development spike only. vLLM/MLX-VLM are not model names and ScaleCUA is not served through OpenRouter.

Use `just check`, `just test`, `just build`, and `just eval-offline` before a pull request. Native input smoke tests are ignored unless run explicitly on a supervised fixture window.

## Safety boundary

Tro cannot automate passwords, OTP/MFA, payments, banking, government, medical or legal actions, permission/security changes, elevated applications, proctored assessments, or permanent deletion. Submit, send, upload, download, settings, unknown fields, and external navigation require a one-action confirmation. Every action is bound to a locally approved app, exact observation, and window generation.

Tro is intentionally bounded computer use, not an “every task” autonomous operator. On macOS it prefers native Accessibility elements and re-resolves an ephemeral PID/window/path locator immediately before an action. When semantic access is unavailable, it says so and visual clicks remain confirmation-gated. Windows computer use stays disabled/incomplete until UIA reaches the same gates.

See `docs/architecture.md`, `docs/privacy.md`, `docs/native-spike.md`, and `docs/manual-qa.md`.
