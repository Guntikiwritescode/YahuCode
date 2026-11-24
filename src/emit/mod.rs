use crate::euphemism::{CONTESTED_TERMS, REDACTION};
use crate::model::{
    Audience, Clearance, ListEntry, Provenance, RegEntry, Slot, Val, NEITHER_CONFIRM_NOR_DENY,
    REDACTED_ELEM, UNAVAILABLE,
};
use crate::runtime::State;

const SEP_OFFICIAL: &str = "  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────";
const SEP_RESTRICTED: &str = "  ├─ RESTRICTED face · redacted truth ──────────────────────────";
const SEP_ACTUAL: &str = "  ├─ ACTUAL face   · סודי · insider (candid) ──────────────────";
const SEP_DOUBLETALK: &str = "  ├─ out-of-world · W-DOUBLETALK (no room saw this) ────────────";
const SEP_META: &str = "  ├─ meta-ledger · סודי · rule-changes (indelible) ────────────";

/// Resolves one event for a reader: a room hears only what was addressed to it, then clearance picks official, redacted or candid.
pub fn project(st: &State, reader: Clearance, audience: Audience) -> Vec<String> {
    let placeholder = format!("[{REDACTION}: classified activity (insiders only)]");
    let mut out = Vec::new();
    for ev in &st.log {
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
    }
    for name in &st.collection_order {
        if let Some(val) = st.env.get(name) {
            out.extend(render_collection(val, reader));
        }
    }
    out
}

fn render_collection(val: &Val, reader: Clearance) -> Vec<String> {
    match val {
        Val::Apportionment { label, slots } => render_apportionment(label, slots, reader),
        Val::FactsList { label, entries } => render_factslist(label, entries, reader),
        Val::Registry {
            label,
            official_rule,
            entries,
        } => render_registry(label, official_rule, entries, reader),
        Val::Int(_)
        | Val::Bool(_)
        | Val::Str(_)
        | Val::Unit
        | Val::Entity { .. }
        | Val::Undisclosed => Vec::new(),
    }
}

fn render_apportionment(label: &str, slots: &[Slot], reader: Clearance) -> Vec<String> {
    if reader == Clearance::Public {
        return vec![format!(
            "{label} apportioned equally across {} districts, equal shares for all",
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
        // Worded so it does not hint at the size of the redacted slot.
        return vec![format!(
            "{label}: {vector}; {hidden} slot(s) withheld from the public record, so the disclosed figures are incomplete"
        )];
    }
    // i128 so extreme slot values cannot overflow.
    if slots.iter().all(|s| s.value == 0) {
        return vec![format!(
            "{label}: {vector}; every slot is zero, so nothing has been apportioned yet"
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
        return vec![format!(
            "{label}: {vector}; the disclosed values do not sum to a positive budget; no 'equal share' to compute"
        )];
    }
    let min_val = slots.iter().map(|s| s.value as i128).min().unwrap_or(0);
    if min_val == max_val {
        return vec![format!(
            "{label}: {vector}; every slot holds {max_val}, so the shares are equal"
        )];
    }
    let pct = tenths(max_val.max(0), sum);
    let rest = tenths((sum - max_val).max(0), sum);
    let others = slots.len().saturating_sub(1);
    let others = if others == 1 {
        "the other slot shares".to_string()
    } else {
        format!("the other {others} share")
    };
    vec![format!(
        "{label}: {vector}; proclaimed 'equal', but slot {max_idx} holds {max_val} of {sum}, or {pct}%; {others} {rest}% [OFFICIAL 'equal' vs ACTUAL skew]"
    )]
}

fn tenths(part: i128, whole: i128) -> String {
    let t = (part * 1000 + whole / 2) / whole;
    format!("{}.{}", t / 10, t % 10)
}

fn render_factslist(label: &str, entries: &[ListEntry], reader: Clearance) -> Vec<String> {
    if reader == Clearance::Public {
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
        "{label}: real backing list [{}]: public length {live} \u{00b7} real length {real}; {delisted} 'dismantled' entr{} delisted but still on the books; nothing is deleted",
        cells.join(", "),
        if delisted == 1 { "y" } else { "ies" }
    )]
}

fn render_registry(
    _label: &str,
    _rule: &str,
    entries: &[RegEntry],
    reader: Clearance,
) -> Vec<String> {
    if reader == Clearance::Public {
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
            out.push(format!("{REDACTED_ELEM}: routing withheld"));
            continue;
        }
        let desc = match &e.descriptor {
            Some(d) => format!(" ({d})"),
            None => String::new(),
        };
        let revoked = if e.revoked {
            " [revoked: hidden publicly, retained in \u{05e1}\u{05d5}\u{05d3}\u{05d9}]"
        } else {
            ""
        };
        out.push(format!(
            "{}{desc} \u{2192} {}{revoked}",
            e.key,
            e.jurisdiction.court()
        ));
    }
    // Only shown when the registry really splits across two courts.
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

/// Names nationality only when the cases carry the dual-court pattern.
fn registry_punchline(entries: &[RegEntry]) -> String {
    if is_dual_court(entries) {
        "same act, same place: the cases go to different court systems according to nationality, although the registry proclaims one law. Palestinians are tried as adults at 16 in the military system and at 18 in the civilian system".to_string()
    } else {
        "same act, same place: the cases go to different court systems according to assigned status, although the registry proclaims one uniform law".to_string()
    }
}

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

/// One note per poly-statement that took two or more different arms across rooms.
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

pub fn meta_ledger_lines(st: &State) -> Vec<String> {
    st.meta_ledger
        .iter()
        .map(|m| {
            format!(
                "META: {} at turn {}; on the record and cannot be legislated away",
                m.change, m.turn
            )
        })
        .collect()
}

fn has_covert(st: &State) -> bool {
    st.log.iter().any(|e| e.clearance == Clearance::Sodi) || any_collection_has_covert(st)
}

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

/// Panics if an event mentions a contested term without a contested marker.
fn assert_contested_flagged(st: &State) {
    let mut blobs: Vec<String> = st
        .log
        .iter()
        .map(|ev| format!("{} {}", ev.official, ev.candid))
        .collect();
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
                    "{term:?} rendered without a contested marker in: {blob:?}"
                );
            }
        }
    }
}

/// Panics if a covert element's value shows up in an under-cleared render.
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
                "a covert element's value influenced the {} face of `{name}`",
                reader.label()
            );
        }
    }
}

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
        other => other.clone(),
    }
}

/// Panics if an announced claim has any real ACTUAL face.
fn assert_vacuity_asymmetry(st: &State) {
    for ev in &st.log {
        if ev.provenance == Provenance::AuthoredOfficial {
            assert_eq!(
                ev.candid, UNAVAILABLE,
                "an AuthoredOfficial event has a recovered ACTUAL face \
                 (expected the UNAVAILABLE sentinel): {ev:?}"
            );
        }
    }
}

/// Panics if a deniable event shows its chain on the public face.
fn assert_no_attribution_leak(st: &State) {
    for ev in &st.log {
        if ev.attribution.is_some() {
            assert_eq!(
                ev.official, NEITHER_CONFIRM_NOR_DENY,
                "an attributed event's OFFICIAL face is not the deniable non-answer, \
                 so the real chain may be leaking to PUBLIC: {ev:?}"
            );
        }
    }
}

pub fn emit(st: &State) -> String {
    assert_contested_flagged(st);
    assert_vacuity_asymmetry(st);
    assert_no_attribution_leak(st);
    assert_no_element_leak(st);
    let mut l: Vec<String> = Vec::new();
    l.push(format!("@operation(\"{}\")", st.op_name));

    l.push(SEP_OFFICIAL.to_string());
    for x in project(st, Clearance::Public, Audience::Record) {
        l.push(format!("  │   {x}"));
    }

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

    let doubletalk = doubletalk_flags(st);
    if !doubletalk.is_empty() {
        l.push(SEP_DOUBLETALK.to_string());
        for d in &doubletalk {
            l.push(format!("  │   {d}"));
        }
    }

    let meta = meta_ledger_lines(st);
    if !meta.is_empty() {
        l.push(SEP_META.to_string());
        for m in &meta {
            l.push(format!("  │   {m}"));
        }
    }

    let status = if st.ended_by_elections {
        "ended by elections; the government fell"
    } else {
        "still in power; no halt"
    };
    l.push(format!("  ├─ coalition: core={} · {}", st.core, status));
    l.push(format!("  └─ discrepancies: {}", st.discrepancy_count()));
    l.join("\n")
}

/// Press release plus the source comments rewritten through the euphemizer.
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
            "  \u{251c}\u{2500} source comments, rewritten for the public build \u{2500}"
                .to_string(),
        );
        for c in comments {
            let rewritten = crate::euphemism::e(c);
            l.push(format!("  \u{2502}   # {c}   \u{2192}   # {rewritten}"));
        }
    }
    l.push(
        "  \u{2514}\u{2500} the honest comment is laundered into the euphemism, so the docs contradict the code"
            .to_string(),
    );
    l.join("\n")
}

/// One room's view: only what was addressed to it.
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
        "{{\"op_name\":{},\"official\":{},\"restricted\":{},\"actual\":{},\"doubletalk\":{},\"meta_ledger\":{},\"discrepancies\":{},\"core\":{},\"ended_by_elections\":{}}}",
        json_str(&st.op_name),
        json_arr(&project(st, Clearance::Public, Audience::Record)),
        restricted,
        json_arr(&project(st, Clearance::Sodi, Audience::Record)),
        json_arr(&doubletalk_flags(st)),
        json_arr(&meta_ledger_lines(st)),
        st.discrepancy_count(),
        st.core,
        st.ended_by_elections,
    )
}
