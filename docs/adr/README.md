# Architecture Decision Records

An ADR records a decision that is expensive to reverse: the context, the choice, what was
rejected and the consequences. The implementation plan
([`../../.planning/PLAN.md`](../../.planning/PLAN.md)) remains the source of truth for
scope and work breakdown; an ADR explains **why** a cross-cutting choice was made.

## Index

| ADR | Title | Status |
| --- | --- | --- |
| [0001](0001-pure-rust.md) | Pure Rust, with no C toolchain | Accepted |
| [0002](0002-static-schema-codegen.md) | Compile the vendored XSDs to static tables | Accepted |
| [0003](0003-error-model.md) | Layered report instead of a boolean verdict | Accepted |

## Template

Create `docs/adr/NNNN-short-slug.md` with the next zero-padded number:

```markdown
# ADR NNNN — Title

* **Status:** Proposed | Accepted | Deprecated | Superseded by ADR NNNN
* **Date:** YYYY-MM-DD
* **Decision owners:** project maintainers
* **Supersedes:** — or ADR NNNN
* **Superseded by:** — or ADR NNNN

## Context

What forces the decision? Constraints, plan sections, official sources and risks.

## Decision

What is chosen, stated so an implementer can follow it.

## Alternatives rejected

Each alternative and why it was not chosen.

## Consequences

Positive and negative outcomes, follow-ups and what must be revisited.

## References

PLAN sections, source sections (`P1`/`P2`/`P3 §…`), issues and related ADRs.
```

## Process

* Proposed ADRs are reviewed in the pull request that needs the decision; the status
  becomes Accepted when merged.
* Do not rewrite an Accepted ADR to change its decision. Add a new ADR and mark the old one
  `Superseded by ADR NNNN`.
* Cite PLAN sections and official sources; do not invent rules or behaviour. Decisions
  about uncertain (`[?]`) rules follow the policy in
  [`../release-policy.md`](../release-policy.md).
* Keep the template headings (`Context`, `Decision`, `Alternatives rejected`,
  `Consequences`) consistent across ADRs.
