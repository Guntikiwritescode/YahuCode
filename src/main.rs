//! YahuCode CLI entry point.
//!
//! During PR1 the foundational base (core model + euphemism) is being converged; the
//! full CLI (arg parsing, file IO, `--json`, the two/three-face render) lands with the
//! emitter in PR2. For now this is a placeholder that prints the banner so the binary
//! target builds cleanly.

fn main() {
    println!("YahuCode — foundational base (see `cargo test`).");
}
