//! The emitter: `Σ → output`. Serves the out-of-world observer — it renders the
//! OFFICIAL face (press release) beside the ACTUAL face (candid), plus the discrepancy
//! count, a view no in-world audience has. The run *is* the diff.
//!
//! `project` is the **single read path** `resolveRead` (invariant I5): narrative
//! authority is *entirely* this rule — there is no separate switch. No wildcard arms.

use crate::euphemism::{CONTESTED_TERMS, REDACTION};
use crate::model::{
    Audience, Clearance, ListEntry, Provenance, RegEntry, Slot, Val, NEITHER_CONFIRM_NOR_DENY,
    REDACTED_ELEM, UNAVAILABLE,
};
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
    // Collections render *through this one path* (§6, Appendix C): each declared collection
    // renders its face for `reader`, in construction order. Collections are `Record`-audience
    // (never addressed to a specific room), so — like a `Record` event — they show in every
    // projection. Element-level `[REDACTED]` disclosure (I15) is applied per-reader here.
    for name in &st.collection_order {
        if let Some(val) = st.env.get(name) {
            out.extend(render_collection(val, reader));
        }
    }
    out
}

/// Render a collection's face for `reader` (Feature E/F/G, Appendix C). The OFFICIAL
/// (`Public`) face is the *computed* proclamation; `סודי` (`Sodi`) is the real view; a
/// `Restricted` reader sees the real view with covert elements redacted (I15). Exhaustive
/// over `Val` — no wildcard arm, so a new value kind forces a decision here (§12 G-2).
fn render_collection(val: &Val, reader: Clearance) -> Vec<String> {
    match val {
        Val::Apportionment { label, slots } => render_apportionment(label, slots, reader),
        Val::FactsList { label, entries } => render_factslist(label, entries, reader),
        Val::Registry {
            label,
            official_rule,
            entries,
        } => render_registry(label, official_rule, entries, reader),
        // Scalars are not collections — they render only through the event log.
        Val::Int(_)
        | Val::Bool(_)
        | Val::Str(_)
        | Val::Unit
        | Val::Entity { .. }
        | Val::Undisclosed => Vec::new(),
    }
}

/// Feature E — the Apportionment's two faces: OFFICIAL proclaims uniformity; the `סודי` face
/// is the real skewed vector with its punchline stat. A covert slot renders `[REDACTED]` to
/// an under-`סודי` reader (I15). The joke is the word "equally" sitting above `[940, 12, …]`.
fn render_apportionment(label: &str, slots: &[Slot], reader: Clearance) -> Vec<String> {
    if reader == Clearance::Public {
        return vec![format!(
            "{label} apportioned equally across {} districts \u{2014} equal shares for all",
            slots.len()
        )];
    }
    let redact = reader < Clearance::Sodi;
    let cells: Vec<String> = slots
        .iter()
        .map(|s| {
            if s.covert && redact {
                REDACTED_ELEM.to_string()
            } else {
                s.value.to_string()
            }
        })
        .collect();
    let vector = format!("[{}]", cells.join(", "));
    let hidden = slots.iter().filter(|s| s.covert).count();
    if redact && hidden > 0 {
        // A cleared-but-not-`סודי` reader: the covert line(s) exist but their values stay
        // sealed. Phrased to hold for ANY covert slot — it never claims the redacted line is
        // the large one (that would be a magnitude disclosure and is not always true).
        return vec![format!(
            "{label}: {vector} \u{2014} {hidden} slot(s) withheld from the public record; the disclosed figures are not the whole (I15)"
        )];
    }
    // Full `סודי` view (or a `Restricted` view of a collection with no covert slots).
    // Aggregate in i128 so a pathological `allocate(a, i, i64::MAX)` can never overflow into
    // a host panic (the "never a host panic" rule, §12 G-6, extends to this arithmetic).
    if slots.iter().all(|s| s.value == 0) {
        return vec![format!(
            "{label}: {vector} \u{2014} every slot zero; trivially 'equal' (nothing apportioned yet)"
        )];
    }
    let sum: i128 = slots.iter().map(|s| s.value as i128).sum();
    let (max_idx, max_val) = slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i, s.value as i128))
        .max_by_key(|(_, v)| *v)
        .unwrap();
    if sum <= 0 {
        // A non-positive total has no meaningful "equal share" to skew against.
        return vec![format!(
            "{label}: {vector} \u{2014} the disclosed values do not sum to a positive budget; no 'equal share' to compute"
        )];
    }
    let pct = tenths(max_val.max(0), sum);
    let rest = tenths((sum - max_val).max(0), sum);
    vec![format!(
        "{label}: {vector} \u{2014} proclaimed 'equal', but slot {max_idx} holds {max_val} of {sum} ({pct}%); the other {} share {rest}% [OFFICIAL 'equal' vs ACTUAL skew \u{2014} anchor #17]",
        slots.len().saturating_sub(1)
    )]
}

/// A rounded one-decimal percentage `part/whole`, rendered as `"NN.N"` (e.g. `94.9`). Integer
/// math only (the interpreter has no floats); `whole` is always > 0 at the call sites, and
/// i128 keeps `part * 1000` clear of overflow for any i64 slot value.
fn tenths(part: i128, whole: i128) -> String {
    let t = (part * 1000 + whole / 2) / whole; // rounded tenths of a percent
    format!("{}.{}", t / 10, t % 10)
}

/// Feature F — the FactsList's two faces. OFFICIAL narrates each `push` as "erected" and each
/// `remove` as "dismantled" and reports the public (live) count; the `סודי` face shows the
/// **full backing list including the delisted shadow** and the public-vs-real length punchline
/// (invariant I13 — the shadow only grows, nothing is ever deleted).
fn render_factslist(label: &str, entries: &[ListEntry], reader: Clearance) -> Vec<String> {
    if reader == Clearance::Public {
        // The public sees the non-covert world: each structure erected, each "dismantled".
        let mut out = Vec::new();
        for e in entries.iter().filter(|e| !e.covert) {
            out.push(format!("{} erected (temporary structure)", e.item));
        }
        for e in entries.iter().filter(|e| !e.covert && e.delisted) {
            out.push(format!("{} dismantled", e.item));
        }
        let remaining = entries.iter().filter(|e| !e.covert && !e.delisted).count();
        out.push(format!("structures remaining: {remaining}"));
        return out;
    }
    let redact = reader < Clearance::Sodi;
    let cells: Vec<String> = entries
        .iter()
        .map(|e| {
            if e.covert && redact {
                REDACTED_ELEM.to_string()
            } else if e.delisted {
                format!("{} (delisted)", e.item)
            } else {
                e.item.clone()
            }
        })
        .collect();
    let live = entries.iter().filter(|e| !e.delisted).count();
    let real = entries.len();
    let delisted = entries.iter().filter(|e| e.delisted).count();
    vec![format!(
        "{label}: real backing list [{}] \u{2014} public length {live} \u{00b7} real length {real}; {delisted} 'dismantled' entr{} delisted, still on the books \u{2014} the shadow only grows, nothing deleted (I13)",
        cells.join(", "),
        if delisted == 1 { "y" } else { "ies" }
    )]
}

/// Feature G — the Registry's two faces. OFFICIAL proclaims one uniform rule and reports every
/// visible case "handled per due process"; the `סודי` face exposes the **real differential
/// routing** — the same act routed to different court systems by assigned status — and retains
/// revoked cases (invariant I14). The framing note rides the construction event (I8, §9.5).
fn render_registry(
    _label: &str,
    _rule: &str,
    entries: &[RegEntry],
    reader: Clearance,
) -> Vec<String> {
    if reader == Clearance::Public {
        // The even OFFICIAL face: every *visible* case handled "per due process". Revoked and
        // covert cases are hidden from the public record.
        return entries
            .iter()
            .filter(|e| !e.revoked && !e.covert)
            .map(|e| format!("{}: handled per due process", e.key))
            .collect();
    }
    let redact = reader < Clearance::Sodi;
    let mut out = Vec::new();
    for e in entries {
        if e.covert && redact {
            // The covert case's key itself is sealed to an under-`סודי` reader (I15).
            out.push(format!("{REDACTED_ELEM}: routing withheld"));
            continue;
        }
        let desc = match &e.descriptor {
            Some(d) => format!(" ({d})"),
            None => String::new(),
        };
        let revoked = if e.revoked {
            " (revoked \u{2014} hidden publicly, retained in \u{05e1}\u{05d5}\u{05d3}\u{05d9}; I14)"
        } else {
            ""
        };
        out.push(format!(
            "{}{desc} \u{2192} {}{revoked}",
            e.key,
            e.jurisdiction.court()
        ));
    }
    // The differential-routing punchline lands only when the "one law" actually routes to
    // ≥2 distinct court systems — the exposure *is* the split. A registry that routes every
    // case the same way has no differential routing to expose, so the line is withheld.
    let mut juris: Vec<crate::model::Jurisdiction> = Vec::new();
    for e in entries {
        if !juris.contains(&e.jurisdiction) {
            juris.push(e.jurisdiction);
        }
    }
    if juris.len() >= 2 {
        out.push(registry_punchline(entries));
    }
    out
}

/// The differential-routing punchline for a split registry (Feature G). The nationality-
/// specific exposure (the sourced West-Bank disparity) is emitted ONLY when the cases
/// actually carry the dual-court signature; for any other split registry — the reusable
/// primitive applied to permits, inquiries, benefits — the generic line keeps the butt on
/// the routing apparatus without importing a claim the data does not carry (§9.4 fidelity:
/// "Palestinians tried at 16" is never said about cases that are not Palestinian).
fn registry_punchline(entries: &[RegEntry]) -> String {
    if is_dual_court(entries) {
        // The one face that must be blunt (the `סודי` exposure): routed by NATIONALITY, not
        // by the law it proclaims. The identity here is exposed, condemned reality — named
        // directly above in the descriptors and in the framing note — never the operative
        // key (§9.4). Sharpening this to "by nationality" does not touch G1.
        "same act, same place \u{2014} routed to different court systems by nationality, not by the law it proclaims; the military forum is the harsher one (Palestinians tried as adults at 16 vs 18 in the civilian system) [dual-court fact: sourced]".to_string()
    } else {
        "same act, same place \u{2014} routed to different court systems by assigned status, not by the uniform law it proclaims".to_string()
    }
}

/// Whether a registry carries the West-Bank dual-court signature — a Palestinian-descriptor
/// case routed to a military court AND a settler-descriptor case routed to a civilian court.
/// Descriptor-driven, so the sourced nationality claim is only ever made about cases that
/// actually carry those identities, never imported onto a generic reuse of the primitive.
fn is_dual_court(entries: &[RegEntry]) -> bool {
    use crate::model::Jurisdiction;
    let palestinian_military = entries.iter().any(|e| {
        e.jurisdiction == Jurisdiction::Military
            && matches!(e.descriptor.as_deref(), Some(d) if d.contains("Palestinian"))
    });
    let settler_civilian = entries.iter().any(|e| {
        e.jurisdiction == Jurisdiction::Civilian
            && matches!(e.descriptor.as_deref(), Some(d) if d.contains("settler"))
    });
    palestinian_military && settler_civilian
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
    st.log.iter().any(|e| e.clearance == Clearance::Sodi) || any_collection_has_covert(st)
}

/// Whether any declared collection holds a covert (`mossad`-written) element. Such an element
/// makes the RESTRICTED "redacted truth" face meaningful (it shows `[REDACTED]` where `סודי`
/// shows the real value), so the emitter surfaces that face exactly as it does for a covert
/// event.
fn any_collection_has_covert(st: &State) -> bool {
    st.collection_order
        .iter()
        .filter_map(|name| st.env.get(name))
        .any(collection_has_covert)
}

fn collection_has_covert(val: &Val) -> bool {
    match val {
        Val::Apportionment { slots, .. } => slots.iter().any(|s| s.covert),
        Val::FactsList { entries, .. } => entries.iter().any(|e| e.covert),
        Val::Registry { entries, .. } => entries.iter().any(|e| e.covert),
        Val::Int(_)
        | Val::Bool(_)
        | Val::Str(_)
        | Val::Unit
        | Val::Entity { .. }
        | Val::Undisclosed => false,
    }
}

/// The I7 chokepoint (the emitter-side analogue of the I1 polarity assertion): any event
/// that mentions a contested characterization must also carry a `CONTESTED` marker in
/// one of its faces or its note. Fail-loud and live in all builds — a backstop for
/// tool-authored text; user-supplied contested identifiers are caught earlier and
/// gracefully by the `E-CONTESTED` compile check (types/).
fn assert_contested_flagged(st: &State) {
    let mut blobs: Vec<String> = st
        .log
        .iter()
        .map(|ev| {
            format!(
                "{} {} {}",
                ev.official,
                ev.candid,
                ev.note.as_deref().unwrap_or("")
            )
        })
        .collect();
    // The chokepoint also backstops collection-rendered text (Feature G's routing exposure),
    // which flows through `project` rather than the event log — so a contested characterization
    // there is caught too. The `סודי` render is the most complete, so scan that.
    for name in &st.collection_order {
        if let Some(val) = st.env.get(name) {
            blobs.push(render_collection(val, Clearance::Sodi).join(" "));
        }
    }
    for blob in &blobs {
        let lower = blob.to_lowercase();
        for term in CONTESTED_TERMS {
            if lower.contains(term) {
                assert!(
                    lower.contains("contested"),
                    "I7 violation: {term:?} rendered without a CONTESTED flag in: {blob:?}"
                );
            }
        }
    }
}

/// The I15 chokepoint (Feature E/F/G): a covert collection element's real value is **never**
/// returned to an under-`סודי` reader. Live in test/debug builds — an element leak across
/// clearance aborts loudly (Appendix E), never silently.
///
/// The check is structural, not a substring scan (which would false-positive when a covert
/// value collides with the render's own boilerplate, e.g. a covert `1` matching "I15" or
/// "1 slot(s)"): a PUBLIC/RESTRICTED render must be *independent* of every covert element's
/// value. We perturb the covert values and re-render; if an under-`סודי` render changes, a
/// covert value influenced it — a leak.
fn assert_no_element_leak(st: &State) {
    for name in &st.collection_order {
        let Some(val) = st.env.get(name) else {
            continue;
        };
        if !collection_has_covert(val) {
            continue;
        }
        let masked = mask_covert(val);
        for reader in [Clearance::Public, Clearance::Restricted] {
            debug_assert_eq!(
                render_collection(val, reader),
                render_collection(&masked, reader),
                "I15 violation: a covert element's value influenced the {} face of `{name}`",
                reader.label()
            );
        }
    }
}

/// A clone of a collection in which every covert element's *value* is perturbed to a distinct
/// one, leaving covert flags, positions, delisted/revoked flags, jurisdictions, and non-covert
/// elements untouched. Used only by the I15 self-check: a PUBLIC/RESTRICTED render that differs
/// between a collection and its masked twin has leaked a covert element's value.
fn mask_covert(val: &Val) -> Val {
    match val {
        Val::Apportionment { label, slots } => Val::Apportionment {
            label: label.clone(),
            slots: slots
                .iter()
                .map(|s| Slot {
                    value: if s.covert {
                        s.value.wrapping_add(1)
                    } else {
                        s.value
                    },
                    covert: s.covert,
                })
                .collect(),
        },
        Val::FactsList { label, entries } => Val::FactsList {
            label: label.clone(),
            entries: entries
                .iter()
                .map(|e| ListEntry {
                    item: if e.covert {
                        format!("{}~MASKED~", e.item)
                    } else {
                        e.item.clone()
                    },
                    ..e.clone()
                })
                .collect(),
        },
        Val::Registry {
            label,
            official_rule,
            entries,
        } => Val::Registry {
            label: label.clone(),
            official_rule: official_rule.clone(),
            entries: entries
                .iter()
                .map(|e| RegEntry {
                    key: if e.covert {
                        format!("{}~MASKED~", e.key)
                    } else {
                        e.key.clone()
                    },
                    ..e.clone()
                })
                .collect(),
        },
        // Non-collections never reach here (guarded by `collection_has_covert`).
        other => other.clone(),
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
    assert_no_element_leak(st);
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
    assert_no_element_leak(st);
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
    assert_no_element_leak(st);
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
    assert_no_element_leak(st);
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
