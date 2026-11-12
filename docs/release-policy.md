# Release policy

This document defines how versions, the JSON contract, the MSRV and deprecations are
managed. The release roadmap is PLAN §8; the definition of done is PLAN §9. This policy is
normative for maintainers and is part of the governance set linked from
[`../README.md`](../README.md).

## Version sources

* The crate version is the workspace `version` in the root `Cargo.toml`
  (`[workspace.package]`). `fattura_core::VERSION` is derived from
  `CARGO_PKG_VERSION`, not hand-edited.
* The JSON report contract has its own version, `REPORT_SCHEMA_VERSION` (currently `"1"`).
* Published versions are immutable. Never reuse or move a version; crates.io and tags are
  append-only.

## Semantic versioning for the crate

The crate follows [Semantic Versioning](https://semver.org/):

* **Before 1.0 (`0.y.z`)** — `PATCH` is for compatible fixes and documentation; `MINOR` is
  for features and, when unavoidable, breaking API changes. A breaking change is never
  smuggled into a patch; it is listed under `Changed`/`Removed` in the changelog with a
  migration note.
* **From 1.0** — incompatible public API changes require a major version. Additive,
  backwards-compatible changes are minor; fixes are patch.
* Internal modules, generated tables, private items and test utilities are not part of the
  public API. `pub` items are, even when re-exported only through a module path.
* Removing a feature, changing a default feature, or raising the MSRV is a user-visible
  change and follows the rules below.

The planned release content is fixed by PLAN §8 (`0.1.0` = E1–E4 and E9.1–E9.3; `0.2.0` =
E5–E6; `0.3.0` = E7–E8; `0.4.0` = E10.1–E10.3; `1.0.0` = frozen contract and complete
in-scope rules). The roadmap may move in time, but a release must not claim content it does
not contain.

## JSON contract versioning

The JSON projection of `Report` is the cross-language contract used by the CLI, bindings and
downstream CI. It is versioned independently of the crate:

* `REPORT_SCHEMA_VERSION` is an integer-like string and denotes a **breaking** contract
  version. `1` is the initial contract (E9.1).
* A **schema bump is required** for removing or renaming a field, changing a type, unit,
  nullability or encoded meaning, adding a required field, changing the severity/level
  semantics of an existing code, or changing exit-code/report semantics in a way that
  breaks consumers.
* The following may ship within the same schema version: additive optional fields, new
  rule ids/findings, new `not_checked` entries, and clarifications that do not change
  encoded meaning. Consumers must ignore unknown fields.
* A crate **minor** release may carry a schema bump; a crate **patch** release never does.
  A schema bump requires a changelog entry, refreshed `insta` snapshots and a migration
  note. Downstream consumers should pin and assert on `schema_version`.
* When in doubt, bump: a silent semantic change is worse than a visible version change.

## MSRV policy

* MSRV is **Rust 1.85** (edition 2024), as recorded in `rust-version` and tested in CI
  (PLAN §8).
* CI tests both 1.85 and current stable. A feature that cannot build on 1.85 is not merged
  unless the MSRV is deliberately raised.
* **The MSRV may only increase in a minor release, never in a patch.** The increase is
  announced in `CHANGELOG.md` and the release notes, with the new `rust-version` and the
  reason (for example a required dependency or language feature).
* Before 1.0, an MSRV increase follows the pre-1.0 minor rule; from 1.0 it is a minor
  change, not a major one. In both cases it is never a surprise patch change.
* Dependencies must honour the MSRV; `cargo +1.85 check` is a required CI job.

## Deprecation policy

* **Rust API:** mark the item `#[deprecated(since = "x.y.z", note = "use …")]`, keep it
  working, and document the replacement in rustdoc and the changelog. Keep a deprecated
  item for **at least one minor release** before removal. From 1.0, removal happens only
  in a major release; before 1.0, removal happens at a minor boundary after the
  deprecation has shipped.
* **JSON contract:** add the replacement field/meaning first and document the old one as
  deprecated in the schema description and changelog. Removal or semantic change is a
  schema-breaking change and follows the rules above; a field cannot disappear in a patch.
* **CLI:** deprecated flags keep working for at least one minor release, print a warning
  to stderr, and keep deterministic exit codes until removal.
* Deprecations are not silent: every deprecation has a `CHANGELOG.md` entry under
  `Deprecated`.

## Release checklist

Run this checklist from a clean checkout of the release commit. Do not tag before every
item is true.

1. **Scope** — the release matches PLAN §8 and the milestone; deferred work is listed, not
   implied to be present.
2. **Gates** — the aggregate `CI gate` is green: fmt, clippy, test (stable and MSRV
   1.85), doc (`-D warnings`), wasm, `cargo-deny`, 100% coverage and bench. No gate was
   weakened or skipped for the release.
3. **Versions** — workspace `version` bumped; `Cargo.lock` updated; no hand-edited
   `VERSION`; `REPORT_SCHEMA_VERSION` decision recorded (bumped or explicitly unchanged).
4. **Contract** — JSON snapshots refreshed; golden/corpus tests pass; migration notes
   written if the schema changed; `schema_version` asserted in tests.
5. **Changelog** — `[Unreleased]` moved to a dated release section; entries name the
   official control codes and PLAN/source references; breaking changes, deprecations,
   MSRV changes and security fixes are called out.
6. **Docs** — `cargo doc -D warnings` is clean; rustdoc examples compile; README status and
   links are current; ADRs/release policy are updated; **no publication placeholders
   remain** (including the `conduct@example.invalid` contact in `CODE_OF_CONDUCT.md`).
7. **Packaging** — `cargo publish --dry-run` for each published crate; verify the package
   contents exclude `.planning/` and any fixture whose redistribution is unresolved; check
   licence files and `readme` paths.
8. **Tag and publish** — create a signed annotated tag `vX.Y.Z`; publish crates in
   dependency order; create the GitHub release with release notes, the schema version, the
   MSRV, and checksums for any binary/WASM artifacts.
9. **Post-release** — verify docs.rs, a clean install, the CLI, and the WASM/bindings
   artifacts when they exist; close the milestone; open follow-ups for anything deferred.
10. **Security** — if the release contains a security fix, publish the GitHub Security
    Advisory and follow [`SECURITY.md`](../SECURITY.md). Do not disclose before the fix is
    available.

Release managers must not push directly to a protected branch: the release commit goes
through the same pull-request gates as any other change.

## `[?]` policy

The research sources mark claims `[V]` (verified against the primary source), `[S]`
(secondary source) and `[?]` (to be verified). For releases:

* A `[?]` item is **never** emitted as an `error` finding. It may be a `warning` or `info`,
  or it may be absent and listed in `not_checked` where appropriate.
* Every `[?]` item has a `needs-verification` GitHub issue. Verification means reading the
  official primary source (Allegato A / Elenco controlli / XSD as applicable), recording
  the document, version, section and date, and adding positive and negative tests.
* Promotion to `error` requires the primary-source citation, the exact official control
  code, a severity justification (why a false positive is acceptable) and reviewer
  agreement. Differential behaviour of another implementation is not proof; the SdI is the
  oracle (PLAN §7.3).
* A release must not silently turn a `[?]` warning into an error. If verification lands in
  a release, the changelog says so under `Changed` and the issue is closed with the
  citation.
* When an official source is ambiguous, keep the finding as a warning and document the
  ambiguity. Over-validation is worse than under-validation (PLAN §10, R6).

## Security releases

Security fixes follow [`SECURITY.md`](../SECURITY.md): private report, coordinated
disclosure, patch release, GitHub Security Advisory, and a `Security` entry in
`CHANGELOG.md`. A security release may be cut outside the normal cadence, but it still
passes the required gates and still follows the MSRV/schema rules (a security patch does
not raise the MSRV or break the JSON contract).
