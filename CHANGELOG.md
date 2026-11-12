# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate
follows Semantic Versioning; the JSON report contract is versioned independently through
`REPORT_SCHEMA_VERSION` (see [`docs/release-policy.md`](docs/release-policy.md)). Rule
changes cite the official SdI control code and the source document.

## [Unreleased]

No unreleased changes yet.

## [0.1.0] - TBD

**Planned foundation release (milestone M1).** The date is set when the release is cut.
Per PLAN §8, `0.1.0` also covers E2–E4 and E9.1–E9.3; their entries will be added as they
land. Items below are the E1 foundation work.

### Added

* **E1.1 — Workspace skeleton and crate boundaries.** Cargo workspace with
  `fattura-core` (I/O-free, network-free, clock-injected library), `fattura-cli`,
  `fattura-ffi` and `xtask`; edition 2024, MSRV 1.85, shared workspace lints, dual
  MIT/Apache-2.0 licensing.
* **E1.2 — Diagnostic model.** The shared diagnostic vocabulary: `Finding`, `SdiCode`,
  `Severity` (`error`/`warning`/`info`), `Level` (L0–L5), `RuleId`, `ElementPath` and the
  layered `Report`; stable rule identifiers such as `ade.00423`.
* **E1.3 — Exact decimal arithmetic core.** `i128` fixed-point and rational arithmetic
  for amounts, rates, rounding and tolerances; no `f64` in domain arithmetic; decimal
  parse/format round-trip tests.
* **E1.4 — CI quality gates and supply-chain policy.** Required fmt, clippy, test, MSRV,
  rustdoc, wasm, `cargo-deny`, coverage and benchmark jobs, plus the workspace lint
  policy (no panics on user input, no `unsafe` outside the FFI crate).
* **E1.5 — Project documentation and governance.** `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `SECURITY.md`, this changelog, the ADR set in
  [`docs/adr/`](docs/adr/), the release policy, GitHub issue/PR templates, and expanded
  crate-level rustdoc.
* Layered-report philosophy: the library reports conformance to L0–L3 and explicitly
  lists L4/L5 as `not_checked`; it never returns a boolean "valid" (PLAN §1.2–§1.3).
* Initial JSON report contract, `REPORT_SCHEMA_VERSION = "1"`.

### Changed

* Documented that `[?]` research items are never emitted as hard `error` findings until
  verified against a primary source (PLAN §11;
  [`docs/release-policy.md`](docs/release-policy.md)).
* Clarified the public licence expression as MIT OR Apache-2.0.

### Security

* Documented the no-network-calls guarantee: no DNS, OCSP/CRL, AdE/SdI, IndicePA or VIES
  access; the core also performs no file I/O and reads no wall clock. Signature trust and
  revocation material is caller-injected; without it the status is `Unverifiable`, never
  `Valid` (PLAN §1.2, §5.1; [`SECURITY.md`](SECURITY.md)).

[Unreleased]: https://github.com/lorenzo-vecchio/fattura-validator/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lorenzo-vecchio/fattura-validator/releases/tag/v0.1.0
