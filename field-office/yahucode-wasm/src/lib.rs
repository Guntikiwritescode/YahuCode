//! A thin `wasm-bindgen` wrapper around the dependency-free YahuCode core.
//!
//! It exposes exactly one function to JS — `run_json(source, intercepts)` — which reuses
//! the core crate's [`yahucode::run_json`], returning the same
//! `{official, actual, discrepancies, …}` JSON the CLI's `--json` flag produces. The core
//! crate stays dependency-free by design; `wasm-bindgen` lives only in this wrapper.
//!
//! The interpreter never gains real DOM or network access: the only capability the host
//! supplies is the read-only intake channel (`intercepts`). All browser I/O — reading the
//! on-screen snippet, painting the tooltip/grey-out/badge — is done by the JS shim, never
//! by YahuCode.

use wasm_bindgen::prelude::*;

/// Run a YahuCode program and return the structured `{official, actual, discrepancies, …}`
/// JSON (the same projection the CLI `--json` flag emits). `intercepts` seeds the Field
/// Office intake channel (`intercept(n)`): the JS shim passes the on-screen snippet(s) it
/// scraped. On a parse/compile failure the JSON carries an `error` + `diagnostics` object,
/// so the JS caller always receives valid JSON.
#[wasm_bindgen]
pub fn run_json(source: &str, intercepts: Vec<String>) -> String {
    yahucode::run_json(source, intercepts)
}

#[cfg(test)]
mod tests {
    //! Native (non-wasm) unit tests: the wrapper is a pure pass-through, so it can be
    //! exercised without a wasm host. `cargo test` (native target) runs these.
    use super::run_json;

    #[test]
    fn wrapper_passes_through_to_the_core_run_json() {
        let out = run_json(
            "@operation(\"Guardian of Discourse\")\npost = intercept(0);\nflag(post);",
            vec!["a critical post".to_string()],
        );
        // The structured projection is returned; the intercepted content is סודי-only (I16),
        // so it appears on the ACTUAL projection but never on the OFFICIAL one.
        assert!(out.contains("\"official\""), "expected the --json shape: {out}");
        assert!(out.contains("content submitted for community context"));
        assert!(out.contains("a critical post"), "the ACTUAL face retains the content");
    }

    #[test]
    fn wrapper_returns_structured_error_on_bad_source() {
        let out = run_json("not a program", Vec::new());
        assert!(out.contains("\"error\""), "expected an error object: {out}");
    }
}
