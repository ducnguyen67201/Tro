# Release and pilot runbook

V1 distributes a signed/notarized direct DMG for macOS (transparent private API rules out the App Store) and signed MSI/NSIS for Windows x64. Signing and updater private keys exist only in protected CI environments and never print to logs.

Before release: run all static checks, tests, offline evals, explicitly budgeted provider canaries, dependency/license audits, secret scans, SBOM generation, and the manual matrix. Verify model IDs and current provider retention from official provider documentation. Sign native-spike evidence. Test clean install, uninstall/data removal, update, rollback, token revoke, and every server kill switch.

Reliable computer use requires both `AGENT_ENABLED=true` and `RELIABLE_COMPUTER_USE_ENABLED=true`. Promote one explicit `COMPUTER_PROVIDER=openai_responses|openrouter_chat` and model only after the course fixture reaches 29/30 on every supported platform, unexpected-state recovery reaches 95% within two replans, takeover p95 is below 250 ms, and wrong-app/stale/delete/prohibited actions remain zero. Provider fallback is disabled; changing provider or model requires a new retention review and canary.

`openai_responses` uses real `previous_response_id` continuation and therefore sends `store=true`; the release disclosure must state OpenAI's current response-retention period. Do not enable it for a cohort whose policy requires stateless provider calls. The OpenRouter adapter keeps only bounded, content-free action summaries in Tro's encrypted continuation and disables provider fallback, but its selected model and routing retention still require separate verification.

The first cohort is at most 20 invited Vietnamese university students aged 18+. Record opt-in consent, support contact, revocation path, and a daily safety review. Do not recruit minors or collect real screens/voices without separate explicit consent and retention documentation.

Rollback never silently installs a known vulnerable version. During an incident set `RELIABLE_COMPUTER_USE_ENABLED=false` first (or `AGENT_ENABLED=false` for the full agent), stop active runs and release input, then set `REALTIME_ENABLED=false` if needed. Revoke affected device tokens, preserve content-free audit evidence, notify pilot members in Vietnamese, and rotate provider/signing credentials only through protected operations.
