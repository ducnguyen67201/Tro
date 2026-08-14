# Native capability spike

The stable port layer and first adapters are implemented with exact pins: xcap 0.9.8, cpal 0.18.1, enigo 0.6.1, and Tauri 2.11. The app builds capture bytes in memory, creates per-monitor click-through overlay windows, exposes push-to-talk capability, inserts Unicode through enigo, keeps the active login session only in process memory, registers global shortcuts, and makes emergency stop release common modifiers/buttons.

## Evidence status — 2026-08-13

| Gate                              | macOS ARM                                 | macOS Intel       | Windows 11 x64    |
| --------------------------------- | ----------------------------------------- | ----------------- | ----------------- |
| Dependency compile                | Passed on local Apple Silicon host        | CI/manual pending | CI/manual pending |
| On-demand capture, no temp file   | Manual pending                            | Manual pending    | Manual pending    |
| Transparent overlay click-through | Manual pending                            | Manual pending    | Manual pending    |
| cpal record/play loop             | Adapter capability only                   | Pending           | Pending           |
| Native Realtime Vietnamese turn   | Transport/provider live test pending      | Pending           | Pending           |
| Unicode fixture insertion         | Supervised smoke pending                  | Pending           | Pending           |
| Stop releases input under 250 ms  | Unit architecture present; timing pending | Pending           | Pending           |

## App-scoped computer-use evidence — 2026-08-14

| Gate                                               | macOS ARM                                                                                    | macOS Intel          | Windows 11 x64                             |
| -------------------------------------------------- | -------------------------------------------------------------------------------------------- | -------------------- | ------------------------------------------ |
| Stable app catalog and local approval              | Automated tests passed; debug app bundle built                                               | Build/manual pending | Build/manual pending                       |
| Exact-window capture and binding                   | xcap adapter implemented; signed-device correlation pending                                  | Pending              | Pending                                    |
| AX/UIA control view and native semantic invocation | Bounded AX adapter/native locator implemented and automated pure tests pass; signed supervised acceptance pending | Pending | Disabled until a separate UIA parity tranche |
| Stale app/window/layout rejection                  | Fake-port trajectory implemented; native focus theft pending                                 | Pending              | Pending                                    |
| Physical takeover p95 under 250 ms                 | Injection lease and shortcut path implemented; general pointer monitor/signed timing pending | Pending              | Pending                                    |
| Course 5, 30 repetitions                           | Not run                                                                                      | Not run              | Not run                                    |

No release claim is made for native AX/UIA coverage or broad background control. The new runtime remains behind `RELIABLE_COMPUTER_USE_ENABLED`; enabling an invited pilot is blocked until signed hardware fills every required cell with build, OS, architecture, provider/model, fixture seed, and tester evidence.

Current implementation evidence on 2026-08-14: the macOS code compiles with `axuielement` 0.9.1 (raw FFI feature disabled), and focused tests cover geometry, role/risk mapping, secure redaction behavior, policy escalation, ScaleCUA protocol/redirect handling, and the content-free telemetry allowlist. A supervised ignored fixture test exists for source binding, nonzero AX output, native focus/set-value/press, secure rejection, and stale zero-input. It has not been run in a signed isolated browser during this implementation, so native acceptance, provider quality, distribution, and every 30-run reliability cell remain blocked.

Serving evidence is also pending: vLLM 0.26.0 and the ScaleCUA checkpoint/revision are pinned as a canary candidate, while MLX-VLM 0.6.8 is documented only as an M4 Max 48 GB spike. No GPU run, conversion, live paid-provider call, commercial-license approval, or performance claim is recorded here.

This file intentionally does not claim a cross-platform gate that has not run on signed hardware. Product release is blocked until every cell is signed by a tester with OS/build/hardware identifiers. If a pin fails, preserve the port and replace only the adapter with ScreenCaptureKit/Windows.Graphics.Capture or Quartz/SendInput. Windows elevated targets remain unsupported by design.
