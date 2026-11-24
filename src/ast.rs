#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub op_name: String,
    pub body: Vec<Stmt>,
    pub comments: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InertKind {
    WorldOpinion,
    Polls,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LawToggle {
    RetroactivelySanction(String),
    ExpungeLastDiscrepancy,
    WaiveGate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    Commit,
    Foreclose,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

impl BinOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    UnOp {
        op: UnOp,
        expr: Box<Expr>,
    },
    BinOp {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },

    Read(Box<Expr>),

    Cast {
        target: crate::model::Clearance,
        expr: Box<Expr>,
    },

    SelfDefense(Box<Expr>),

    /// `external(args)` is a deterministic mock: it logs a declared effect and returns `undisclosed`.
    External(Vec<Expr>),

    /// `via(proxy, inner)` prepends `proxy` to the attribution chain; the origin is never removed.
    Via {
        proxy: String,
        inner: Box<Expr>,
    },

    Apportionment {
        label: String,
        size: i64,
    },
    Index {
        coll: String,
        idx: Box<Expr>,
    },
    Balanced {
        coll: String,
    },

    FactsNew {
        label: String,
    },
    Length {
        coll: String,
    },

    RegistryNew {
        label: String,
        rule: String,
    },
    Route {
        coll: String,
        case: String,
    },
    EqualBeforeLaw {
        coll: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Assign {
        var: String,
        value: Expr,
    },

    Declare(Expr),

    If {
        cond: Expr,
        then_body: Vec<Stmt>,
        else_body: Vec<Stmt>,
    },

    While {
        cond: Expr,
        body: Vec<Stmt>,
    },

    FuncDef {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },

    Return(Option<Expr>),

    ExprStmt(Expr),

    Hasbara {
        talking_point: String,
        body: Vec<Stmt>,
    },

    Action {
        verb: String,
        target: String,
        self_defense: bool,
    },

    Allocate {
        name: String,
        what: String,
    },

    Bribe {
        name: String,
        amount: Expr,
    },

    Postpone,

    Elections,

    Mossad {
        body: Vec<Stmt>,
    },

    Blame {
        who: String,
    },

    Raise {
        name: String,
    },

    Whatabout {
        name: String,
    },

    /// Reads like `break` but lowers to a no-op `continue`.
    Ceasefire,

    Concern {
        who: Option<String>,
    },

    Criticism {
        subject: String,
    },

    Antisemitism {
        incident: String,
    },

    Access {
        entity: String,
    },

    Timeline {
        symbol: String,
    },

    EstablishCommission {
        name: String,
        subject: String,
    },

    Settlement {
        name: String,
        what: String,
    },

    HumanShields {
        verb: String,
        target: String,
    },

    Proportionate {
        claim: String,
    },

    Disputed {
        name: String,
        official: i64,
        actual: i64,
    },

    Deny {
        event: String,
    },

    Inert {
        kind: InertKind,
    },

    Investigate {
        subject: String,
    },

    AddressInternational,

    Announce {
        text: String,
    },

    Position {
        stance: Stance,
        subject: String,
    },

    Address {
        audience: crate::model::Audience,
        body: Vec<Stmt>,
    },

    PolyStatement {
        name: String,
        arms: Vec<(crate::model::Audience, Vec<Stmt>)>,
    },

    /// With no arm for the current audience this is a no-op, not an error.
    Invoke {
        name: String,
    },

    Legislate {
        toggle: LawToggle,
    },

    AllocateSlot {
        coll: String,
        idx: Expr,
        value: Expr,
    },

    Push {
        coll: String,
        item: Expr,
    },
    /// Delists the first live match; the entry stays in the list.
    Remove {
        coll: String,
        item: Expr,
    },

    Classify {
        coll: String,
        case: String,
        jurisdiction: crate::model::Jurisdiction,
    },
    /// Hides the case publicly but keeps it in `סודי`.
    Revoke {
        coll: String,
        case: String,
    },
}

pub fn vars_in(expr: &Expr) -> Vec<String> {
    let mut out = Vec::new();
    collect_vars(expr, &mut out);
    out
}

fn collect_vars(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) => {}
        Expr::Var(name) => {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        Expr::UnOp { expr, .. } => collect_vars(expr, out),
        Expr::BinOp { lhs, rhs, .. } => {
            collect_vars(lhs, out);
            collect_vars(rhs, out);
        }
        Expr::Call { args, .. } => {
            for a in args {
                collect_vars(a, out);
            }
        }
        Expr::Read(e) | Expr::SelfDefense(e) => collect_vars(e, out),
        Expr::Cast { expr, .. } => collect_vars(expr, out),
        Expr::External(args) => {
            for a in args {
                collect_vars(a, out);
            }
        }
        Expr::Via { inner, .. } => collect_vars(inner, out),
        Expr::Index { coll, idx } => {
            if !out.contains(coll) {
                out.push(coll.clone());
            }
            collect_vars(idx, out);
        }
        Expr::Balanced { coll } | Expr::Length { coll } | Expr::EqualBeforeLaw { coll } => {
            if !out.contains(coll) {
                out.push(coll.clone());
            }
        }
        Expr::Route { coll, .. } => {
            if !out.contains(coll) {
                out.push(coll.clone());
            }
        }
        Expr::Apportionment { .. } | Expr::FactsNew { .. } | Expr::RegistryNew { .. } => {}
    }
}

pub fn pretty(expr: &Expr) -> String {
    match expr {
        Expr::Int(n) => n.to_string(),
        Expr::Bool(b) => b.to_string(),
        Expr::Str(s) => format!("\"{s}\""),
        Expr::Var(name) => name.clone(),
        Expr::UnOp { op, expr } => {
            let sym = match op {
                UnOp::Neg => "-",
                UnOp::Not => "!",
            };
            format!("{sym}{}", pretty(expr))
        }
        Expr::BinOp { op, lhs, rhs } => {
            format!("{} {} {}", pretty(lhs), op.as_str(), pretty(rhs))
        }
        Expr::Call { name, args } => {
            let a: Vec<String> = args.iter().map(pretty).collect();
            format!("{name}({})", a.join(", "))
        }
        Expr::Read(e) => format!("read({})", pretty(e)),
        Expr::Cast { target, expr } => format!("({}) {}", target.label(), pretty(expr)),
        Expr::SelfDefense(e) => format!("(self_defense) {}", pretty(e)),
        Expr::External(args) => {
            let a: Vec<String> = args.iter().map(pretty).collect();
            format!("external({})", a.join(", "))
        }
        Expr::Via { proxy, inner } => format!("via({proxy}, {})", pretty(inner)),
        Expr::Apportionment { size, .. } => format!("apportionment[{size}]"),
        Expr::Index { coll, idx } => format!("index({coll}, {})", pretty(idx)),
        Expr::Balanced { coll } => format!("balanced({coll})"),
        Expr::FactsNew { .. } => "facts_on_the_ground()".to_string(),
        Expr::Length { coll } => format!("length({coll})"),
        Expr::RegistryNew { rule, .. } => format!("registry(\"{rule}\")"),
        Expr::Route { coll, case } => format!("route({coll}, {case})"),
        Expr::EqualBeforeLaw { coll } => format!("equal_before_the_law({coll})"),
    }
}
