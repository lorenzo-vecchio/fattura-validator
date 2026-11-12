# ADR 0001 — Pure Rust, with no C toolchain

* **Status:** Accepted
* **Date:** 2026-11-12
* **Decision owners:** project maintainers
* **Supersedes:** —
* **Superseded by:** —

## Context

`fattura-validator` must validate invoices offline and be embeddable from WASM, a C ABI,
Python and Node. The core is specified as I/O-free, network-free and clock-injected
(PLAN §1.2, §2.1). A C dependency would complicate cross-compilation, static linking,
Windows/musl builds, sandboxed WASM, supply-chain review and reproducible builds. The
project also processes untrusted files, so memory-safety and a small attack surface matter.
PLAN §2.3 makes "no C toolchain, no OpenSSL, no libxml2" a hard rule.

## Decision

The workspace is **pure Rust**:

* `fattura-core` uses only Rust crates and keeps `#![forbid(unsafe_code)]`.
* XML reading uses a streaming pure-Rust reader, not libxml2 and not a DOM.
* XSD conformance uses generated static tables, not a runtime C XSD engine
  ([ADR 0002](0002-static-schema-codegen.md)).
* Signature, archive and DER/CMS work uses pure-Rust crates behind features; no OpenSSL.
* Dependencies must build for `wasm32-unknown-unknown`; anything that breaks it is
  rejected. `unsafe` is not permitted outside the optional FFI crate.
* An external tool such as `xmllint` may be used in CI as a **test oracle**, but never as a
  build or runtime dependency.

## Alternatives rejected

### libxml2 bindings (or `xmllint` library mode)

Rejected: brings a C toolchain, a DOM, a large runtime XSD engine and platform-specific
build/link problems; cannot be shipped to WASM; expands the parser attack surface and the
supply chain. `xmllint` remains useful only as a differential-test oracle.

### OpenSSL (or a system TLS stack)

Rejected: the library performs no network I/O, so it needs no TLS. For signature and
digest work, pure-Rust `rsa`/`p256`/`sha1`/`sha2` crates avoid system-library variance,
CVE coupling and cross-compilation failures. OpenSSL would also be unusable in the WASM
target.

### A C XSD engine

Rejected for the same reasons, plus performance and determinism: a general XSD engine
interprets the schema at runtime, while the invoice XSDs are small and known. They are
compiled ahead of time instead (ADR 0002).

## Consequences

* **Positive:** one toolchain, `cargo build` works without system packages, WASM and FFI
  are viable, memory safety is enforced by Rust, builds are reproducible and auditable.
* **Positive:** the dependency graph has no TLS/network client to abuse (see
  [`SECURITY.md`](../../SECURITY.md)).
* **Negative:** the project owns the XML reader, the schema codegen and the local crypto
  integration. Correctness must be proven with differential tests against `xmllint` and
  reference vectors, not inherited from a mature C library.
* **Negative:** some ecosystem crates that wrap C cannot be used; a future requirement
  that genuinely needs one must go through a new ADR, not a drive-by dependency.
* **Follow-up:** keep the `wasm32` and `cargo-deny` CI jobs as required checks; document
  any pure-Rust limitation as a known difference rather than silently accepting it.

## References

* PLAN §1.2 (no network, no repair), §2.1 (workspace and core constraints), §2.3
  (dependency policy and hard rules), §5.1 (signature scope), §10 R3/R5.
* P3 §7 (what cannot be validated offline).
* [ADR 0002 — static schema code generation](0002-static-schema-codegen.md).
