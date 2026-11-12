# Security policy

## Supported versions

`fattura-validator` is pre-1.0 and follows the release policy in
[`docs/release-policy.md`](docs/release-policy.md). Security fixes are applied to the
latest published line; older pre-1.0 lines are not maintained.

| Version | Supported |
| --- | --- |
| `0.1.x` (current development line, milestone M1) | ✅ Yes, once published |
| `< 0.1.0` | ❌ No |

If you are testing an unreleased commit, reproduce against `main` and state the commit in
your report. There is no long-term-support branch before 1.0.

## Private disclosure

**Do not open a public issue, discussion, or pull request for a suspected
vulnerability.**

Use GitHub Security Advisories:

<https://github.com/lorenzo-vecchio/fattura-validator/security/advisories/new>

Please include:

* affected version(s) or commit;
* the smallest input that reproduces the problem (sanitize or synthesize it — never send
  a real invoice, a real tax identifier, or personal data);
* expected and actual behaviour, including the report/finding if one is produced;
* impact: panic/abort, excessive memory or CPU, incorrect acceptance/rejection, or
  signature-status confusion;
* any suggested fix or mitigation.

We aim to acknowledge reports within **7 days** and to agree a coordinated disclosure
timeline. Please allow up to **90 days** before public disclosure; we will credit
reporters who wish to be named. Confirmed issues are fixed in a patch release, published
as a GitHub Security Advisory, and recorded under `Security` in
[`CHANGELOG.md`](CHANGELOG.md). Good-faith research that follows this policy is welcome.

## The library makes no network calls by design

This is a defining property, not an implementation detail (PLAN §1.2, §2.1, §5.2; P3 §7):

* `fattura-core` performs **no network I/O**: no DNS resolution, no TCP/HTTP, no OCSP, no
  CRL download, no Agenzia delle Entrate/SdI service, no IndicePA, no VIES, no ANAC/CUP
  lookup, and no telemetry or analytics.
* The core also performs **no file I/O** and reads no wall clock. The caller supplies the
  input bytes and the validation date (`as_of`); the core is usable from WASM, FFI and
  sandboxes.
* The CLI and future bindings are specified to preserve this property: they may read the
  file the user names, but they must not contact external services.

Consequently, the following classes **do not apply to the library itself**: server-side
request forgery (SSRF), DNS rebinding, outbound data exfiltration, OCSP/CRL privacy
leakage, telemetry privacy, and "validator phones home" supply-chain concerns. There is no
network client in the dependency graph for the core to be abused. If a future change
introduces a network dependency, it is a design and security review failure, not a
feature.

External checks that require online data (P.IVA/CF existence, channel registry, IPA,
VIES, *dichiarazioni d'intento*, historical duplicates, certificate revocation) are
**reported as not checked**, with their official codes, rather than approximated
(PLAN §5.2; ADR [0003](docs/adr/0003-error-model.md)).

## Signature trust and revocation

Local signature work is bounded by what can be proven offline (PLAN §1.2, §5.1):

* Integrity and certificate dates can be checked locally where the algorithm and the
  certificate are available.
* Revocation (CRL/OCSP) and the AgID trust list require data the library does not fetch.
  Trust material is **caller-injected**; without it the status is `Unverifiable`, never
  `Valid`.
* The library never claims that a signature is "valid" merely because it parsed, and it
  makes no statement about the legal validity of a document.

Reports that the validator accepts a forged signature, or reports `Valid` without the
evidence required by the plan, are security issues — report them privately.

## Scope for security reports

In scope:

* panics, aborts, stack overflows or unbounded memory/CPU on hostile input (malformed XML,
  entity expansion, deep nesting, truncated/corrupt ZIP, zip bombs, pathological `.p7m`,
  invalid encodings);
* incorrect findings that cause a file to be accepted when it should be rejected, or a
  false `error` caused by a parser/arithmetic bug;
* signature-status confusion (for example `Valid` or `Unverifiable` reported incorrectly);
* dependency vulnerabilities reachable from the core, including licence/supply-chain
  issues handled by `cargo-deny`.

Out of scope:

* findings about online registries or services the project explicitly does not query
  (PLAN §5.2, P3 §7);
* the mere absence of a rule that is still tracked as `needs-verification` / `[?]`;
* vulnerabilities in a downstream application's own file handling, network layer or
  clock injection;
* reports that require the caller to disable a documented guard or pass attacker-chosen
  trust material.

## Dependency and supply-chain policy

The workspace is pure Rust with no C toolchain, OpenSSL or libxml2 (PLAN §2.3; ADR
[0001](docs/adr/0001-pure-rust.md)). CI runs `cargo deny check` for advisories, licences,
bans and sources. New dependencies require a justification in the pull request and must
remain `wasm32-unknown-unknown`-clean.
