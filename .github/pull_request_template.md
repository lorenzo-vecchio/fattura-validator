# Pull request

## Summary

<!-- What does this change and why? Reference the issue (e.g. Refs #15). -->

## Linked issue / PLAN section

<!-- e.g. E1.5, PLAN §9, P3 §2.2. Every rule needs a source citation. -->

## What changed

<!-- Bullet list. Call out public API, JSON contract, severity or schema changes. -->

## Report / JSON contract

- [ ] No `Report`/JSON change
- [ ] JSON change, and `REPORT_SCHEMA_VERSION` was bumped (breaking) or explicitly kept
      (additive/optional only) with a migration note
- [ ] `insta` snapshots refreshed and reviewed (no blind `cargo insta accept`)

## Definition of done (PLAN §9)

- [ ] **Tests** — positive and negative cases for every rule/behaviour; boundary and
      property tests where arithmetic is involved.
- [ ] **Coverage** — new/changed lines are covered; coverage did not decrease; any
      generated-code exclusion is justified.
- [ ] **Source citation** — each rule cites the official control code and source
      (`P3 §…`, PLAN section) in a doc comment; no invented rules.
- [ ] **No panics** — no `unwrap`/`expect` outside tests; no panic on user input; hostile
      input path considered.
- [ ] **No `f64`** — domain arithmetic uses the exact `num` core; no float arithmetic or
      float comparison in validation logic.
- [ ] **No C / OpenSSL / libxml2** — no new C dependency; `wasm32-unknown-unknown` still
      builds; no runtime XSD engine.
- [ ] **No runtime regex, DOM or network/file I/O in `fattura-core`** — streaming,
      borrowed bytes; clock and input are injected; no DNS/OCSP/AdE/IPA/VIES.
- [ ] **`[?]` policy** — unverified items are warnings/info, never `error`, and have a
      `needs-verification` issue.
- [ ] **Snapshots / corpus** — fixtures include provenance and checksums; snapshot diff
      reviewed; no unclear-redistribution fixture added.
- [ ] **Docs** — rustdoc and README updated for user-visible behaviour; `CHANGELOG.md`
      updated; ADR/release policy updated if a cross-cutting decision changed.
- [ ] **Gates** — `CI gate` is green: fmt, clippy, test (stable + MSRV 1.85), doc,
      wasm, `cargo-deny`, coverage and bench.

## Reviewer notes

<!-- Decisions taken, trade-offs, follow-ups. Mention anything intentionally deferred. -->

## Security

- [ ] This PR does not introduce network access, telemetry, or a new trust/revocation
      assumption.
- [ ] If it changes signature status or trust injection, the `Unverifiable`/`Valid`
      boundary is documented and tested.
