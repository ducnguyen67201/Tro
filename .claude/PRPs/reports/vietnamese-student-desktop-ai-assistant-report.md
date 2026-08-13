# Implementation Report: Vietnamese-First Student Desktop AI Assistant

## Summary

Implemented a greenfield Tauri 2, React/TypeScript, Rust, and Axum foundation for **Tro**, a Vietnamese-first desktop learning assistant for university students aged 18+. The repository now contains the desktop shell and Vietnamese UI, shared safety contracts and state machines, a server-side OpenAI broker, invite/device APIs, privacy and release documentation, 300 evaluation fixtures, CI workflows, automated tests, and a locally built Apple Silicon debug application and DMG.

The implementation is a **pre-pilot foundation**, not a production-complete HeyClicky-equivalent. Live Realtime audio streaming, end-to-end desktop agent execution, native permission adapters, provider-backed quality evaluations, signed Windows/macOS packages, and hardware-matrix validation remain release blockers.

## Assessment vs. Reality

| Metric           |                               Predicted in plan |                                                                                           Actual |
| ---------------- | ----------------------------------------------: | -----------------------------------------------------------------------------------------------: |
| Complexity       |                                              XL |                                                                                               XL |
| Files            |                                             ~92 |                 219 repository files after report/archive, including generated Tauri icon assets |
| Tasks            |                                              18 |                                                                           6 complete, 12 partial |
| Evaluation cases |                 300 minimum across five corpora |                                                                       300/300 structurally valid |
| Target platforms | macOS 14.2+ Apple Silicon/Intel; Windows 11 x64 | Apple Silicon debug bundle built locally; Intel macOS and Windows require CI/hardware validation |

The plan correctly identified this as an XL project. The native/provider integration and distribution work cannot be honestly completed without platform hardware, credentials, signing identities, and supervised manual testing.

## Task Status

| Task                                   | Status   | Result                                                                                                                                                                                                              |
| -------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1. Reproducible monorepo               | Complete | Pinned Rust and pnpm workspaces, lockfiles, formatting/lint/typecheck/test/build commands, Docker Compose, and environment template.                                                                                |
| 2. Native capability slice             | Partial  | Tauri plugins and Rust adapters compile; local macOS capture/audio/input capability code exists. Live streaming, Intel macOS, Windows, mixed-DPI, and hardware tests remain.                                        |
| 3. Canonical contracts/state machines  | Complete | Typed coordinate, frame, action, confirmation, assistant, agent, error, permission, and API contracts with invariant tests.                                                                                         |
| 4. API/database/device auth            | Partial  | Axum service, Postgres migration, invite/device routes, HMAC device tokens, health endpoint, body limit, and migration smoke test implemented. Production-grade rate limiting and deployment hardening remain.      |
| 5. Realtime credential broker          | Complete | Authenticated client-secret endpoint, server-held provider key, expiry validation, and 60-second token-rotation overlap implemented. Live client session orchestration remains in Task 10.                          |
| 6. Computer-use API loop               | Partial  | Agent create/turn/stop APIs, provider adapter, encrypted continuation IDs, usage limits, kill switches, and typed action proposals implemented. Provider-backed loop integration is not certified.                  |
| 7. Tauri shell/topology/tray/shortcuts | Complete | Desktop shell, tray, global shortcut, assistant window, per-monitor overlay creation, click-through overlays, and emergency shortcut registration implemented.                                                      |
| 8. Permission onboarding/diagnostics   | Partial  | Vietnamese permission UI and typed permission states exist; OS-native request/status/open-settings adapters are placeholders.                                                                                       |
| 9. Capture/coordinates                 | Partial  | In-memory frame types, xcap capture, redaction/zeroization, normalized coordinates, and mapping tests exist. Active-foreground selection, downscaling, mixed-DPI topology, and real focus context remain.           |
| 10. Audio/Realtime orchestration       | Partial  | Native audio capability checks and transport boundary exist. Bidirectional WebSocket/PCM streaming, reconnection, barge-in, and session lifecycle are not implemented.                                              |
| 11. Vietnamese tutor/model tools       | Complete | Versioned Vietnamese tutor and agent-policy prompts, tutoring/academic-integrity policy, prompt assembly, and tests implemented.                                                                                    |
| 12. Accessible visual guidance         | Complete | Non-interactive overlay renderer, normalized target mapping, labels, SVG guidance, screen-reader text, and renderer tests implemented. Hardware multi-monitor QA remains.                                           |
| 13. Vietnamese-first dictation         | Partial  | NFC normalization and secure-field safety helpers exist. Focused-app insertion, clipboard fallback/restoration, and production dictation wiring remain.                                                             |
| 14. Guarded execution/emergency stop   | Partial  | Exhaustive risk classifier, confirmation policy, cancellable input ports, state machine, and emergency-stop path exist. It is not wired into an end-to-end API-to-native execution loop.                            |
| 15. React product states/settings      | Complete | Vietnamese-first onboarding, assistant bar, transcript, agent status, confirmation, settings, privacy, English fallback strings, responsive styling, and UI tests implemented.                                      |
| 16. Observability/budgets/recovery     | Partial  | Server-side allowlisted telemetry, content rejection, encrypted continuation storage, budgets, and kill switches exist. Full client telemetry/crash cleanup and incident flow remain.                               |
| 17. Evaluation/test matrix             | Partial  | Five deterministic corpora with 300 cases, runner, automated tests, and CI jobs exist. The `live` runner is intentionally fixture-only; provider-quality and supervised native matrices remain.                     |
| 18. Package/sign/update/pilot gate     | Partial  | Release workflow, updater configuration boundary, operational docs, and local debug `.app`/`.dmg` build exist. Signing, notarization, NSIS, protected updater artifacts, rollback drill, and pilot approval remain. |

## Validation Results

### Passed

- `pnpm install --frozen-lockfile`
- `cargo fetch --locked`
- `pnpm format:check`
- `pnpm lint`
- `pnpm typecheck`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `pnpm test --run`
- `cargo test --workspace --locked` (supervised native smoke test intentionally ignored)
- `cargo build --workspace --locked`
- `pnpm --filter @tro/desktop build`
- `cargo run -p eval-runner -- --offline` — 300/300 cases
- Budget-gated fixture mode — 300/300 cases; explicitly reports `live-fixture-only`, not a provider-quality test
- Axum/Postgres Docker smoke test — migration applied and `/healthz` returned `{"status":"ok"}`
- `pnpm audit --audit-level high` — no known vulnerabilities
- `cargo audit` — no reachable denied advisories; documented ignore for an uncompiled SQLx optional RSA dependency
- `cargo deny check --hide-inclusion-graph` — advisories, licenses, bans, and sources passed; duplicate-version warnings remain informational
- `pnpm --filter @tro/desktop tauri build --debug` — local Apple Silicon `.app` and `.dmg` produced

### Not Yet Certifiable

- Real Vietnamese/English voice latency, barge-in, session recovery, and provider response quality
- End-to-end screenshot-to-overlay and proposed-action-to-native-execution flows
- Secure-field detection and clipboard restoration in real target applications
- macOS Intel and Windows 11 behavior, mixed-DPI multi-monitor mapping, permissions, and emergency stop
- Signed/notarized macOS DMG, signed Windows NSIS, updater signatures, rollback, and clean-machine installation
- Accessibility audit with VoiceOver/Narrator and supervised student pilot

## Major Files and Components

| Area           | Paths                                                                                              | Action                                                                                                                       |
| -------------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Workspace      | `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, lockfiles, `justfile`, toolchain/config files | Created reproducible Rust/TypeScript monorepo and root gates.                                                                |
| Contracts      | `crates/contracts/`                                                                                | Added cross-boundary types, validation, state machines, risk policy, redacted frames, and tests.                             |
| Tutor core     | `crates/tutor-core/`, `prompts/`                                                                   | Added Vietnamese tutoring, academic-integrity behavior, and versioned prompts.                                               |
| API            | `services/api/`                                                                                    | Added Axum routes, Postgres migration, auth, provider broker/agent boundaries, budgets, encryption, and telemetry filtering. |
| Desktop native | `apps/desktop/src-tauri/`                                                                          | Added Tauri shell, tray/hotkey/windows, overlays, capture/audio/input/vault adapters, policy engine, and commands.           |
| Desktop UI     | `apps/desktop/src/`                                                                                | Added Vietnamese-first product screens, overlay/confirmation components, styling, localization, and tests.                   |
| Evaluations    | `evals/`, `tools/eval-runner/`                                                                     | Added 300 deterministic cases and offline/budget-gated runner.                                                               |
| Operations     | `.github/workflows/`, `docs/`, `SECURITY.md`, `.env.example`                                       | Added CI/release scaffolding, privacy architecture, QA, release, and security runbooks.                                      |

## Deviations from Plan

1. **No Git repository or branch was created.** The target directory was empty and the plan explicitly required Git initialization only when requested.
2. **No proprietary HeyClicky assets or implementation details were copied.** Tro follows the public interaction model while using its own Vietnamese-first visual and product implementation.
3. **Realtime remains a port, not a working stream.** A fake implementation would obscure the actual engineering risk; the repository exposes the native/API boundaries needed for a real WebSocket and PCM pipeline.
4. **The agent executor is safety-complete at the policy layer but not integration-complete.** Risk classification and cancellation are testable; live action execution is deliberately not claimed without end-to-end confirmation and hardware QA.
5. **The live evaluation command is explicitly labeled fixture-only.** It verifies opt-in and budget controls but does not pretend to measure provider output without credentials and a scoring harness.
6. **Dependency remediation updated transitive versions.** Tauri-compatible resolver updates removed reachable high-severity advisories. The remaining RSA advisory appears only in SQLx's optional MySQL lockfile graph and is documented as an audit ignore because it is not compiled for this PostgreSQL service.

## Issues Encountered

- Empty target required a full repository scaffold rather than incremental changes.
- Tauri's generated platform icons increased the file count beyond the plan estimate.
- SQLx's lockfile includes optional MySQL/RSA packages even when only PostgreSQL is enabled. Target-aware dependency inspection showed the vulnerable RSA crate is not in the compiled graph.
- Signing, notarization, Windows packaging, and provider-backed validation require external secrets and platform-specific infrastructure that were not available locally.

## Tests Added

- Contract validation and serialization round trips
- Assistant and agent state transitions
- Risk policy and confirmation decisions
- Coordinate normalization and mapping
- Vietnamese tutor and academic-integrity policies
- Dictation NFC/secure-field safety helper
- API health, device authentication, agent state, and telemetry filtering
- React assistant-bar and confirmation interaction tests
- Ignored supervised native smoke test for hardware-only execution
- Five evaluation corpora totaling 300 cases

## Artifacts

- Debug macOS application: `target/debug/bundle/macos/Tro.app`
- Debug Apple Silicon DMG: `target/debug/bundle/dmg/Tro_0.1.0_aarch64.dmg`
- Evaluation reports: `target/eval-reports/latest.json` and `target/eval-reports/latest.md`

## Recommended Next Steps

1. Implement native Realtime WebSocket/PCM streaming with explicit reconnect, expiry, and barge-in tests.
2. Wire onboarding to invite/device registration and implement real macOS/Windows permission adapters.
3. Connect the desktop agent coordinator to the API, overlay targeting, confirmation dialog, cancellable input executor, and emergency stop.
4. Complete foreground-window capture, multi-monitor/mixed-DPI mapping, Vietnamese dictation insertion, and clipboard restoration.
5. Replace fixture-only live evaluation with provider calls, deterministic scoring, latency/cost capture, and Vietnamese voice test recordings.
6. Run the full macOS Apple Silicon/Intel and Windows 11 manual matrix, accessibility audit, signed packaging, updater rollback drill, and security/privacy review before inviting students.

## Completion Status

**Partial — pre-pilot foundation implemented and automated local gates passed. Production/pilot release remains blocked on the integration and platform gates listed above.**
