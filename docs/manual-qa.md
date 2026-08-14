# Manual QA

Run on macOS 14.2+ ARM and Intel plus Windows 11 x64. Use a dedicated fixture application; never point native input smoke tests at personal applications.

- Clean install opens in Vietnamese, launches Google sign-in in the system browser, and returns to Tro after account selection. Quitting and reopening Tro requires Google sign-in again and never opens a Keychain or Credential Manager prompt.
- Cancel Google consent, let the login expire, send a callback with the wrong state, and occupy the first loopback port. Tro must fail safely, never accept a mismatched callback, and allow a clean retry.
- Deny, grant, and revoke microphone, screen, and delayed input/accessibility permissions. Tro stays useful and explains recovery without repeated prompts.
- Exercise silence, long speech, Vietnamese without diacritics, Northern/Central/Southern accents, and Vietnamese-English code-switching.
- Test single monitor and mixed-DPI dual monitors including negative origin, fullscreen, Spaces/virtual desktops, monitor unplug, and layout change mid-turn.
- Verify overlays pass all mouse/keyboard input, never steal focus, clear on state exit, and do not appear in captured frames.
- Insert the consented Vietnamese text fixture into TextEdit/Notepad, browser, Office, Electron, and the Tro fixture. Block secure/unknown fields and preserve preview on failure. Clipboard fallback must be explicit.
- Start agent only from explicit intent. Confirm send/delete/upload/download/settings/external navigation/personal data exactly once. Block credentials, OTP, payments, banking, legal, medical, government, permission/security, elevation, and proctored exams.
- Trigger emergency stop during capture, provider wait, click, drag, key chord, typing, and wait. Verify release within 250 ms and no automatic resume after crash/update/restart.
- Inspect app data, DB, logs, telemetry, crash reports, and provider dashboard for prohibited raw content.
