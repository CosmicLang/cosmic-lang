# Rust Toolchain and Dependency Policy

| | |
|---|---|
| **Status** | Active — Phase 0 |
| **Applies to** | the entire `cosmic-lang` Cargo workspace |

## 1. Pinned toolchain

* The workspace pins Rust **1.99.0** in `rust-toolchain.toml`
  (`channel = "1.99.0"`, components `clippy`, `rustfmt`, profile `minimal`).
  Tools that honor the file (`rustup` proxies, CI) use exactly this version.
* CI additionally requests the same version explicitly
  (`dtolnay/rust-toolchain@v1`, `toolchain: 1.99.0`) so a workflow change
  cannot silently drift from the pin.
* `Cargo.toml` declares `rust-version = "1.99"` for the workspace packages,
  matching the pin. **Raising the pin is a reviewed change**: update
  `rust-toolchain.toml`, the CI toolchain input, `rust-version`, and note the
  change in the pull request; use the new version's features only after the
  bump lands.
* Phase 0 was developed and validated locally with rustc/cargo 1.99.0
  (Homebrew) on macOS 27.0.1 and designed to be validated by CI on Linux.

## 2. Committed lockfile

`Cargo.lock` is committed. The workspace ships a binary (`cosmic`), so the
lockfile is part of the reproducible development contract. Dependency
additions must regenerate and commit the lockfile in the same change.

## 3. Dependency policy

* **Phase 0 uses zero external dependencies** (standard library only). This is
  deliberate: the foundation must be reviewable byte-for-byte.
* A new dependency needs, in its pull request: (a) the concrete problem it
  solves, (b) why the standard library cannot solve it, (c) an explicit
  version requirement, and (d) a note on maintenance/licensing of the crate.
  `serde` for machine-readable output and a CLI argument framework are the
  expected first candidates **when a command surface exists that needs them**
  — not before.
* Do not add dependencies to mirror other compilers' stacks.

## 4. Lint and code policy

* `unsafe_code = "forbid"` workspace-wide. If unsafe ever becomes necessary:
  minimize, encapsulate in a dedicated module, document the safety argument
  (invariants + why safe), and add targeted tests; review is mandatory.
* `missing_docs`, `missing_debug_implementations` warn; CI turns all warnings
  into errors (`RUSTFLAGS: -D warnings`, `RUSTDOCFLAGS: -D warnings`).
* `clippy::dbg_macro`, `clippy::todo`, `clippy::unimplemented` are denied.
  Code that is not implemented must fail loudly and visibly (the CLI returns
  exit code 2 with a message; it never stubs success).
* Suppressing warnings (e.g. `#[allow]` without justification) to green CI is
  prohibited. Fix the cause or document the exception in review.

## 5. Reproducibility

No release artifacts exist in Phase 0, so no reproducibility claim is made.
The committed lockfile, pinned toolchain and zero-dependency policy are the
foundation for a future reproducible-build gate; that gate will be added when
the first distributable binary exists.
