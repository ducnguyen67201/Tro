# Security operations

## Incident response

1. Disable the affected server feature before investigation.
2. Revoke affected device tokens and invalidate active agent continuations.
3. Preserve content-free request IDs, versions, timestamps, stable reason codes, and decision categories; do not request raw student content.
4. Patch, rerun adversarial policy and native smoke gates, then stage a signed update.
5. Notify the invite cohort with impact, recovery, removal, and support instructions.

## Device revocation

Set the device status to `revoked`, revoke all device token rows, mark active agent runs stopped, and reject new secret/turn issuance immediately. The desktop returns to idle and asks for a new invite. Revocation cannot be reversed by refresh-token overlap.

## Model change control

Provider model and voice IDs are configuration only. Every change requires protocol fixtures, Vietnamese quality/adversarial evals, a budget-capped live canary, retention review, and rollback configuration.

## Dependency advisory exceptions

`RUSTSEC-2023-0071` is temporarily ignored for SQLx 0.8's optional MySQL RSA dependency. Tro enables only Postgres; `cargo tree --target all -i rsa` returns no path, so the affected cryptography is not compiled into API or desktop artifacts. Remove the exception when SQLx stops locking the optional driver or a fixed RSA release is available. No reachable high-severity advisory is accepted.
