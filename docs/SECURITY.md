# Security Policy

## Supported Versions

| Version | Status |
|---|---|
| `main` branch | Actively maintained |
| Tagged releases | Bug fixes only (critical severity) |

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

Email **security@stellarforge.io** with the following:

1. **Description:** A clear description of the vulnerability, including affected components.
2. **Reproduction steps:** Minimal steps to reproduce, including any relevant contract addresses or transaction IDs.
3. **Impact assessment:** Your estimate of the exploitability and potential impact (funds at risk, scope, etc.).
4. **Suggested fix (optional):** Any ideas on how to remediate.

### Response timeline

| Stage | Target |
|---|---|
| Acknowledgement | Within 48 hours |
| Severity assessment | Within 5 business days |
| Fix timeline communicated | Within 7 business days |
| Critical fix deployed | Within 14 days of confirmed severity |

### Disclosure policy

We follow coordinated disclosure:
- We ask reporters to allow us 90 days to resolve the issue before public disclosure.
- We will publish a full incident report after resolution.
- Responsible reporters will be credited by name (or alias) in the security advisory, unless they prefer anonymity.

## Bug Bounty

A formal bug bounty program will be announced when Phase 2 contracts are deployed on mainnet. Until then, we offer:
- Public acknowledgement for confirmed vulnerabilities
- Invitation to the `security-contributors` GitHub team
- `SFORGE` token allocation for critical findings (discretionary, at core team's determination)

## Audit Reports

All completed audit reports are published in [`docs/audits/`](audits/) as they become available.
