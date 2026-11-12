# Contributing to fattura-validator

Thanks for helping build an offline validator for Italian electronic invoices. This
document covers the development environment, the quality gates, the rules that are not
negotiable, and what "done" means. The design record is
[`.planning/PLAN.md`](.planning/PLAN.md); every behaviour change must be traceable to a
section there (or to an official SdI source cited from it).

By participating you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md). Security
issues go through the private process in [SECURITY.md](SECURITY.md), never a public issue.

## Development setup

The workspace is a pure-Rust Cargo workspace. **No C toolchain, OpenSSL, libxml2, or
system XML library is required** (PLAN §2.3, ADR
[0001](docs/adr/0001-pure-rust.md)).

Requirements:

* Rust **1.85** or newer (edition 2024). CI pins 1.85 as the MSRV and also builds stable.
* `rustfmt` and `clippy` components.
* Optional but expected for the gates below:
  * `cargo-deny` (`cargo install cargo-deny`)
  * `cargo-llvm-cov` (`cargo install cargo-llvm-cov`)
  * the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
  * `cargo-fuzz` for fuzz targets (nightly toolchain)

Clone and build:

```sh
git clone https://github.com/lorenzo-vecchio/fattura-validator
cd fattura-validator
cargo build --workspace
cargo test --workspace
```

The crate map and module tree are in [README.md](README.md) and PLAN §2.1. The core
crate is I/O-free, network-free and clock-injected; time and bytes are always injected by
the caller, so tests stay deterministic (PLAN §2.1).

## Gate commands

Run the same checks locally that CI runs on every pull request. The workflow definitions
under `.github/workflows/` are authoritative; the list below mirrors them:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo build -p fattura-core --target wasm32-unknown-unknown --locked
cargo deny check advisories bans licenses sources
cargo llvm-cov -p fattura-core --all-features --locked \
  --fail-under-lines 100 --fail-under-regions 100 --fail-under-functions 100
cargo bench -p fattura-core --locked --no-run
```

MSRV check, matching the CI matrix entry for Rust 1.85.0:

```sh
cargo +1.85.0 test --workspace --all-targets --all-features --locked
```

A pull request is not mergeable until the aggregate **`CI gate`** job is green. It requires
**fmt, clippy, test (stable and MSRV), doc, wasm, deny, coverage and bench** to succeed; a
skipped or failed job fails the gate. Do not weaken a gate to make a change pass; fix the
change or open an issue explaining why the gate itself is wrong.

## Hard rules

These follow from PLAN §2.3, §9 and the product definition in PLAN §1:

1. **No panic on user input.** The library returns a report; it must not unwind on
   malformed XML, truncated ZIPs, hostile `.p7m`, or bad encodings. `unwrap` and `expect`
   are denied outside tests by workspace lints. Tests may `#[allow]` locally when a panic
   *is* the assertion.
2. **No `f64` in domain arithmetic.** Amounts, quantities, rates, tolerance comparisons
   and rounding use the exact fixed-point/rational core in `num` (PLAN §2.3, §7.4).
   `clippy::float_arithmetic` and `clippy::float_cmp` are denied at crate level.
3. **No C dependencies and no OpenSSL/libxml2.** The dependency policy is pure Rust and
   `wasm32`-clean. A dependency that breaks `wasm32-unknown-unknown` is rejected.
4. **No runtime XSD engine and no runtime regex.** XSDs are vendored and compiled to
   static tables by `xtask` (PLAN §2.3, E3.2; ADR
   [0002](docs/adr/0002-static-schema-codegen.md)). Patterns become hand-written
   matchers.
5. **No DOM and no network I/O in `fattura-core`.** One streaming pass over borrowed
   bytes. No `std::fs`, `std::net`, or `SystemTime` in the core (PLAN §2.1). The
   validator makes no DNS, OCSP, CRL, AdE, IPA or VIES calls by design
   ([SECURITY.md](SECURITY.md)).
6. **The library never returns a boolean "valid".** It returns the layered report and
   lists what was not checked (PLAN §1.2, ADR [0003](docs/adr/0003-error-model.md)).
7. **`[?]` is never a hard `Error`.** See below.

## The `[?]` rule

The research sources mark each claim `[V]` (verified against the primary source), `[S]`
(secondary source) or `[?]` (to be verified). An item marked `[?]`:

* may be implemented as a `warning` or `info` finding at most — **never** as an `error`;
* must have a `needs-verification` GitHub issue;
* must not be cited as official behaviour in docs or rustdoc until the open question is
  resolved and a primary source is recorded.

The same rule applies to open questions in PLAN §11. Over-validation is worse than
under-validation (PLAN §10, R6). If you believe a `[?]` item should become an error, the
pull request must include the primary-source citation, the exact control code, and
positive and negative tests.

## Tests and fixtures

Test conventions:

* Unit tests live next to the code in `#[cfg(test)]` modules; workspace-level conformance
  tests live in `tests/`; property tests use `proptest`; snapshots use `insta`.
* Every rule needs a **positive** case (no finding) and a **negative** case (the expected
  finding, code, severity, level and path). A corpus snapshot is required when the rule
  affects the public `Report`.
* Determinism is mandatory. Inject `as_of`/the clock; never read wall-clock time in a
  test. Never rely on hash-map iteration order in a snapshot.
* Test data must be small and reviewable. A minimal XML fragment beats a full invoice
  unless the test is specifically about a full document.

Fixture and provenance conventions (PLAN §2.1, §7.1):

* Vendored XSDs, dated tables, trust stores and corpus manifests live under
  `crates/fattura-core/data/`.
* Every corpus entry has provenance in `data/corpus/SOURCES.md`: origin, version, date,
  checksum (SHA-256), licence/redistribution note, and the expected report snapshot.
* Never add a fixture of unclear origin. If redistribution is uncertain, record the
  checksum and keep the file out of the repository until it is resolved (PLAN §10, R5).
* Generated schema/tables are produced by `xtask`; do not hand-edit generated files.
  Regenerate them and include the generator input/checksum change in the same PR.

Snapshot updates must be deliberate: review the diff, confirm every changed finding is
explained by the change, and never run `cargo insta accept` blindly.

## Coverage policy

Coverage is a merge gate, not a vanity metric:

* CI requires **100% line, region and function coverage** for `fattura-core`
  (`cargo llvm-cov --fail-under-lines 100 --fail-under-regions 100
  --fail-under-functions 100`). A pull request must keep it at 100%.
* New or changed lines need tests. Do not add an exclusion or lower a threshold to land a
  change; if a generated or trivial item genuinely cannot be covered, change the policy
  deliberately in a separate PR with a justification.
* Every implemented rule has both positive and negative tests; arithmetic changes include
  boundary/property tests (tolerances `0.009…`/`0.010…`, rounding edges).
* Fuzz regressions belong in the corpus once minimized.

## Definition of done

The project-wide DoD is PLAN §9. In addition to the checklists in the pull request
template, a change is done when:

* [ ] Code and tests are present; `cargo fmt --check` and `clippy -D warnings` are clean.
* [ ] Every rule cites its source (`P3 §…`, official control code, or PLAN section) in a
      doc comment.
* [ ] Positive and negative tests exist; a corpus snapshot is updated where relevant.
* [ ] A `Report`/JSON change includes a schema-version decision and refreshed snapshots.
* [ ] User-visible behaviour is documented in rustdoc and, where relevant, `README.md`
      and `CHANGELOG.md`.
* [ ] No `unwrap`/`expect` outside tests; no `f64` in domain arithmetic; no new C
      dependency; no runtime regex or DOM.
* [ ] `[?]` items remain warnings/info, with a `needs-verification` issue.
* [ ] The full gate set above is green.

## Commit style

History is linear on `main`. Use the repository's existing style:

```text
E<epic>.<item>: <imperative summary>

Why this change is needed, which PLAN section or official source it implements,
and any schema/behaviour decision.
```

Examples: `E1.5: add project governance documentation`, `E5.3: implement 00421 with exact
arithmetic`. Keep the subject under ~72 characters, use the body for rationale, and
reference the issue (`Refs #15`). Do not mix refactors with rule behaviour in one commit.
Squash fixups before merging; no merge commits (linear history is enforced).

## Branch protection expectations

`main` is protected:

* Direct pushes are disabled; every change goes through a pull request.
* The required status check is the aggregate **`CI gate`**, which depends on fmt, clippy,
  test (stable and MSRV), doc, wasm, deny, coverage and bench.
* At least one maintainer review is required; stale approvals are dismissed on new
  commits.
* Linear history is required (rebase or squash, no merge commits).
* Force pushes and branch deletion are disabled.

Rulesets are configured in GitHub settings and mirrored by the workflows. If a required
job is flaky, open an issue; do not bypass it.

## Reporting bugs, rules and features

Use the issue forms in [`.github/ISSUE_TEMPLATE/`](.github/ISSUE_TEMPLATE/):

* **Bug report** — wrong finding, crash, panic, or report regression.
* **Rule proposal** — a new SdI control. It requires the official control code, a source
  citation, the verification status (`[V]`/`[S]`/`[?]`) and a severity justification.
* **Feature request** — API, CLI, binding or performance work.

Security reports must not use these forms; follow [SECURITY.md](SECURITY.md).

## Licence

Contributions are accepted under the dual MIT/Apache-2.0 licence of the project (see
[README.md](README.md#license)). By submitting a pull request you confirm you have the
right to contribute the code under those terms.
