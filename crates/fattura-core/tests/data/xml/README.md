# XML ingestion fixtures

These files are **synthetic** test fixtures, authored for E2.1 on 2026-11-18.
They are modelled on the structure in `.planning/sources/01-standard-versioni-struttura-xml.md`
§5.1–§6 and are *not* official AdE files. The official examples
(`IT01234567890_FPA01.xml`, `_FPA02`, `_FPA03`, `_FPR01`–`_FPR03`, and the 1.9.1
"Esempi fattura, hash e ricevute" bundle) are not vendored yet; E3.1 owns
vendoring with checksums and an explicit redistribution decision (PLAN §10 R5).

* `fpr12_mixed_prefix.xml` — `p:` prefix on the root only, unprefixed children
  (`elementFormDefault` is `unqualified`), `xsi:schemaLocation`, XML stylesheet
  processing instruction, comments, a predefined entity and a CDATA section.
* `fsm10_default_prefix.xml` — simplified root in a default namespace, `FSM10`
  transmission format, zero-rate line with `Natura`.

Do not edit these files to fit an implementation: a fixture change is a contract
decision and must be reviewed with the test that consumes it.
