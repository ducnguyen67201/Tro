# Privacy and retention

Tro captures after a user shortcut/tray action or the next turn of an explicit agent session. It does not passively listen or continuously record. Overlay/status windows are hidden for capture. Captures are compressed in memory, bounded to the configured dimensions/bytes, stripped of metadata, and zeroized after use.

By default Tro stores no screenshots, audio, transcripts, prompts, dictated text, assistant responses, clipboard history, or conversation history. The Postgres schema deliberately has no content columns. Provider calls should use non-persistent operation where supported; release owners must re-verify current provider retention before every model change and disclose unavoidable retention in Vietnamese.

Essential records are limited to opaque identifiers, version/platform, timestamps, aggregate usage, stable error/action category, allow/confirm/block decision, request/run IDs, and revocation state. Google account linkage is stored only as a keyed hash of Google's stable subject identifier; Tro does not retain the Google email, profile, access token, or ID token. Optional content-free product telemetry is off until consent. Audit events expire on the operating schedule; active agent continuations expire within 30 minutes.

Device session tokens live only in process memory and are zeroized when replaced or dropped. Quitting Tro removes the local login session, so the user signs in with Google again on the next launch. Pilot support can revoke a device immediately.
