# fattura-validator — implementation plan

**Status:** implementation — milestone M1 complete · **Plan date:** 12 November 2026 · **Target:** Rust library (crate) that
validates Italian *fatture elettroniche* offline, with language bindings as a later milestone.

Research inputs live in [`sources/`](sources/) and are the normative reference for this plan:

* [Part 1/3](sources/01-standard-versioni-struttura-xml.md) — standards, versions, XML tree, types, file constraints, signature.
* [Part 2/3](sources/02-codifiche-code-list.md) — complete official code lists, deprecations, quirks.
* [Part 3/3](sources/03-validazione-offline-controlli.md) — control catalog, arithmetic, algorithms, tables, test strategy.

Section references below (e.g. `P1 §6`, `P3 §3.5`) point into those files.

---

## 1. Product definition

### 1.1 What it does

`fattura-validator` takes the bytes of an Italian electronic invoice — a bare `.xml`,
an `.xml.p7m` (CAdES), or a `.zip` batch — and produces a **layered report** stating
which offline checks passed, which failed, and which could not be performed.

It validates, in this order:

| Level | Name | What is checked |
| --- | --- | --- |
| **L0** | File | name shape, size, extension, archive integrity, encoding, `.p7m` envelope |
| **L1** | Well-formedness | XML parses; declared encoding matches bytes; BOM/PIs/comments tolerated |
| **L2** | XSD conformance | element order, cardinality, sequence/choice, enumerations, patterns, lengths, numeric ranges |
| **L3** | Content rules | SdI controls `00400`–`00477` that need only the document, plus documented warnings and recommended (non-SdI) checks |
| **L4** | External registries | **not checked** — P.IVA/CF existence, channel registry, IPA, VIES, *dichiarazioni d'intento* |
| **L5** | Lifecycle | **not checked** — historical duplicates, delivery, notifications |

### 1.2 What it explicitly does not do

* It never returns a boolean "valid". It returns `conforms_to: [L0, L1, L2, L3]` and a
  list of `not_checked` entries naming L4/L5 checks by their official code.
* It does not perform network calls. No DNS, no OCSP, no AdE services, no IPA.
* It does not "fix" input. No normalization of `12,50` → `12.50`, no trimming, no
  re-encoding. A file the SdI would reject must be reported, not repaired.
* It does not assert signature *validity* beyond what can be proven locally. With
  revocation unavailable, the signature status is `Unverifiable`, never `Valid`.

### 1.3 Report philosophy

Every finding carries: official SdI code (when one exists), severity, level, element
path, rule id, and a human message. Severities:

| Severity | Meaning |
| --- | --- |
| `error` | The SdI would reject the file according to the rules encoded here. |
| `warning` | Likely rejection, deprecated construct, or ambiguity — must be reviewed. |
| `info` | Domain advice (E-series checks), or a fact about what was skipped. |

Items marked `[?]` in the source documents are **never** `error` until verified (see §11).

---

## 2. Architecture

### 2.1 Workspace layout

```
fattura-validator/
├── Cargo.toml                  # [workspace]
├── crates/
│   ├── fattura-core/           # the library: no I/O, no C deps
│   │   ├── src/
│   │   │   ├── lib.rs          # public API: validate(), Options, Report
│   │   │   ├── diag/           # Finding, SdiCode, Severity, Level, ElementPath
│   │   │   ├── num/            # exact fixed-point + rational arithmetic
│   │   │   ├── xml/            # streaming reader, encoding, local-name dispatch
│   │   │   ├── model/          # typed document: header, body, lines, riepilogo
│   │   │   ├── schema/         # generated static XSD model + validator walk
│   │   │   ├── tables/         # effective-dated code lists (generated data)
│   │   │   ├── rules/          # L3 rules, one module per group
│   │   │   ├── id/             # P.IVA, CF, IBAN, BIC, CIG/CUP, email
│   │   │   ├── file/           # name, size, zip, p7m, encoding
│   │   │   └── sign/           # XAdES / CAdES (feature = "signature")
│   │   └── data/               # vendored XSDs, tables, trust store, corpus manifests
│   ├── fattura-cli/            # binary: fattura-validate
│   └── fattura-ffi/            # C ABI (later); wasm/python/node live elsewhere (E10)
├── xtask/                      # codegen + table/schema update tooling
├── tests/                      # workspace-level conformance tests
├── benches/                    # criterion
├── fuzz/                       # cargo-fuzz targets
└── .planning/
```

`fattura-core` must stay free of `std::fs`, `std::net`, and `std::time::SystemTime`:
time enters through an injected clock so tests are deterministic and bindings are
sandbox-friendly. This is also what makes it usable from WASM and FFI.

### 2.2 Validation pipeline

```
bytes ──► L0 sniff (extension/name/size/zip/p7m) ──► payload bytes
                                                       │
                                                       ▼
                     L1 reader: encoding detect ──► quick-xml events (zero-copy)
                                                       │
                            ┌──────────────────────────┼──────────────────────┐
                            ▼                          ▼                      ▼
                    typed model (model/)      schema walk (schema/)    L3 rules (rules/)
                            │                          │                      │
                            └──────────────► findings ◄┘──────────────────────┘
                                                       │
                                              report assembly
                                                       │
                                          L4/L5 "not checked" disclosure
```

One pass builds the model and runs the checks that need only local context. Checks
that need whole-document aggregation (riepilogo coverage, duplication, batch
comparisons) run after the pass on the collected model. No DOM is built, except in the
optional signature path, where canonicalization needs a subtree.

### 2.3 Dependency policy

| Concern | Choice | Rationale |
| --- | --- | --- |
| XML | `quick-xml` (streaming, borrowed) | fastest pure-Rust reader; no C, no DOM |
| XSD | **no runtime XSD engine** | codegen a static schema model (E3.2); avoids libxml2 and is far faster |
| Decimal | own `i128` fixed-point | exactness, no `f64`, no bigint allocs |
| Dates | `jiff` (or `time`) | `xs:date`/`dateTime` parsing with explicit timezone policy |
| Report | `serde` + `serde_json` | stable JSON contract for every binding |
| Archive | `zip` (feature `zip`) | pure Rust; guarded against zip bombs |
| CAdES | `cms`, `x509-parser`, `der` (feature `p7m`) | pure Rust DER/CMS |
| Crypto | `rsa`, `p256`, `sha1`, `sha2` (feature `signature`) | verify digests/signatures |
| Dev | `criterion`, `proptest`, `insta`, `dhat`, `cargo-fuzz` | benchmarks, properties, snapshots, alloc tracking, fuzzing |

Hard rules: **no C toolchain, no OpenSSL, no libxml2**. Anything that breaks
`wasm32-unknown-unknown` is rejected. Workspace lints deny `unwrap`/`expect` outside
tests, deny float arithmetic in `num`/`rules`, and deny `unsafe` outside `fattura-ffi`.

### 2.4 Performance design

* Single pass, borrowed slices, no `String` for names — dispatch on byte slices.
* No runtime regex. XSD patterns are compiled to small hand-written matchers by codegen
  (E3.3), so hot fields (`CAP`, `Provincia`, `Divisa`, amounts) are a few comparisons.
* Findings are the only allocations on the happy path. Element paths are built lazily,
  only when a finding is emitted.
* Aggregations use `SmallVec` keyed by scale-8 integer amounts, not hash maps of strings.
* Budgets (CI-enforced, E9.4): 10 KB / 20-line invoice ≤ **150 µs**; 5 MB file ≤ **60 ms**;
  core structural throughput ≥ **100 MB/s**; zero heap allocations per simple-type check.

---

## 3. Ruleset and date model

The definition of "valid" is time-dependent in two independent ways. The API must keep
them separate and name them:

1. **Document date** (`DatiGeneraliDocumento/Data`) decides which code lists apply:
   `N2/N3/N6` are legal before 2021-01-01 and rejected after (`00445`); `Z` is replaced by
   `ZO`; `RF03` was removed; `TD29`, `RF20`, `ESENZSPORT`, `00327` appear in 1.9/1.9.1.
2. **Validation date** (`as_of`, defaulting to *now*) decides what "current" means for
   format checks and for the "is this date in the future" rule (`00403`).

```rust
pub struct Options {
    pub as_of: Option<CivilDate>,        // default: injected clock
    pub ruleset: RulesetId,              // default: latest embedded ("ade-1.9.1")
    pub severity: SeverityProfile,       // Sdi | Recommended | Permissive
    pub limits: Limits,                  // size caps, error cap, zip guards
    pub features: FeatureSet,            // signature, zip, p7m
}
```

Every table and rule carries an effective window plus a source citation. The invariant:
*"is this document conformant to the rules in force at date X?"* must be answerable, and
a finding must be reproducible years later. Codes are selected from the document date
unless the official rule explicitly uses transmission time — documented per rule.

---

## 4. Public API sketch

```rust
// Rust
pub fn validate(input: &[u8], options: &Options) -> Report;

pub struct Report {
    pub schema_version: &'static str,       // JSON contract version
    pub document: DocumentInfo,             // format, version, bodies, counts
    pub levels: LevelSummary,               // per-level: Checked | Passed | Findings(n)
    pub findings: Vec<Finding>,
    pub not_checked: Vec<NotChecked>,        // L4/L5 with official codes
}

pub struct Finding {
    pub rule: RuleId,                       // stable id, e.g. "ade.00423"
    pub code: Option<SdiCode>,              // None for E-series / informational
    pub severity: Severity,                 // Error | Warning | Info
    pub level: Level,                       // L0..L3
    pub path: ElementPath,                  // Body[2]/DatiBeniServizi/DettaglioLinee[3]/PrezzoTotale
    pub message: String,
    pub hint: Option<String>,
}
```

The JSON form of `Report` is the contract used by all bindings and the CLI; it is
versioned and snapshot-tested. Bindings expose `validate(bytes, options_json) -> json`
plus typed wrappers.

---

## 5. Coverage map

### 5.1 Fully offline and in scope

* **File/format:** `00001` (shape only), `00003`, `00106`, `00200`, `00201` (counter policy),
  `00103`.
* **Content:** `00400`, `00401`, `00403` (vs today), `00409`, `00411`, `00413`, `00414`,
  `00415`, `00417`, `00418`, `00419`, `00420`, `00421`, `00422`, `00423`, `00424`, `00425`,
  `00427`, `00428`, `00429`, `00430`, `00437`, `00438`, `00443`, `00444`, `00445`, `00460`,
  `00471`–`00476`.
* **Local but with policy:** `00426` (only "PEC set ⇒ `0000000`" as error; the other branch
  is a configurable warning), `00313` (`XXXXXXX` vs foreign recipient), `00330` (local
  blacklist of SdI mailboxes), `00459` (warning, `RF03`).
* **Partial signature:** `00100`, `00102`, `00105`, `00107` with local crypto and a
  versioned trust store; `00101`/`00104` only as `Unverifiable` or opt-in.
* **Recommended (no SdI code):** E1–E18 from P3 §4.

### 5.2 Out of scope, disclosed by code

`00002`, `00101`, `00104` (unless opt-in trust store), all `003xx` registry checks
(`00300`–`00306`, `00311`–`00312`, `00320`–`00327`, `00398`–`00399`), `00404`, `00477`.
Each appears in `report.not_checked` with the reason (P3 §7).

---

## 6. Work breakdown

Each item below becomes a GitHub issue. `needs-verification` marks items blocked on an
open question from the sources — they must be resolved against official PDFs before the
behaviour is frozen. Dependencies are listed so the board can be ordered.

### [E1 — Foundation and diagnostics](https://github.com/lorenzo-vecchio/fattura-validator/issues/1) (tracking issue #1)

| Key | Title | Depends on |
| --- | --- | --- |
| [E1.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/11) | Workspace skeleton and crate boundaries | — |
| [E1.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/12) | Diagnostic model: codes, severities, levels, paths, report | E1.1 |
| [E1.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/13) | Exact decimal arithmetic core | E1.1 |
| [E1.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/14) | CI quality gates and supply-chain policy | E1.1 |
| [E1.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/15) | Project documentation and governance | E1.1 |

### [E2 — L1 XML ingestion and typed model](https://github.com/lorenzo-vecchio/fattura-validator/issues/2) (tracking issue #2)

| Key | Title | Depends on |
| --- | --- | --- |
| [E2.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/16) | Streaming XML reader with namespace-agnostic local-name matching | E1.1 |
| [E2.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/17) | Typed document model for FPA12/FPR12 | E2.1, E1.2 |
| [E2.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/18) | Typed document model for FSM10 (semplificata) | E2.2, E3.1 |
| [E2.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/19) | Multi-body lotto support and per-body scoping | E2.2 |
| [E2.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/20) | XML robustness: encodings, BOM, PIs, whitespace, malformed input | E2.1 |

### [E3 — L2 XSD conformance](https://github.com/lorenzo-vecchio/fattura-validator/issues/3) (tracking issue #3)

| Key | Title | Depends on |
| --- | --- | --- |
| [E3.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/21) | Vendor official XSDs with checksums and provenance manifest | — |
| [E3.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/22) | Generate a static schema model from the XSDs | E3.1, E2.1 |
| [E3.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/23) | XSD simple-type facet engine (XSD regex, lengths, ranges, whitespace) | E3.2 |
| [E3.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/24) | Structural validation over the schema model (order, cardinality, choice) | E3.3 |
| [E3.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/25) | Differential XSD conformance tests against xmllint + relaxed-policy set | E3.4, E9.5 |

### [E4 — Code lists and effective-dated rulesets](https://github.com/lorenzo-vecchio/fattura-validator/issues/4) (tracking issue #4)

| Key | Title | Depends on |
| --- | --- | --- |
| [E4.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/27) | Effective-dated table infrastructure | E1.2 |
| [E4.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/28) | SdI closed enumerations from the XSD | E4.1, E3.1 |
| [E4.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/29) | Deprecation windows and validity cutovers | E4.2 |
| [E4.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/30) | CausalePagamento with CU-yearly versions | E4.1 |
| [E4.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/31) | AltriDatiGestionali/TipoDato quasi-enumeration | E4.1 |
| [E4.6](https://github.com/lorenzo-vecchio/fattura-validator/issues/32) | External open lists: ISO 3166-1, ISO 4217, provinces, Incoterms | E4.1 |

### [E5 — L3 content rules (004xx)](https://github.com/lorenzo-vecchio/fattura-validator/issues/5) (tracking issue #5)

| Key | Title | Depends on |
| --- | --- | --- |
| [E5.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/33) | Natura ↔ AliquotaIVA consistency | E2.2, E4.2 |
| [E5.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/34) | Riepilogo coverage and matching (00419/00443/00444) | E5.1 |
| [E5.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/35) | Invoice arithmetic: 00421, 00422, 00423 | E1.3, E5.2 |
| [E5.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/36) | Withholding, payment and discount structure | E5.3 |
| [E5.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/37) | Transmission coherence: 00427, 00428, 00426, 00313 | E2.2, E4.2 |
| [E5.6](https://github.com/lorenzo-vecchio/fattura-validator/issues/38) | TipoDocumento rule matrix 00471–00476 | E2.4, E4.2 |
| [E5.7](https://github.com/lorenzo-vecchio/fattura-validator/issues/39) | Deprecated codes and FSM10 amount cap (00445, 00459, 00460) | E4.3 |
| [E5.8](https://github.com/lorenzo-vecchio/fattura-validator/issues/40) | Intra-file duplication (00409) and duplicate warnings (00404) | E2.4 |
| [E5.9](https://github.com/lorenzo-vecchio/fattura-validator/issues/41) | Recommended non-SdI checks E1–E18 | E5.3, E6.* |

### [E6 — Identifier validation](https://github.com/lorenzo-vecchio/fattura-validator/issues/6) (tracking issue #6)

| Key | Title | Depends on |
| --- | --- | --- |
| [E6.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/42) | Partita IVA checksum | E1.2 |
| [E6.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/43) | Codice Fiscale for individuals, including omocodia | E6.1 |
| [E6.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/44) | IBAN mod-97, country structure and ABI/CAB consistency | E6.1 |
| [E6.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/45) | Geographic and currency codes | E4.6 |
| [E6.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/46) | CIG/CUP formats and email/PEC heuristics | E4.6 |

### [E7 — L0 file layer](https://github.com/lorenzo-vecchio/fattura-validator/issues/7) (tracking issue #7)

| Key | Title | Depends on |
| --- | --- | --- |
| [E7.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/47) | File-name validation (00001) | E1.2 |
| [E7.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/48) | Size limits (00003) | E1.2 |
| [E7.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/49) | ZIP archive support (00106) | E7.1, E2.1 |
| [E7.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/50) | `.p7m` CAdES envelope parsing | E7.2 |
| [E7.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/51) | Encoding detection and declared-vs-actual checks | E2.5 |

### [E8 — Signature verification](https://github.com/lorenzo-vecchio/fattura-validator/issues/8) (tracking issue #8)

| Key | Title | Depends on |
| --- | --- | --- |
| [E8.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/52) | XAdES presence and shape rules (00103) | E2.1 |
| [E8.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/53) | XML-DSig integrity verification (00102) | E8.1 |
| [E8.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/54) | Certificate validity window (00100, 00105, 00107) | E8.2 |
| [E8.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/55) | CAdES parity and trust/revocation status model (00101, 00104) | E8.3, E7.4 |

### [E9 — API CLI reporting and performance](https://github.com/lorenzo-vecchio/fattura-validator/issues/9) (tracking issue #9)

| Key | Title | Depends on |
| --- | --- | --- |
| [E9.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/56) | Public Rust API and JSON contract | E1.2 |
| [E9.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/57) | CLI with deterministic exit codes | E9.1 |
| [E9.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/58) | Report rendering: JSON, text, JUnit | E9.1 |
| [E9.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/59) | Performance harness and budgets | E2.2, E5.3 |
| [E9.5](https://github.com/lorenzo-vecchio/fattura-validator/issues/60) | Fuzzing and regression corpus | E2.5, E3.5 |

### [E10 — Bindings and distribution](https://github.com/lorenzo-vecchio/fattura-validator/issues/10) (tracking issue #10)

| Key | Title | Depends on |
| --- | --- | --- |
| [E10.1](https://github.com/lorenzo-vecchio/fattura-validator/issues/61) | Stable C ABI crate | E9.1 |
| [E10.2](https://github.com/lorenzo-vecchio/fattura-validator/issues/62) | WebAssembly package | E9.1, E10.1 |
| [E10.3](https://github.com/lorenzo-vecchio/fattura-validator/issues/63) | Python and Node packages | E10.1 |
| [E10.4](https://github.com/lorenzo-vecchio/fattura-validator/issues/64) | Release automation and cross-language conformance suite | E9.3, E10.1–E10.3 |

---

## 7. Testing and conformance strategy

1. **Golden documents.** Official FPA01–03 / FPR01–03 examples, plus the 1.9.1
   "esempi fattura, hash e ricevute" bundle. Each has an expected `Report` snapshot
   (`insta`). Provenance and checksum recorded in `data/corpus/SOURCES.md`.
2. **Differential XSD.** For every corpus file, compare our L2 verdict with
   `xmllint --schema <official.xsd>`; divergences are either bugs or documented
   relaxations (email regex, XSD-regex engine quirks). Stored as a known-difference list.
3. **Differential rules.** Where a reference implementation (FatturaElettronica.NET,
   `slam/php-validatore-fattura-elettronica`) can be run offline, compare findings per
   control code — never adopt behaviour blindly; the SdI is the oracle.
4. **Property-based.** Arithmetic (cascading mixed discounts, rounding boundaries,
   tolerance edges `0.009…` / `0.010…`), decimal parse/format round-trips, path rendering.
5. **Fuzzing.** XML mutations (BOM, truncated, mis-encoded, deep nesting, entity tricks),
   ZIP bombs and pathological `.p7m`; no panics, no unbounded memory.
6. **Hand-written edge cases.** TD16 (Natura + IVA ≠ 0), TD04 duplicate exception,
   multi-body lotto, `0000000` with/without PEC, `XXXXXXX` with IT recipient, `N2` before
   and after 2021, `RF03`, `Z`/`ZO`, omocodia CF, `00000000000` P.IVA.
7. **Cross-language conformance.** Every binding runs the same corpus and must produce
   byte-identical JSON (E10.4).

---

## 8. Milestones and release plan

The GitHub milestones mirror the epics (M1–M10). Releases cut across them:

| Release | Content |
| --- | --- |
| `0.1.0` | E1–E4, E9.1–E9.3: parse, L2 conformance, code lists, API + CLI + JSON. |
| `0.2.0` | E5, E6: full offline `004xx`, identifiers, warnings, performance budgets. |
| `0.3.0` | E7, E8: zip/p7m/encoding, signature verification. |
| `0.4.0` | E10.1–E10.3: C ABI, WASM, Python, Node. |
| `1.0.0` | Frozen JSON contract, MSRV policy, 100% of in-scope rules covered, docs complete. |

MSRV: edition 2024 → Rust **1.85**, tested in CI and bumped only on a minor release.

---

## 9. Definition of done (applies to every issue)

* Code + tests, `cargo fmt` / `clippy -D warnings` clean.
* Every rule cites its source (`P3 §…`, official code) in a doc comment.
* Positive and negative test cases; a corpus snapshot where relevant.
* `Report` JSON change → schema version decision + snapshot update.
* User-visible behaviour documented in rustdoc and `README`.
* No `unwrap`/`expect` outside tests; no `f64` in domain arithmetic; no new C dependency.

---

## 10. Risks

| # | Risk | Mitigation |
| --- | --- | --- |
| R1 | AdE documentation is not machine-readable; rules live in PDF prose | Source-tagged rules, verification issues, differential testing, changelog watch |
| R2 | Rules change ~yearly (CU) and per provvedimento | Effective-dated data tables; updates without code changes; `as_of` selection |
| R3 | XSD codegen may be immature in pure Rust | Fallback: custom XSD reader → own static model; XSDs are simple (sequence + simpleType facets) |
| R4 | XML-DSig canonicalization is easy to get subtly wrong | Test vectors, interop suite, status `Unverifiable` instead of false `Valid` |
| R5 | Vendoring official XSDs/corpus may have unclear redistribution terms | Keep provenance + checksums, attribution, opt-out via build feature; confirm before 1.0 |
| R6 | Over-validation (false errors) is worse than under-validation | `[?]` → warning only; severity profiles; no blocking email-regex check |
| R7 | Performance claims unproven | CI-enforced benchmarks from the first milestone (E9.4) |
| R8 | Decimal/rounding divergences | Exact i128 + rational reference model, property tests, documented tolerances per control |

---

## 11. Open questions (tracked as `needs-verification` issues)

From P1 §11 and P3 §8.1:

1. `00426` status: v1.8 has the double implication; v2.0 reportedly omits it; 1.9.1 allows
   `0000000` without PEC. Implement only "PEC ⇒ `0000000`" as error until verified.
2. Exact `00460` threshold and derogation in 1.9.1 (400 €, forfettari/RF20).
3. Complete FSM10 tree and differences from XSD 1.0.2 (missing/optional nodes).
4. `xs:dateTime` timezone offset and fractional-seconds policy.
5. Sign of `Arrotondamento` in the `00422` formula (algebraic addend vs direction).
6. Whether `Appendice 1` of Allegato A 1.9.1 adds controls beyond Elenco v2.0
   (`00327`, `ESENZSPORT`).
7. PEC/email length and blacklist rules beyond `EmailType` (local `00330` list).
8. Whether any official Schematron/XSD 1.1 assertions exist (currently believed: none).

---

## Appendix A — module ↔ control map

| Module | Controls / items |
| --- | --- |
| `file/name` | 00001 |
| `file/limits` | 00003, 00201 counter |
| `file/zip` | 00106, batch 00002-style warning |
| `file/p7m` | 00106, CAdES envelope |
| `xml/encoding` | L1, 00106 (corrupt) |
| `schema/*` | 00200 |
| `rules/natura` | 00400, 00401, 00413, 00414, 00420, 00429, 00430, 00444, 00445 |
| `rules/riepilogo` | 00419, 00443, 00444 |
| `rules/amounts` | 00421, 00422, 00423, 00424 |
| `rules/withholding` | 00411, 00415, 00437, 00438 |
| `rules/document` | 00417, 00418, 00425, 00471–00476, 00460, 00459 |
| `rules/transmission` | 00426, 00427, 00428, 00313, 00330 |
| `rules/duplicates` | 00409, 00404 (warning) |
| `id/*` | 00300–00306 shape/checksum only, 00320/00324 heuristics |
| `sign/*` | 00100, 00102, 00103, 00105, 00107 (+00101/00104 status only) |
| `rules/recommended` | E1–E18 |

## Appendix B — glossary

* **SdI** — Sistema di Interscambio, the exchange system run by Agenzia delle Entrate.
* **FPA12 / FPR12 / FSM10** — the three transmission formats (PA, private, simplified).
* **Lotto** — one file containing multiple `FatturaElettronicaBody` elements.
* **Riepilogo** — `DatiRiepilogo`, the per-VAT-rate summary block.
* **Natura** — the VAT exemption/reverse-charge reason code (`N1`…`N7`).
* **Omocodia** — CF disambiguation by replacing digits with letters.
* **Ruleset** — a dated snapshot of tables and rules (`ade-1.9.1`).
