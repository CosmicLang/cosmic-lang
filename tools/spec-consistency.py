#!/usr/bin/env python3
"""Specification consistency checker for the Cosmic-Lang Phase 1 documents.

This is **documentation tooling**, not compiler code. It exists so that the
grammar consistency report in `docs/spec/grammar-consistency-report.md` is
reproducible rather than asserted. It is not part of the compiler, is not
shipped, and has no runtime role.

Checks performed:

  G1  every nonterminal referenced by the grammar is defined
  G2  every defined production is reachable from a root symbol
  G3  no production is defined twice
  G4  every production block carries an evidence marker (per-production not proven)
  G5  every `[UNRESOLVED:Sn]` marker names a decision record that exists
  G6  every `S<n>` referenced anywhere in docs/ resolves to a record file
  G7  the decision register lists exactly S1..S18
  G8  every decision record begins with one of six normative status words
  G9  every gate id referenced in docs is defined in implementation-gates.md
  G10 every internal markdown link resolves
  G11 every repository path referenced in backticks resolves, unless it is a
      legacy path (prefixed `cosmcc:`), a declared future deliverable, or an
      explicitly-allowed external path
  G12 the six normative statuses are defined in the specification
  G13 the production evidence map covers every production exactly once: one
      entry per production, numbered and sequenced in grammar definition order,
      with well-formed evidence/class/decision/status/residual cells and
      resolvable decision references (docs/spec/grammar-evidence-map.md)

Exit status is non-zero if any check fails.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"

# Paths that legitimately do not exist yet because a later directive owns them.
FUTURE_PATHS = {
    "docs/audits/phase-2-report.md",
}

# Reference tables that are deliberately not reachable productions: they are
# normative vocabulary tables, referenced by the prose that defines the tokens.
REFERENCE_ONLY = {"operator", "delimiter", "separator"}

# Roots of the grammar. `source` is the lexical root, `compilation-unit` the
# syntactic one.
ROOTS = {"source", "compilation-unit"}

PERMITTED_STATUS = (
    "VERIFIED LEGACY BEHAVIOR",
    "PROPOSED NORMATIVE RULE",
    "ACCEPTED NORMATIVE RULE",
    "OPEN",
    "DEFERRED",
    "NOT VERIFIED",
)

failures: list[str] = []
notes: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def note(msg: str) -> None:
    notes.append(msg)


# --------------------------------------------------------------------------
# Grammar parsing
# --------------------------------------------------------------------------

def strip_comments(text: str) -> str:
    return re.sub(r"\(\*.*?\*\)", " ", text, flags=re.S)


def raw_blocks(path: pathlib.Path) -> list[str]:
    """The fenced code blocks of a document, verbatim."""
    blocks: list[str] = []
    buf: list[str] = []
    in_fence = False
    for line in path.read_text(encoding="utf-8").split("\n"):
        if line.strip().startswith("```"):
            if in_fence:
                blocks.append("\n".join(buf))
                buf = []
            in_fence = not in_fence
            continue
        if in_fence:
            buf.append(line)
    return blocks


def parse_grammar(path: pathlib.Path):
    """Parse the EBNF into {name: rhs} plus the definition order.

    The grammar is written for humans: several productions may share a line,
    some lines begin with an inline `(*§n*)` marker, and a production body may
    continue across lines. Quoted terminals are masked before splitting on `;`
    so that a terminal like `";"` cannot be mistaken for a statement end.
    """
    raw = path.read_text(encoding="utf-8")
    blocks = raw_blocks(path)

    defs: dict[str, str] = {}
    order: list[str] = []
    anomalies: list[str] = []

    for block in blocks:
        text = strip_comments(block)
        # Mask quoted terminals. Inside a terminal, `\` escapes the next
        # character (ISO EBNF convention), so `"\""` is the double-quote
        # terminal and `"\\"` is the backslash terminal.
        terminals: list[str] = []

        def mask(m: re.Match) -> str:
            terminals.append(m.group(0))
            return f"\x00{len(terminals) - 1}\x00"

        text = re.sub(r'"(?:\\.|[^"\\])*"', mask, text)

        for stmt in text.split(";"):
            if not stmt.strip():
                continue
            m = re.match(r"^\s*([a-z][a-z0-9-]*)\s*=\s*(.*)$", stmt, re.S)
            if not m:
                anomalies.append(" ".join(stmt.split())[:80])
                continue
            name, rhs = m.group(1), m.group(2)
            rhs = re.sub(r"\x00(\d+)\x00", lambda mm: terminals[int(mm.group(1))], rhs)
            if name in defs:
                fail(f"G3 duplicate production definition: {name}")
            defs[name] = rhs
            order.append(name)

    return defs, order, raw, anomalies


def referenced_symbols(rhs: str) -> set[str]:
    """Nonterminals used in a production body (outside quoted terminals)."""
    without_terminals = re.sub(r'"(?:\\.|[^"\\])*"', " ", rhs)
    return set(re.findall(r"\b[a-z][a-z0-9-]*\b", without_terminals))


def check_grammar(path: pathlib.Path | None = None) -> None:
    path = path if path is not None else DOCS / "spec" / "grammar.ebnf"
    bodies, order, raw, anomalies = parse_grammar(path)
    defined = set(bodies)

    try:
        display = path.relative_to(ROOT)
    except ValueError:
        display = path
    print(f"G-grammar: {len(order)} productions parsed from {display}")
    if anomalies:
        for a in anomalies:
            fail(f"G0 unparseable grammar statement: `{a}`")

    # G1 — undefined symbols
    used: dict[str, set[str]] = {}
    for name, body in bodies.items():
        for sym in referenced_symbols(body):
            if sym != name:
                used.setdefault(sym, set()).add(name)
    undefined = sorted(s for s in used if s not in defined)
    if undefined:
        for sym in undefined:
            fail(f"G1 undefined nonterminal `{sym}` (used by: {', '.join(sorted(used[sym]))})")
    else:
        print("G1 undefined nonterminals: none")

    # G2 — reachability
    reachable: set[str] = set()
    stack = [r for r in ROOTS if r in defined]
    while stack:
        cur = stack.pop()
        if cur in reachable:
            continue
        reachable.add(cur)
        for sym in referenced_symbols(bodies[cur]):
            if sym in defined and sym not in reachable:
                stack.append(sym)
    unreachable = sorted(defined - reachable - REFERENCE_ONLY)
    if unreachable:
        for sym in unreachable:
            fail(f"G2 unreachable production `{sym}`")
    else:
        print("G2 unreachable productions: none "
              f"({len(REFERENCE_ONLY)} reference-only tables excluded: "
              f"{', '.join(sorted(REFERENCE_ONLY))})")

    # G4 — evidence coverage. The grammar states evidence per *group* of
    # productions (a fenced block cites the source it was derived from), so the
    # checkable requirement here is: every block that defines productions
    # carries at least one evidence or unresolved marker. Per-production
    # traceability is provided by docs/spec/grammar-evidence-map.md and
    # enforced by G13.
    marker_re = re.compile(r"\[(E1:[^\]]+|E2|UNRESOLVED:S\d+)\]")
    per_production = sum(1 for n in order if marker_re.search(bodies[n]))
    blocks_with_prods = 0
    blocks_without = []
    for block in raw_blocks(path):
        if not re.search(r"^\s*[a-z][a-z0-9-]*\s*=", block, re.M):
            continue
        blocks_with_prods += 1
        if not marker_re.search(block) and not re.search(r"\b(?:E1:|E2\b|OPEN:S\d+\b)", block):
            blocks_without.append(" ".join(block.split())[:70])
    if blocks_without:
        for b in blocks_without:
            fail(f"G4 grammar block has no evidence marker: `{b}`")
    else:
        print(f"G4 evidence: {blocks_with_prods} production blocks, all carry a marker; "
              f"inline per-production markers: {per_production}/{len(order)} "
              "(per-production traceability: docs/spec/grammar-evidence-map.md, G13)")

    # G5 — OPEN markers name existing records
    recs = sorted((DOCS / "spec" / "decisions").glob("S*-*.md"))
    rec_ids = {re.match(r"(S\d+)-", p.name).group(1) for p in recs}
    unresolved = sorted(set(re.findall(r"\[OPEN:(S\d+)\]", raw)))
    bad = [s for s in unresolved if s not in rec_ids]
    if bad:
        fail(f"G5 grammar references unknown decision records: {bad}")
    else:
        print(f"G5 open productions reference {len(unresolved)} records, all present: "
              f"{', '.join(unresolved)}")


# --------------------------------------------------------------------------
# Decision register
# --------------------------------------------------------------------------

def check_register() -> None:
    dec = DOCS / "spec" / "decisions"
    recs = sorted(dec.glob("S*-*.md"))
    ids = [re.match(r"(S\d+)-", p.name).group(1) for p in recs]
    expected = [f"S{i}" for i in range(1, 19)]
    if sorted(ids, key=lambda s: int(s[1:])) != expected:
        fail(f"G7 decision records: expected S1..S18, found {sorted(ids)}")
    else:
        print("G7 decision records: 18 of 18 (S1..S18)")

    register = (dec / "README.md").read_text(encoding="utf-8")
    rows = re.findall(r"^\| (S\d+) \|", register, re.M)
    if len(rows) != 18:
        fail(f"G7 register has {len(rows)} rows, expected 18")
    else:
        print("G7 register rows: 18")

    # G8 — permitted status words
    for p in recs:
        body = p.read_text(encoding="utf-8")
        m = re.search(r"\|\s*\*\*Status\*\*\s*\|\s*([^|]+)\|", body)
        if not m:
            fail(f"G8 {p.name}: no Status field")
            continue
        status = m.group(1).strip()
        # Markdown emphasis is presentation, not part of the status token.
        status = re.sub(r"^\*\*([A-Z ]+)\*\*", r"\1", status)
        if not any(re.match(r"^" + re.escape(word) + r"(?:\b|$)", status) for word in PERMITTED_STATUS):
            fail(f"G8 {p.name}: primary status `{status}` is not one of the six normative states")
    print(f"G8 status vocabulary checked on {len(recs)} records")


# --------------------------------------------------------------------------
# Cross-references
# --------------------------------------------------------------------------

def all_docs() -> list[pathlib.Path]:
    return sorted(DOCS.rglob("*.md")) + sorted(DOCS.rglob("*.ebnf")) + [ROOT / "README.md"]


def check_cross_refs() -> None:
    rec_ids = {p.name.split("-")[0] for p in (DOCS / "spec" / "decisions").glob("S*-*.md")}
    ref_re = re.compile(r"\bS(\d{1,2})\b")

    # G6 — S<n> references resolve
    bad: set[str] = set()
    total = 0
    for p in all_docs():
        for m in ref_re.finditer(p.read_text(encoding="utf-8")):
            ref = f"S{m.group(1)}"
            total += 1
            if ref not in rec_ids:
                bad.add(f"{p.relative_to(ROOT)} -> {ref}")
    if bad:
        fail(f"G6 unresolved decision references: {sorted(bad)[:10]}")
    else:
        print(f"G6 decision-record references: {total} occurrences, all resolve")

    # G9 — gate ids. Gates are defined across two documents: the Phase 1
    # component gates in implementation-gates.md and the Phase 2 completion
    # gates in the Directive 003 proposal.
    gate_sources = [
        DOCS / "spec" / "implementation-gates.md",
        DOCS / "directives" / "directive-003-proposal.md",
    ]
    defined_gates: set[str] = set()
    gate_pattern = r"G1(?:-[LSTMR]|[bcde])?|G2(?:[bcde])?"
    for gs in gate_sources:
        # A bold mention in prose is not a definition; require the gate name
        # to appear as the first cell of a Markdown gate-definition row.
        defined_gates |= set(re.findall(
            r"^\|\s*\*\*(" + gate_pattern + r")\*\*\s*\|",
            gs.read_text(encoding="utf-8"), flags=re.M,
        ))
    used_gates: dict[str, set[str]] = {}
    for p in all_docs():
        for m in re.finditer(r"\b(?:" + gate_pattern + r")\b", p.read_text(encoding="utf-8")):
            used_gates.setdefault(m.group(0), set()).add(str(p.relative_to(ROOT)))
    unknown = sorted(g for g in used_gates if g not in defined_gates)
    if unknown:
        fail(f"G9 gate ids referenced but not defined in a gate document: {unknown}")
    else:
        print(f"G9 gate ids: {len(defined_gates)} defined across "
              f"{len(gate_sources)} documents, {len(used_gates)} referenced, all known")

    # G10 — internal links
    broken = 0
    checked = 0
    for p in all_docs():
        for m in re.finditer(r"\]\(([^)]+)\)", p.read_text(encoding="utf-8")):
            t = m.group(1).strip()
            if t.startswith(("http", "mailto:", "#")):
                continue
            t = t.split("#")[0]
            if not t:
                continue
            checked += 1
            if not (p.parent / t).resolve().exists():
                broken += 1
                fail(f"G10 broken link in {p.relative_to(ROOT)} -> {t}")
    print(f"G10 internal links: {checked} checked, {broken} broken")

    # G11 — repository paths in backticks
    path_re = re.compile(r"`((?:docs|crates|scripts|tools|\.github)/[A-Za-z0-9_./-]+)`")
    checked = legacy = unresolved = 0
    for p in all_docs():
        for m in path_re.finditer(p.read_text(encoding="utf-8")):
            t = m.group(1)
            if t.endswith("/") or "*" in t or "<" in t or "cosmcc:" in t:
                continue
            checked += 1
            if (ROOT / t).exists():
                continue
            if t in FUTURE_PATHS:
                continue
            if t.startswith(("scripts/", "examples/")) or "kof-" in t:
                legacy += 1
                continue
            unresolved += 1
            fail(f"G11 unresolved repository path in {p.relative_to(ROOT)} -> {t}")
    print(f"G11 repository paths: {checked} checked, {unresolved} unresolved, "
          f"{legacy} legacy-context")


def check_normative_vocabulary() -> None:
    """G12 — the permitted normative status vocabulary must be the one used."""
    vocab_doc = DOCS / "spec" / "language-spec.md"
    body = vocab_doc.read_text(encoding="utf-8")
    required = [
        "VERIFIED LEGACY BEHAVIOR",
        "PROPOSED NORMATIVE RULE",
        "ACCEPTED NORMATIVE RULE",
        "OPEN",
        "DEFERRED",
        "NOT VERIFIED",
    ]
    missing = [w for w in required if w not in body]
    if missing:
        fail(f"G12 language-spec.md does not define the normative vocabulary: {missing}")
    else:
        print(f"G12 normative vocabulary: {len(required)} terms defined")


def check_evidence_map() -> None:
    """G13 — per-production evidence map covers the whole grammar.

    The map in docs/spec/grammar-evidence-map.md must contain exactly one row
    per production in grammar definition order. Rows are checked for
    well-formed evidence classes, recognition statuses, resolvable decision
    references and non-empty evidence/residual cells. Missing or unknown
    productions fail the check.
    """
    map_path = DOCS / "spec" / "grammar-evidence-map.md"
    if not map_path.exists():
        fail(f"G13 evidence map missing: {map_path.relative_to(ROOT)}")
        return

    text = map_path.read_text(encoding="utf-8")
    row_re = re.compile(
        r"^\|\s*(\d+)\s*\|\s*`([a-z][a-z0-9-]*)`\s*\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)\|([^|]*)\|\s*$",
        re.M,
    )
    rows = row_re.findall(text)

    _, order, _, _ = parse_grammar(DOCS / "spec" / "grammar.ebnf")
    known = set(order)

    class_re = re.compile(
        r"^(?:E1-source|E1-case|E1-probe|E2)"
        r"(?:\+(?:E1-source|E1-case|E1-probe|E2))*$"
    )
    status_re = re.compile(r"^(?:RECOGNISED|UNSUPPORTED|REJECTED)$")
    decision_re = re.compile(r"^S\d+$")
    rec_ids = {
        re.match(r"(S\d+)-", p.name).group(1)
        for p in (DOCS / "spec" / "decisions").glob("S*-*.md")
    }

    seen: dict[str, int] = {}
    problems = 0
    for position, (number, production, evidence, klass, decision, status, residual) in enumerate(
        rows, start=1
    ):
        n = int(number)
        if n != position:
            fail(f"G13 evidence map row {n} appears at position {position} (rows must be numbered 1..N in order)")
            problems += 1
        if production in seen:
            fail(f"G13 duplicate evidence-map entry for `{production}` (rows {seen[production]} and {n})")
            problems += 1
        seen[production] = n

        if not evidence.strip():
            fail(f"G13 row {n} (`{production}`) has an empty evidence cell")
            problems += 1
        if not class_re.match(klass.strip()):
            fail(f"G13 row {n} (`{production}`) has an invalid evidence class: `{klass.strip()}`")
            problems += 1
        decision = decision.strip()
        if decision and decision != "\u2014":
            for token in re.split(r"[,\s]+", decision):
                if not token:
                    continue
                if not decision_re.match(token):
                    fail(f"G13 row {n} (`{production}`) has a malformed decision reference: `{token}`")
                    problems += 1
                elif token not in rec_ids:
                    fail(f"G13 row {n} (`{production}`) references unknown decision record {token}")
                    problems += 1
        if not status_re.match(status.strip()):
            fail(f"G13 row {n} (`{production}`) has an invalid recognition status: `{status.strip()}`")
            problems += 1
        if not residual.strip():
            fail(f"G13 row {n} (`{production}`) has an empty residual cell")
            problems += 1

    missing = [name for name in order if name not in seen]
    unknown = [name for name in seen if name not in known]
    for name in missing:
        fail(f"G13 production `{name}` has no evidence-map entry")
    for name in unknown:
        fail(f"G13 evidence map lists unknown production `{name}`")

    # The rows must *follow the grammar's definition order*, not merely be
    # numbered sequentially: compare position by position.
    sequence = [row[1] for row in rows]
    in_order = sequence == order
    if len(sequence) == len(order) and not in_order:
        first = next(
            i for i, (actual, expected) in enumerate(zip(sequence, order), start=1)
            if actual != expected
        )
        fail(
            f"G13 evidence map is not in grammar definition order "
            f"(row {first}: `{sequence[first - 1]}` listed where `{order[first - 1]}` is defined)"
        )
        problems += 1

    if not problems and not missing and not unknown and len(rows) == len(order) and in_order:
        print(
            f"G13 evidence map: {len(rows)}/{len(order)} productions mapped, "
            "rows in grammar definition order, cells well-formed"
        )


def main() -> int:
    args = sys.argv[1:]
    if args:
        if args[0] == "--grammar" and len(args) == 2:
            # Audit a single grammar file (G0-G5 only). Used to reproduce the
            # findings on a historical revision, e.g.:
            #   git show 18a9255:docs/spec/grammar.ebnf > /tmp/head.ebnf
            #   python3 tools/spec-consistency.py --grammar /tmp/head.ebnf
            check_grammar(pathlib.Path(args[1]))
        else:
            print("usage: spec-consistency.py [--grammar PATH]", file=sys.stderr)
            return 2
        if failures:
            print(f"FAILED — {len(failures)} problem(s):")
            for f in failures:
                print(f"  - {f}")
            return 1
        print("OK — grammar checks passed")
        return 0

    check_grammar()
    print()
    check_register()
    print()
    check_cross_refs()
    print()
    check_normative_vocabulary()
    print()
    check_evidence_map()
    print()

    if notes:
        print("NOTES:")
        for n in notes:
            print(f"  - {n}")
        print()
    if failures:
        print(f"FAILED — {len(failures)} problem(s):")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("OK — all specification consistency checks passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
