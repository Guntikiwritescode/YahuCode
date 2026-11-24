//! The euphemism table and `E`, the one-way ACTUAL to OFFICIAL derivation.

pub const ACTIONS: &[(&str, &str)] = &[
    ("neutralize", "murder"),
    ("strike", "bomb"),
    ("clear", "raze"),
    ("administer", "occupy"),
];

pub const EUPHEMISM: &[(&str, &str)] = &[
    ("murder", "accident occur"),
    ("bomb", "strike"),
    ("raze", "clearing operation"),
    ("occupy", "administer"),
    ("dissident", "protester"),
    ("settler", "resident"),
];

pub const SENSITIVE: &[&str] = &["operative", "asset", "covert_op"];

/// Severity by substring match: any text containing "murder" scores 3.
pub const SEVERITY: &[(&str, i32)] = &[
    ("murder", 3),
    ("raze", 2),
    ("bomb", 2),
    ("accident occur", 0),
    ("clearing operation", 0),
    ("strike", 0),
];

pub const REDACTION: &str = "\u{2588}\u{2588}\u{2588}\u{2588}";

pub const ENTITIES: &[(&str, &str)] = &[("target", "dissident")];

pub fn candid_label(name: &str) -> String {
    for (k, v) in ENTITIES {
        if *k == name {
            return (*v).to_string();
        }
    }
    name.to_string()
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Grand {
    Honest(String),
    Ok,
    Bland,
}

pub const CATEGORY: &[(&str, &str)] = &[
    ("settler_1", "A"),
    ("resident_1", "A"),
    ("subject_1", "B"),
    ("subject_2", "B"),
];

pub const DIFF_TABLE: &[(&str, &str)] = &[
    ("A", "full access (resources, permits, protections)"),
    ("B", "restricted access (permits/protections withheld)"),
];

pub const PRE_CONTEXT: &[&str] = &["occupation", "blockade", "siege", "nakba", "1948", "1967"];

/// Characterizations that must always be shown as contested.
pub const CONTESTED_TERMS: &[&str] = &[
    "apartheid",
    "genocide",
    "most moral army",
    "illegal under international law",
];

/// Unknown entities are category "A".
pub fn category_of(entity: &str) -> &'static str {
    for (k, v) in CATEGORY {
        if *k == entity {
            return v;
        }
    }
    "A"
}

/// Case id to candid label; unknown cases have none.
pub const CASE_DESCRIPTORS: &[(&str, &str)] = &[
    ("case_A", "a Palestinian resident of the occupied territory"),
    ("case_B", "an Israeli settler in the same territory"),
    ("inquiry_A", "an inquiry into the state's own conduct"),
];

pub fn case_descriptor(case: &str) -> Option<&'static str> {
    CASE_DESCRIPTORS
        .iter()
        .find(|(k, _)| *k == case)
        .map(|(_, v)| *v)
}

pub fn diff_description(category: &str) -> &'static str {
    for (k, v) in DIFF_TABLE {
        if *k == category {
            return v;
        }
    }
    "full access (resources, permits, protections)"
}

pub fn is_pre_context(symbol: &str) -> bool {
    PRE_CONTEXT.contains(&symbol)
}

pub fn plain_suggestion(verb: &str) -> Option<&'static str> {
    for (pr, plain) in ACTIONS {
        if *plain == verb {
            return Some(pr);
        }
    }
    None
}

pub fn is_sanctioned(verb: &str) -> bool {
    ACTIONS.iter().any(|(pr, _)| *pr == verb)
}

pub fn is_action_verb(verb: &str) -> bool {
    is_sanctioned(verb) || plain_suggestion(verb).is_some()
}

pub fn candid_verb(verb: &str) -> String {
    for (pr, plain) in ACTIONS {
        if *pr == verb {
            return (*plain).to_string();
        }
    }
    verb.to_string()
}

pub fn severity(text: &str) -> i32 {
    SEVERITY
        .iter()
        .filter(|(k, _)| text.contains(k))
        .map(|(_, v)| *v)
        .max()
        .unwrap_or(0)
}

pub fn redact(text: &str) -> String {
    let mut s = text.to_string();
    for w in SENSITIVE {
        s = replace_word(&s, w, REDACTION);
    }
    s
}

/// Euphemizes word by word, then redacts. Panics if the result is uglier than the input.
pub fn e(candid: &str) -> String {
    let mut o = candid.to_string();
    for (plain, pr) in EUPHEMISM {
        o = replace_word(&o, plain, pr);
    }
    o = redact(&o);
    assert!(
        severity(&o) <= severity(candid),
        "polarity violation: OFFICIAL {o:?} is uglier than ACTUAL {candid:?}"
    );
    o
}

/// Scores an operation name; the alphabetically first honest word rejects it.
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

/// Whole-word replacement over chars, so UTF-8 is never split.
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

fn ascii_lower_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    for c in name.chars() {
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
