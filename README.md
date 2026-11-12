# fattura-validator

Offline validator for Italian electronic invoices (*fatture elettroniche*), written in
Rust. Validates the file, the XML, the XSD conformance, and the SdI content rules that
can be checked without any network access — then states clearly what it could **not**
check.

> **Status: planning.** The implementation plan and the research it is based on live in
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
let report = fattura_validator::validate(&bytes, &Options::default())?;
for finding in &report.findings {
    eprintln!("{:?} {}: {}", finding.severity, finding.code, finding.message);
}
println!("not checked: {:?}", report.not_checked);
```

A CLI and bindings for C, WebAssembly, Python and Node are on the roadmap.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
