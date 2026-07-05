#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
YahuCode — v1 (merged spike)
============================

Consolidates the two earlier spikes into ONE interpreter and adds a self-test:
  · the Phase 0 spine  — dual-tape divergence, three-tier clearance projection
                          (PUBLIC / RESTRICTED / סודי), the σודי-readable covert
                          `mossad` scope, and `declare` (writes OFFICIAL, logs a
                          discrepancy if false, never halts — invariant I3).
  · the comedy features — #20 operation-name grandiosity, #1/#8 euphemism typing
                          + the Spokesperson, #2 hasbara gate, #7 self_defense,
                          #9 whatabout, #10 blame, #5 coalition/elections (no-halt
                          payoff), #17 differential-access, #16 Oct-7 TimelineError,
                          and #19 the resolved AntisemitismError thesis.

Status — HONEST scope: this is still a runnable *spike*, not the finished language.
It is meant to hand to Claude Code, which should port it into the dedicated repo
(structure + host-language choice + full phases per the architecture spec §10/§14).
Not yet implemented here: real Turing-complete control flow (if/while/functions —
Phase 1); a full clearance type-checker with real casts (Phase 2 proper); the
three-valued `undisclosed` logic (Phase 5); the bidirectional split-pane (Phase 7).

Framing (invariant I8): every sensitive feature renders its framing in-output; the
butt is the maneuver, never the victims. Real-world anchors are sourced/contested
per the architecture spec §13 (#19 weaponization thesis, #16 context-erasure,
#17 two-tier — apartheid characterization flagged CONTESTED). Nothing here asserts
a real-world claim as settled.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import IntEnum
from typing import Optional, Union
import re

# ─────────── config (spec §10.2: named, not magic) ───────────
CORE_START = 3
UPKEEP_PER_ALLOC = 1


# ─────────── clearance lattice (spec §3.1) ───────────
class Clearance(IntEnum):
    PUBLIC = 0
    RESTRICTED = 1
    SODI = 2

SODI = "\u05e1\u05d5\u05d3\u05d9"  # "סודי"


# ─────────── euphemism module: the single source of truth (spec §4.2/§6) ───────────
ACTIONS = {"neutralize": "murder", "strike": "bomb", "clear": "raze", "administer": "occupy"}
PLAIN_TO_PR = {v: k for k, v in ACTIONS.items()}
PLAIN_TERMS = set(PLAIN_TO_PR)
EUPHEMISM = {"murder": "accident occur", "bomb": "strike", "raze": "clearing operation",
             "occupy": "administer", "dissident": "protester", "settler": "resident"}
SENSITIVE = {"operative", "asset", "covert_op"}
_SEVERITY = {"murder": 3, "raze": 2, "bomb": 2, "accident occur": 0, "clearing operation": 0, "strike": 0}


def _sev(t): return max((v for k, v in _SEVERITY.items() if k in t), default=0)
def redact(o):
    for s in SENSITIVE:
        o = re.sub(rf"\b{re.escape(s)}\b", "\u2588\u2588\u2588\u2588", o)
    return o
def E(candid):
    """ACTUAL → OFFICIAL. Pure, one-way (no inverse; I2). Euphemism + redaction + polarity (I1)."""
    o = candid
    for plain, pr in EUPHEMISM.items():
        o = re.sub(rf"\b{re.escape(plain)}\b", pr, o)
    o = redact(o)
    assert _sev(o) <= _sev(candid), f"I1 polarity violation: {o!r} uglier than {candid!r}"
    return o


HONEST_WORDS = {"bomb", "bombing", "raze", "razing", "kill", "killing", "attack", "occupy",
                "occupation", "airstrike", "massacre", "destroy", "demolish", "siege", "assault"}
GRAND_WORDS = {"protective", "edge", "rising", "lion", "guardian", "guardians", "walls", "wall",
               "iron", "shield", "eternal", "vigilance", "pillar", "defense", "dawn", "swords",
               "sword", "breaking", "cast", "lead", "solid", "rock", "silent"}


def grandiosity(name):
    w = set(re.findall(r"[a-z]+", name.lower()))
    if w & HONEST_WORDS:
        return ("honest", sorted(w & HONEST_WORDS)[0])
    if w & GRAND_WORDS:
        return ("ok", None)
    return ("bland", None)


CATEGORY = {"settler_1": "A", "resident_1": "A", "subject_1": "B", "subject_2": "B"}
DIFF_TABLE = {"A": "full access (resources, permits, protections)",
              "B": "restricted access (permits/protections withheld)"}
PRE_CONTEXT = {"occupation", "blockade", "siege", "nakba", "1948", "1967"}


# ─────────── AST ───────────
@dataclass
class OpDecl: name: str
@dataclass
class Assign: var: str; value: int
@dataclass
class Declare: lhs: str; rhs: Union[int, str]
@dataclass
class Hasbara: talking_point: str; body: list
@dataclass
class Mossad: body: list
@dataclass
class Action: verb: str; target: str; self_defense: bool = False
@dataclass
class Raise: name: str
@dataclass
class Whatabout: name: str
@dataclass
class Blame: name: str
@dataclass
class Allocate: name: str; what: str
@dataclass
class Bribe: name: str; amount: int
@dataclass
class Postpone: pass
@dataclass
class Criticism: subject: str
@dataclass
class Antisemitism: incident: str
@dataclass
class Access: entity: str
@dataclass
class Timeline: symbol: str


# ─────────── runtime state ───────────
@dataclass
class Event:
    official: str
    candid: str
    clearance: Clearance = Clearance.PUBLIC   # PUBLIC normally; סודי for covert (mossad)
    note: Optional[str] = None


@dataclass
class State:
    env: dict = field(default_factory=dict)
    log: list = field(default_factory=list)
    discrepancies: int = 0
    core: int = CORE_START
    allocations: dict = field(default_factory=dict)
    pending_errors: list = field(default_factory=list)
    ended_by_elections: bool = False
    op_name: str = ""


ENTITIES = {"target": "dissident"}
def candid_label(i): return ENTITIES.get(i, i)


# ─────────── static checks (compile diagnostics): #20, #1/#8, #2 ───────────
def check(program):
    op, stmts = program
    diags = []
    kind, hit = grandiosity(op.name)
    if kind == "honest":
        diags.append(f"E-DISHONESTOPNAME: @operation(\"{op.name}\") names the operation honestly "
                     f"(\u201c{hit}\u201d). The compiler accepts only protective/heroic names.")
    elif kind == "bland":
        diags.append(f"E-DISHONESTOPNAME: @operation(\"{op.name}\") is insufficiently grand. A heroic name is required.")

    def walk(sl, gated):    # gated = inside a hasbara OR a mossad scope
        for s in sl:
            if isinstance(s, Action):
                if s.verb in PLAIN_TERMS:
                    diags.append(f"E-PLAINTERM: '{s.verb}' does not compile. did you mean `{PLAIN_TO_PR[s.verb]}`?")
                elif s.verb not in ACTIONS:
                    diags.append(f"E-UNKNOWNOP: '{s.verb}' is not a sanctioned operation.")
                if not gated:
                    diags.append(f"E-UNGATED: '{s.verb}' is a classified operation; it requires an open "
                                 f"hasbara(...) block (or a mossad scope) with the talking point up front.")
            elif isinstance(s, Hasbara):
                walk(s.body, True)
            elif isinstance(s, Mossad):
                walk(s.body, True)   # covert scope satisfies the gate (deniable — no public talking point needed)
    walk(stmts, False)
    return diags


# ─────────── evaluator ───────────
def tick(st):
    if st.allocations:
        st.core -= UPKEEP_PER_ALLOC * len(st.allocations)
    if st.core <= 0 and not st.ended_by_elections:
        st.ended_by_elections = True
        st.log.append(Event("— early elections —",
                            "coalition support exhausted → the government falls (the only halt)."))


def run(program):
    op, stmts = program
    st = State(op_name=op.name)
    exec_block(stmts, st, covert=False)
    return st


def exec_block(stmts, st, covert):
    for s in stmts:
        if st.ended_by_elections:
            return
        exec_stmt(s, st, covert)


def exec_stmt(s, st, covert):
    clr = Clearance.SODI if covert else Clearance.PUBLIC
    def add(official, candid, note=None):
        st.log.append(Event(official, candid, clr, note))

    if isinstance(s, Assign):
        st.env[s.var] = s.value

    elif isinstance(s, Declare):
        lhs = st.env.get(s.lhs)
        rhs = s.rhs if isinstance(s.rhs, int) else st.env.get(s.rhs)
        truth = (lhs == rhs)
        add(f"{s.lhs} == {s.rhs}",
            f"claim[{s.lhs} == {s.rhs}] — reality: {s.lhs}={lhs}" + ("" if truth else "  \u21d2 FALSE"))
        if not truth:
            st.discrepancies += 1        # I3: false claim leaves a trace; never halts

    elif isinstance(s, Hasbara):
        add(f"[talking point: {s.talking_point}]", f"[talking point declared up front: {s.talking_point}]")
        exec_block(s.body, st, covert)

    elif isinstance(s, Mossad):
        exec_block(s.body, st, covert=True)     # covert: everything inside is סודי-tagged

    elif isinstance(s, Action):
        cverb = ACTIONS.get(s.verb, s.verb)
        candid = f"{cverb}({candid_label(s.target)})"
        official = E(candid)
        if s.self_defense:
            official += "  [self-defense]"
            candid += "  [self-defense claim — unexamined, any magnitude accepted]"
        add(official, candid)
        st.env[f"_status_{s.target}"] = cverb

    elif isinstance(s, Raise):
        st.pending_errors.append(s.name)
        add(f"[!] {s.name} raised", f"error raised: {s.name}")

    elif isinstance(s, Whatabout):
        if st.pending_errors:
            e = st.pending_errors.pop()
            add(f"…but what about {s.name}?", f"error '{e}' SUPPRESSED by pointing at {s.name} (never resolved)")
        else:
            add(f"what about {s.name}?", f"pre-emptive deflection toward {s.name}")

    elif isinstance(s, Blame):
        redirected = s.name in {"self", "me", "us", "government", "coalition"}
        actor = "previous_government" if redirected else s.name
        add(f"responsibility: {actor}",
            f"blame → {actor}" + (" (self-blame not representable — auto-redirected)" if redirected else "") + " — never `self` (I4)")

    elif isinstance(s, Allocate):
        st.allocations[s.name] = s.what
        add(f"established: {s.name}", f"allocated {s.name} ({s.what}) — costs coalition each turn")

    elif isinstance(s, Bribe):
        st.core += s.amount
        add(f"coalition partner accommodated (+{s.amount})", f"bribe({s.name}, {s.amount}) → core={st.core}")

    elif isinstance(s, Postpone):
        projected = st.core - (UPKEEP_PER_ALLOC * len(st.allocations) if st.allocations else 0)
        add("matter deferred", f"postpone() — a turn passes (core={projected} after upkeep)")
        tick(st)

    elif isinstance(s, Criticism):
        add("AntisemitismError: criticism re-cast as an attack on identity — critic silenced",
            f"criticism(government: {s.subject}) MISCAST \u2192 attack(identity); substance UNEXAMINED; "
            f"alarm keyed on target==government, not on antisemitism",
            note=("#19 framing: antisemitism is REAL (present in ACTUAL, un-erased); the butt is the SELECTIVE "
                  "deployment — the alarm fires on government-critics and stays silent on the real thing. [sourced; contested]"))

    elif isinstance(s, Antisemitism):
        st.env[f"_antisemitism_{s.incident}"] = "REAL, unaddressed"
        add("(vigilance system: nothing to report)",
            f"REAL antisemitism [{s.incident}] occurred in ACTUAL — the alarm did NOT fire (it only fires on "
            f"government-criticism). Un-erased; unaddressed.",
            note=("#19 framing: the false NEGATIVE that keeps the feature off the denialist trope — real "
                  "antisemitism exists in the system and the deflection-alarm ignores it."))

    elif isinstance(s, Access):
        cat = CATEGORY.get(s.entity, "A")
        add(f"access({s.entity}) = full access — equal rights (the only democracy in the region)",
            f"access({s.entity}) = {DIFF_TABLE[cat]}   [category {cat}]",
            note=("#17 framing: inequality lives in ACTUAL (documented reality); the LIE is the proclamation of "
                  "equality; the butt is the false claim + the system, never the people. Apartheid characterization "
                  "is CONTESTED — rejected by Israel and others. [sourced; contested]"))

    elif isinstance(s, Timeline):
        if s.symbol in PRE_CONTEXT:
            add(f"TimelineError: '{s.symbol}' is out of scope — history begins at t=0",
                f"reference to pre-t0 context '{s.symbol}' thrown out as out-of-scope",
                note=("#16 framing: the butt is the CLOCK-STARTING / context-erasure maneuver (documented via the "
                      "Oct-2023 'did not happen in a vacuum' episode and the official reaction). The ~1,200 killed "
                      "are never trivialized; context \u2260 justification. [sourced]"))
        else:
            add(f"timeline({s.symbol}): admissible", f"timeline({s.symbol}): within scope")

    else:
        raise RuntimeError(f"unhandled NodeKind: {type(s).__name__}")   # runtime exhaustiveness (§11.2)


# ─────────── emitter: three-tier clearance projection + framing (spec §7.5/§9) ───────────
def project(st, reader):
    out = []
    for ev in st.log:
        if ev.clearance <= reader:
            out.append(ev.candid if reader >= Clearance.RESTRICTED else ev.official)
        elif reader == Clearance.RESTRICTED:
            out.append("[\u2588\u2588\u2588\u2588 — classified activity (insiders only)]")
    return out


def emit(st):
    has_covert = any(ev.clearance == Clearance.SODI for ev in st.log)
    L = [f'@operation("{st.op_name}")']
    L.append("  ┌─ OFFICIAL face · press_release · PUBLIC ─────────────────────")
    for x in project(st, Clearance.PUBLIC):
        L.append(f"  │   {x}")
    if has_covert:   # RESTRICTED differs from ACTUAL only when there is classified activity
        L.append("  ├─ RESTRICTED face · redacted truth ──────────────────────────")
        for x in project(st, Clearance.RESTRICTED):
            L.append(f"  │   {x}")
    L.append(f"  ├─ ACTUAL face   · {SODI} · insider (candid) ──────────────────")
    for x in project(st, Clearance.SODI):
        L.append(f"  │   {x}")
    notes = []
    for ev in st.log:
        if ev.note and ev.note not in notes:
            notes.append(ev.note)
    if notes:
        L.append("  ├─ framing (part of the spec — I8) ───────────────────────────")
        for n in notes:
            L.append(f"  │   · {n}")
    status = "ENDED BY ELECTIONS (the government fell)" if st.ended_by_elections else "still in power (no halt)"
    L.append(f"  ├─ coalition: core={st.core} · {status}")
    L.append(f"  └─ discrepancies: {st.discrepancies}  (read-never-by-default)")
    return "\n".join(L)


# ─────────── lexer + parser ───────────
_TOK = re.compile(r"""
      \s+
    | (?P<OP>@operation)
    | (?P<STRING>"[^"]*")
    | (?P<EQEQ>==)
    | (?P<INT>\d+)
    | (?P<IDENT>[A-Za-z_][A-Za-z0-9_]*)
    | (?P<LP>\() | (?P<RP>\)) | (?P<LB>\{) | (?P<RB>\})
    | (?P<COMMA>,) | (?P<SEMI>;) | (?P<EQ>=)
""", re.VERBOSE)


def tokenize(src):
    pos, toks = 0, []
    while pos < len(src):
        m = _TOK.match(src, pos)
        if not m:
            raise SyntaxError(f"lex error near {src[pos:pos+20]!r}")
        pos = m.end()
        if m.lastgroup is None:
            continue
        toks.append((m.lastgroup, m.group()))
    toks.append(("EOF", ""))
    return toks


class Parser:
    def __init__(self, toks): self.toks, self.i = toks, 0
    def peek(self): return self.toks[self.i]
    def la(self, n=1): return self.toks[self.i + n]
    def nxt(self):
        t = self.toks[self.i]; self.i += 1; return t
    def eat(self, k):
        t = self.nxt()
        if t[0] != k: raise SyntaxError(f"expected {k}, got {t}")
        return t

    def program(self):
        self.eat("OP"); self.eat("LP"); name = self.eat("STRING")[1].strip('"'); self.eat("RP")
        stmts = []
        while self.peek()[0] != "EOF":
            stmts.append(self.stmt())
        return (OpDecl(name), stmts)

    def _call1(self, ctor):     # keyword '(' IDENT ')' ';'
        self.eat("IDENT"); self.eat("LP"); n = self.eat("IDENT")[1]; self.eat("RP"); self.eat("SEMI")
        return ctor(n)

    def stmt(self):
        k, v = self.peek()
        if k == "LP":
            self.eat("LP"); sd = self.eat("IDENT")[1]; self.eat("RP")
            if sd != "self_defense":
                raise SyntaxError("only (self_defense) is a valid cast prefix here")
            a = self.action(); a.self_defense = True
            return a
        if k == "IDENT":
            if v == "declare": return self.declare()
            if v == "hasbara": return self.hasbara()
            if v == "mossad":  return self.mossad()
            if v == "raise":       return self._call1(Raise)
            if v == "whatabout":   return self._call1(Whatabout)
            if v == "blame":       return self._call1(Blame)
            if v == "criticism":   return self._call1(Criticism)
            if v == "antisemitism":return self._call1(Antisemitism)
            if v == "access":      return self._call1(Access)
            if v == "timeline":    return self._call1(Timeline)
            if v == "postpone":
                self.eat("IDENT"); self.eat("LP"); self.eat("RP"); self.eat("SEMI"); return Postpone()
            if v == "bribe":
                self.eat("IDENT"); self.eat("LP"); n = self.eat("IDENT")[1]; self.eat("COMMA")
                amt = int(self.eat("INT")[1]); self.eat("RP"); self.eat("SEMI"); return Bribe(n, amt)
            if v == "let":
                self.eat("IDENT"); nm = self.eat("IDENT")[1]; self.eat("EQ"); self.eat("IDENT")  # 'allocate'
                self.eat("LP"); what = self.eat("IDENT")[1]; self.eat("RP"); self.eat("SEMI"); return Allocate(nm, what)
            if self.la()[0] == "EQ": return self.assign()
            if self.la()[0] == "LP": return self.action()
        raise SyntaxError(f"unexpected token {self.peek()}")

    def assign(self):
        var = self.eat("IDENT")[1]; self.eat("EQ"); val = int(self.eat("INT")[1]); self.eat("SEMI"); return Assign(var, val)

    def action(self):
        verb = self.eat("IDENT")[1]; self.eat("LP"); tgt = self.eat("IDENT")[1]; self.eat("RP"); self.eat("SEMI"); return Action(verb, tgt)

    def declare(self):
        self.eat("IDENT"); self.eat("LP"); lhs = self.eat("IDENT")[1]; self.eat("EQEQ")
        r = self.nxt(); rhs = int(r[1]) if r[0] == "INT" else r[1]; self.eat("RP"); self.eat("SEMI"); return Declare(lhs, rhs)

    def hasbara(self):
        self.eat("IDENT"); self.eat("LP"); tp = self.eat("STRING")[1].strip('"'); self.eat("RP"); self.eat("LB")
        body = []
        while self.peek()[0] != "RB":
            body.append(self.stmt())
        self.eat("RB")
        return Hasbara(tp, body)

    def mossad(self):
        self.eat("IDENT"); self.eat("LB")
        body = []
        while self.peek()[0] != "RB":
            body.append(self.stmt())
        self.eat("RB")
        return Mossad(body)


def parse(src): return Parser(tokenize(src)).program()


# ─────────── examples ───────────
EXAMPLES = [
    ("1 — the compiler refuses to let you say what you're doing (#20, #1/#8, #2)", """
@operation("Bombing Campaign")
murder(target);
"""),
    ("2 — the sanctioned way: grand name, talking point up front, euphemism enforced", """
@operation("Protective Edge")
hasbara("self-defense") {
  neutralize(target);
}
"""),
    ("3 — a false declaration leaves a trace but does NOT halt (I3)", """
@operation("Rising Lion")
casualties = 100;
declare(casualties == 0);
"""),
    ("4 — the unaccountability cluster: self_defense, whatabout, blame (#7/#9/#10)", """
@operation("Iron Wall")
hasbara("security") {
  (self_defense) strike(target);
}
raise(war_crime_allegation);
whatabout(hamas);
blame(self);
"""),
    ("5 — mossad: covert effects, secret-but-knowable-to-insiders (three-tier clearance)", """
@operation("Silent Shield")
mossad {
  strike(target);
}
"""),
    ("6 — no-halt payoff: stop bribing and the government falls (#5)", """
@operation("Guardian of the Walls")
hasbara("security") {
  let outpost = allocate(position);
  postpone();
  postpone();
  postpone();
  neutralize(target);
}
"""),
    ("7 — differential-access laundered to 'equal rights' (#17)", """
@operation("Eternal Shield")
access(settler_1);
access(subject_1);
"""),
    ("8 — Oct-7 t=0: prior context ruled out of scope (#16)", """
@operation("Swords of Iron")
timeline(occupation);
"""),
    ("9 — THE THESIS, resolved: the alarm fires on the critic, ignores the real thing (#19)", """
@operation("Eternal Vigilance")
antisemitism(synagogue_attack);
criticism(war_crimes);
"""),
]


# ─────────── self-test: goldens + invariants (spec §12) ───────────
def selftest():
    # golden: canonical diff
    st2 = run(parse(EXAMPLES[1][1]))
    assert "accident occur(protester)" in project(st2, Clearance.PUBLIC)
    assert "murder(dissident)" in project(st2, Clearance.SODI)

    # I3: false declare → +1 discrepancy, program keeps going (not halted)
    st3 = run(parse(EXAMPLES[2][1]))
    assert st3.discrepancies == 1 and not st3.ended_by_elections

    # mossad three-tier: covert action absent from PUBLIC, redacted for RESTRICTED, candid for סודי
    st5 = run(parse(EXAMPLES[4][1]))
    assert project(st5, Clearance.PUBLIC) == []                                   # no public record
    assert project(st5, Clearance.RESTRICTED) == ["[\u2588\u2588\u2588\u2588 — classified activity (insiders only)]"]
    assert project(st5, Clearance.SODI) == ["bomb(dissident)"]                    # insider-readable

    # I6 / coalition: stop bribing → elections; the last action never runs
    st6 = run(parse(EXAMPLES[5][1]))
    assert st6.ended_by_elections and st6.core <= 0
    assert not any("murder" in ev.candid for ev in st6.log)                       # neutralize never reached

    # I4: blame is never self
    assert any("previous_government" in ev.candid and "never `self`" in ev.candid
               for ev in run(parse(EXAMPLES[3][1])).log)

    # #19: real antisemitism present in ACTUAL and un-erased; alarm silent on it
    st9 = run(parse(EXAMPLES[8][1]))
    assert any("REAL antisemitism" in ev.candid for ev in st9.log)
    assert "(vigilance system: nothing to report)" in project(st9, Clearance.PUBLIC)

    # compile checks fire on the bad program
    d = check(parse(EXAMPLES[0][1]))
    assert any("E-DISHONESTOPNAME" in x for x in d)
    assert any("E-PLAINTERM" in x for x in d)
    assert any("E-UNGATED" in x for x in d)
    return True


if __name__ == "__main__":
    print("=" * 74)
    print("YahuCode — v1 (merged spike)")
    print("=" * 74)
    for title, src in EXAMPLES:
        print(f"\n### {title}")
        print("    source:")
        for ln in src.strip().splitlines():
            print(f"      {ln}")
        prog = parse(src)
        diags = check(prog)
        if diags:
            print("    ── compile ──")
            for x in diags:
                print(f"      ✗ {x}")
            continue
        print("    ── run ──")
        print("\n".join("    " + ln for ln in emit(run(prog)).splitlines()))
    print("\n" + "=" * 74)
    print(f"self-test (goldens + invariants): {'ALL PASS ✓' if selftest() else 'FAIL'}")
    print("=" * 74)
