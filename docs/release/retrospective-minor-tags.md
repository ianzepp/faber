# Retrospective Minor Tag Proposal

This document proposes marker-only minor version tags for the development
history after `v0.7.0`.

These tags are not evidence that binaries were built or published at the time.
They are retrospective source-history markers intended to make the version
number communicate the amount of development that happened before the next real
compiled release.

## Recommendation

Use annotated tags from `v0.8.0` through `v0.32.0` as retrospective markers.
Reserve `v0.33.0` for the first renewed compiled, tested, published release.

Do not create GitHub Releases with assets for these marker tags unless each tag
is separately checked out, built, and documented as a historical build.

This ladder is target-fit to the goal of making `v0.33.0` the next real
compiled release. The history contains enough distinct development epochs to
justify a ladder in this range, but `v0.32.0` is not a uniquely discovered
natural endpoint. A stricter epoch-only pass would likely produce fewer tags; a
more granular pass through the dense January work could justify more.

## Method

The candidate boundaries were selected from `git log v0.7.0..HEAD` using:

- chronological commit clustering
- dominant path churn and subject vocabulary
- phase/checkpoint commits
- commits immediately before or at a clear development-theme shift
- preference for source-history meaning over equal commit spacing

## Candidate Tags

| Version | Commit | Date | Commits since previous | Commit subject | Boundary rationale |
| ------- | ------ | ---- | ---------------------- | -------------- | ------------------ |
| `v0.8.0` | `14bb1368` | 2025-12-31 | 16 | Move function modifiers to postfix position, add curata keyword | End initial bootstrap AST/lexor shaping before TypeScript pivot |
| `v0.9.0` | `5ff64f33` | 2025-12-31 | 18 | Update bootstrap-ts.md: all blockers resolved | Bootstrap TypeScript blocker resolution |
| `v0.10.0` | `81830d15` | 2025-12-31 | 26 | Complete bootstrap parser: add remaining statement parsers | Bootstrap parser completion |
| `v0.11.0` | `9b090eaf` | 2026-01-01 | 32 | Fix bootstrap compiler: exports, enum names, and stdlib methods | Bootstrap semantic/codegen/CLI viability |
| `v0.12.0` | `49fd9d05` | 2026-01-02 | 16 | Complete Phase 0: Convert all parser/semantic/codegen to modern syntax | Modern syntax phase 0 conversion complete |
| `v0.13.0` | `6e39b3b5` | 2026-01-02 | 31 | Rename fons/primus to fons/faber and fons/proprius to fons/rivus | Faber/Rivus split and directory rename |
| `v0.14.0` | `fffc2b23` | 2026-01-05 | 114 | Mark rivus-modules.md as completed and move to completa | Rivus module resolution milestone |
| `v0.15.0` | `b781954d` | 2026-01-06 | 23 | Migrate 47 lista methods to norma and add indexed placeholders | Norma lista migration and registry generation |
| `v0.16.0` | `421a3dd1` | 2026-01-06 | 13 | Add conversio test suite (57 tests) | Conversio operators implemented and tested |
| `v0.17.0` | `c6b5cb4a` | 2026-01-06 | 25 | Fix norma template escaping, add cross-module type resolution | Unified norma registry stabilized after template/type fixes |
| `v0.18.0` | `3964f8a5` | 2026-01-07 | 33 | Regenerate norma registry from updated stdlib specs | Expanded stdlib/HAL definitions and registry regeneration |
| `v0.19.0` | `8d766e23` | 2026-01-08 | 58 | Wire up norma-registry integration for Zig codegen (#61) | Rivus multi-target codegen wired to norma |
| `v0.20.0` | `34144198` | 2026-01-09 | 34 | Fix build-exempla to use stdin for all compilers | Artifex/build pipeline and JSON norma registry stabilized |
| `v0.21.0` | `ca024cb5` | 2026-01-12 | 41 | Use norma registry for namespace calls | Target capability validation and norma namespace calls |
| `v0.22.0` | `4f3a4c5a` | 2026-01-13 | 40 | refactor(rivus): Archive broken rs/zig/cpp codegens | TypeScript-only simplification and archival of broken codegens |
| `v0.23.0` | `08182170` | 2026-01-15 | 80 | Rename faber compile command to emit | Go/Rivus work plus build/emit command surface |
| `v0.24.0` | `685cd602` | 2026-01-16 | 37 | feat(cli): add comprehensive CLI options to all commands | CLI annotation and option support before syntax pruning |
| `v0.25.0` | `eefa1a11` | 2026-01-17 | 43 | Migrate imports to § ex sectional syntax | Language syntax pruning and sectional imports |
| `v0.26.0` | `52d86e6b` | 2026-01-18 | 40 | fix(nanus): Multiple codegen improvements for rivus compilation | Nanus compatibility and Rivus compilation support |
| `v0.27.0` | `f6772dbe` | 2026-01-21 | 80 | feat(fons): add --stdin-filename to all compilers | Multi-compiler stdin/stdout tooling consolidation |
| `v0.28.0` | `f88cd3f7` | 2026-01-22 | 40 | refactor(build): split exempla into codegen (stage 4) and verify (stage 5) | Canonical nanus-ts/bootstrap and exempla staging |
| `v0.29.0` | `5cfda845` | 2026-01-26 | 52 | Added test rivus CLI entry point | Norma/HAL multi-runtime restructure |
| `v0.30.0` | `992a1d35` | 2026-01-29 | 74 | feat(radix): add bidirectional type inference | Initial Rust radix compiler through bidirectional inference |
| `v0.31.0` | `8db5673f` | 2026-02-16 | 89 | fix(radix): improve unnecessary cast warning to handle aliases and skip construction | Radix correctness burst and glyph conversion unification |
| `v0.32.0` | `750bfa09` | 2026-05-20 | 94 | Update CLI docs and examples | Post-radix repo collapse, pruning, and CLI framework rework |

## Suggested Annotated Tag Message

```text
Retrospective version marker.

This tag was created after the fact to make post-v0.7.0 Faber development
history navigable and to reserve an honest minor-version ladder before the next
compiled release. It was not originally published as a binary release.
```

## Validation Status

These candidates have not been checked out or built individually. Treat all
entries as `marker-only` until a separate validation pass proves otherwise.
