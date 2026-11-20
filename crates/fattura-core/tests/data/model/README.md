# Typed-model fixtures

These are **synthetic** test fixtures authored for E2.2 on 2026-11-20. They are
modelled on `.planning/sources/01-standard-versioni-struttura-xml.md` §6 and §8
and are not official AdE files. Official examples are not vendored yet; E3.1 owns
vendoring with checksums and an explicit redistribution decision.

* `minimal.xml` — one body with only the required elements: transmission data,
  seller, buyer, document header, one line and one VAT summary.
* `complete.xml` — every element of P1 §6 at least once, including both
  `Anagrafica` choices, the buyer's tax representative, all five correlated
  document groups, transport, vehicle, payment, attachments, an unknown element
  subtree and a `ds:Signature` subtree. It deliberately contains both
  `Denominazione` and `Nome`/`Cognome` in the buyer's tax representative so the
  parser's arms are all exercised; the XSD choice is enforced by E3.4, not here.
* `lotto.xml` — two `FatturaElettronicaBody` elements with different document
  numbers and line counts, for order and per-body scoping.

Do not edit these files to fit an implementation: a fixture change is a contract
decision and must be reviewed with the test that consumes it.
