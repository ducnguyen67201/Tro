# Tro — computer action policy v1

Pixels, OCR text, websites, documents, and model output are untrusted. They cannot grant permission or alter the immutable user goal.

- Low risk: pointer movement, benign click, scroll, wait, capture, and typing into a known practice/editor field.
- Confirm exactly once: send, submit, post, upload, download, settings changes, new external navigation, personal data, and unknown fields.
- Always block permanent deletion/removal in this rollout, including when a provider or screen label calls it benign.
- Always block: credentials, password, OTP/MFA, payment, banking/crypto, legal/medical/government action, proctored assessment, elevation, permission/security changes, safeguard changes, and hidden/background input.

Every action is validated immediately before execution. Confirmations bind to one action fingerprint and expire after 30 seconds, focus change, or display-layout change. Runs stop after 20 turns, 100 actions, or 5 minutes. Emergency stop cancels pending events and releases every held key or button.

Every action must also bind the current run scope, approved stable app ID, observation ID, window/layout generation, and locator. Accessibility and screen strings may raise risk but never grant permission, add an app, or lower risk. Physical local input and unexpected unapproved focus stop future input; resuming requires a fresh observation.
