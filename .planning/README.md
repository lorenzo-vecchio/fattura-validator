# `.planning/` — research inputs and implementation plan

This folder is the design record for **fattura-validator**. It is not part of the
published crate; it exists so that every implementation decision can be traced
back to a source.

| File | What it is |
| --- | --- |
| `PLAN.md` | The implementation plan: architecture, work breakdown, acceptance criteria, risks. |
| `sources/01-standard-versioni-struttura-xml.md` | Part 1/3 — standards, versions, XML structure, types, file constraints, signature. |
| `sources/02-codifiche-code-list.md` | Part 2/3 — complete official code lists (enumerations, deprecations, quirks). |
| `sources/03-validazione-offline-controlli.md` | Part 3/3 — offline validation: control catalog, arithmetic, algorithms, tables, test strategy. |

## Provenance of the sources

The three source documents were produced on **17 September 2026** as a desk
research report over the official Agenzia delle Entrate material (XSD 1.2.3 /
1.0.2, Allegato A — Specifiche tecniche 1.9.1, Elenco controlli v1.8 and v2.0,
CU instructions). They carry their own reliability legend:

* `[V]` — verified directly against the primary source.
* `[S]` — reported by a reliable secondary source, not line-by-line verified.
* `[?]` — to be verified **before** being encoded as a blocking rule.

Every `[?]` item is tracked as a GitHub issue labelled `needs-verification`.
Nothing marked `[?]` may be implemented as a hard `Error`.

## Ground rule

The validator never claims "this invoice is valid". It reports conformance to
the levels it can actually check offline (L0–L3) and explicitly lists what it
did not check (L4–L5). See `PLAN.md` §1.
