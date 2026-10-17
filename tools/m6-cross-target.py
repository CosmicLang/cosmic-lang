#!/usr/bin/env python3
"""A3 M-6 cross-target context: identifier classification across available runtimes.

The M-6 debt item (`docs/spec/implementation-gates.md` §6) is *"cross-target
behaviour of every probe in this record — re-running the probes on
``--target js`` and ``--target script``"*. That instrument needs the legacy
compiler, which is not available in this environment and must never be
downloaded or vendored, so **M-6 itself is NOT MEASURED**.

What this script does instead, and states as such in the generated document:
it compares the accepted Rust identifier-classification authority against
the host runtimes that happen to be available (CPython ``str.isidentifier``,
Node's ``\\p{ID_Start}/\\p{ID_Continue}``-based IdentifierName rule, and the
JDK ``Character.isJavaIdentifierStart/Part`` rules) for the M-7 fragment
set. This is cross-**authority** context for the identifier boundary; it is
not the M-6 instrument, not a target run, and no legacy-compatibility claim
may rest on it.

Writes ``docs/audits/phase-2-a3-m6-cross-target.md``. Stdlib only, offline;
missing runtimes are reported as ``n/a`` with the reason.
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HARNESS = ROOT / "target" / "release" / "cosmic-diff-harness"
OUT_MD = ROOT / "docs" / "audits" / "phase-2-a3-m6-cross-target.md"

# (probe id, exact fragment, accepted-Rust classification label)
FRAGMENTS = [
    ("m7_ascii_word", "camelCase", "identifier (accepted)"),
    ("m7_underscore", "_", "identifier (accepted)"),
    ("m7_dollar", "$", "identifier (accepted)"),
    ("m7_digits_continue", "a1b", "identifier (accepted)"),
    (
        "m7_digit_start",
        "1abc",
        "not one identifier (number `1` + identifier `abc`)",
    ),
    ("m7_precomposed_letter", "caf\u00e9", "identifier (accepted)"),
    ("m7_non_ascii_letters", "\u03a9mega", "identifier (accepted)"),
    ("m7_non_bmp_letter", "\U00010400\U00010428", "identifier (accepted)"),
    (
        "m7_combining_mark_continue",
        "e\u0301",
        "identifier + refused combining mark (LEX005)",
    ),
    (
        "m7_leading_combining",
        "\u0301name",
        "refused combining mark + identifier (LEX005)",
    ),
    ("m7_emoji", "\U0001f600", "refused (LEX005)"),
    (
        "m7_normalization_not_applied",
        "caf\u00e9 cafe\u0301",
        "identifier, whitespace, identifier, refused mark (LEX005)",
    ),
]


def esc(text: str) -> str:
    out = []
    for ch in text:
        code = ord(ch)
        if 32 <= code < 127:
            out.append(ch)
        elif code <= 0xFFFF:
            out.append(f"\\u{code:04X}")
        else:
            out.append(f"\\U{code:08X}")
    return "".join(out)


def run(cmd, **kwargs):
    return subprocess.run(cmd, capture_output=True, text=True, **kwargs)


def rust_observations(work: Path) -> dict[str, str]:
    if not HARNESS.exists():
        build = run(
            ["cargo", "build", "--release", "-p", "cosmic-diff-harness"], cwd=ROOT
        )
        if build.returncode != 0 or not HARNESS.exists():
            raise RuntimeError(f"cannot build harness: {build.stderr.strip()}")
    out_json = work / "probes.json"
    result = run([str(HARNESS), "--emit-probes-json", str(out_json)])
    if result.returncode != 0:
        raise RuntimeError(f"--emit-probes-json failed: {result.stderr.strip()}")
    data = json.loads(out_json.read_text(encoding="utf-8"))
    return {case["id"]: case["observed"] for case in data["cases"]}


def node_classifications(work: Path, fragments: list[str]) -> list[dict] | None:
    node = shutil.which("node")
    if not node:
        return None
    script = work / "classify.js"
    script.write_text(
        """
const fs = require('fs');
const frags = fs.readFileSync(process.argv[2], 'utf8').split('\\n');
const lang = /^[\\p{ID_Start}_$][\\p{ID_Continue}\\u200C\\u200D_$]*$/u;
const prop = /^\\p{ID_Start}\\p{ID_Continue}*$/u;
for (const s of frags) {
  console.log(JSON.stringify({ lang: lang.test(s), prop: prop.test(s) }));
}
""",
        encoding="utf-8",
    )
    frag_file = work / "fragments.txt"
    frag_file.write_text("\n".join(fragments), encoding="utf-8")
    result = run([node, str(script), str(frag_file)])
    if result.returncode != 0:
        return None
    return [json.loads(line) for line in result.stdout.splitlines() if line.strip()]


def java_classifications(work: Path, fragments: list[str]) -> tuple[list[bool] | None, str]:
    javac = shutil.which("javac")
    java = shutil.which("java")
    if not javac or not java:
        return None, "javac/java not on PATH — n/a"
    source = work / "Classify.java"
    source.write_text(
        """
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

public class Classify {
    public static void main(String[] args) throws Exception {
        List<String> lines = Files.readAllLines(Path.of(args[0]), StandardCharsets.UTF_8);
        for (String line : lines) {
            boolean ok = !line.isEmpty();
            for (int i = 0; ok && i < line.length(); ) {
                int cp = line.codePointAt(i);
                boolean part = (i == 0)
                    ? Character.isJavaIdentifierStart(cp)
                    : Character.isJavaIdentifierPart(cp);
                ok = part;
                i += Character.charCount(cp);
            }
            System.out.println(ok ? "true" : "false");
        }
    }
}
""",
        encoding="utf-8",
    )
    compile_result = run([javac, str(source)], cwd=work)
    if compile_result.returncode != 0:
        return None, "javac/java stub present but non-functional (no Java Runtime installed) — n/a"
    frag_file = work / "fragments_java.txt"
    frag_file.write_text("\n".join(fragments), encoding="utf-8")
    result = run([java, "-cp", str(work), "Classify", str(frag_file)], cwd=work)
    if result.returncode != 0:
        return None, "javac/java present but no Java Runtime is installed (stub fails) — n/a"
    classifications = [line.strip() == "true" for line in result.stdout.splitlines() if line.strip()]
    return classifications, "run"


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="cosmic-a3-m6-") as tmp_name:
        work = Path(tmp_name)
        try:
            observations = rust_observations(work)
        except RuntimeError as error:
            print(f"m6-cross-target: {error}", file=sys.stderr)
            return 1

        # Pin the probe contract: accepted fragments must be exactly one
        # identifier token; refused ones must carry LEX005.
        for probe_id, fragment, label in FRAGMENTS:
            observed = observations.get(probe_id)
            if observed is None:
                print(f"m6-cross-target: probe {probe_id} missing", file=sys.stderr)
                return 1
            if label == "identifier (accepted)":
                if observed != f"identifier:{fragment}":
                    print(
                        f"m6-cross-target: probe {probe_id} changed: {observed!r}",
                        file=sys.stderr,
                    )
                    return 1
            elif label.startswith("not one identifier"):
                if observed.startswith("identifier:"):
                    print(
                        f"m6-cross-target: probe {probe_id} is no longer split: {observed!r}",
                        file=sys.stderr,
                    )
                    return 1
            elif "LEX005" not in observed:
                print(
                    f"m6-cross-target: probe {probe_id} no longer refused: {observed!r}",
                    file=sys.stderr,
                )
                return 1

        fragments = [fragment for _, fragment, _ in FRAGMENTS]
        python_results = [fragment.isidentifier() for fragment in fragments]
        node_results = node_classifications(work, fragments)
        java_results, java_note = java_classifications(work, fragments)
        python_version = sys.version.split()[0]
        node_version = None
        if shutil.which("node"):
            node_version = run(["node", "--version"]).stdout.strip() or None

    lines: list[str] = []
    lines.append("# M-6 cross-target context: identifier classification (Increment A3)")
    lines.append("")
    lines.append("| | |")
    lines.append("|---|---|")
    lines.append(
        "| **M-6 status** | **NOT MEASURED.** The M-6 instrument is re-running the probe "
        "record on `--target js` and `--target script` of the legacy compiler "
        "(`docs/spec/implementation-gates.md` §6). No legacy compiler is available in this "
        "environment and vendoring or downloading one is prohibited, so the instrument "
        "cannot be executed. |"
    )
    lines.append(
        "| **This document** | Cross-**authority** context only: the accepted Rust "
        "classification authority versus the host runtimes that happened to be available "
        "(CPython, Node, JDK) for the M-7 fragment set. It is not a target run and no "
        "legacy-compatibility claim may rest on it. |"
    )
    lines.append(
        "| **Rust authority** | `cosmic-lexer` via the A3 probe measurements "
        "(`docs/audits/phase-2-a3-probes.md`); classification labels pinned by the "
        "`cosmic-diff-harness` probe test. |"
    )
    lines.append("")
    lines.append("## Authorities and their rules")
    lines.append("")
    lines.append("| runtime | rule used by this table | version |")
    lines.append("|---|---|---|")
    lines.append(
        "| Rust (`cosmic-lexer`, accepted authority) | `char::is_alphabetic` at start; "
        "`char::is_alphanumeric` or `_`/`$` in continuation; everything else refused "
        "(D-L3) | workspace toolchain |"
    )
    lines.append(
        f"| CPython `str.isidentifier()` | XID_Start/XID_Continue per UAX #31 | "
        f"{python_version} |"
    )
    lines.append(
        "| Node IdentifierName | `^[\\p{ID_Start}_$][\\p{ID_Continue}\\u200C\\u200D_$]*$` "
        "with `u` flag (language rule, also computed without the `$`/`_` extras) | "
        + (f"{node_version}" if node_version else "runtime unavailable — n/a")
        + " |"
    )
    lines.append(
        "| JDK `Character.isJavaIdentifierStart/Part` | Java identifier rules (include "
        "`$`, `_` and combining marks) | "
        + ("run" if java_results is not None else java_note)
        + " |"
    )
    lines.append("")
    lines.append("## Fragment table")
    lines.append("")
    lines.append(
        "| fragment | accepted Rust authority | CPython | Node (IdentifierName) | Node "
        "(property-only) | JDK | divergence from Rust |"
    )
    lines.append("|---|---|---|---|---|---|---|")
    for index, (probe_id, fragment, label) in enumerate(FRAGMENTS):
        rust_ok = label == "identifier (accepted)"
        python_ok = python_results[index]
        if node_results is not None:
            node_lang = "pass" if node_results[index]["lang"] else "refuse"
            node_prop = "pass" if node_results[index]["prop"] else "refuse"
        else:
            node_lang = "n/a"
            node_prop = "n/a"
        if java_results is not None:
            java_cell = "pass" if java_results[index] else "refuse"
        else:
            java_cell = "n/a"
        divergent = []
        if python_ok != rust_ok:
            divergent.append("CPython")
        if node_results is not None and node_results[index]["lang"] != rust_ok:
            divergent.append("Node")
        if java_results is not None and java_results[index] != rust_ok:
            divergent.append("JDK")
        lines.append(
            f"| `{esc(fragment)}` (`{probe_id}`) | {label} | "
            f"{'pass' if python_ok else 'refuse'} | {node_lang} | {node_prop} | "
            f"{java_cell} | {', '.join(divergent) if divergent else 'none'} |"
        )
    lines.append("")
    lines.append("## Reading the table")
    lines.append("")
    lines.append(
        "- `pass`/`refuse` are whole-fragment classifications by each runtime's own rule; "
        "they are not statements about the legacy compiler's targets."
    )
    lines.append(
        "- Rows with a divergence are the interesting boundary cases: they show where the "
        "accepted Rust authority draws the identifier class differently from the available "
        "host runtimes. Any legacy-target re-run must be run with the real M-6 instrument "
        "before compatibility may be claimed."
    )
    lines.append(
        "- The `$` fragment is a known authority difference: it is an accepted identifier "
        "character in the Rust authority and in Java/JS language rules, but not in "
        "XID_Start/XID_Continue."
    )
    lines.append("")
    lines.append("## Record fields")
    lines.append("")
    lines.append("| field | value |")
    lines.append("|---|---|")
    lines.append(
        "| `command` | `target/release/cosmic-diff-harness --emit-probes-json <tmp>`; "
        "CPython in-process `str.isidentifier`; `node classify.js <fragments>`; "
        "`javac Classify.java && java Classify <fragments>` |"
    )
    lines.append(
        "| `caveats` | runtimes were whatever the host provides (versions in the authority "
        "table); missing runtimes are `n/a`, never guessed; fragment classification by a "
        "runtime is context, not the M-6 instrument |"
    )
    lines.append("")
    lines.append(
        "Regenerated with `python3 tools/m6-cross-target.py` (stdlib only, offline)."
    )
    lines.append("")

    OUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUT_MD.write_text("\n".join(lines), encoding="utf-8")
    print(f"m6-cross-target: wrote {OUT_MD.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
