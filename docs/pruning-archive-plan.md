# Faber Pruning Archive Plan

**Status**: planning artifact, no files moved yet.

This document lists repository surfaces that no longer match the likely future direction: `radix-rs` as the primary compiler, Rust/WASM as the preferred execution path, and a smaller main repository that does not carry old bootstrap or self-hosting experiments as active project domains.

The sibling archive repository is `../faber-archivum/`. It already contains old `faber-codegen/`, `rivus-codegen/`, and `prettier/` material, so the recommended archive destination is real rather than hypothetical.

## Canonical Main-Repo Surface

Keep these in the main repository unless a later decision changes the project direction:

| Path | Reason |
| ---- | ------ |
| `radix/crates/radix/` | New home for the active compiler and current CI gate. Move from `compilers/radix-rs/`. |
| `radix/crates/norma/` | New home for the Rust stdlib runtime. Move from `runtimes/norma-rs/`. |
| `radix/stdlib/norma/` | New home for Faber stdlib definitions and `@ verte` metadata. Move from `stdlib/norma/`. |
| `radix/Cargo.toml` | Workspace root for compiler/runtime crates. |
| `.github/workflows/ci.yml` | Already scoped to `radix-rs` format and tests. |
| `docs/grammatica/`, `EBNF.md` | Current language contract and prose documentation. Some pages still need cleanup after pruning. |
| `docs/wasm-execution-plan.md` | Future direction memo. |
| `examples/exempla/` | Keep the general feature corpus, but prune examples that only exist for archived surfaces. |
| `README.md`, `AGENTS.md`, `project.yaml`, `package.json` | Keep, but rewrite after the archive move so they stop advertising archived paths as local project surfaces. |

## Collapse The Active Tree

After archiving old compiler surfaces, collapse the three active pieces into a single `radix/` workspace:

```text
radix/
  Cargo.toml                 # workspace root
  crates/
    radix/                   # current compilers/radix-rs crate
      Cargo.toml
      src/
      tests/
    norma/                   # current runtimes/norma-rs crate
      Cargo.toml
      src/
        lib.rs
        hal/
        json.rs
        toml.rs
        yaml.rs
  stdlib/
    norma/                   # current stdlib/norma/*.fab definitions
      hal/
      innatum/
      json.fab
      toml.fab
      yaml.fab
```

This keeps Rust crates as Rust crates, keeps Faber stdlib definitions out of Rust `src/`, and gives future WASM work one natural home (`radix/crates/norma-wasm`, `radix/crates/faber-wasm-host`, or similar).

Do not put the Faber stdlib files under `radix/crates/radix/src/`; that would blur compiler source with language/library source. The compiler should consume `radix/stdlib/norma/` as data/spec input.

Migration caveat: `stdlib/norma/*.fab` currently encodes relative `@ subsidia` paths such as `../norma-rs/json.rs` and `../../norma-rs/hal/solum.rs`. The collapse should either rewrite them to the new layout or, preferably, replace filesystem-relative runtime references with logical runtime identifiers so stdlib metadata stops depending on repo layout.

## Move To `faber-archivum` First

These are high-confidence archive candidates. They are not paying rent in the main repo under a `radix-rs`-first future.

| Main repo path | Suggested archive path | Why archive |
| -------------- | ---------------------- | ----------- |
| `compilers/nanus-ts/` | `bootstrap/nanus-ts/` | TypeScript bootstrap compiler. Root test/lint/typecheck still point here, but that is compatibility residue, not a future-facing gate. |
| `compilers/nanus-go/` | `bootstrap/nanus-go/` | Go bootstrap compiler. No recent feature work beyond repo reorganization/housekeeping. Contains a tracked built binary at `compilers/nanus-go/nanus-go`, which should not stay in the main repo. |
| `compilers/nanus-rs/` | `bootstrap/nanus-rs/` | Rust bootstrap compiler, now redundant beside the real Rust compiler in `radix-rs`. |
| `compilers/nanus-py/` | `bootstrap/nanus-py/` | Python bootstrap compiler. Python is not part of current `radix-rs targets`. |
| `compilers/rivus/` | `self-hosting/rivus/` | Self-hosted compiler source. It is explicitly experimental/deprioritized, parser behavior is known to be unreliable, and current work does not try to make it pass. |
| `compilers/rivus-cli/` | `self-hosting/rivus-cli/` | CLI shim for the archived self-hosting path. |
| `docs/reference/faber-ts/` | `reference/faber-ts/` | Earlier TypeScript reference compiler. Useful historically, especially for CLI design, but not a live implementation surface. |
| `docs/reference/rivus-cli/` | `reference/rivus-cli/` | Reference CLI sources for the old `rivus` path. Archive with `rivus`. |
| `tests/proba/rivus.test.ts` | `tests/proba/rivus.test.ts` | Test runner for `opus/rivus` generated output. Depends on the archived self-hosting path. |
| `tests/proba/rivus-compile.ts` | `tests/proba/rivus-compile.ts` | Same as above; imports generated `opus/rivus/...` modules. |
| `examples/exempla/rivus-cli-annotated/` | `examples/rivus-cli-annotated/` | Rich historical CLI example. Keep a pointer from `docs/grammatica/cli.md`, but the file itself belongs with archived CLI/reference material. |

## Move Or Delete With The Same Break

These are root-level scripts/configs whose only job is to keep archived surfaces convenient. They should move with the old code or be deleted from the main repo.

| Path | Recommendation |
| ---- | -------------- |
| `scripta/build-nanus-ts.ts` | Move to archive with `nanus-ts`. |
| `scripta/build-nanus-go.ts` | Move to archive with `nanus-go`. |
| `scripta/build-nanus-rs.ts` | Move to archive with `nanus-rs`. |
| `scripta/build-nanus-py.ts` | Move to archive with `nanus-py`. |
| `scripta/build-bootstrap.ts` | Move to archive; bootstrap orchestration should not remain in the main repo. |
| `scripta/build-rivus.ts` | Move to archive with `rivus`. |
| `scripta/build-faber.ts` | Move to archive; it exists to prove old self-hosting. |
| `scripta/verify-nanus.ts` | Move to archive; cross-bootstrap consistency is no longer a main quality gate. |
| `scripta/rivus-features.ts` | Move to archive; introspects the archived `rivus` codebase. |
| `scripta/rivus-zig` | Move to archive; depends on generated `opus/faber-ts/...`. |
| `scripta/build-golden.ts` | Move to archive with `tests/golden`; default compiler list is `nanus-*` plus old `faber`/`rivus`. |
| `scripta/golden.ts` | Move to archive with `tests/golden`; defaults to `nanus-ts`. |
| `scripta/test-report` | Move to archive with the `tests/proba` harness unless that harness is rewritten for `radix-rs`. |
| `scripta/test` | Replace with either `bun run test:radix-rs` or delete. Current script only runs `compilers/nanus-ts/index.test.ts`. |
| `scripta/lint` | Replace with a `radix-rs` or docs-specific lint, or delete. Current script only lints `compilers/nanus-ts`. |
| `scripta/typecheck` | Move to archive or delete. Current script typechecks only `tsconfig.nanus.json`. |
| `tsconfig.nanus.json` | Move to archive with `nanus-ts`. |

After this break, update `package.json` so root scripts stop exposing `build:nanus-*`, `build:bootstrap`, `test:nanus-ts`, `test:legacy`, `lint:nanus-ts`, `typecheck:legacy`, and `verify:nanus`.

## Review Before Moving

These paths are mixed. They contain useful data but also old multi-target assumptions.

| Path | Recommended handling |
| ---- | -------------------- |
| `scripta/build.ts` | Rewrite or replace. Despite its header saying "radix-rs focused", it still builds `nanus-rs` and compiles `rivus` as stages. |
| `scripta/build-exempla.ts` | Keep only if simplified around `radix-rs`; current options still support `nanus-*`, `rivus-*`, and old targets (`zig`, `py`, `rs`, `go`). |
| `scripta/prettier` | Revise globs after moving `compilers/rivus`, `compilers/nanus-*`, and `tests/proba`. |
| `tests/proba/` YAML files | Salvage useful language feature cases into `radix-rs` tests or a new `tests/spec/` corpus, then archive the TypeScript harness. The current TS capability tests import non-current `../../faber/...` modules and use old syntax such as `fit`/`fiet`. |
| `tests/golden/` | Likely archive. These are nanus-era TS golden outputs, not the active `radix-rs` test surface. Keep only if a current `radix-rs` golden workflow is introduced. |
| `runtimes/norma-ts/` | Defer until the TypeScript target decision is explicit. `radix-rs` can still emit TS files, but future WASM direction makes this a likely archive candidate. |
| `runtimes/norma-go/` | Defer until the Go target decision is explicit. Current `radix-rs` emits Go files, but package/runtime support is not the future focus. |
| `runtimes/norma-py/` | Likely archive with `nanus-py` unless Python runtime artifacts are needed as reference material. Python is not a current `radix-rs` target. |
| `docs/faber-language-critique.md` | Keep or archive depending on whether you want historical language critique in the main docs. It references `compilers/rivus/` heavily. |
| `docs/radix-diagnostics-plan.md` | Keep for now. Although it uses `rivus` as a stress corpus, the subject is `radix-rs` diagnostics. After moving `rivus`, change examples to point at the archive or replace with smaller fixtures. |
| `docs/grammatica/cli.md` | Keep. It is now clearly aspirational and points to historical artifacts; update links after the move. |
| `docs/grammatica/verba.md` | Revise. It currently tells readers to update bootstrap/self-hosted lexicons. |
| `docs/grammatica/targets.md` | Revise after the move so bootstrap compilers are described as archived, not repository-local maintenance surfaces. |
| `docs/demos/` | Review individually. Some demos mention `rivus fetch` and old manifest behavior. |

## Cleanup That Is Not Archival

These are generated or local artifacts. Do not move them into `faber-archivum`; delete or ignore them when doing the actual pruning pass.

| Path/pattern | Handling |
| ------------ | -------- |
| `compilers/nanus-ts/.*.bun-build` | Ignored build artifacts. Delete locally; do not archive. |
| `opus/` | Build output. Already ignored; do not archive. |
| `compilers/radix-rs/target/` | Rust build output. Already ignored; do not archive. |
| `.codex/worktrees/` | Local worktrees. Already ignored; do not archive. |
| `.DS_Store` | Local Finder metadata. Delete locally; do not archive. |

## Suggested Move Order

1. Copy or move the high-confidence source trees into `../faber-archivum/` under stable folders:
   - `bootstrap/nanus-*`
   - `self-hosting/rivus`
   - `reference/faber-ts`
   - `reference/rivus-cli`
2. Commit the archive repository first, so the old code has a durable destination.
3. Remove the moved paths from the main repository.
4. Move active compiler/runtime/stdlib paths into the new `radix/` workspace:
   - `compilers/radix-rs/` -> `radix/crates/radix/`
   - `runtimes/norma-rs/` -> `radix/crates/norma/`
   - `stdlib/norma/` -> `radix/stdlib/norma/`
5. Add `radix/Cargo.toml` as the workspace root and update Cargo commands accordingly.
6. Rewrite `package.json`, `README.md`, `AGENTS.md`, and `project.yaml` around the smaller canonical surface.
7. Replace root scripts with a narrow set:
   - `ci`
   - `check:radix`
   - `test:radix`
   - optional `build:radix`
   - optional future `build:wasm`
8. Sweep docs for now-broken local links to `compilers/nanus-*`, `compilers/rivus`, `compilers/radix-rs`, `stdlib/norma`, `runtimes/norma-rs`, `docs/reference/faber-ts`, and `docs/reference/rivus-cli`.
9. Decide whether to preserve `tests/proba` YAML cases by porting them into `radix` tests before archiving the harness.
10. Run the remaining gate: `bun run ci`.

## Expected Main-Repo Shape After Pruning

The main repository should roughly collapse to:

```text
radix/
  Cargo.toml
  crates/
    radix/
    norma/
  stdlib/
    norma/
docs/grammatica/
docs/wasm-execution-plan.md
docs/pruning-archive-plan.md
examples/exempla/        # after removing archive-only examples
tests/                   # only if converted to radix-rs/current spec tests
scripta/                 # only radix/current utility scripts
```

That leaves one compiler truth in the main repo. The old paths remain available in `faber-archivum`, but they stop shaping the build, docs, and decision surface of current Faber.
