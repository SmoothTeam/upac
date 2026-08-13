# Security Policy

Upac runs with elevated privileges (mounting, `/etc` merges, package installation) and verifies
package/hook authenticity via Ed25519/X.509 signing (`upac-pki`). Vulnerabilities here can have
real system-level impact, so please report them privately rather than through a public issue.

## Supported Versions

Upac is pre-1.0 and under active rewrite (`lib-rs` branch). There is no long-term support branch —
only the latest commit on `main` is supported. Security fixes are not backported to older tags.

## Reporting a Vulnerability

Please use one of the following instead of opening a public issue:

- GitHub's [private vulnerability reporting](https://github.com/SmoothTeam/upac/security/advisories/new)
  (Security tab → "Report a vulnerability")
- Email: aksenovpaveldmitrievich@gmail.com

Include what you'd normally include in a report: affected component/crate, a reproduction if you
have one, and the impact as you see it (e.g. signature bypass, privilege escalation, arbitrary
write outside the deploy tree).

This is a solo-maintained project, so there's no formal SLA — reports are handled on a best-effort
basis, but security reports get priority over everything else in the queue. A fix (or at minimum an
acknowledgment and mitigation advice) will be published via a GitHub Security Advisory once
resolved.

## Scope

Particularly relevant areas:

- `upac-pki` (`lib/pki`) — hook/package signature scheme
- `upac-lib`'s composefs/deploy handling (`lib/lib/src/composefs`, `lib/lib/src/deploy`) — atomicity
  and integrity of system state
- The C ABI boundary (`upac-abi`, `lib/abi`) — memory safety across the FFI
