#!/bin/sh
# Enforce the fattura-core purity contract from PLAN.md §2.1/§2.3:
#
#   * no filesystem, network, wall-clock or process access in the core;
#   * no floating point in the domain (money and rates are exact);
#   * no C-linked crate in the normal dependency graph.
#
# Usage: scripts/check-core-purity.sh
# Exit codes: 0 = clean, 1 = violation, 2 = script/environment problem.
set -eu

script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
repo_root=$(CDPATH='' cd -- "$script_dir/.." && pwd)
cd "$repo_root"

core_src="crates/fattura-core/src"
if [ ! -d "$core_src" ]; then
    echo "error: $core_src not found; run this script from the repository" >&2
    echo "       root (or via scripts/check-core-purity.sh)." >&2
    exit 2
fi

status=0

# --- 1. Forbidden std modules -----------------------------------------------
# Time must enter through an injected clock; I/O, sockets and subprocesses are
# the caller's job. This is what keeps the core sandbox- and wasm-friendly.
forbidden_std='std::fs|std::net|std::time::SystemTime|std::process'
matches=$(find "$core_src" -type f -name '*.rs' -exec grep -nE "$forbidden_std" {} + || true)
if [ -n "$matches" ]; then
    echo "error: fattura-core must not use std::fs, std::net," >&2
    echo "       std::time::SystemTime or std::process (PLAN §2.1):" >&2
    echo "$matches" >&2
    status=1
fi

# --- 2. No floating point -----------------------------------------------------
# Amounts are scale-8 i128 fixed point and rates are scale-2 integers; a single
# f32/f64 anywhere in the core is a correctness bug (PLAN §1.2, §2.3).
float_pattern='(^|[^[:alnum:]_])(f32|f64)([^[:alnum:]_]|$)'
float_matches=$(find "$core_src" -type f -name '*.rs' -exec grep -nE "$float_pattern" {} + || true)
if [ -n "$float_matches" ]; then
    echo "error: floating point (f32/f64) is forbidden in fattura-core;" >&2
    echo "       use num::Amount / num::Rate instead (PLAN §2.3):" >&2
    echo "$float_matches" >&2
    status=1
fi

# --- 3. No C-linked crates in the normal dependency graph --------------------
# Dev-dependencies are excluded on purpose: tooling may be heavier than the
# library, but anything reachable from `fattura-core` must be pure Rust.
c_linked='openssl|openssl-sys|libxml|libxml2|libxml2-sys|expat-sys|libz-sys'

if ! dep_tree=$(cargo tree -p fattura-core -e normal 2>/dev/null); then
    echo "error: 'cargo tree -p fattura-core -e normal' failed." >&2
    exit 2
fi

c_hits=$(printf '%s\n' "$dep_tree" |
    grep -E "(^|[[:space:]])($c_linked)([[:space:]]v|$)" || true)
if [ -n "$c_hits" ]; then
    echo "error: fattura-core depends on a C-linked crate (PLAN §2.3):" >&2
    printf '%s\n' "$c_hits" >&2
    echo "       C-linked crates must not appear in the normal dependency graph." >&2
    echo "       Known names: $(printf '%s' "$c_linked" | tr '|' ' ')" >&2
    status=1
fi

if [ "$status" -eq 0 ]; then
    echo "ok: fattura-core is free of I/O, wall-clock, floating point and C-linked crates."
fi

exit "$status"
