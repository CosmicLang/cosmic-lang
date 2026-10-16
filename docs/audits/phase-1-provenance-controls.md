# Phase 1 — Provenance and License Decision Controls (Directive 002-R1)

**Status: PROPOSED NORMATIVE RULE for engineering controls.**
**License: OPEN — no license selection, approval or publication.**

## Actual provenance

The specification was constructed after **human and agent inspection of the
legacy Java source** at `CosmicLang/cosmcc` SHA
`284358f43db885ccd8b52b11f897ac0defec10de`, and from behavioral probes
executed against its existing compiler in isolated temporary directories.
This is **not a formally isolated clean-room implementation**: a clean-room
claim would require documented separation of specification researchers from
implementation authors and an information-transfer protocol not evidenced
here. The team may accurately call its Rust work **independently authored,
behavior-guided reimplementation** where review evidence supports that
description, without asserting a copyright or license conclusion.

Evidence records cite source paths, revision SHA, target, commands, output,
exit codes and interpretation. New Rust code and authored test inputs must be
reviewed for independent authorship; the legacy source checkout and generated
legacy fixtures remain **outside** the Cosmic-Lang repository. Do not copy
legacy implementation code, documentation passages, generated source, or
tests into the repo, PRs, CI artifacts, or distributed packages. Do not
mechanically translate legacy code. The differential harness should invoke an
external legacy executable as a black-box oracle with newly authored programs
and redact potentially sensitive/third-party output.

## Review and license gates

Before each future change involving semantics, reviewers should record (a) the
source evidence path and immutable revision, (b) independently authored test
input and expected behavior, (c) evidence-versus-normative status, (d) whether
any implementation code is derived by copying, and (e) license of newly added
dependencies. A contribution checklist is maintained in `CONTRIBUTING.md`.

**Owner options (none selected):** A GPL-3.0 subject to compatibility and
copyright-owner review; B permissive independent work (engineering preference,
specific license such as MIT or Apache-2.0 to be chosen later); C other copyleft
subject to qualified compatibility analysis; D no outgoing license and no public
release/redistribution plan. Avoid implying that selecting GPL-3.0 alone
grants permission to redistribute separately protected material in every
context. Avoid claiming all independently authored reimplementation is
necessarily non-derivative: that is a legal determination, not a compiler
engineering result.

Qualified counsel must examine legacy GPL obligations, derivative-work
analysis, trademarks, generated outputs and CI logs, any copying risk,
patent provisions, PortuKof/KofScript semantics and future contributor terms.
The repository `LICENSE` file remains absent pending the owner.
