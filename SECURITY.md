# Security policy

Report vulnerabilities privately to the project maintainers; do not include real screenshots, recordings, transcripts, tokens, or student data. The pilot does not offer a public bug bounty.

Never commit provider keys, invite codes, device tokens, signing material, student content, or raw media. The desktop stores its revocable device token in the operating-system credential vault. All model-produced computer actions are untrusted and pass a fail-closed policy immediately before execution.

Supported security updates cover the current pilot release on macOS 14.2+ and Windows 11 x64. The server-side `AGENT_ENABLED` and `REALTIME_ENABLED` switches are incident controls.
