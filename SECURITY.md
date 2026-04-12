<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright 2025-2026 ecoPrimals Project -->

# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| v3.x    | Yes       |
| v2.x    | Security fixes only |
| < v2.0  | No        |

## Reporting a Vulnerability

**Do not open a public issue for security vulnerabilities.**

Report vulnerabilities by emailing **security@biomeos.org** with:

1. **Description** of the vulnerability
2. **Steps to reproduce** (minimal reproduction case preferred)
3. **Impact assessment** — what an attacker could achieve
4. **Affected components** — which crate(s) or module(s)

We will acknowledge receipt within 48 hours and provide a detailed response
within 7 business days, including:

- Confirmation of the vulnerability
- Severity assessment (using CVSS where appropriate)
- Planned fix timeline
- Credit attribution (unless you prefer anonymity)

## Security Design Principles

biomeOS follows the ecoPrimals security model:

- **Zero unsafe code** in production (`#![forbid(unsafe_code)]` workspace-wide)
- **Zero C dependencies** at the application level (ecoBin v3.0 compliant)
- **No telemetry or phone-home** — sovereignty by design
- **Dark Forest beacon genetics** for privacy-preserving discovery
- **BearDog delegation** for all cryptographic operations (biomeOS never handles keys)
- **Genetic lineage verification** for primal authenticity
- **Circuit breakers** to prevent cascade failures across primals

## Scope

This policy covers the biomeOS workspace (`crates/*`, `src/`, `tests/`).
For vulnerabilities in other ecoPrimals primals (BearDog, Songbird, NestGate,
Squirrel, Toadstool), report to the respective primal's security contact
or to the ecosystem address above.

## License

This security policy is part of the biomeOS documentation, licensed under
CC-BY-SA 4.0 as part of the scyBorg triple-copyleft framework.
