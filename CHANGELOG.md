# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate
follows Semantic Versioning; the JSON report contract is versioned independently through
`REPORT_SCHEMA_VERSION` (see [`docs/release-policy.md`](docs/release-policy.md)). Rule
changes cite the official SdI control code and the source document.

## [Unreleased]

### Added

* **E2.2 — Typed FPA12/FPR12 document model.** A streaming, one-pass builder over the E2.1
  reader produces the complete element tree of `P1 §6`: header (transmission, seller, tax
  representatives, buyer, intermediary), body (document data, correlated documents, SAL,
  DDT, transport, main invoice, lines, VAT summary, vehicle, payments, attachments), with
  every nested block. Money is `num::Amount`, rates are `num::Rate`, `xs:date` fields are
  `model::CivilDate` (pure integer calendar, no clock, no `f64`); `xs:dateTime` keeps its
  raw lexical form for E3.3. Repetitions preserve document order in `SmallVec`-backed
  lists, so the 1-based index matches finding paths. Unknown elements are retained with
  their byte offset and depth for XSD reporting. Text borrows from the input unless an
  entity reference forces a decoded copy. A malformed numeric or date value becomes
  `None` and is left to the schema layer, so one bad value never hides the rest of the
  document. Model benchmarks (`benches/model.rs`): a **12.9 KB** exhaustive invoice parses in
  **88–94 µs** (PLAN §2.4 budget: 150 µs) and a **5 MB lotto** in **43–54 ms**
  (88–112 MiB/s; budget: 60 ms) on the development machine;
  `size_of::<Document>()` is **6,368 bytes** (recorded by
  `document_value_stays_small_enough_to_return_by_value`).
* **E2.1 — Streaming XML reader.** A `quick-xml`-backed reader over `&[u8]` that yields
  borrowed element/attribute names and text with byte offsets and element depth. Element
  matching is namespace-agnostic on local names, because the official schema leaves
  children unqualified (`P1 §5.1`); prefixes are recorded for reporting only. Comments
  and processing instructions are skipped; whitespace is never trimmed (`P3 §8`). The
  reader exposes the root prefix, namespace and `@versione`, and a one-pass `scan`
  returns L1 findings. Depth and text-run guards are typed warnings, never aborts.
* XML reader benchmarks (`benches/xml.rs`): 20-line scan **43 µs**, 5 MiB scan **35 ms**,
  throughput **≥140 MiB/s** on the development machine (PLAN §2.4 budgets: 150 µs and
  60 ms). An initial implementation recomputed line/column for every event and measured
  1.7 ms for the 20-line case; positions are now computed only on the error path.

### Security

* DTDs and entity declarations are rejected before parsing, removing the external-entity
  (XXE) and entity-expansion attack surface for invoice input (`P3 §8` plumbing items
  1–6). Undefined entity references and malformed character references are L1 findings.

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
