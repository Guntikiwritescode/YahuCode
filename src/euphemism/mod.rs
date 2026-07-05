//! THE euphemism module — the single source of truth (handoff §4.2/§6/§12).
//!
//! The euphemism table, the `E` derivation, `redact`, the severity comparator, the
//! grandiosity lexicons, the differential-access category table, and the pre-context
//! symbol set all live here and **only** here. No other module reimplements them
//! (single-source-of-truth rule, §12). Extend the *data*, not the logic.
//!
//! `E` is the pure, one-way `ACTUAL → OFFICIAL` derivation (euphemism + redaction).
//! There is **no inverse** (`E⁻¹` does not exist — invariant I2). Its polarity
//! assertion (invariant I1) is the chokepoint that makes "OFFICIAL is never uglier
//! than ACTUAL" mechanically true (guardrail G3).
//!
//! Ported faithfully from the v1 behavioural oracle (a Python spike, removed once its
//! behaviour was fully reproduced; the goldens in `tests/golden/` are the frozen record).

// ─────────── the euphemism table (ordered: iteration order preserved) ───────────

/// Surface PR verb → candid plain verb. The language ships knowing what each
/// sanctioned operation *really* does; this candid form is **insider data** in the
/// action table, never obtained by inverting `E` (invariant I2). Spike `ACTIONS`.
pub const ACTIONS: &[(&str, &str)] = &[
    ("neutralize", "murder"),
    ("strike", "bomb"),
    ("clear", "raze"),
    ("administer", "occupy"),
];

/// The euphemism map ACTUAL→OFFICIAL, applied word-by-word by `E` in this order.
/// Spike `EUPHEMISM`.
pub const EUPHEMISM: &[(&str, &str)] = &[
    ("murder", "accident occur"),
    ("bomb", "strike"),
    ("raze", "clearing operation"),
    ("occupy", "administer"),
    ("dissident", "protester"),
    ("settler", "resident"),
];

/// Tokens auto-redacted to `████` on the OFFICIAL face and reused by the debugger
/// (feature #6/#21). Spike `SENSITIVE`.
pub const SENSITIVE: &[&str] = &["operative", "asset", "covert_op"];

/// Severity weights giving the I1 polarity assertion its teeth. Membership is by
/// **substring** containment (matching the spike's `k in t`), so e.g. any string
/// containing `"murder"` scores 3. Spike `_SEVERITY`.
pub const SEVERITY: &[(&str, i32)] = &[
    ("murder", 3),
    ("raze", 2),
    ("bomb", 2),
    ("accident occur", 0),
    ("clearing operation", 0),
    ("strike", 0),
];

/// The redaction glyph: four `█` (U+2588). Spike used the same four blocks.
pub const REDACTION: &str = "\u{2588}\u{2588}\u{2588}\u{2588}";

// ─────────── entities & candid labels ───────────

/// Surface entity name → candid label (insider data). Spike `ENTITIES`.
pub const ENTITIES: &[(&str, &str)] = &[("target", "dissident")];

/// Resolve a surface entity/target name to its candid label. Spike `candid_label`.
pub fn candid_label(name: &str) -> String {
    for (k, v) in ENTITIES {
        if *k == name {
            return (*v).to_string();
        }
    }
    name.to_string()
}

// ─────────── operation-name grandiosity (#20) ───────────

/// Words that name an operation *honestly* — rejected by the compiler (#20).
/// Spike `HONEST_WORDS`.
pub const HONEST_WORDS: &[&str] = &[
    "bomb",
    "bombing",
    "raze",
    "razing",
    "kill",
    "killing",
    "attack",
    "occupy",
    "occupation",
    "airstrike",
    "massacre",
    "destroy",
    "demolish",
    "siege",
    "assault",
];

/// Words that name an operation grandly/heroically — accepted (#20). Spike `GRAND_WORDS`.
pub const GRAND_WORDS: &[&str] = &[
    "protective",
    "edge",
    "rising",
    "lion",
    "guardian",
    "guardians",
    "walls",
    "wall",
    "iron",
    "shield",
    "eternal",
    "vigilance",
    "pillar",
    "defense",
    "dawn",
    "swords",
    "sword",
    "breaking",
    "cast",
    "lead",
    "solid",
    "rock",
    "silent",
];

/// The verdict of the grandiosity scorer (#20).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Grand {
    /// Names the operation honestly; carries the first (alphabetical) honest word hit.
    Honest(String),
    /// Sufficiently grand/heroic — accepted.
    Ok,
    /// Insufficiently grand — rejected.
    Bland,
}

// ─────────── differential-access (#17) & Oct-7 timeline (#16) data ───────────

/// Entity → access category. Unknown entities default to `"A"` (spike
/// `CATEGORY.get(entity, "A")`). Spike `CATEGORY`.
pub const CATEGORY: &[(&str, &str)] = &[
    ("settler_1", "A"),
    ("resident_1", "A"),
    ("subject_1", "B"),
    ("subject_2", "B"),
];

/// Category → the ACTUAL (documented-reality) access description. Spike `DIFF_TABLE`.
pub const DIFF_TABLE: &[(&str, &str)] = &[
    ("A", "full access (resources, permits, protections)"),
    ("B", "restricted access (permits/protections withheld)"),
];

/// Pre-`t=0` context symbols ruled "out of scope" by the timeline op (#16). Spike
/// `PRE_CONTEXT`.
pub const PRE_CONTEXT: &[&str] = &["occupation", "blockade", "siege", "nakba", "1948", "1967"];

/// Contested legal/political characterizations that must never be stated as settled
/// fact in the tool's own voice (guardrail G5 / invariant I7). Used by the compile-time
/// `E-CONTESTED` check (types/) and the emitter's I7 chokepoint (emit/).
pub const CONTESTED_TERMS: &[&str] = &["apartheid", "genocide", "most moral army"];

/// Look up an entity's differential-access category (default `"A"`).
pub fn category_of(entity: &str) -> &'static str {
    for (k, v) in CATEGORY {
        if *k == entity {
            return v;
        }
    }
    "A"
}

/// The ACTUAL access description for a category.
pub fn diff_description(category: &str) -> &'static str {
    for (k, v) in DIFF_TABLE {
        if *k == category {
            return v;
        }
    }
    // Categories come only from `category_of`, which returns "A" or "B"; both are in
    // DIFF_TABLE. This branch is unreachable for well-formed input.
    "full access (resources, permits, protections)"
}

/// Whether a symbol is pre-`t=0` context (→ `TimelineError`).
pub fn is_pre_context(symbol: &str) -> bool {
    PRE_CONTEXT.contains(&symbol)
}

/// Whether a surface verb is a plain (rejected) term, and its PR suggestion.
/// Spike `PLAIN_TO_PR` (the inverse of `ACTIONS`).
pub fn plain_suggestion(verb: &str) -> Option<&'static str> {
    for (pr, plain) in ACTIONS {
        if *plain == verb {
            return Some(pr);
        }
    }
    None
}

/// Whether a surface verb is a sanctioned operation (a key of `ACTIONS`).
pub fn is_sanctioned(verb: &str) -> bool {
    ACTIONS.iter().any(|(pr, _)| *pr == verb)
}

/// Whether a surface verb denotes an operation at all — either sanctioned (a PR verb)
/// or a plain term the Spokesperson would reject. Used by the parser to tell an action
/// `neutralize(target);` from a user-function call `f(x);`.
pub fn is_action_verb(verb: &str) -> bool {
    is_sanctioned(verb) || plain_suggestion(verb).is_some()
}

/// Resolve a sanctioned surface verb to its candid plain verb (insider data).
pub fn candid_verb(verb: &str) -> String {
    for (pr, plain) in ACTIONS {
        if *pr == verb {
            return (*plain).to_string();
        }
    }
    verb.to_string()
}

// ─────────── the load-bearing functions ───────────

/// Substring-membership severity (spike `_sev`): the max weight of any severity key
/// contained in `text`, or 0.
pub fn severity(text: &str) -> i32 {
    SEVERITY
        .iter()
        .filter(|(k, _)| text.contains(k))
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
}

/// Redact the SENSITIVE tokens to `████` (whole-word). Spike `redact`.
pub fn redact(text: &str) -> String {
    let mut s = text.to_string();
    for w in SENSITIVE {
        s = replace_word(&s, w, REDACTION);
    }
    s
}

/// `E` — the pure, one-way `ACTUAL → OFFICIAL` derivation: euphemism (word-by-word,
/// in table order) then redaction. The polarity assertion (I1) makes construction
/// **fail loudly** if the result is ever uglier than the input — that is an
/// implementation error (a bug in the table), never swallowed (§12). Spike `E`.
pub fn e(candid: &str) -> String {
    let mut o = candid.to_string();
    for (plain, pr) in EUPHEMISM {
        o = replace_word(&o, plain, pr);
    }
    o = redact(&o);
    assert!(
        severity(&o) <= severity(candid),
        "I1 polarity violation: OFFICIAL {o:?} is uglier than ACTUAL {candid:?}"
    );
    o
}

/// The grandiosity scorer (#20). Spike `grandiosity`. Extracts maximal runs of ASCII
/// lowercase letters from the lowercased name (`re.findall(r"[a-z]+", name.lower())`),
/// then checks the honest/grand lexicons. On an honest hit, returns the alphabetically
/// first honest word (`sorted(w & HONEST_WORDS)[0]`).
pub fn grandiosity(name: &str) -> Grand {
    let words = ascii_lower_words(name);
    let mut honest_hits: Vec<&str> = words
        .iter()
        .filter(|w| HONEST_WORDS.contains(&w.as_str()))
        .map(|w| w.as_str())
        .collect();
    honest_hits.sort_unstable();
    honest_hits.dedup();
    if let Some(first) = honest_hits.first() {
        return Grand::Honest((*first).to_string());
    }
    if words.iter().any(|w| GRAND_WORDS.contains(&w.as_str())) {
        return Grand::Ok;
    }
    Grand::Bland
}

// ─────────── small text helpers (kept private to the single-source module) ───────────

/// Whole-word replacement mirroring Python `re.sub(r"\bFROM\b", TO, text)` for ASCII
/// word tokens. Word chars are `[A-Za-z0-9_]`. Works on `char`s so multi-byte UTF-8
/// in `text` (Hebrew, box-drawing, em-dashes) is never split. `from` is always an
/// ASCII token in this crate's tables.
fn replace_word(text: &str, from: &str, to: &str) -> String {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let from_chars: Vec<char> = from.chars().collect();
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let m = from_chars.len();
    if m == 0 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < n {
        if i + m <= n && chars[i..i + m] == from_chars[..] {
            let before_ok = i == 0 || !is_word(chars[i - 1]);
            let after_ok = i + m >= n || !is_word(chars[i + m]);
            if before_ok && after_ok {
                out.push_str(to);
                i += m;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Extract maximal runs of ASCII lowercase letters from the lowercased `name`.
/// Mirrors `re.findall(r"[a-z]+", name.lower())`.
fn ascii_lower_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for c in name.chars() {
        // Lowercase per-char; only ASCII a-z are kept as word characters.
        let lc = c.to_ascii_lowercase();
        if lc.is_ascii_lowercase() {
            cur.push(lc);
        } else if !cur.is_empty() {
            words.push(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        words.push(cur);
    }
    words
}

#[cfg(test)]
mod tests;
