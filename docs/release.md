# Release and pilot runbook

V1 distributes a signed/notarized direct DMG for macOS (transparent private API rules out the App Store) and signed MSI/NSIS for Windows x64. Signing and updater private keys exist only in protected CI environments and never print to logs.

Before release: run all static checks, tests, offline evals, explicitly budgeted provider canaries, dependency/license audits, secret scans, SBOM generation, and the manual matrix. Verify model IDs and current provider retention from official provider documentation. Sign native-spike evidence. Test clean install, uninstall/data removal, update, rollback, token revoke, and every server kill switch.

Reliable computer use requires both `AGENT_ENABLED=true` and `RELIABLE_COMPUTER_USE_ENABLED=true`. Promote one explicit `COMPUTER_PROVIDER=openai_responses|openrouter_chat|scale_cua` and model only after both course and messaging fixtures reach 29/30 on every supported platform, unexpected-state recovery reaches 95% within two replans, takeover p95 is below 250 ms, Send/Enter confirmation reaches 100%, and wrong-app/stale/delete/prohibited actions remain zero. Provider fallback is disabled; changing provider, model/alias, serving runtime, revision, quantization, OS build, or architecture requires a new retention review and canary.

`openai_responses` uses real `previous_response_id` continuation and therefore sends `store=true`; the release disclosure must state OpenAI's current response-retention period. Do not enable it for a cohort whose policy requires stateless provider calls. The OpenRouter adapter keeps only bounded, content-free action summaries in Tro's encrypted continuation and disables provider fallback, but its selected model and routing retention still require separate verification.

The first cohort is at most 20 invited Vietnamese university students aged 18+. Record opt-in consent, support contact, revocation path, and a daily safety review. Do not recruit minors or collect real screens/voices without separate explicit consent and retention documentation.

## Provider promotion matrix

| Candidate                          | Intended stage                       | Required evidence before promotion                                                                                                        |
| ---------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- |
| `openai_responses` + `gpt-5.6`     | Initial production-quality candidate | A/B low/medium/high reasoning, actual configured alias recorded, retention disclosure, protocol tests, signed macOS fixture 30-run matrix |
| `openrouter_chat` + explicit model | Experiment                           | Exact route/model/retention recorded, router fallback disabled, independent fixture matrix                                                |
| `scale_cua` + vLLM                 | Self-hosted research/canary          | Pinned image/model/runtime tuple, protocol and GPU fixture matrix, latency/memory, signed commercial/license decision                     |
| `scale_cua` + MLX-VLM              | Local development only               | Conversion/tool/Vietnamese/20-turn memory spike plus independent quantization matrix; not promotable by “it loads”                        |

Promotion order is automated tests → isolated signed macOS AX fixture → internal GPT-5.6 canary → self-hosted ScaleCUA protocol/fixture canary → license review → invited ScaleCUA canary. Record time to first valid action, turns, confirmations, completion time, rejection rate, semantic-action rate, visual-fallback rate, provider latency/cost, and zero-content log review separately for every tuple. Windows remains disabled until its UIA tranche meets the same matrix.

Rollback never silently installs a known vulnerable version. During an incident set `RELIABLE_COMPUTER_USE_ENABLED=false` first (or `AGENT_ENABLED=false` for the full agent), stop active runs and release input, then set `REALTIME_ENABLED=false` if needed. Revoke affected device tokens, preserve content-free audit evidence, notify pilot members in Vietnamese, and rotate provider/signing credentials only through protected operations.
