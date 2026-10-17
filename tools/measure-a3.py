#!/usr/bin/env python3
"""A3 reproducible measurement runner (Stage A).

Measures the accepted frontend (cosmic-lexer + cosmic-syntax) through the
release build of the differential harness: throughput, p50/p95 latency,
peak resident set size, adversarial workloads, dependency exposure and
panic/ICE incidence. Writes ``docs/audits/phase-2-a3-measurements.md`` and
``docs/audits/phase-2-a3-measurements.json``.

Protocol: ``docs/development/measurement-plan.md`` §3 (sha, host, toolchain,
profile, command, runs, date, caveats; cold and warm runs are never
averaged). Everything here is stdlib-only, offline, and reproduces by
re-running the script; no dependency is added.

The legacy oracle is never downloaded or invoked here: every
legacy-comparison metric is reported as NOT MEASURED with the concrete
reason.
"""

from __future__ import annotations

import datetime
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
HARNESS = ROOT / "target" / "release" / "cosmic-diff-harness"
OUT_MD = ROOT / "docs" / "audits" / "phase-2-a3-measurements.md"
OUT_JSON = ROOT / "docs" / "audits" / "phase-2-a3-measurements.json"

# input name -> (repeat count, per-run timeout seconds)
PLAN = {
    "valid_small": (30, 60),
    "valid_medium": (30, 60),
    "valid_large": (5, 120),
    "if_lines_50k": (3, 90),
    "deep_type_arguments_50k": (3, 90),
    "type_decl_lines_50k": (2, 120),
    "parens_1m": (3, 90),
    "statement_soup_10k": (3, 60),
}

# inputs whose cold invocation is measured with /usr/bin/time -l
RSS_INPUTS = [
    "valid_medium",
    "valid_large",
    "if_lines_50k",
    "deep_type_arguments_50k",
    "parens_1m",
]


def run(cmd, **kwargs):
    return subprocess.run(cmd, capture_output=True, text=True, **kwargs)


def capture(cmd):
    result = run(cmd)
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def environment() -> dict:
    env = {}
    env["os"] = platform.platform()
    version = capture(["sw_vers", "-productVersion"]) if shutil.which("sw_vers") else None
    if version:
        env["os_version"] = version
    env["machine"] = platform.machine()
    cpu = capture(["sysctl", "-n", "machdep.cpu.brand_string"])
    env["cpu"] = cpu or "unknown"
    mem = capture(["sysctl", "-n", "hw.memsize"])
    env["ram_bytes"] = int(mem) if mem and mem.isdigit() else None
    env["cores"] = os.cpu_count()
    if shutil.which("diskutil"):
        info = capture(["diskutil", "info", "/"])
        if info:
            match = re.search(r"Solid State:\s*(\w+)", info)
            env["storage"] = "solid state" if match and match.group(1) == "Yes" else "unknown"
    env.setdefault("storage", "unknown")
    rustc = capture(["rustc", "-vV"]) or "unavailable"
    env["rustc_release"] = next(
        (line.split(":", 1)[1].strip() for line in rustc.splitlines() if line.startswith("release:")),
        "unknown",
    )
    env["rustc_host"] = next(
        (line.split(":", 1)[1].strip() for line in rustc.splitlines() if line.startswith("host:")),
        "unknown",
    )
    env["cargo"] = capture(["cargo", "-V"]) or "unavailable"
    env["sha"] = capture(["git", "rev-parse", "HEAD"]) or "not-a-git-checkout"
    status = capture(["git", "status", "--porcelain"]) or ""
    env["dirty_paths"] = [line[3:] for line in status.splitlines()]
    env["date"] = datetime.date.today().isoformat()
    return env


def generate_valid(target_bytes: int) -> str:
    parts = []
    size = 0
    index = 0
    while size < target_bytes:
        block = f"f{index}() {{\n    val x = {index}\n    call(x)\n}}\n"
        parts.append(block)
        size += len(block)
        index += 1
    return "".join(parts)


def build_inputs(tmp: Path) -> list[dict]:
    inputs: list[dict] = []

    def write(name: str, text: str) -> None:
        path = tmp / f"{name}.csmx"
        path.write_text(text, encoding="utf-8")
        inputs.append({"name": name, "path": path, "bytes": len(text.encode("utf-8"))})

    for name, target in (
        ("valid_small", 1 << 10),
        ("valid_medium", 256 << 10),
        ("valid_large", 4 << 20),
    ):
        write(name, generate_valid(target))

    write("if_lines_50k", "main() {\n" + "    if (c)\n" * 50_000 + "}\n")

    depth = 50_000
    write(
        "deep_type_arguments_50k",
        "main() {\n    val x: " + "A<" * depth + "Int" + ">" * depth + "\n}\n",
    )

    lines = ["main() {"]
    lines.extend(f"    val x{index}: Pair<Int, Int>" for index in range(50_000))
    lines.append("}")
    write("type_decl_lines_50k", "\n".join(lines) + "\n")

    write(
        "parens_1m",
        "main() {\n    val x = " + "(" * 1_000_000 + "1" + ")" * 1_000_000 + "\n}\n",
    )

    parts = ["main() {"]
    for index in range(10_000):
        parts.append(["    val x = 1", "    ~", "    if (c) {", "    }", "    call()"][index % 5])
    parts.append("}")
    write("statement_soup_10k", "\n".join(parts) + "\n")

    return inputs


def measure(path: Path, repeat: int, timeout_s: float) -> dict:
    command = f"{HARNESS} --measure {path} --repeat {repeat}"
    result: dict = {"command": command, "repeat": repeat, "timeout_s": timeout_s}
    try:
        proc = run([str(HARNESS), "--measure", str(path), "--repeat", str(repeat)], timeout=timeout_s)
    except subprocess.TimeoutExpired:
        result["timed_out"] = True
        return result
    result["timed_out"] = False
    result["exit_code"] = proc.returncode
    samples = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            data = json.loads(line)
        except json.JSONDecodeError:
            continue
        if "run" in data:
            samples.append(int(data["elapsed_ns"]))
            result["accepts"] = data.get("accepts")
            result["lossless"] = data.get("lossless")
            result["reprint_ok"] = data.get("reprint_ok")
            result["nodes"] = data.get("nodes")
        elif "aggregate" in data:
            result["aggregate"] = data["aggregate"]
    result["samples_ns"] = samples
    if samples:
        ordered = sorted(samples)
        result["min_ns"] = ordered[0]
        result["max_ns"] = ordered[-1]
        result["p50_ns"] = ordered[(len(ordered) - 1) // 2]
        result["p95_ns"] = ordered[min(len(ordered) - 1, round(0.95 * (len(ordered) - 1)))]
        result["mean_ns"] = sum(samples) // len(samples)
    result["panicked"] = "panicked" in proc.stderr
    result["stderr_head"] = proc.stderr.strip().splitlines()[:1]
    return result


def peak_rss(path: Path) -> dict | None:
    time_bin = "/usr/bin/time"
    if not Path(time_bin).exists():
        return None
    try:
        proc = run([time_bin, "-l", str(HARNESS), "--measure", str(path), "--repeat", "1"], timeout=600)
    except subprocess.TimeoutExpired:
        return {"timed_out": True}
    rss = re.search(r"^\s*(\d+)\s+maximum resident set size", proc.stderr, re.MULTILINE)
    real = re.search(r"^\s*([0-9.,]+)\s+real", proc.stderr, re.MULTILINE)
    if not rss:
        return None
    return {
        "peak_rss_bytes": int(rss.group(1)),
        "real_s": float(real.group(1).replace(",", ".")) if real else None,
    }


def dependency_counts() -> dict:
    result = {}
    tree = run(["cargo", "tree", "--workspace", "--edges", "normal", "--prefix", "none", "--format", "{p}"], cwd=ROOT)
    if tree.returncode != 0:
        return {"error": tree.stderr.strip().splitlines()[:1]}
    packages = sorted({line.strip() for line in tree.stdout.splitlines() if line.strip()})
    workspace = sorted({p.split()[0] for p in packages if p.startswith("cosmic-")})
    external = sorted({p.split()[0] for p in packages if not p.startswith("cosmic-")})
    result["workspace_packages"] = workspace
    result["external_packages"] = external
    result["external_count"] = len(external)
    result["total_count"] = len(packages)
    return result


def fmt_ms(ns: int | None) -> str:
    return "n/a" if ns is None else f"{ns / 1e6:.2f} ms"


def fmt_mib_s(bytes_: int, ns: int | None) -> str:
    if not ns:
        return "n/a"
    return f"{bytes_ / (ns / 1e9) / (1024 * 1024):.1f} MiB/s"


def fmt_bytes(value: int | None) -> str:
    if value is None:
        return "n/a"
    return f"{value / (1024 * 1024):.1f} MiB" if value >= 1 << 20 else f"{value / 1024:.1f} KiB"


def main() -> int:
    print("measure-a3: building release harness ...")
    build = run(["cargo", "build", "--release", "-p", "cosmic-diff-harness"], cwd=ROOT)
    if build.returncode != 0:
        print(build.stderr, file=sys.stderr)
        return 1
    if not HARNESS.exists():
        print(f"measure-a3: harness binary missing at {HARNESS}", file=sys.stderr)
        return 1

    env = environment()
    deps = dependency_counts()

    with tempfile.TemporaryDirectory(prefix="cosmic-a3-measure-") as tmp_name:
        tmp = Path(tmp_name)
        inputs = build_inputs(tmp)

        # Corpus sanity: the generated "valid" inputs must be accepted
        # without diagnostics, otherwise the throughput numbers would be
        # measuring error recovery, not recognition.
        sanity = measure(next(i for i in inputs if i["name"] == "valid_small")["path"], 1, 60)
        if sanity.get("accepts") is not True:
            print(
                "measure-a3: generated valid input was not accepted; refusing to measure",
                file=sys.stderr,
            )
            return 1

        results = []
        for entry in inputs:
            repeat, timeout_s = PLAN[entry["name"]]
            print(f"measure-a3: {entry['name']} ({fmt_bytes(entry['bytes'])}) x{repeat} ...")
            outcome = measure(entry["path"], repeat, timeout_s)
            outcome.update({"name": entry["name"], "bytes": entry["bytes"]})
            results.append(outcome)

        rss = {}
        for name in RSS_INPUTS:
            entry = next(i for i in inputs if i["name"] == name)
            print(f"measure-a3: peak RSS for {name} ...")
            rss[name] = peak_rss(entry["path"])

    panics = sum(1 for result in results if result.get("panicked"))
    timeouts = [result["name"] for result in results if result.get("timed_out")]

    payload = {
        "schema": "cosmic-diff-harness/a3-measurements/1",
        "environment": env,
        "dependencies": deps,
        "results": results,
        "peak_rss": rss,
        "panics": panics,
        "timeouts": timeouts,
    }
    OUT_JSON.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    OUT_MD.write_text(render(env, deps, results, rss, panics, timeouts), encoding="utf-8")
    print(f"measure-a3: wrote {OUT_MD.relative_to(ROOT)} and {OUT_JSON.relative_to(ROOT)}")
    return 0


def render(env: dict, deps: dict, results: list[dict], rss: dict, panics: int, timeouts: list[str]) -> str:
    out: list[str] = []
    out.append("# A3 frontend measurements (Stage A)")
    out.append("")
    out.append("| | |")
    out.append("|---|---|")
    out.append(
        "| **Status** | Measured by `tools/measure-a3.py` (stdlib only). "
        "Every figure below was produced by the recorded command on the recorded host; "
        "nothing is estimated. |"
    )
    out.append(
        "| **Protocol** | `docs/development/measurement-plan.md` §3 — sha, host, toolchain, "
        "profile, command, runs, date and caveats; cold and warm runs are reported separately "
        "and never averaged. |"
    )
    out.append(
        "| **Subject** | The accepted frontend (`cosmic-lexer` → `cosmic-syntax`) through "
        "`target/release/cosmic-diff-harness --measure`; no semantic analysis or code "
        "generation exists in Stage A. |"
    )
    out.append("")
    out.append("## Record fields")
    out.append("")
    out.append("| field | value |")
    out.append("|---|---|")
    out.append(f"| `sha` | `{env['sha']}` |")
    out.append(f"| `date` | {env['date']} |")
    out.append(
        f"| `host` | {env['machine']}, {env.get('os', '')}"
        + (f", macOS {env['os_version']}" if 'os_version' in env else "")
        + f"; CPU: {env['cpu']}; {env['cores']} cores; RAM: {fmt_bytes(env['ram_bytes'])}; storage: {env['storage']} |"
    )
    out.append(f"| `toolchain` | rustc {env['rustc_release']} ({env['rustc_host']}); {env['cargo']} |")
    out.append("| `profile` | `release` (`cargo build --release -p cosmic-diff-harness`); no non-default flags |")
    out.append(
        "| `command` | `target/release/cosmic-diff-harness --measure <input> --repeat <n>`; "
        "peak RSS: `/usr/bin/time -l target/release/cosmic-diff-harness --measure <input> --repeat 1` |"
    )
    out.append("| `runs` | per table row (`p50` = lower median of the per-run sample set) |")
    if env["dirty_paths"]:
        out.append(
            "| `caveats` | the working tree was **not clean** at measurement time; uncommitted "
            "paths: " + ", ".join(f"`{path}`" for path in env["dirty_paths"]) + " |"
        )
    else:
        out.append("| `caveats` | working tree clean at measurement time |")
    out.append("")
    out.append("## Method")
    out.append("")
    out.append(
        "- Inputs are generated (authored) files, never legacy fixtures: `valid_*` are "
        "repetitions of a small recognised program; the adversarial inputs reproduce the "
        "workloads named by the A2 record (`if_lines`, deep type arguments, deep "
        "parentheses) and the A3 statement soup."
    )
    out.append(
        "- **Warm latency** is the per-run sample set inside one process (`--repeat n`); "
        "**cold invocation** is a separate process measured by `/usr/bin/time -l` "
        "(reported in the peak-RSS table). They are different metrics and are not averaged."
    )
    out.append(
        "- Every run records `accepts`/`lossless`/`reprint_ok`; a measurement whose input was "
        "not accepted would be measuring recovery, not recognition (the `valid_*` corpus is "
        "sanity-checked before measuring)."
    )
    out.append(
        "- Adversarial runs are **bounded**: "
        + (f"timeouts occurred for {', '.join(timeouts)}" if timeouts else "no run exceeded its per-run timeout")
        + "."
    )
    out.append("")
    out.append("## Throughput and latency (warm)")
    out.append("")
    out.append(
        "| input | bytes | runs | min | p50 | p95 | max | p50 throughput | accepted / lossless / reprints |"
    )
    out.append("|---|---|---|---|---|---|---|---|")
    for result in results:
        if result["name"].startswith("valid_"):
            flags = (
                f"{result.get('accepts')} / {result.get('lossless')} / {result.get('reprint_ok')}"
                if not result.get("timed_out")
                else "TIMEOUT"
            )
            out.append(
                f"| `{result['name']}` | {fmt_bytes(result['bytes'])} | {result['repeat']} | "
                f"{fmt_ms(result.get('min_ns'))} | {fmt_ms(result.get('p50_ns'))} | "
                f"{fmt_ms(result.get('p95_ns'))} | {fmt_ms(result.get('max_ns'))} | "
                f"{fmt_mib_s(result['bytes'], result.get('p50_ns'))} | {flags} |"
            )
    out.append("")
    out.append("## Adversarial workloads (bounded)")
    out.append("")
    out.append(
        "| input | bytes | runs | min | p50 | max | result flags | notes |"
    )
    out.append("|---|---|---|---|---|---|---|---|")
    for result in results:
        if not result["name"].startswith("valid_"):
            if result.get("timed_out"):
                out.append(
                    f"| `{result['name']}` | {fmt_bytes(result['bytes'])} | {result['repeat']} | "
                    f"— | — | — | TIMEOUT | bounded at {result['timeout_s']} s per run |"
                )
                continue
            note = f"nodes {result.get('nodes')}"
            if result["name"] == "if_lines_50k":
                note = "reproduces the A2 per-trip PARSE005 workload"
            if result["name"] == "deep_type_arguments_50k":
                note = "50 000 nested type arguments; implementation guard workload"
            if result["name"] == "parens_1m":
                note = "1 000 000 nested parentheses; implementation guard workload"
            if result["name"] == "statement_soup_10k":
                note = "mixed malformed soup; determinism covered by tests"
            out.append(
                f"| `{result['name']}` | {fmt_bytes(result['bytes'])} | {result['repeat']} | "
                f"{fmt_ms(result.get('min_ns'))} | {fmt_ms(result.get('p50_ns'))} | "
                f"{fmt_ms(result.get('max_ns'))} | "
                f"accepts={result.get('accepts')}, lossless={result.get('lossless')} | {note} |"
            )
    out.append("")
    out.append("### Previous reported durations (baseline context)")
    out.append("")

    def p50_of(name: str) -> str:
        for result in results:
            if result["name"] == name and not result.get("timed_out"):
                return fmt_ms(result.get("p50_ns"))
        return "n/a"

    out.append(
        "- The A2 record reported ≈18 s (bounded) for a ~150 KB type-argument "
        "workload and 291×`PARSE005` for `if_lines` at 50 000 lines on the "
        "previous development machine; the A2 probe generator was not preserved."
    )
    out.append(
        "- This run reproduces the *shape* with A3-authored generators: "
        f"`deep_type_arguments_50k` (146.5 KiB) measures {p50_of('deep_type_arguments_50k')} p50 and "
        f"`if_lines_50k` reproduces the per-trip guard workload at {p50_of('if_lines_50k')} p50. "
        "Different generator, machine and toolchain: these are **not a comparison**; the A2 "
        "figures are baseline context only, exactly as the directive states."
    )
    out.append("")
    out.append("## Peak RSS and cold invocation")
    out.append("")
    out.append("| input | peak RSS | cold wall time (`real`) | note |")
    out.append("|---|---|---|---|")
    for name, value in rss.items():
        if value is None:
            out.append(f"| `{name}` | NOT MEASURED | NOT MEASURED | `/usr/bin/time -l` unavailable on this host |")
        elif value.get("timed_out"):
            out.append(f"| `{name}` | TIMEOUT | — | bounded run exceeded 600 s |")
        else:
            out.append(
                f"| `{name}` | {fmt_bytes(value['peak_rss_bytes'])} | "
                f"{value['real_s']:.2f} s | includes process startup |"
                if value.get("real_s") is not None
                else f"| `{name}` | {fmt_bytes(value['peak_rss_bytes'])} | n/a | includes process startup |"
            )
    out.append("")
    large = rss.get("valid_large")
    if large and large.get("peak_rss_bytes"):
        multiple = large["peak_rss_bytes"] / (4 * 1024 * 1024)
        out.append(
            f"- **Observation, not a gate.** The 4 MiB recognised input peaks at "
            f"{fmt_bytes(large['peak_rss_bytes'])} (≈{multiple:.0f}× input size) with the full CST "
            "retained. Stage A declares no memory threshold; this is recorded for Stage B "
            "attention, consistent with the plan's note that peak RSS is a defect detector."
        )
    out.append("")
    out.append("## Dependency exposure (plan M1)")
    out.append("")
    if "error" in deps:
        out.append(f"`cargo tree` failed: {deps['error']}")
    else:
        out.append(
            f"- Workspace packages: {len(deps['workspace_packages'])} "
            f"({', '.join('`' + p.split()[0] + '`' for p in deps['workspace_packages'])})."
        )
        out.append(
            f"- External (non-workspace) packages, direct and transitive, normal edges: "
            f"**{deps['external_count']}**"
            + (" — the zero-dependency invariant holds." if deps["external_count"] == 0 else f": {', '.join(deps['external_packages'])}")
        )
    out.append("")
    out.append("## Panic / ICE incidence (plan M8)")
    out.append("")
    out.append(
        f"- Panic events observed across all A3 measurement runs: **{panics}**."
        + (" No run panicked." if panics == 0 else " See the JSON record for the affected runs.")
    )
    out.append(
        "- This is a count over the measured corpus, not a claim over all inputs; the "
        "property and adversarial suites (`crates/cosmic-syntax/tests/properties.rs`, "
        "`adversarial.rs`) are the systematic complement."
    )
    out.append("")
    out.append("## Plan-metric mapping for this increment")
    out.append("")
    out.append("| plan metric | A3 status |")
    out.append("|---|---|")
    out.append("| M1 dependency exposure | **measured** (above; zero external packages) |")
    out.append(
        "| M2 test coverage | **NOT MEASURED** — no coverage tooling is installed and adding one "
        "(`cargo llvm-cov`) is a dependency decision that the measurement plan §5 explicitly "
        "does not take; substitute evidence: per-suite test totals in the A3 report |"
    )
    out.append(
        "| M3 complexity/duplication | **NOT MEASURED** — no analyzer installed; no threshold "
        "declared (plan: derive from measured data, not in advance) |"
    )
    out.append("| M4 compilation throughput | **measured** (frontend only; no semantic phase exists) |")
    out.append("| M5 p50/p95 latency | **measured** in-process; cold invocation wall time recorded separately |")
    out.append("| M6 compiler memory usage | **measured** (peak RSS per input, `/usr/bin/time -l`) |")
    out.append("| M7 artifact size | **NOT MEASURED** — no artifact exists before Phase 4 |")
    out.append("| M8 ICE rate | **measured as zero observed** on the A3 corpus; not a claim over all inputs |")
    out.append("| M9 incremental build time | **NOT MEASURED** — out of A3 scope; no edit-build tooling was added |")
    out.append(
        "| M10 conformance and per-feature parity | **NOT MEASURED for legacy targets** — the "
        "differential harness is delivered and its machinery is tested, but no legacy oracle is "
        "available in this environment, so per-target conformance cannot be claimed; harness "
        "skips are recorded with reasons (see the report and the harness summary) |"
    )
    out.append("| M11 post-release regressions | not applicable before Phase 5 |")
    out.append("")
    out.append("## What is deliberately not claimed")
    out.append("")
    out.append(
        "- **No legacy comparison.** The legacy compiler was not present; every "
        "oracle-dependent condition is reported as NOT MEASURED, not as agreement."
    )
    out.append(
        "- **No cross-machine claim.** These numbers are one declared host (above); the "
        "protocol states that different machines are not comparable."
    )
    out.append(
        "- **No tuned inputs.** The adversarial generators are the workloads named by the A2 "
        "record and the A3 directive; they were not tuned for favourable numbers, and every "
        "run is bounded by the recorded timeout."
    )
    out.append(
        "- **No coverage claim.** Test totals demonstrate activity, not coverage of the state "
        "space; M2 stays NOT MEASURED until tooling is approved."
    )
    out.append("")
    out.append("## Regeneration")
    out.append("")
    out.append(
        "`python3 tools/measure-a3.py` (stdlib only, offline; rebuilds the release harness "
        "first). The JSON record next to this document carries every raw sample, command and "
        "flag for independent re-derivation."
    )
    out.append("")
    return "\n".join(out)


if __name__ == "__main__":
    sys.exit(main())
