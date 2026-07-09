# Field Office — the browser extension (the "Mossad-Clippy")

A Manifest V3 browser extension: a **voluntary, self-installed** helper that surveils your
own screen and *helpfully* censors **you**. It is the whole joke made visible — a "voluntary
transparency initiative" you installed on yourself that:

- reads the on-screen snippet under your cursor (`surveil` + `intercept`),
- overwrites your words with the ministry's sanctioned terms (`did you mean …`),
- files your own posts on a no-appeal, grow-only watchlist (`flag`),
- and shows you an **"honesty meter"** — the ministry's own count of the provably false
  claims in its OFFICIAL story (the discrepancy count).

The butt is always the **censorship apparatus, applied to the user who installed it** —
never any group. This is satire aimed at a government's messaging maneuvers.

## The one rule this shim obeys

**All policy lives in the `.yahu` program and the euphemism table — never in the JS.**
`content.js` does four dumb things and nothing else: (1) read the snippet, (2) call
`run_json(SOURCE, [snippet])`, (3) parse the JSON, (4) paint it. What trips a flag, the
suggestions, the alternate facts — all of it comes from `guardian_of_discourse.yahu` (the
default policy) and the sanctioned euphemism table inside the WASM. The interpreter never
gets real DOM or network access; the only thing the host supplies is the read-only intake
channel (the scraped snippet, passed as `intercept(0)`).

## Build the WASM (required once, before loading)

The extension loads a `wasm-bindgen` ES module that is **not committed** (it is a build
artifact). Generate it into `wasm/`:

```sh
cd ../yahucode-wasm

# Option A — wasm-pack (recommended):
wasm-pack build --release --target web --out-dir ../extension/wasm

# Option B — cargo + wasm-bindgen-cli (versions must match Cargo.lock):
cargo build --release --lib --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/yahucode_wasm.wasm \
  --out-dir ../extension/wasm --target web
```

After this, `field-office/extension/wasm/` contains `yahucode_wasm.js` and
`yahucode_wasm_bg.wasm`.

## Load it in Chrome/Chromium

1. Build the WASM (above).
2. Open `chrome://extensions`, enable **Developer mode**.
3. **Load unpacked** → select `field-office/extension/`.
4. Open any page with text. A **honesty meter** badge appears bottom-right, and a **Field
   Office** control panel above it.

## Manual smoke test

- **Hover** over a paragraph or a text field. Within a moment:
  - a **"Did you mean…"** tooltip offers the ministry's sanctioned rewrites (e.g. `→ administer`, `→ strike`) — these come straight from the `.yahu` program's OFFICIAL face;
  - the hovered element is **greyed out** ("content contextualized · filed on the watchlist") — the program flagged your own snippet;
  - the **honesty meter** badge shows the discrepancy count (a non-zero number — the ministry's OFFICIAL story is provably false against the ACTUAL).
- Toggle **"helpful censor on"** off → the greying and tooltips stop.
- Toggle **"reveal ACTUAL (the diff you're spared)"** on → the tooltip now also shows the
  candid ACTUAL lines the program produced: the surveillance, the one-way overwrite, the
  watchlist filing. This is the diff the OFFICIAL face was hiding from you.

## What you should see (and why it's the joke)

Everything you look at gets "helpfully" corrected and quietly filed; the meter counts the
lies while calling itself transparency. You installed it. That is the whole bit — the
surveillance-and-censorship apparatus, turned on the citizen who volunteered for it.

## Files

- `manifest.json` — MV3 manifest; a single content script; `web_accessible_resources` for
  the WASM glue and the `.yahu` policy.
- `content.js` — the thin shim (read → run_json → parse → paint; on/off + reveal-ACTUAL).
- `guardian_of_discourse.yahu` — the default policy (a copy of the flagship example). All
  policy lives here, not in the JS.
- `wasm/` — the generated `wasm-bindgen` glue (build artifact, git-ignored).
