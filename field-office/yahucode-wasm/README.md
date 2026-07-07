# yahucode-wasm — the browser wrapper

A **separate, thin** `wasm-bindgen` wrapper around the dependency-free YahuCode core. It
exposes exactly one function to JS:

```rust
run_json(source: &str, intercepts: Vec<String>) -> String
```

which reuses the core crate's `yahucode::run_json` and returns the same
`{official, actual, restricted, discrepancies, notes, …}` JSON the CLI's `--json` flag
produces. On a parse/compile failure it returns `{"error": …, "diagnostics": [ … ]}`.

## Why a separate crate

The core `yahucode` crate is **dependency-free by design** (its `Cargo.toml` has no
`[dependencies]`). `wasm-bindgen` lives *only* here, so that invariant is preserved. This
crate depends on the core by path and on `wasm-bindgen`, and is built separately — the core
crate's gates (`cargo build && cargo test && cargo clippy`) never pull in `wasm-bindgen`.

## Verify the raw wasm build

```sh
# From this directory. The core lib also builds for wasm unchanged (no std feature blocks it).
cargo build --lib --target wasm32-unknown-unknown
```

## Build the JS glue for the extension

The extension loads the `wasm-bindgen`-generated ES module. Generate it with either
`wasm-pack` or `wasm-bindgen-cli` (the CLI version must match the `wasm-bindgen` crate
version in `Cargo.lock`):

```sh
# Option A — wasm-pack (recommended): emits the .wasm + .js glue into pkg/
wasm-pack build --release --target web

# Option B — cargo + wasm-bindgen-cli (matching versions)
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen \
  target/wasm32-unknown-unknown/release/yahucode_wasm.wasm \
  --out-dir ../extension/wasm --target web
```

Either way, copy the generated `yahucode_wasm.js` and `yahucode_wasm_bg.wasm` into
`field-office/extension/wasm/` (git-ignored — it is a build artifact). The extension's
content script imports them:

```js
import init, { run_json } from './wasm/yahucode_wasm.js';
await init();
const json = run_json(SOURCE, [snippet]);
```

## Native tests

The wrapper is a pure pass-through, so it is unit-tested on the native target without a
wasm host:

```sh
cargo test
```
