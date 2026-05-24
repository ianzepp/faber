# Epic 6.1 Delivery: Norma Classification Baseline

## Interpreted Problem

Epic 6 starts the migration of `norma` from "stdlib implementation linked into
each generated program" toward canonical Faber contracts split between
compiler-owned core behavior, host-owned effects, and temporary native Rust
support. The first slice must classify the current surfaces before changing
compiler lowering, host provisioning, strict-mode validation, or deleting any
implementation.

## Normalized Spec

Create durable Epic 6 artifacts under
`docs/factory/faber-execution-roadmap/epic-6/`:

- an Epic 6 ledger,
- a module/file-level `norma` classification,
- a first host-effect proof record for `consolum`.

The classification vocabulary is exactly:

- `core-language`: pure language/library semantics the compiler or target
  runtime can own directly,
- `host-effect`: outside-world effects and IO that route semantically through
  host syscall capability surfaces,
- `rust-bridge`: native Rust support kept for current HIR-to-Rust output while
  the host path matures,
- `needs-decision`: surfaces requiring later review before a final ownership
  move.

## Repo-Aware Baseline

Primary inputs:

- `docs/factory/faber-execution-roadmap/goal.md`, Epic 6 section.
- `hosts/macos-arm64/ARCHITECTURE.md`, especially the core/capability split and
  Norma Direction sections.
- `hosts/macos-arm64/SYSCALL_MODEL.md`, especially the Relationship To Norma
  and routing/error model sections.
- `stdlib/norma/`, including `hal/consolum.fab`.
- `crates/norma/`, including `hal/consolum.rs`.

Current repo evidence:

- `stdlib/norma/hal/consolum.fab` already describes console stdin/stdout/stderr
  as a HAL pactum and references `crates/norma/hal/consolum.rs` through
  `@ subsidia`.
- `crates/norma/hal/consolum.rs` provides direct `std::io` and `tokio::io`
  implementations for the same interface.
- `hosts/macos-arm64/src/hal/host.rs` already records
  `norma:hal/consolum` as the first HAL migration candidate.
- `hosts/macos-arm64` already has a local frame/kernel route proof with
  `host:echo` and `E_NO_ROUTE`.

## Stage Graph

1. Inventory all files under `stdlib/norma/` and `crates/norma/`.
2. Assign one primary classification to each file and record member-level
   exceptions where a file mixes host effects and pure helpers.
3. Classify `stdlib/norma/hal/consolum.fab` as `host-effect`.
4. Classify `crates/norma/hal/consolum.rs` as `rust-bridge`.
5. Record canonical syscall identities for every current `consolum` member.
6. Record that `nota`, `vide`, `mone`, and related language output semantics
   are semantically host IO even when native Rust output keeps direct printing.
7. Validate the docs only; no compiler, host provisioning, or runtime behavior
   changes are part of this slice.

## Checkpoints

This slice is complete when:

- every current `stdlib/norma` and `crates/norma` file has a classification,
- `consolum` syscall identities are recorded without new annotation syntax,
- existing Rust bridge files are preserved,
- strict-mode host manifests are treated as future consumers, not implemented,
- the worktree diff is documentation-only,
- markdown/diff consistency checks pass.

## Companion Skill Plan

- `factory`: owns the outer Epic 6 execution loop.
- `delivery`: produced this scoped execution spec.
- `poker-face`: should audit the docs against the Epic 6.1 checkpoint before
  committing.

## Gate Plan

Run:

```bash
git diff --check -- docs/factory/faber-execution-roadmap/epic-6
find docs/factory/faber-execution-roadmap/epic-6 -type f -maxdepth 1 | sort
git status --short
```

Optional content checks:

```bash
find stdlib/norma crates/norma -maxdepth 3 -type f | sort
rg -n "stdlib/norma|crates/norma|consolum:" docs/factory/faber-execution-roadmap/epic-6
```

No Cargo test is required for this docs-only classification slice unless the
diff expands beyond documentation.

## Open Questions

- Whether pure data-format transforms such as JSON/TOML/YAML eventually become
  compiler intrinsics, target-runtime library calls, or host-provided utility
  capabilities remains a later implementation decision. They are classified as
  `core-language` contracts in this slice because they are pure in-memory
  transforms, while their existing Rust files remain `rust-bridge`.
- The planned `ems`, `vfs`, and `llm` directories need future design before
  becoming executable contracts; they are classified as `needs-decision` with
  likely host-effect direction where applicable.
