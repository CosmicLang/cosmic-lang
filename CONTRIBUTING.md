# Cosmic-Lang contribution policy

| | |
|---|---|
| **Status** | **In force** — adopted with the Apache-2.0 licensing package (owner authorization 003-L4, 2026-10-16) |
| **Applies to** | every contribution to `CosmicLang/cosmic-lang` |
| **Outbound licence** | [Apache License, Version 2.0](LICENSE) |

## 1. Provenance rules (mandatory)

Contributions must be authored independently. Do not import, paste, mechanically
translate, bundle, or redistribute legacy `CosmicLang/cosmcc` code, fixtures,
generated outputs or protected documentation. The pinned legacy executable may
be consulted as an external read-only behavioral oracle; test programs must be
newly authored in isolated directories.

1. No importing GPL-3.0-protected legacy implementation code.
2. No mechanical translation of legacy source code.
3. No unauthorized copying of legacy documentation, fixtures, or generated
   artifacts.
4. All compiler implementation and tests must be independently authored.
5. Contributors must have the necessary rights to submit their contributions.
6. Any suspected provenance violation must be escalated for review and must
   not be merged while unresolved.

Every semantics-changing contribution should document the immutable evidence
source and executed target, observed output and exit status, the applicable
decision record, the normative rule's actual owner-acceptance status, and any
operation-specific feature gate. Document uncertain provenance and send it for
human review before committing.

## 2. Developer Certificate of Origin (DCO)

This project uses the **Developer Certificate of Origin, Version 1.1**
(https://developercertificate.org/) as its inbound contribution policy. Every
contributed commit must carry a `Signed-off-by` trailer certifying the DCO:

```
Signed-off-by: Your Name <your@email>
```

`git commit -s` adds it automatically; use `git rebase --signoff` to add it to
existing commits.

The full text of the Developer Certificate of Origin, Version 1.1:

```text
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.


Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

## 3. Dependencies

The workspace currently has **zero external dependencies**; no licence-audit
tooling is installed while that remains true. Before the first dependency is
added — and for every later one — the pull request that introduces it must:

* record the dependency's exact version, source and SPDX licence identifier;
* confirm the licence is compatible with this project's outbound
  **Apache-2.0** licence (permissive licences compatible with Apache-2.0 are
  acceptable; GPL/AGPL/SSPL-family or unclear licences require explicit owner
  review before any use);
* state the justification required by `docs/development/toolchain-policy.md`
  §3.

Unrecorded dependencies are not merged.

## 4. Licensing

By contributing, you agree that your contribution is provided under the
Apache License, Version 2.0 (the project's outbound licence). This document is
a project contribution policy, not legal advice; the project's licence was
adopted under the owner's risk-acceptance governance and has not been reviewed
by qualified counsel (see `docs/spec/phase-2-entry-readiness.md` §9).
