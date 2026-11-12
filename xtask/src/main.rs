//! Code generation and table maintenance tasks.
//!
//! `cargo xtask <command>` regenerates the static schema model and effective-dated
//! tables from the vendored XSDs. Output is committed, never generated at build
//! time, so builds stay deterministic and offline.

fn main() {
    eprintln!("xtask: no tasks implemented yet (see E3.2, E4.1)");
}
