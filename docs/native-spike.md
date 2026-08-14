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
| AX/UIA control view and native semantic invocation | Honest visual-fallback boundary only; native AX acceptance not signed                        | Pending              | Foreground-only adapter acceptance pending |
| Stale app/window/layout rejection                  | Fake-port trajectory implemented; native focus theft pending                                 | Pending              | Pending                                    |
| Physical takeover p95 under 250 ms                 | Injection lease and shortcut path implemented; general pointer monitor/signed timing pending | Pending              | Pending                                    |
| Course 5, 30 repetitions                           | Not run                                                                                      | Not run              | Not run                                    |

No release claim is made for native AX/UIA coverage or broad background control. The new runtime remains behind `RELIABLE_COMPUTER_USE_ENABLED`; enabling an invited pilot is blocked until signed hardware fills every required cell with build, OS, architecture, provider/model, fixture seed, and tester evidence.

Local automated evidence on 2026-08-14: the locked workspace tests and warnings-as-errors clippy gate passed on `aarch64-apple-darwin`; the offline evaluator passed 315/315 cases; and Tauri produced `target/debug/bundle/macos/Tro.app`. These results validate the contracts, fallback runtime, and local build, but not native AX traversal/invocation, provider quality, signed distribution, or the 30-run hardware reliability gate.

This file intentionally does not claim a cross-platform gate that has not run on signed hardware. Product release is blocked until every cell is signed by a tester with OS/build/hardware identifiers. If a pin fails, preserve the port and replace only the adapter with ScreenCaptureKit/Windows.Graphics.Capture or Quartz/SendInput. Windows elevated targets remain unsupported by design.
