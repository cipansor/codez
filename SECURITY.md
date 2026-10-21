# Security Policy

## Supported versions

Codez is under active development. Security fixes are applied to the `main` branch;
there are no maintained release branches yet.

## Reporting a vulnerability

Please **do not** open a public issue for security problems. Instead, report privately
using GitHub's [private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability)
("Report a vulnerability" under the repository's **Security** tab).

Include as much of the following as you can:

- A description of the issue and its impact
- Steps to reproduce, or a proof-of-concept
- The affected route, handler, or component
- Any suggested remediation

You can expect an acknowledgement within a few days. We will keep you updated as we
investigate and coordinate a fix and disclosure timeline with you.

## Scope notes

Codez is a demo-oriented platform: storage is in-memory by default, and there is no
authentication middleware yet, so several endpoints trust the caller's identity. Please
focus reports on issues that are independent of the missing auth layer, such as:

- Request handling in the Axum router (`crates/backend/src/routes/`)
- Injection, path traversal, or unsafe deserialization in handlers
- SSRF protections around repository webhooks
- Anything that lets one request corrupt shared state for others

Known limitation (not a vulnerability to report): handlers read the acting user from the
request rather than from a verified session, because auth middleware does not exist yet.