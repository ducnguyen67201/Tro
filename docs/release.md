# Release and pilot runbook

V1 distributes a signed/notarized direct DMG for macOS (transparent private API rules out the App Store) and signed MSI/NSIS for Windows x64. Signing and updater private keys exist only in protected CI environments and never print to logs.

Before release: run all static checks, tests, offline evals, provider canaries, dependency/license audits, secret scans, SBOM generation, and the manual matrix. Verify model IDs and current provider retention. Sign native-spike evidence. Test clean install, uninstall/data removal, update, rollback, token revoke, and both server kill switches.

The first cohort is at most 20 invited Vietnamese university students aged 18+. Record opt-in consent, support contact, revocation path, and a daily safety review. Do not recruit minors or collect real screens/voices without separate explicit consent and retention documentation.

Rollback never silently installs a known vulnerable version. During an incident set `AGENT_ENABLED=false` or `REALTIME_ENABLED=false`, revoke affected device tokens, preserve content-free audit evidence, notify pilot members in Vietnamese, and rotate provider/signing credentials only through protected operations.
