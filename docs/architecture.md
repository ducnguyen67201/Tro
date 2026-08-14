# Architecture

Tro is a ports-and-adapters desktop product. React renders sanitized state; Rust owns assistant and agent state, permissions, native media, overlays, policy, cancellation, and input. The Axum broker protects provider credentials, verifies Google identities, issues revocable device sessions, applies usage limits, and proposes—but never executes—computer actions.

## Trust boundaries

- Webview input is untrusted and enters Rust through narrow Tauri commands with length, enum, state, and origin checks. No shell or broad filesystem capability is exposed.
- Network requests use a revocable opaque device token. The provider API key exists only in the API process.
- Google sign-in uses the system browser, an ephemeral IPv4 loopback listener, OAuth authorization code flow, PKCE, state, and nonce. The API validates the signed OpenID Connect token and discards Google tokens after exchanging them for a Tro device token.
- Every pixel and every model action is untrusted. Screen content cannot alter consent, goal, policy, limits, or confirmation.
- Logs and telemetry accept stable metadata only. Raw media, text, coordinates paired with window identity, secrets, and provider bodies are forbidden.

## Desktop flow

`Google system-browser login → loopback callback → API OIDC verification → opaque device token in the OS credential vault`.

`Global shortcut → Assistant state → hide overlays → in-memory capture → cpal audio → Realtime adapter → sanitized transcript / overlay event`.

Agent mode is separate: `explicit goal → capture → API action proposal → schema validation → ActionPolicy → allow / one-action confirmation / block → serialized InputBackend → new observation`. Emergency stop cancels the token, releases held inputs, clears overlays and confirmations, and never resumes after restart.

Reliable computer use adds an app-scoped state machine before that input boundary: `resolve stable local app identity → local app approval → activate/verify window → exact-window observation → one observation-bound proposal → local evidence/policy → execute → adaptive stabilization → fresh observation`. Every proposal carries an `observation_id` and an application, element, or frame locator. Element IDs are ephemeral and are discarded with their observation. The API can propose actions and retain an encrypted provider continuation, but it cannot approve apps, resolve native handles, lower local risk, or execute input.

`ApplicationBackend`, `ObservationBackend`, `ActionExecutor`, `UserActivityBackend`, and `ComputerUseBackend` are injected ports. Exact-window pixels and bounded accessibility content remain in memory only. When native semantic access is absent, the observation declares the degradation and coordinate fallback receives conservative local policy; it never silently becomes unrestricted global input.

## Conventions

Rust libraries return typed errors and adapters do one capability. API code follows route → service → repository. Coordinators own cancellation and state. Provider types do not cross adapters. JSON and stable codes use `snake_case`; Rust/React types use `PascalCase`. Runtime paths do not use `unwrap`, `expect`, or `panic`. Test-only static invariants may use clear `expect` messages.

Normalized screen coordinates are inclusive `0..=1`, finite, and converted to physical desktop pixels only by `CoordinateMapper`. Screen buffers and secret text redact debug output and zeroize on drop.
