# fattura-validator

Offline validator for Italian electronic invoices (*fatture elettroniche*), written in
Rust. Validates the file, the XML, the XSD conformance, and the SdI content rules that
can be checked without any network access — then states clearly what it could **not**
check.

> **Status: M1 (E1 — foundation) in progress.** The workspace, diagnostic vocabulary,
> exact arithmetic, CI gates and project governance are landing under milestone M1. The
> implementation plan and the research it is based on live in
> [`.planning/`](.planning/). Work is tracked in the
> [issue tracker](https://github.com/lorenzo-vecchio/fattura-validator/issues).

## Workspace

| Crate | Role |
| --- | --- |
| [`crates/fattura-core`](crates/fattura-core) | The library: I/O-free, network-free, clock-injected, `wasm32`-clean. |
| [`crates/fattura-cli`](crates/fattura-cli) | `fattura-validate` binary (stub until E9.2). |
| [`crates/fattura-ffi`](crates/fattura-ffi) | Stable C ABI (stub until E10.1). |
| [`xtask`](xtask) | Code generation for the static schema model and dated tables. |

The architecture, module map and work breakdown are in
[`.planning/PLAN.md`](.planning/PLAN.md) (§2.1 for the module tree, §6 for the
milestones).

## Governance and documentation

| Document | What it covers |
| --- | --- |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Development setup, gate commands, test/fixture conventions, hard rules and definition of done. |
| [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) | Contributor Covenant 2.1 and the enforcement process. |
| [SECURITY.md](SECURITY.md) | Supported versions, private disclosure, and the no-network-calls design. |
| [CHANGELOG.md](CHANGELOG.md) | Keep a Changelog history, including the M1 foundation work. |
| [`docs/adr/`](docs/adr/) | Architecture Decision Records: pure Rust, static schema codegen, layered report. |
| [`docs/release-policy.md`](docs/release-policy.md) | SemVer for the crate and JSON contract, MSRV, deprecations, release checklist and `[?]` policy. |
| [`.planning/PLAN.md`](.planning/PLAN.md) | The implementation plan: scope, architecture, work breakdown, risks and open questions. |

Issue forms and the pull-request checklist live in
[`.github/`](.github/). Security reports must use
[GitHub Security Advisories](https://github.com/lorenzo-vecchio/fattura-validator/security/advisories/new),
not a public issue.

## Why

Most validators either stop at XSD validation or silently drift from the SdI rules:
rounding tolerances, effective-dated code lists, multi-body *lotti*, `.p7m` and ZIP
handling are where they diverge. This project aims to be:

* **Offline and deterministic** — no network calls, no AdE services, no registries.
* **Layered** — reports L0 (file) … L3 (content) and discloses L4/L5 as not checked.
* **Exact** — decimal arithmetic with per-control tolerances, never floating point.
* **Fast** — single-pass streaming parser, no DOM, no runtime regex, no C dependencies.
* **Portable** — the core is pure Rust and `wasm32`-clean, ready for language bindings.

## Scope

| In scope (offline) | Out of scope (requires online data) |
| --- | --- |
| File naming, size, encoding, ZIP, `.p7m` | P.IVA / CF existence and status |
| XML well-formedness | Channel registry, IndicePA codes |
| XSD conformance (1.2.3 / 1.0.2) | *Dichiarazioni d'intento* |
| SdI content rules `00400`–`00476` | Historical duplicates (`00404`) |
| Checksums: P.IVA, CF, IBAN | Certificate revocation (CRL/OCSP) |
| Signature integrity and dates | Signature trust chain (AgID, opt-in) |

**The library never says "this invoice is valid".** It reports conformance to the levels
it can verify, with every finding tied to an official code and an element path.

## Usage (target API)

```rust
let report = fattura_core::validate(&bytes, &Options::default());
for finding in &report.findings {
    eprintln!("{:?} {}: {}", finding.severity, finding.code, finding.message);
}
println!("not checked: {:?}", report.not_checked);
```

A CLI and bindings for C, WebAssembly, Python and Node are on the roadmap.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
