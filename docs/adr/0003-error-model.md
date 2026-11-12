# ADR 0003 — Layered report instead of a boolean verdict

* **Status:** Accepted
* **Date:** 2026-11-12
* **Decision owners:** project maintainers
* **Supersedes:** —
* **Superseded by:** —

## Context

An offline validator cannot answer "is this invoice valid?" in the sense used by the SdI.
Several controls need online registries (Anagrafica tributaria, IndicePA, channel registry,
VIES, AdE declarations, historical duplicates) or revocation data (CRL/OCSP); the exact
list is in P3 §7 and PLAN §5.2. Returning a single boolean would therefore either
overclaim ("valid" although L4/L5 were never checked) or underclaim ("invalid" because an
online check was unavailable). It would also discard the distinction between an error, a
warning, an informational finding and a check that was not performed.

The product definition is explicitly a layered report: L0 file, L1 well-formedness, L2 XSD,
L3 content rules, L4 external registries (**not checked**), L5 lifecycle (**not checked**)
(PLAN §1.1). Special rules apply: `[?]` research items are never hard errors (PLAN §11),
the validator never repairs input, and signature status is `Unverifiable` rather than
`Valid` when revocation/trust is unavailable (PLAN §1.2).

## Decision

The public result is a **structured, layered report**, never a boolean:

* `Report` carries the contract version, document information, a per-level summary,
  `findings`, and `not_checked` entries for L4/L5 with their official codes
  (PLAN §4, §5.2).
* `Level` is `L0`–`L5`; only L0–L3 can be `Checked`/`Passed`/`Findings(n)`. L4 and L5 are
  represented as **not checked**, with a reason.
* Every `Finding` carries a stable rule id, optional official SdI code, severity
  (`error`/`warning`/`info`), level, element path and human message/hint.
* Severity is not a single global verdict. Consumers choose a severity profile
  (`Sdi`/`Recommended`/`Permissive`, PLAN §3) and decide policy; the library never says
  "valid".
* The JSON projection of `Report` is the cross-language contract and is independently
  versioned by `REPORT_SCHEMA_VERSION`; changes require a schema decision and snapshot
  updates.
* `[?]` items may be `warning`/`info`, never `error`, until verified against a primary
  source.

## Alternatives rejected

### `bool` (or `Result<(), Error>`)

Rejected: it cannot represent partial evidence, multiple simultaneous findings, or the
L4/L5 "not checked" state. It forces an all-or-nothing answer and encourages the false
claim that an offline tool has established legal validity. It also loses severity and
element paths needed for actionable output.

### Panics or a single exception

Rejected: hostile input and multiple findings are normal, not exceptional. The core is
embedded in WASM/FFI where unwinding is undesirable or unavailable; panics are denied on
user input. An exception-style API also cannot return a partial report.

### One flat severity for the whole file

Rejected: it cannot express warnings versus errors, informational E-series advice, or
`[?]` findings that must not block. It also prevents severity profiles and hides which
levels were actually checked.

### A flat list of findings without level/not-checked state

Rejected: consumers could not distinguish "checked and passed" from "not checked", which
is the central product guarantee.

## Consequences

* **Positive:** the report is honest about the offline boundary, machine-readable and
  suitable for CI policies, and preserves every finding's provenance.
* **Positive:** new checks and levels can be added without changing the "no boolean"
  contract; schema versioning makes evolution explicit.
* **Negative:** the API and JSON are larger than a boolean, and consumers must inspect
  `levels`/`not_checked` rather than a single field.
* **Negative:** severity/schema changes are user-visible and require changelog notes,
  schema decisions and snapshot refreshes.
* **Follow-up:** E1.2 defines the diagnostic types; E9.1 freezes the public API and JSON
  contract. `[?]` verification is tracked as `needs-verification` issues.

## References

* PLAN §1.1–§1.3 (levels, what is not done, severities), §3 (date model and profiles),
  §4 (public API sketch), §5.2 (out-of-scope codes), §11 (open questions), §9 (DoD).
* P3 §7 (what cannot be validated offline and why).
* [`CHANGELOG.md`](../../CHANGELOG.md), [`docs/release-policy.md`](../release-policy.md).
