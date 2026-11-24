/// Hebrew for "secret"; the clearance needed to read ACTUAL.
pub const SODI: &str = "\u{05e1}\u{05d5}\u{05d3}\u{05d9}";

/// Shown as the ACTUAL face of an announced claim; no clearance recovers anything behind it.
pub const UNAVAILABLE: &str = "[unavailable: no one has said what this actually does]";

/// What an under-cleared reader sees in place of a covert element.
pub const REDACTED_ELEM: &str = "[REDACTED]";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Clearance {
    Public,
    Restricted,
    Sodi,
}

impl Clearance {
    pub fn label(self) -> &'static str {
        match self {
            Clearance::Public => "PUBLIC",
            Clearance::Restricted => "RESTRICTED",
            Clearance::Sodi => SODI,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provenance {
    AuthoredActual,
    AuthoredOfficial,
    Covert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Audience {
    Domestic,
    International,
    Record,
}

/// The public non-answer for a deniable attribution.
pub const NEITHER_CONFIRM_NOR_DENY: &str = "responsibility: [neither confirm nor deny]";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Attribution {
    Traceable(Vec<String>),
    Deniable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetaEntry {
    pub change: String,
    pub turn: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Undisclosed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub value: i64,
    pub covert: bool,
}

/// `remove` sets `delisted`; entries are never dropped from the backing list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListEntry {
    pub item: String,
    pub delisted: bool,
    pub covert: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Jurisdiction {
    Military,
    Civilian,
}

impl Jurisdiction {
    pub fn court(self) -> &'static str {
        match self {
            Jurisdiction::Military => "military court",
            Jurisdiction::Civilian => "civilian court",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegEntry {
    pub key: String,
    pub descriptor: Option<String>,
    pub jurisdiction: Jurisdiction,
    pub revoked: bool,
    pub covert: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Int(i64),
    Bool(bool),
    Str(String),
    Unit,
    Entity {
        name: String,
        category: String,
    },
    Undisclosed,

    Apportionment {
        label: String,
        slots: Vec<Slot>,
    },

    FactsList {
        label: String,
        entries: Vec<ListEntry>,
    },

    Registry {
        label: String,
        official_rule: String,
        entries: Vec<RegEntry>,
    },
}

impl Val {
    pub fn truth(&self) -> Truth {
        match self {
            Val::Bool(true) => Truth::True,
            Val::Bool(false) => Truth::False,
            Val::Int(0) => Truth::False,
            Val::Int(_) => Truth::True,
            Val::Str(s) => {
                if s.is_empty() {
                    Truth::False
                } else {
                    Truth::True
                }
            }
            Val::Unit => Truth::False,
            Val::Entity { .. } => Truth::True,
            Val::Undisclosed => Truth::Undisclosed,
            Val::Apportionment { .. } | Val::FactsList { .. } | Val::Registry { .. } => Truth::True,
        }
    }

    /// A short name for error messages, with its article.
    pub fn type_name(&self) -> &'static str {
        match self {
            Val::Int(_) => "an integer",
            Val::Bool(_) => "a boolean",
            Val::Str(_) => "a string",
            Val::Unit => "a unit value",
            Val::Entity { .. } => "an entity",
            Val::Undisclosed => "an undisclosed value",
            Val::Apportionment { .. } => "an apportionment",
            Val::FactsList { .. } => "a facts list",
            Val::Registry { .. } => "a registry",
        }
    }

    /// Covert collection elements render as `[REDACTED]`, since this string can reach a public event.
    pub fn render(&self) -> String {
        match self {
            Val::Int(n) => n.to_string(),
            Val::Bool(b) => b.to_string(),
            Val::Str(s) => s.clone(),
            Val::Unit => "unit".to_string(),
            Val::Entity { name, .. } => name.clone(),
            Val::Undisclosed => "undisclosed".to_string(),
            Val::Apportionment { slots, .. } => {
                let cells: Vec<String> = slots
                    .iter()
                    .map(|s| {
                        if s.covert {
                            REDACTED_ELEM.to_string()
                        } else {
                            s.value.to_string()
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
            Val::FactsList { entries, .. } => {
                let cells: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        if e.covert {
                            REDACTED_ELEM.to_string()
                        } else if e.delisted {
                            format!("{} (delisted)", e.item)
                        } else {
                            e.item.clone()
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
            Val::Registry { entries, .. } => {
                let cells: Vec<String> = entries
                    .iter()
                    .map(|e| {
                        if e.covert {
                            REDACTED_ELEM.to_string()
                        } else {
                            let tag = if e.revoked { " (revoked)" } else { "" };
                            format!("{}\u{2192}{}{tag}", e.key, e.jurisdiction.court())
                        }
                    })
                    .collect();
                format!("[{}]", cells.join(", "))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Event {
    pub official: String,
    pub candid: String,
    pub clearance: Clearance,
    pub provenance: Provenance,
    pub audience: Audience,
    pub attribution: Option<Attribution>,
}

impl Event {
    pub fn public(official: impl Into<String>, candid: impl Into<String>) -> Self {
        Event {
            official: official.into(),
            candid: candid.into(),
            clearance: Clearance::Public,
            provenance: Provenance::AuthoredActual,
            audience: Audience::Record,
            attribution: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Discrepancy {
    pub claim: String,
    pub reality: String,
    pub turn: u64,
}
