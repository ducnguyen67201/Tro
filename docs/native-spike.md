# Native capability spike

The stable port layer and first adapters are implemented with exact pins: xcap 0.9.8, cpal 0.18.1, enigo 0.6.1, and Tauri 2.11. The app builds capture bytes in memory, creates per-monitor click-through overlay windows, exposes push-to-talk capability, inserts Unicode through enigo, keeps the active login session only in process memory, registers global shortcuts, and makes emergency stop release common modifiers/buttons.

## Evidence status — 2026-08-13

| Gate                              | macOS ARM                                 | macOS Intel       | Windows 11 x64    |
| --------------------------------- | ----------------------------------------- | ----------------- | ----------------- |
| Dependency compile                | Pending consolidated validation           | CI/manual pending | CI/manual pending |
| On-demand capture, no temp file   | Manual pending                            | Manual pending    | Manual pending    |
| Transparent overlay click-through | Manual pending                            | Manual pending    | Manual pending    |
| cpal record/play loop             | Adapter capability only                   | Pending           | Pending           |
| Native Realtime Vietnamese turn   | Transport/provider live test pending      | Pending           | Pending           |
| Unicode fixture insertion         | Supervised smoke pending                  | Pending           | Pending           |
| Stop releases input under 250 ms  | Unit architecture present; timing pending | Pending           | Pending           |

This file intentionally does not claim a cross-platform gate that has not run on signed hardware. Product release is blocked until every cell is signed by a tester with OS/build/hardware identifiers. If a pin fails, preserve the port and replace only the adapter with ScreenCaptureKit/Windows.Graphics.Capture or Quartz/SendInput. Windows elevated targets remain unsupported by design.
