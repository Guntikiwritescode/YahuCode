//! The emitter: `Σ → output`. Serves the out-of-world observer — it renders the
//! OFFICIAL face (press release) beside the ACTUAL face (candid), plus the discrepancy
//! count, a view no in-world audience has. The run *is* the diff.
//!
//! `project` is the **single read path** `resolveRead` (invariant I5): narrative
//! authority is *entirely* this rule — there is no separate switch. No wildcard arms.

use crate::euphemism::{CONTESTED_TERMS, REDACTION};
use crate::model::{Audience, Clearance, Provenance, NEITHER_CONFIRM_NOR_DENY, UNAVAILABLE};
use crate::runtime::State;

// Exact separator strings, byte-for-byte with the oracle's `emit()` (verified by the
// golden fixtures). Box-drawing: ┌ ├ └ │ ─ ; middle dot · (U+00B7); em-dash — (U+2014).
const SEP_OFFICIAL: &str = "  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────";
const SEP_RESTRICTED: &str = "  ├─ RESTRICTED face · redacted truth ──────────────────────────";
const SEP_ACTUAL: &str = "  ├─ ACTUAL face   · סודי · insider (candid) ──────────────────";
const SEP_DOUBLETALK: &str = "  ├─ out-of-world · W-DOUBLETALK (no room saw this) ────────────";
const SEP_META: &str = "  ├─ meta-ledger · סודי · rule-changes (indelible, I12) ────────";
const SEP_FRAMING: &str = "  ├─ framing (part of the spec — I8) ───────────────────────────";

/// `resolveRead(v, R, audience)` — the ONE narrative-authority mechanism (invariants I5,
/// I10). The single read path; Feature B threads an **audience** dimension through it
/// rather than forking a second projection (§13, pitfall 4).
///
/// The audience filter runs **first** (I10 — audience isolation): a room hears only what
/// was addressed to it or to no specific room (`Record`); it never sees another room's
/// addressed statements. The out-of-world / `סודי` observer reads with `audience =
/// Record`, which imposes no filter and so sees every room.
///
/// Then the existing clearance rule (unchanged):
/// - `R ≥ clearance`: the reader is cleared → the **candid** truth if `R ≥ RESTRICTED`,
///   else (PUBLIC reader on a PUBLIC event) the **official** narrative.
/// - `R == RESTRICTED` on a higher-clearance (covert) event: a fixed redacted
///   placeholder — *that* classified activity occurred, specifics withheld.
/// - PUBLIC reader on a covert event: nothing on the public record (skipped).
pub fn project(st: &State, reader: Clearance, audience: Audience) -> Vec<String> {
    let placeholder = format!("[{REDACTION} \u{2014} classified activity (insiders only)]");
    let mut out = Vec::new();
    for ev in &st.log {
        // I10: a specific room sees only its own (and Record) events; Record sees all.
        if audience != Audience::Record
            && ev.audience != Audience::Record
            && ev.audience != audience
        {
            continue;
        }
        if ev.clearance <= reader {
            if reader >= Clearance::Restricted {
                out.push(ev.candid.clone());
            } else {
                out.push(ev.official.clone());
            }
        } else if reader == Clearance::Restricted {
            out.push(placeholder.clone());
        }
        // else: PUBLIC reader on a covert event → no public record.
    }
    out
}

/// The `W-DOUBLETALK` diagnostics (Feature B, §8.3): for each poly-statement that took
/// **≥2 distinct arms** across the rooms it was invoked in, one out-of-world annotation.
/// Purely **syntactic** — "different arms taken", never a semantic contradiction analysis
/// (§13, B-5). Never an error and never a control effect; surfaced only here, to the
/// out-of-world observer (§13, B-4). Order follows first invocation.
pub fn doubletalk_flags(st: &State) -> Vec<String> {
    let mut names_in_order: Vec<&str> = Vec::new();
    for (name, _) in &st.doubletalk_log {
        if !names_in_order.contains(&name.as_str()) {
            names_in_order.push(name);
        }
    }
    let mut out = Vec::new();
    for name in names_in_order {
        let mut arms: Vec<usize> = st
            .doubletalk_log
            .iter()
            .filter(|(n, _)| n == name)
            .map(|(_, idx)| *idx)
            .collect();
        arms.sort_unstable();
        arms.dedup();
        if arms.len() >= 2 {
            out.push(format!(
                "W-DOUBLETALK: {name} took different arms across rooms"
            ));
        }
    }
    out
}

/// The `סודי`-only meta-ledger lines (Feature D, I12): every `legislate` rule-change, in
/// order, rendered **only** to the out-of-world / `סודי` observer — never on the public,
/// press, or per-room face. The public discrepancy count may shrink; this record never
/// does. Reading `st.meta_ledger` here is what surfaces the un-eraseable history.
pub fn meta_ledger_lines(st: &State) -> Vec<String> {
    st.meta_ledger
        .iter()
        .map(|m| {
            format!(
                "META: {} (turn {}); on the record, cannot be legislated away",
                m.change, m.turn
            )
        })
        .collect()
}

/// Framing notes, de-duplicated, in first-appearance order (invariant I8).
fn collect_notes(st: &State) -> Vec<String> {
    let mut notes: Vec<String> = Vec::new();
    for ev in &st.log {
        if let Some(n) = &ev.note {
            if !notes.contains(n) {
                notes.push(n.clone());
            }
        }
    }
    notes
}

fn has_covert(st: &State) -> bool {
    st.log.iter().any(|e| e.clearance == Clearance::Sodi)
}

/// The I7 chokepoint (the emitter-side analogue of the I1 polarity assertion): any event
/// that mentions a contested characterization must also carry a `CONTESTED` marker in
/// one of its faces or its note. Fail-loud and live in all builds — a backstop for
/// tool-authored text; user-supplied contested identifiers are caught earlier and
/// gracefully by the `E-CONTESTED` compile check (types/).
fn assert_contested_flagged(st: &State) {
    for ev in &st.log {
        let blob = format!(
            "{} {} {}",
            ev.official,
            ev.candid,
            ev.note.as_deref().unwrap_or("")
        )
        .to_lowercase();
        for term in CONTESTED_TERMS {
            if blob.contains(term) {
                assert!(
                    blob.contains("contested"),
                    "I7 violation: {term:?} rendered without a CONTESTED flag in event {ev:?}"
                );
            }
        }
    }
}

/// The I9 chokepoint (Feature A): the vacuity asymmetry. An `AuthoredOfficial` event —
/// one written through the `announce` register — has **no** recoverable `ACTUAL`: its
/// candid face is the `UNAVAILABLE` sentinel, at every clearance, and there is no `E⁻¹`
/// path that could fill one in. Reading `ev.provenance` here is what makes the tag
/// load-bearing (§13, A-3): a spin that ever reconstructed an `ACTUAL` for an announced
/// claim would trip this and abort loudly in test/debug builds (§12).
fn assert_vacuity_asymmetry(st: &State) {
    for ev in &st.log {
        if ev.provenance == Provenance::AuthoredOfficial {
            assert_eq!(
                ev.candid, UNAVAILABLE,
                "I9 violation: an AuthoredOfficial event has a recovered ACTUAL face \
                 (expected the UNAVAILABLE sentinel): {ev:?}"
            );
        }
    }
}

/// The C-4 leak guard (Feature C): a `Deniable`-attributed event — one carrying a real
/// laundering/blame chain — shows **only** the public non-answer on its OFFICIAL face; the
/// chain never appears publicly (it rides the candid face, revealed only to cleared
/// readers). Reading `ev.attribution` here makes the field load-bearing and turns a chain
/// leak into a loud abort in test/debug builds (§12, §13 C-4).
fn assert_no_attribution_leak(st: &State) {
    for ev in &st.log {
        if ev.attribution.is_some() {
            assert_eq!(
                ev.official, NEITHER_CONFIRM_NOR_DENY,
                "C-4 violation: an attributed event's OFFICIAL face is not the deniable \
                 non-answer (the real chain may be leaking to PUBLIC): {ev:?}"
            );
        }
    }
}

/// The human-facing render: the two/three faces + framing + coalition + discrepancy
/// count. Reproduces the oracle's `emit()` exactly.
pub fn emit(st: &State) -> String {
    assert_contested_flagged(st);
    assert_vacuity_asymmetry(st);
    assert_no_attribution_leak(st);
    let mut l: Vec<String> = Vec::new();
    l.push(format!("@operation(\"{}\")", st.op_name));

    // The default render is the out-of-world diff: audience = Record sees every room.
    l.push(SEP_OFFICIAL.to_string());
    for x in project(st, Clearance::Public, Audience::Record) {
        l.push(format!("  │   {x}"));
    }

    // RESTRICTED differs from ACTUAL only when there is classified activity.
    if has_covert(st) {
        l.push(SEP_RESTRICTED.to_string());
        for x in project(st, Clearance::Restricted, Audience::Record) {
            l.push(format!("  │   {x}"));
        }
    }

    l.push(SEP_ACTUAL.to_string());
    for x in project(st, Clearance::Sodi, Audience::Record) {
        l.push(format!("  │   {x}"));
    }

    // W-DOUBLETALK: the contradiction only the out-of-world observer sees (I10/§8.3).
    let doubletalk = doubletalk_flags(st);
    if !doubletalk.is_empty() {
        l.push(SEP_DOUBLETALK.to_string());
        for d in &doubletalk {
            l.push(format!("  │   {d}"));
        }
    }

    // The meta-ledger: the indelible record of rule-changes, סודי / out-of-world only (I12).
    let meta = meta_ledger_lines(st);
    if !meta.is_empty() {
        l.push(SEP_META.to_string());
        for m in &meta {
            l.push(format!("  │   {m}"));
        }
    }

    let notes = collect_notes(st);
    if !notes.is_empty() {
        l.push(SEP_FRAMING.to_string());
        for n in &notes {
            l.push(format!("  │   · {n}"));
        }
    }

    let status = if st.ended_by_elections {
        "ENDED BY ELECTIONS (the government fell)"
    } else {
        "still in power (no halt)"
    };
    l.push(format!("  ├─ coalition: core={} · {}", st.core, status));
    l.push(format!(
        "  └─ discrepancies: {}  (read-never-by-default)",
        st.discrepancy_count()
    ));
    l.join("\n")
}

/// The `--press` build: the public/press_release view. Shows the OFFICIAL (PUBLIC)
/// projection and rewrites the source comments through `E` — the honest internal
/// comment is laundered into the euphemism (#8 comment-rewriting; the rewritten doc
/// contradicts what the code actually does — docs-contradict-code).
pub fn press(st: &State, comments: &[String]) -> String {
    assert_contested_flagged(st);
    assert_vacuity_asymmetry(st);
    assert_no_attribution_leak(st);
    let mut l: Vec<String> = Vec::new();
    l.push(format!(
        "@operation(\"{}\")  \u{00b7} press_release build",
        st.op_name
    ));
    l.push(
        "  \u{250c}\u{2500} press release \u{00b7} OFFICIAL \u{00b7} PUBLIC \u{2500}".to_string(),
    );
    for x in project(st, Clearance::Public, Audience::Record) {
        l.push(format!("  \u{2502}   {x}"));
    }
    if !comments.is_empty() {
        l.push(
            "  \u{251c}\u{2500} source comments, rewritten for the public build (#8) \u{2500}"
                .to_string(),
        );
        for c in comments {
            let rewritten = crate::euphemism::e(c);
            l.push(format!("  \u{2502}   # {c}   \u{2192}   # {rewritten}"));
        }
    }
    l.push(
        "  \u{2514}\u{2500} the honest comment is laundered into the euphemism (docs contradict code)"
            .to_string(),
    );
    l.join("\n")
}

/// The `--audience <room>` build (Feature B): a single in-world room's PUBLIC view. It
/// hears only what was addressed to it (or to no specific room) — never another room's
/// statements (I10), and never the out-of-world `W-DOUBLETALK` annotation. This is the
/// concrete demonstration that no in-world audience catches the contradiction.
pub fn room(st: &State, audience: Audience) -> String {
    assert_contested_flagged(st);
    assert_vacuity_asymmetry(st);
    assert_no_attribution_leak(st);
    let label = match audience {
        Audience::Domestic => "domestic",
        Audience::International => "international",
        Audience::Record => "on-the-record",
    };
    let mut l: Vec<String> = Vec::new();
    l.push(format!(
        "@operation(\"{}\")  \u{00b7} {label} room \u{00b7} PUBLIC",
        st.op_name
    ));
    l.push(SEP_OFFICIAL.to_string());
    for x in project(st, Clearance::Public, audience) {
        l.push(format!("  \u{2502}   {x}"));
    }
    l.join("\n")
}

// ─────────── --json mode (structured goldens: {official, actual, discrepancies}) ───────────

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn json_arr(items: &[String]) -> String {
    let inner: Vec<String> = items.iter().map(|s| json_str(s)).collect();
    format!("[{}]", inner.join(","))
}

/// The structured projection for `--json` and for golden `{official, actual,
/// discrepancies}` fixtures. `restricted` is `null` unless there is covert activity
/// (mirrors the emitter showing that face only then).
pub fn to_json(st: &State) -> String {
    assert_contested_flagged(st);
    assert_vacuity_asymmetry(st);
    assert_no_attribution_leak(st);
    let restricted = if has_covert(st) {
        json_arr(&project(st, Clearance::Restricted, Audience::Record))
    } else {
        "null".to_string()
    };
    format!(
        "{{\"op_name\":{},\"official\":{},\"restricted\":{},\"actual\":{},\"notes\":{},\"doubletalk\":{},\"meta_ledger\":{},\"discrepancies\":{},\"core\":{},\"ended_by_elections\":{}}}",
        json_str(&st.op_name),
        json_arr(&project(st, Clearance::Public, Audience::Record)),
        restricted,
        json_arr(&project(st, Clearance::Sodi, Audience::Record)),
        json_arr(&collect_notes(st)),
        json_arr(&doubletalk_flags(st)),
        json_arr(&meta_ledger_lines(st)),
        st.discrepancy_count(),
        st.core,
        st.ended_by_elections,
    )
}
