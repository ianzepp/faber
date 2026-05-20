# Faber Pruning Delivery Plan

**Status**: executable delivery plan. No pruning work has been performed by this document.

This plan turns `docs/pruning-archive-plan.md` into phased work packets. Each phase is intended to be small enough for a bounded implementation agent, including `gpt-5.4-mini`, as long as the agent is given the phase section, the repository root, and the archive repository path.

Primary repos:

- Main repo: `/Users/ianzepp/work/ianzepp/faber`
- Archive repo: `/Users/ianzepp/work/ianzepp/faber-archivum`

Important rule: do not run Git commands that create locks in parallel in the same repository. Commit boundaries below are intentional.

## Interpreted Problem

The Faber repo currently carries several historical compiler surfaces (`nanus-*`, `rivus`, old TypeScript reference code, old test harnesses, and old scripts) even though current work is centered on `radix-rs` and a future Rust/WASM execution path. The desired result is a smaller main repo with one canonical compiler tree and an archive repo that preserves old code.

The work must:

- preserve historical code in `faber-archivum` before deleting it from the main repo
- collapse active compiler/runtime/stdlib pieces into a new `radix/` workspace
- remove or rewrite root scripts that advertise archived surfaces
- update docs and project metadata so they match the new layout
- keep the final main repo passing its scoped CI gate

## Normalized Spec

Target main-repo shape:

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
docs/pruning-archive-delivery-plan.md
examples/exempla/
scripta/
```

Archive destinations:

```text
faber-archivum/
  bootstrap/
    nanus-ts/
    nanus-go/
    nanus-rs/
    nanus-py/
  self-hosting/
    rivus/
    rivus-cli/
  reference/
    faber-ts/
    rivus-cli/
  tests/
    proba/
    golden/
  scripts/
    bootstrap-and-self-hosting/
```

Final root commands should be narrow:

```bash
bun run ci
bun run check:radix
bun run test:radix
bun run build:radix
```

Legacy commands such as `build:nanus-*`, `build:bootstrap`, `test:nanus-ts`, `test:legacy`, `lint:nanus-ts`, `typecheck:legacy`, `verify:nanus`, `test:report`, and `golden` should either be gone from the main repo or explicitly moved to the archive.

## Repo-Aware Baseline

Current canonical pieces:

- `compilers/radix-rs/` is the active compiler.
- `runtimes/norma-rs/` is the Rust runtime crate.
- `stdlib/norma/` contains Faber stdlib definitions and `@ verte` / `@ subsidia` metadata.
- `.github/workflows/ci.yml` already gates only `compilers/radix-rs`.

Known legacy pieces:

- `compilers/nanus-*` are bootstrap compilers.
- `compilers/rivus/` and `compilers/rivus-cli/` are self-hosting surfaces.
- `docs/reference/faber-ts/` and `docs/reference/rivus-cli/` are historical reference material.
- `tests/proba` and `tests/golden` are mixed, but much of their current harness is old TypeScript/rivus/nanus oriented.
- `scripta/build.ts` claims to be radix-focused but still builds `nanus-rs` and compiles `rivus`.

Layout-sensitive caveat:

- `stdlib/norma/*.fab` files currently contain relative `@ subsidia` paths such as `../norma-rs/json.rs` and `../../norma-rs/hal/solum.rs`. The collapse phase must rewrite these or replace them with logical runtime identifiers.

## Stage Graph

```text
Phase 0: Preflight inventory
  -> Phase 1: Archive legacy surfaces
  -> Phase 2: Remove archived surfaces from main repo
  -> Phase 3: Collapse active tree into radix workspace
  -> Phase 4: Rewrite scripts, metadata, docs, and CI
  -> Phase 5: Salvage or archive tests/examples
  -> Phase 6: Final validation and cleanup
```

Phase 1 must commit in `faber-archivum` before Phase 2 deletes anything from `faber`.

Phases 2 and 3 should not run in parallel because they both move top-level paths and update references.

Phase 4 can be split by file families after Phase 3 lands.

## Phase 0: Preflight Inventory

Objective: confirm the worktree state and produce a short inventory before moving files.

Owned paths:

- read-only over both repos

Steps:

1. In main repo, run:

   ```bash
   git status --short --untracked-files=all
   git log --oneline -5
   ```

2. In archive repo, run:

   ```bash
   git status --short --untracked-files=all
   git log --oneline -5
   find . -maxdepth 3 -type f | sort | sed -n '1,220p'
   ```

3. Confirm no uncommitted user work would be overwritten.

Acceptance:

- Both repo states are known.
- Any dirty worktree is reported before proceeding.
- No files moved.

Commit:

- None unless an inventory note is added intentionally.

## Phase 1: Archive Legacy Surfaces

Objective: copy old surfaces from the main repo into `faber-archivum` and commit the archive repo first.

Owned write paths in archive repo:

- `bootstrap/nanus-ts/`
- `bootstrap/nanus-go/`
- `bootstrap/nanus-rs/`
- `bootstrap/nanus-py/`
- `self-hosting/rivus/`
- `self-hosting/rivus-cli/`
- `reference/faber-ts/`
- `reference/rivus-cli/`
- `tests/proba/`
- `tests/golden/`
- `scripts/bootstrap-and-self-hosting/`
- archive `README.md`

Read paths in main repo:

- `compilers/nanus-*`
- `compilers/rivus/`
- `compilers/rivus-cli/`
- `docs/reference/faber-ts/`
- `docs/reference/rivus-cli/`
- `tests/proba/`
- `tests/golden/`
- old scripts listed in `docs/pruning-archive-plan.md`

Suggested commands from main repo root:

```bash
mkdir -p ../faber-archivum/bootstrap ../faber-archivum/self-hosting ../faber-archivum/reference ../faber-archivum/tests ../faber-archivum/scripts/bootstrap-and-self-hosting
rsync -a --delete compilers/nanus-ts/ ../faber-archivum/bootstrap/nanus-ts/
rsync -a --delete compilers/nanus-go/ ../faber-archivum/bootstrap/nanus-go/
rsync -a --delete compilers/nanus-rs/ ../faber-archivum/bootstrap/nanus-rs/
rsync -a --delete compilers/nanus-py/ ../faber-archivum/bootstrap/nanus-py/
rsync -a --delete compilers/rivus/ ../faber-archivum/self-hosting/rivus/
rsync -a --delete compilers/rivus-cli/ ../faber-archivum/self-hosting/rivus-cli/
rsync -a --delete docs/reference/faber-ts/ ../faber-archivum/reference/faber-ts/
rsync -a --delete docs/reference/rivus-cli/ ../faber-archivum/reference/rivus-cli/
rsync -a --delete tests/proba/ ../faber-archivum/tests/proba/
rsync -a --delete tests/golden/ ../faber-archivum/tests/golden/
```

Copy scripts deliberately:

```bash
cp scripta/build-nanus-ts.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-nanus-go.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-nanus-rs.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-nanus-py.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-bootstrap.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-rivus.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-faber.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/verify-nanus.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/rivus-features.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/rivus-zig ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/build-golden.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/golden.ts ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/test-report ../faber-archivum/scripts/bootstrap-and-self-hosting/
cp scripta/typecheck ../faber-archivum/scripts/bootstrap-and-self-hosting/typecheck-nanus-ts
```

Update archive `README.md` to describe these imported folders.

Acceptance:

- Archive repo contains all copied paths.
- Archive README lists the imported surfaces and says they came from the main Faber repo pruning pass.
- Archive repo commits successfully before the main repo deletes anything.

Archive validation:

```bash
cd ../faber-archivum
git status --short --untracked-files=all
git add .
git commit -m "Archive legacy Faber surfaces"
```

Main repo commit:

- None in this phase.

Subagent packet:

```text
You own only /Users/ianzepp/work/ianzepp/faber-archivum writes. Copy the legacy surfaces from /Users/ianzepp/work/ianzepp/faber exactly as listed in Phase 1 of docs/pruning-archive-delivery-plan.md. Do not delete anything from the main repo. Update archive README.md with a short inventory. Commit only in faber-archivum with message "Archive legacy Faber surfaces". Report copied paths and commit hash.
```

## Phase 2: Remove Archived Surfaces From Main Repo

Objective: delete the surfaces that Phase 1 preserved in the archive repo.

Prerequisite:

- Phase 1 archive commit exists.

Owned write paths in main repo:

- `compilers/nanus-*`
- `compilers/rivus/`
- `compilers/rivus-cli/`
- `docs/reference/faber-ts/`
- `docs/reference/rivus-cli/`
- `tests/proba/`
- `tests/golden/`
- old bootstrap/self-hosting scripts copied in Phase 1
- `tsconfig.nanus.json`

Steps:

1. Confirm archive commit:

   ```bash
   cd ../faber-archivum && git log --oneline -1
   ```

2. In main repo, remove archived paths with `git rm -r`.

3. Delete old scripts with `git rm`.

4. Leave `scripta/build.ts`, `scripta/build-exempla.ts`, and `scripta/prettier` for Phase 4 because they need rewrites rather than simple deletion.

Acceptance:

- `git status --short` shows deletions for archived paths.
- No active `radix` code has been moved yet.
- No docs have been broadly rewritten yet except unavoidable link notes.

Suggested commit:

```bash
git commit -m "Remove archived legacy compiler surfaces"
```

Subagent packet:

```text
You own deletion only in /Users/ianzepp/work/ianzepp/faber. First verify /Users/ianzepp/work/ianzepp/faber-archivum has a recent archive commit. Then remove the archived legacy paths listed in Phase 2. Do not move radix, norma, README, package.json, project.yaml, or CI in this phase. Commit with "Remove archived legacy compiler surfaces". Report any path that was missing or unexpectedly dirty.
```

## Phase 3: Collapse Active Tree Into `radix/`

Objective: move the active compiler, Rust runtime, and Faber stdlib into one workspace.

Owned write paths:

- `compilers/radix-rs/` -> `radix/crates/radix/`
- `runtimes/norma-rs/` -> `radix/crates/norma/`
- `stdlib/norma/` -> `radix/stdlib/norma/`
- `radix/Cargo.toml`
- Cargo manifests and path references needed for the move

Steps:

1. Move directories:

   ```bash
   mkdir -p radix/crates radix/stdlib
   git mv compilers/radix-rs radix/crates/radix
   git mv runtimes/norma-rs radix/crates/norma
   git mv stdlib/norma radix/stdlib/norma
   ```

2. Add `radix/Cargo.toml`:

   ```toml
   [workspace]
   members = [
       "crates/radix",
       "crates/norma",
   ]
   resolver = "2"
   ```

3. Move `radix/crates/norma/lib.rs` into `radix/crates/norma/src/lib.rs` and move sibling Rust modules under `src/` if you want standard Cargo layout. If you keep the current flat layout, preserve `[lib] path = "lib.rs"` in `crates/norma/Cargo.toml`.

4. Update `radix/crates/radix/Cargo.toml` package description if desired, but do not rename the crate unless the CLI and docs are updated in the same phase.

5. Rewrite `@ subsidia rs` paths in `radix/stdlib/norma/*.fab`:

   Minimum viable rewrite:

   - `../norma-rs/json.rs` -> `../../crates/norma/src/json.rs` if standard layout is adopted
   - `../../norma-rs/hal/solum.rs` -> `../../../crates/norma/src/hal/solum.rs`

   Better rewrite:

   - replace filesystem-relative runtime paths with logical identifiers such as `@ subsidia rs "norma::json"` and teach any consumer to resolve them later. Only do this if the consumer behavior is understood.

Acceptance:

- `cargo metadata --manifest-path radix/Cargo.toml` succeeds.
- `cargo test --manifest-path radix/Cargo.toml -p radix` succeeds or failures are clearly unrelated to the move.
- `cargo test --manifest-path radix/Cargo.toml -p norma` succeeds or failures are documented.

Suggested commit:

```bash
git commit -m "Collapse active Faber tree into radix workspace"
```

Subagent packet:

```text
You own the active tree move in /Users/ianzepp/work/ianzepp/faber. Move compilers/radix-rs to radix/crates/radix, runtimes/norma-rs to radix/crates/norma, and stdlib/norma to radix/stdlib/norma. Add radix/Cargo.toml workspace. Update only references required to make cargo metadata and cargo test for the workspace run. Do not rewrite README/package/project docs except if a path is required for build commands. Commit with "Collapse active Faber tree into radix workspace".
```

## Phase 4: Rewrite Scripts, Metadata, CI, And Docs

Objective: make the root repo describe and operate the new shape.

Owned write paths:

- `package.json`
- `.github/workflows/ci.yml`
- `README.md`
- `AGENTS.md`
- `project.yaml`
- `docs/grammatica/targets.md`
- `docs/grammatica/verba.md`
- `docs/grammatica/cli.md`
- `docs/wasm-execution-plan.md`
- `docs/pruning-archive-plan.md`
- `scripta/build.ts`
- `scripta/build-exempla.ts`
- `scripta/prettier`

Required script changes:

- `check:radix`: `cargo fmt --manifest-path radix/Cargo.toml --all -- --check`
- `test:radix`: `cargo test --manifest-path radix/Cargo.toml`
- `ci`: `bun run check:radix && bun run test:radix`
- `build:radix`: `cargo build --release --manifest-path radix/Cargo.toml -p radix`
- remove legacy script names from `package.json`

CI update:

```yaml
- name: Format check
  run: cargo fmt --manifest-path radix/Cargo.toml --all -- --check
- name: Test radix workspace
  run: cargo test --manifest-path radix/Cargo.toml
```

Docs update checklist:

```bash
rg -n "compilers/radix-rs|runtimes/norma-rs|stdlib/norma|compilers/nanus|compilers/rivus|docs/reference/faber-ts|docs/reference/rivus-cli|test:nanus|build:nanus|test:report|golden"
```

Each hit must be one of:

- rewritten to the new path
- explicitly marked as archived with `../faber-archivum/...`
- removed because it advertised old local behavior

Acceptance:

- `package.json` exposes only current scripts.
- CI points at `radix/Cargo.toml`.
- README and AGENTS no longer teach local `nanus` or `rivus` operation.
- `project.yaml` lists only active/current projects in the main repo.
- `rg` sweep has no unintentional stale local references.

Suggested commit:

```bash
git commit -m "Rewrite repo surface around radix workspace"
```

Subagent packet:

```text
You own root metadata/scripts/docs after the radix workspace move. Update package.json, CI, README, AGENTS, project.yaml, selected docs, and current scripts so they refer to radix/Cargo.toml and no longer expose nanus/rivus as main-repo surfaces. Use the Phase 4 checklist. Do not move files. Commit with "Rewrite repo surface around radix workspace".
```

## Phase 5: Tests And Examples Decision

Objective: decide what remains under `tests/` and `examples/` after old harnesses are archived.

Owned write paths:

- `examples/exempla/`
- `tests/`
- `radix/crates/radix/tests/`
- `radix/crates/radix/src/*_test.rs`
- docs that link to examples/tests

Work items:

1. Remove or archive examples that only demonstrate archived surfaces:

   - `examples/exempla/rivus-cli-annotated/` should already be archived in Phase 1 and deleted in Phase 2.
   - Review `examples/exempla/cli/` because CLI lowering is aspirational.

2. Decide whether `tests/proba` YAML cases should be restored as:

   - `radix/crates/radix/tests/spec_cases/`
   - `radix/crates/radix/tests/proba_cases/`
   - or not restored until a new Rust-native harness exists

3. If restoring cases, port only current grammar examples. Skip old syntax such as `fit`, `fiet`, `fiunt`, `fient`.

Acceptance:

- No tests import archived/generated `opus/rivus` or old `../../faber` TypeScript modules.
- Any remaining tests are run by `cargo test --manifest-path radix/Cargo.toml`.
- Examples referenced by README/docs exist and parse under current `radix`.

Suggested commit:

```bash
git commit -m "Align examples and tests with radix workspace"
```

Subagent packet:

```text
You own examples/tests cleanup after archive deletion and radix workspace move. Do not resurrect old TypeScript harnesses. Keep or port only examples/tests that match current EBNF and radix behavior. Ensure docs do not link to deleted examples. Commit with "Align examples and tests with radix workspace".
```

## Phase 6: Final Validation And Cleanup

Objective: prove the final repo is coherent.

Owned paths:

- small final fixes only

Commands:

```bash
git status --short --untracked-files=all
bun run ci
cargo run --manifest-path radix/Cargo.toml -p radix -- targets
rg -n "compilers/nanus|compilers/rivus|compilers/radix-rs|runtimes/norma-rs|stdlib/norma|docs/reference/faber-ts|docs/reference/rivus-cli|nanus-ts|nanus-go|nanus-rs|nanus-py|test:legacy|typecheck:legacy|lint:nanus"
```

Expected stale-reference policy:

- zero stale references in operational docs/scripts
- allowed references only in `docs/pruning-archive-plan.md`, `docs/pruning-archive-delivery-plan.md`, or explicit archive pointers

Cleanup:

- delete ignored local artifacts if they clutter validation (`opus/`, `target/`, `.DS_Store`)
- do not commit generated build outputs

Suggested commit:

```bash
git commit -m "Finish pruning cleanup"
```

If there are no changes after validation, no commit is needed.

## Parallel Workstreams

Safe parallelism:

- Phase 1 archive copy can be performed by one worker while another worker prepares read-only doc reference inventory, but only one worker should run Git commands in `faber-archivum`.
- After Phase 3, Phase 4 can split into docs and scripts if write sets are disjoint.
- Phase 5 can run after Phase 4 starts, but only if its worker owns separate test/example paths and coordinates doc links.

Do not parallelize:

- Phase 1 archive commit and Phase 2 deletion.
- Phase 2 deletion and Phase 3 active tree move.
- Multiple Git operations in the same repo.

## Checkpoints

Checkpoint A: Archive preserved

- Archive repo has a commit containing legacy surfaces.
- Main repo has not deleted anything yet.

Checkpoint B: Main repo no longer carries old compiler trees

- `compilers/nanus-*`, `compilers/rivus`, `compilers/rivus-cli`, and `docs/reference/*` old trees are gone from main.
- No active tree move yet.

Checkpoint C: `radix/` workspace exists

- `cargo metadata --manifest-path radix/Cargo.toml` succeeds.
- `cargo test --manifest-path radix/Cargo.toml` is the primary Rust gate.

Checkpoint D: root contract rewritten

- `package.json`, CI, README, AGENTS, and `project.yaml` match the new layout.

Checkpoint E: final gate

- `bun run ci` passes.
- Stale reference sweep is clean or intentionally documented.

## Companion Skill Plan

- Use `clean-break` during Phases 2 and 4 to avoid preserving compatibility shells.
- Use `zombie-docs` during Phase 4 to repair stale Markdown claims after moves.
- Use `slice` only if a phase needs large line-preserving code movement inside files; directory moves should use `git mv`.
- Use `poker-face` after Phase 6 to compare the final repo against this plan and the pruning archive plan.

## Gate Plan

Minimum gate per phase:

| Phase | Gate |
| ----- | ---- |
| 0 | status/inventory complete |
| 1 | archive commit exists |
| 2 | main deletion commit exists |
| 3 | `cargo metadata --manifest-path radix/Cargo.toml` and `cargo test --manifest-path radix/Cargo.toml` |
| 4 | `bun run ci`; stale reference sweep reviewed |
| 5 | `bun run ci`; examples/docs links reviewed |
| 6 | full final validation commands pass |

## Open Questions

1. Should the `radix` crate package name remain `radix`, or should the CLI/package eventually become `faber` after pruning?
2. Should `@ subsidia` use logical runtime identifiers now, or should this delivery only rewrite filesystem paths and defer logical resolution?
3. Should TypeScript and Go `radix` emitters remain after this prune, or should they become a later clean-break phase?
4. Should `tests/proba` YAML cases be ported immediately, or archived wholesale and reintroduced only when a Rust-native spec harness is designed?
5. Should `examples/exempla/cli/` remain as aspirational examples, or move to the archive until CLI lowering exists?
