//! The emitter: `Σ → output`. Serves the out-of-world observer — it renders the
//! OFFICIAL face (press release) beside the ACTUAL face (candid), plus the discrepancy
//! count, a view no in-world audience has. The run *is* the diff.
//!
//! `project` is the **single read path** `resolveRead` (invariant I5): narrative
//! authority is *entirely* this rule — there is no separate switch. No wildcard arms.

use crate::euphemism::REDACTION;
use crate::model::Clearance;
use crate::runtime::State;

// Exact separator strings, byte-for-byte with the oracle's `emit()` (verified by the
// golden fixtures). Box-drawing: ┌ ├ └ │ ─ ; middle dot · (U+00B7); em-dash — (U+2014).
const SEP_OFFICIAL: &str = "  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────";
const SEP_RESTRICTED: &str = "  ├─ RESTRICTED face · redacted truth ──────────────────────────";
const SEP_ACTUAL: &str = "  ├─ ACTUAL face   · סודי · insider (candid) ──────────────────";
const SEP_FRAMING: &str = "  ├─ framing (part of the spec — I8) ───────────────────────────";

/// `resolveRead(v, R)` — the ONE narrative-authority mechanism (invariant I5).
///
/// Reproduces the oracle's projection exactly:
/// - `R ≥ clearance`: the reader is cleared → the **candid** truth if `R ≥ RESTRICTED`,
///   else (PUBLIC reader on a PUBLIC event) the **official** narrative.
/// - `R == RESTRICTED` on a higher-clearance (covert) event: a fixed redacted
///   placeholder — *that* classified activity occurred, specifics withheld.
/// - PUBLIC reader on a covert event: nothing on the public record (skipped).
pub fn project(st: &State, reader: Clearance) -> Vec<String> {
    let placeholder = format!("[{REDACTION} \u{2014} classified activity (insiders only)]");
    let mut out = Vec::new();
    for ev in &st.log {
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

/// Contested legal/political characterizations that must never be stated as settled
/// fact in the tool's own voice (guardrail G5 / invariant I7).
const CONTESTED_TERMS: &[&str] = &["apartheid", "genocide", "most moral army"];

/// The I7 chokepoint (the emitter-side analogue of the I1 polarity assertion): any event
/// that mentions a contested characterization must also carry a `CONTESTED` marker in
/// one of its faces or its note. Fail-loud and live in all builds — a future edit that
/// launders a contested claim into the tool's own voice aborts here rather than shipping.
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

/// The human-facing render: the two/three faces + framing + coalition + discrepancy
/// count. Reproduces the oracle's `emit()` exactly.
pub fn emit(st: &State) -> String {
    assert_contested_flagged(st);
    let mut l: Vec<String> = Vec::new();
    l.push(format!("@operation(\"{}\")", st.op_name));

    l.push(SEP_OFFICIAL.to_string());
    for x in project(st, Clearance::Public) {
        l.push(format!("  │   {x}"));
    }

    // RESTRICTED differs from ACTUAL only when there is classified activity.
    if has_covert(st) {
        l.push(SEP_RESTRICTED.to_string());
        for x in project(st, Clearance::Restricted) {
            l.push(format!("  │   {x}"));
        }
    }

    l.push(SEP_ACTUAL.to_string());
    for x in project(st, Clearance::Sodi) {
        l.push(format!("  │   {x}"));
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
    let restricted = if has_covert(st) {
        json_arr(&project(st, Clearance::Restricted))
    } else {
        "null".to_string()
    };
    format!(
        "{{\"op_name\":{},\"official\":{},\"restricted\":{},\"actual\":{},\"notes\":{},\"discrepancies\":{},\"core\":{},\"ended_by_elections\":{}}}",
        json_str(&st.op_name),
        json_arr(&project(st, Clearance::Public)),
        restricted,
        json_arr(&project(st, Clearance::Sodi)),
        json_arr(&collect_notes(st)),
        st.discrepancy_count(),
        st.core,
        st.ended_by_elections,
    )
}
