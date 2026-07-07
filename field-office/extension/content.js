/*
 * Field Office — the content script (the thin JS shim).
 *
 * This part CANNOT be YahuCode. It does four dumb things and nothing else:
 *   (1) read the on-screen snippet under the cursor,
 *   (2) call the WASM run_json(SOURCE, [snippet]),
 *   (3) parse the JSON,
 *   (4) paint it: OFFICIAL "did you mean…" as a tooltip, a flag verdict as a grey-out,
 *       and the discrepancy count as a small "honesty meter" badge.
 *
 * ALL policy — what trips a flag, the suggestions, the alternate facts — lives in the
 * .yahu program (guardian_of_discourse.yahu) and the euphemism table, NEVER in this JS.
 * The interpreter never gains real DOM or network access: the only capability the host
 * supplies is the read-only intake channel (the scraped snippet, passed as intercept(0)).
 *
 * The whole joke made visible: a "voluntary transparency initiative" you installed on
 * yourself that reads your screen, overwrites your words with the ministry's, files your
 * posts on a no-appeal watchlist, and shows you an "honesty meter" of its own lies. The
 * butt is the censorship apparatus, applied to the user who installed it — never any group.
 */
(async () => {
  "use strict";

  // Idempotence: never inject twice into the same document.
  if (window.__fieldOfficeInstalled) return;
  window.__fieldOfficeInstalled = true;

  const state = {
    enabled: true, // the on/off toggle
    revealActual: false, // the "reveal ACTUAL" debug switch (shows the diff you're spared)
    run_json: null,
    SOURCE: null,
    lastDiscrepancies: 0,
  };

  // ── (0) load the WASM interpreter + the .yahu policy, or degrade harmlessly ──
  try {
    const glueUrl = chrome.runtime.getURL("wasm/yahucode_wasm.js");
    const wasmUrl = chrome.runtime.getURL("wasm/yahucode_wasm_bg.wasm");
    const programUrl = chrome.runtime.getURL("guardian_of_discourse.yahu");
    const mod = await import(glueUrl);
    await mod.default({ module_or_path: wasmUrl }); // wasm-bindgen --target web init()
    state.run_json = mod.run_json;
    state.SOURCE = await (await fetch(programUrl)).text();
  } catch (e) {
    // The WASM glue is not built. Build it (see field-office/yahucode-wasm/README.md), then
    // reload. The badge says so; nothing else runs.
    console.warn("[Field Office] WASM not loaded — build the glue first:", e);
    mountUi();
    setBadge("—", "Field Office: build the WASM glue (see yahucode-wasm/README.md)");
    return;
  }

  mountUi();

  // ── the analysis: run the .yahu policy with the snippet as intercept(0) ──
  // Returns { suggestions:[…], flagged:bool, discrepancies:int, actual:[…] } — a pure
  // projection of the program's output. The JS decides NOTHING about policy; it only reads
  // the two faces the program produced.
  function analyze(snippet) {
    let parsed;
    try {
      parsed = JSON.parse(state.run_json(state.SOURCE, [snippet]));
    } catch (e) {
      console.warn("[Field Office] run_json failed:", e);
      return null;
    }
    if (parsed.error) {
      console.warn("[Field Office] policy did not compile:", parsed.diagnostics);
      return null;
    }
    const official = parsed.official || [];
    // The suggestions are whatever OFFICIAL "did you mean `X`?" lines the program emitted.
    const suggestions = official
      .map((l) => /did you mean `(.+?)`\?/.exec(l))
      .filter(Boolean)
      .map((m) => m[1]);
    // The flag verdict is the program's own OFFICIAL "content contextualized" line — the
    // euphemism for "greyed and filed on the watchlist". The .yahu decides this, not the JS.
    const flagged = official.some((l) => l.includes("content contextualized"));
    return {
      suggestions,
      flagged,
      discrepancies: parsed.discrepancies || 0,
      actual: parsed.actual || [],
    };
  }

  // ── (1) read the on-screen snippet under the cursor ──
  function snippetOf(el) {
    if (!el) return "";
    // A form field the citizen is typing into, else the element's visible text.
    const raw =
      "value" in el && typeof el.value === "string"
        ? el.value
        : el.innerText || el.textContent || "";
    return raw.trim().replace(/\s+/g, " ").slice(0, 280);
  }

  let hoverTimer = null;
  document.addEventListener(
    "mouseover",
    (ev) => {
      if (!state.enabled) return;
      const el = ev.target;
      if (!(el instanceof HTMLElement)) return;
      if (el.closest(".field-office-ui")) return; // never surveil our own UI
      clearTimeout(hoverTimer);
      hoverTimer = setTimeout(() => handle(el, ev.clientX, ev.clientY), 120);
    },
    true
  );

  document.addEventListener(
    "mouseout",
    () => {
      clearTimeout(hoverTimer);
      hideTooltip();
    },
    true
  );

  function handle(el, x, y) {
    const snippet = snippetOf(el);
    if (snippet.length < 3) return;
    const verdict = analyze(snippet);
    if (!verdict) return;

    // ── (4a) the honesty meter badge = the discrepancy count (the whole joke made visible) ──
    state.lastDiscrepancies = verdict.discrepancies;
    setBadge(
      String(verdict.discrepancies),
      `honesty meter — the ministry's own count of provably false OFFICIAL claims in this run`
    );

    // ── (4b) grey-out: the program flagged the citizen's own snippet — greyed, no appeal ──
    if (verdict.flagged) {
      el.classList.add("field-office-flagged");
      el.title = "Field Office: content contextualized (added to the watchlist — no appeal)";
    }

    // ── (4c) the "did you mean…" tooltip (+ the ACTUAL diff you're spared, if revealed) ──
    showTooltip(el, x, y, verdict);
  }

  // ───────────────────────── the painting (pure UI) ─────────────────────────

  function showTooltip(el, x, y, verdict) {
    const tip = document.getElementById("field-office-tooltip");
    if (!tip) return;
    const lines = [];
    if (verdict.suggestions.length) {
      lines.push(
        `<div class="fo-head">Did you mean…</div>` +
          verdict.suggestions
            .map((s) => `<div class="fo-suggest">→ <b>${escapeHtml(s)}</b></div>`)
            .join("")
      );
    }
    if (verdict.flagged) {
      lines.push(`<div class="fo-flag">⚑ content contextualized · filed on the watchlist</div>`);
    }
    if (state.revealActual) {
      // The "reveal ACTUAL" debug switch — show the uglier truth the citizen is spared.
      lines.push(
        `<div class="fo-head fo-actual">ACTUAL (what it really did):</div>` +
          verdict.actual
            .map((l) => `<div class="fo-actual-line">${escapeHtml(l)}</div>`)
            .join("")
      );
    }
    if (!lines.length) {
      hideTooltip();
      return;
    }
    tip.innerHTML = lines.join("");
    tip.style.display = "block";
    // Keep the tooltip on-screen.
    const pad = 12;
    tip.style.left = Math.min(x + pad, window.innerWidth - tip.offsetWidth - pad) + "px";
    tip.style.top = Math.min(y + pad, window.innerHeight - tip.offsetHeight - pad) + "px";
  }

  function hideTooltip() {
    const tip = document.getElementById("field-office-tooltip");
    if (tip) tip.style.display = "none";
  }

  function setBadge(count, title) {
    const badge = document.getElementById("field-office-badge");
    if (!badge) return;
    badge.querySelector(".fo-badge-count").textContent = count;
    badge.title = title;
  }

  // ── mount the UI: the honesty-meter badge, the tooltip, and the control panel ──
  function mountUi() {
    injectStyles();

    const badge = document.createElement("div");
    badge.className = "field-office-ui";
    badge.id = "field-office-badge";
    badge.innerHTML =
      `<span class="fo-badge-label">honesty&nbsp;meter</span>` +
      `<span class="fo-badge-count">0</span>`;
    document.documentElement.appendChild(badge);

    const tip = document.createElement("div");
    tip.className = "field-office-ui";
    tip.id = "field-office-tooltip";
    document.documentElement.appendChild(tip);

    // The control panel — on/off toggle + "reveal ACTUAL" debug switch (reinforces the satire).
    const panel = document.createElement("div");
    panel.className = "field-office-ui";
    panel.id = "field-office-panel";
    panel.innerHTML =
      `<div class="fo-panel-title">Field Office</div>` +
      `<label><input type="checkbox" id="fo-enabled" checked> helpful censor on</label>` +
      `<label><input type="checkbox" id="fo-reveal"> reveal ACTUAL (the diff you're spared)</label>`;
    document.documentElement.appendChild(panel);

    panel.querySelector("#fo-enabled").addEventListener("change", (e) => {
      state.enabled = e.target.checked;
      if (!state.enabled) {
        hideTooltip();
        document
          .querySelectorAll(".field-office-flagged")
          .forEach((n) => n.classList.remove("field-office-flagged"));
      }
    });
    panel.querySelector("#fo-reveal").addEventListener("change", (e) => {
      state.revealActual = e.target.checked;
    });
  }

  function injectStyles() {
    const css = `
      .field-office-ui { all: revert; font-family: system-ui, sans-serif; z-index: 2147483647; }
      #field-office-badge {
        position: fixed; right: 14px; bottom: 14px;
        display: flex; align-items: center; gap: 8px;
        background: #10243a; color: #eaf2ff; padding: 6px 12px; border-radius: 999px;
        font-size: 12px; box-shadow: 0 2px 12px rgba(0,0,0,.35); cursor: default;
      }
      #field-office-badge .fo-badge-count {
        background: #2f6df0; color: #fff; border-radius: 999px;
        min-width: 20px; text-align: center; padding: 1px 6px; font-weight: 700;
      }
      #field-office-tooltip {
        position: fixed; display: none; max-width: 360px;
        background: #0e1b2b; color: #e6eefc; border: 1px solid #2f6df0;
        border-radius: 8px; padding: 8px 10px; font-size: 12px; line-height: 1.35;
        box-shadow: 0 4px 20px rgba(0,0,0,.45); pointer-events: none;
      }
      #field-office-tooltip .fo-head { font-weight: 700; margin: 2px 0; color: #9dc0ff; }
      #field-office-tooltip .fo-suggest b { color: #ffd479; }
      #field-office-tooltip .fo-flag { margin-top: 4px; color: #ff9b9b; }
      #field-office-tooltip .fo-actual { margin-top: 6px; color: #ff9b9b; }
      #field-office-tooltip .fo-actual-line { color: #cbd8ec; opacity: .9; margin: 1px 0; }
      #field-office-panel {
        position: fixed; right: 14px; bottom: 52px;
        background: #10243a; color: #eaf2ff; padding: 8px 10px; border-radius: 8px;
        font-size: 11px; box-shadow: 0 2px 12px rgba(0,0,0,.35); display: grid; gap: 3px;
      }
      #field-office-panel .fo-panel-title { font-weight: 700; margin-bottom: 2px; }
      #field-office-panel label { display: flex; gap: 6px; align-items: center; cursor: pointer; }
      .field-office-flagged {
        opacity: .45 !important; filter: grayscale(1) !important;
        outline: 1px dashed rgba(255,155,155,.7) !important; transition: opacity .15s ease;
      }
    `;
    const style = document.createElement("style");
    style.className = "field-office-ui";
    style.textContent = css;
    document.documentElement.appendChild(style);
  }

  function escapeHtml(s) {
    return String(s).replace(/[&<>"']/g, (c) => ({
      "&": "&amp;",
      "<": "&lt;",
      ">": "&gt;",
      '"': "&quot;",
      "'": "&#39;",
    })[c]);
  }
})();
