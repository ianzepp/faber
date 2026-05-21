# Faber TOML Front Matter Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-toml-frontmatter/`
**Depends On**: `faber explain` embedded corpus, explain coverage manifest, current `toml` dependency in `crates/faber`
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber's explain corpus currently uses Markdown files with YAML-style front matter delimited by `---`:

```markdown
---
term: "functio"
kind: "keyword"
examples:
  - "examples/exempla/functio/functio.fab"
---
```

The current parser in `crates/faber/src/explain.rs` is a small hand-rolled YAML-like parser. This keeps the implementation narrow, but the file format now reads as YAML and encourages future YAML dependency or compatibility assumptions.

Faber already uses TOML for package manifests (`faber.toml`) and depends on the `toml` crate in `crates/faber`. The explain corpus should use TOML front matter instead:

```markdown
+++
term = "functio"
kind = "keyword"
examples = ["examples/exempla/functio/functio.fab"]
+++
```

The goal is to switch front matter to TOML, update every parser and test that reads it, and remove YAML-style front matter from the Faber-owned explain corpus.

## Normalized Spec

### In Scope

- Convert every `explain/*.md` entry from `---` YAML-style front matter to `+++` TOML front matter.
- Update `crates/faber/src/explain.rs` to parse TOML front matter using the existing `toml` crate.
- Update explain tests and test helpers to expect `+++` delimiters and TOML syntax.
- Update explain documentation and factory docs that describe the corpus format.
- Keep the Markdown body format unchanged.
- Keep the front matter schema unchanged:
  - `term`,
  - `kind`,
  - `category`,
  - `canonical`,
  - `summary`,
  - `syntax`,
  - `examples`,
  - `aliases`,
  - `legacy`,
  - `canonical_term`,
  - `related`.
- Preserve unknown-field validation.
- Preserve duplicate-field validation through TOML parser errors.
- Preserve canonical, legacy, alias, search, list, category, JSON, and coverage behavior.

### Out Of Scope

- Do not remove or redesign the Norma YAML runtime module in this plan. `crates/norma/yaml.rs`, `stdlib/norma/yaml.fab`, and YAML data-format support are a separate language/runtime decision.
- Do not change `faber.toml` package manifest syntax.
- Do not change explain lookup semantics, rendering layout, or JSON shape except where TOML parsing produces equivalent values.
- Do not introduce YAML parsing compatibility shims unless explicitly needed for a transitional phase.
- Do not keep both `---` and `+++` accepted indefinitely.

## Repo-Aware Baseline

Current front matter parser:

- `crates/faber/src/explain.rs::parse_entry` expects the first line to be `---`.
- `parse_entry` scans until the closing `---`.
- `parse_frontmatter` handles a tiny YAML-like subset:
  - `key: "value"`,
  - `key: true`,
  - `key:` followed by indented `- "item"` list entries.
- Unknown fields fail validation.

Current test helper:

- `crates/faber/src/explain_test.rs::frontmatter_value` expects `---` and parses `key: value`.

Current docs:

- `docs/grammatica/explain.md` shows `---` front matter and YAML-style lists.
- `docs/factory/faber-explain-command/plan.md` calls the format "YAML-like frontmatter".
- `docs/factory/faber-explain-coverage-completion/plan.md` shows YAML-style front matter examples.

Potentially related but out of scope:

- `crates/norma/Cargo.toml` depends on `serde_yaml` for the Norma YAML runtime module.
- `crates/norma/yaml.rs` implements YAML data-format functions.
- `docs/factory/stdlib-data-formats/plan.md` already tracks broader data-format/YAML questions.

## Format Contract

Faber explain entries use TOML front matter delimited by `+++`.

Example:

````markdown
+++
term = "proba"
kind = "keyword"
category = "testing"
canonical = true
summary = "Defines a single test case."
syntax = "proba <name> <block>"
aliases = ["test"]
related = ["adfirma"]
+++

`proba` introduces one test case.

```text
proba "arithmetic passes" {
    adfirma 1 + 1 ≡ 2
}
```
````

When written at the top of a real explain file, the front matter delimiters are exactly `+++`:

```markdown
+++
term = "proba"
kind = "keyword"
category = "testing"
canonical = true
summary = "Defines a single test case."
syntax = "proba <name> <block>"
aliases = ["test"]
related = ["adfirma"]
+++
```

Rules:

- The first line must be exactly `+++`.
- The closing delimiter must be exactly `+++` on its own line.
- Front matter content must be valid TOML.
- The TOML root must be a table.
- String fields must be strings.
- Boolean fields must be booleans.
- List fields must be arrays of strings.
- Unknown fields fail validation.
- Required fields remain required.
- Non-canonical entries still require `canonical_term`.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline and inventory | Capture current parser behavior, current corpus count, docs references, and YAML-related surfaces. | Ledger records explain front matter inventory and out-of-scope YAML runtime surfaces. |
| 1 | TOML parser support | Replace hand-rolled YAML-like front matter parsing with `toml` parsing and `+++` delimiters. | Unit tests prove TOML front matter parses and YAML-style delimiters fail. |
| 2 | Corpus migration | Convert all `explain/*.md` files to TOML front matter. | Coverage and registry tests pass with migrated corpus. |
| 3 | Docs migration | Update user docs and factory docs to describe TOML front matter. | No Faber-owned docs describe explain corpus front matter as YAML/YAML-like. |
| 4 | Residue cleanup | Remove old parser helpers and stale YAML-frontmatter assumptions from tests. | `rg` finds no stale `---` front matter assumptions in explain parser/tests/docs. |
| 5 | Validation and closeout | Run full gates, explain command smokes, and update ledger/plan status. | Validation passes and work is committed. |

## Phase Details

### Phase 0: Baseline and Inventory

Steps:

- Inspect `git status --short`.
- Count explain corpus files:

```bash
find explain -maxdepth 1 -name '*.md' | wc -l
```

- Record current front matter parser functions in `crates/faber/src/explain.rs`.
- Record current explain behavior:

```bash
cargo run -p faber -- explain functio
cargo run -p faber -- explain --json functio
cargo run -p faber -- explain --list
cargo test -p faber explain_test
```

- Inventory docs that mention front matter or YAML-like front matter.
- Inventory YAML runtime surfaces and mark them out of scope unless the user expands the plan.

Checkpoint:

- Ledger created.
- No behavior changes.
- YAML runtime module is explicitly classified as out of scope for this plan.

### Phase 1: TOML Parser Support

Steps:

- Update `parse_entry` to require `+++` opening and closing delimiters.
- Replace `parse_frontmatter` with TOML parsing through `toml`.
- Convert TOML values into the existing internal entry model or deserialize into a typed front matter struct.
- Preserve the current schema validation behavior:
  - required fields fail when missing,
  - unknown fields fail,
  - wrong field types fail,
  - non-canonical entries require `canonical_term`.
- Update parser unit tests to use TOML front matter.
- Add explicit negative tests for:
  - missing `+++`,
  - unterminated `+++`,
  - old `---` front matter,
  - unknown field,
  - array with non-string item,
  - scalar where array is required.

Checkpoint:

- TOML-frontmatter fixtures parse.
- YAML-style `---` explain entries fail fast with a clear error.
- No explain behavior changes except accepted front matter syntax.

### Phase 2: Corpus Migration

Steps:

- Convert all root `explain/*.md` files from YAML-style front matter to TOML front matter.
- Preserve all current field values exactly.
- Prefer single-line TOML arrays for short list fields:

```toml
aliases = ["function"]
related = ["→", "incipit"]
```

- Use multi-line TOML arrays only where readability clearly benefits.
- Keep Markdown bodies unchanged except where surrounding documentation examples need delimiter updates.
- Do not rename corpus files in this phase.

Checkpoint:

- Every `explain/*.md` starts with `+++`.
- No `explain/*.md` starts with `---`.
- Coverage tests pass.
- `faber explain` lookup, search, list, category, and JSON outputs are behavior-compatible.

### Phase 3: Docs Migration

Steps:

- Update `docs/grammatica/explain.md`:
  - say entries use TOML front matter,
  - show `+++` examples,
  - show TOML arrays.
- Update `docs/factory/faber-explain-command/plan.md` and related explain factory docs to say TOML front matter rather than YAML-like front matter.
- Update `docs/factory/faber-explain-coverage-completion/plan.md` examples.
- Search for stale references:

```bash
rg -n 'YAML-like|YAML front|frontmatter|front matter|---' docs crates/faber explain
```

- Keep Markdown horizontal rules outside front matter if they are normal prose separators; do not blindly delete all `---` from docs.

Checkpoint:

- User-facing docs describe TOML front matter.
- Factory docs no longer prescribe YAML-like front matter for the explain corpus.
- Any remaining `---` references are normal Markdown separators or historical notes, not current front matter instructions.

### Phase 4: Residue Cleanup

Steps:

- Remove the old `FrontValue` enum if no longer needed.
- Remove the old line-oriented YAML-like parser helpers.
- Update `crates/faber/src/explain_test.rs::frontmatter_value` or replace it with TOML-aware helper logic.
- Confirm `crates/faber` does not add any YAML dependency for front matter.
- Confirm `serde_yaml` is not required by `crates/faber`.

Checkpoint:

- `rg -n 'serde_yaml|yaml' crates/faber Cargo.toml crates/faber/Cargo.toml` shows no YAML dependency or parser references in Faber.
- `crates/faber` front matter logic is TOML-only.
- All explain tests pass.

### Phase 5: Validation and Closeout

Steps:

Run:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
```

Run explain smokes:

```bash
cargo run -p faber -- explain functio
cargo run -p faber -- explain ≡
cargo run -p faber -- explain --json functio
cargo run -p faber -- explain --search equality
cargo run -p faber -- explain --list
cargo run -p faber -- explain --category testing
```

Run residue checks:

```bash
find explain -maxdepth 1 -name '*.md' -exec sh -c 'head -n 1 "$1" | grep -qx "+++"' _ {} \;
rg -n '^---$|YAML-like|YAML front|serde_yaml' explain crates/faber docs/grammatica/explain.md docs/factory/faber-explain-command docs/factory/faber-explain-coverage-completion
```

The residue `rg` may still find deliberate historical notes, Markdown horizontal rules, or out-of-scope Norma YAML references outside the searched paths. Each hit must be reviewed, not mechanically deleted.

Checkpoint:

- Validation passes.
- Plan status and ledger are updated.
- Work is committed.

## Epic Candidates And Scopable Issues

### Issue A: TOML Front Matter Parser

Acceptance:

- `+++` delimiters are required,
- TOML root table parses through the `toml` crate,
- unknown and mistyped fields fail,
- old `---` corpus entries fail.

### Issue B: Explain Corpus Migration

Acceptance:

- all `explain/*.md` files use TOML front matter,
- all embedded entries validate,
- lookup/search/list/category/json behavior is unchanged.

### Issue C: Documentation Update

Acceptance:

- explain docs show TOML front matter,
- factory docs no longer describe YAML-like explain front matter,
- Faber docs clearly distinguish TOML front matter from out-of-scope YAML runtime/data-format support.

### Issue D: Residue Guardrails

Acceptance:

- no YAML parser dependency in `crates/faber`,
- no hand-rolled YAML-like front matter parser remains,
- tests guard against accidental `---` front matter reintroduction.

## Checkpoints

- Phase 1 should be committed before mass corpus migration if possible, because parser failures will be easier to debug.
- Phase 2 may be a large mechanical diff; keep it isolated from parser logic.
- Phase 3 should not rewrite unrelated documentation prose.
- Phase 5 must prove installed-style `faber explain` behavior remains intact.

## Companion Skill Plan

- Use the full `delivery` skill pipeline before implementation phases.
- Use `factory` for multi-phase execution.
- Use `zombie-docs` after docs migration if stale front matter claims remain hard to classify.
- Use `poker-face` before closeout because a delimiter migration is easy to overclaim while leaving stale docs or helpers behind.

## Gate Plan

Required gate before final closeout:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
```

Required explain smokes:

```bash
cargo run -p faber -- explain functio
cargo run -p faber -- explain ≡
cargo run -p faber -- explain --json functio
cargo run -p faber -- explain --search equality
cargo run -p faber -- explain --list
cargo run -p faber -- explain --category testing
```

Required corpus checks:

- every root `explain/*.md` begins with `+++`,
- no root `explain/*.md` begins with `---`,
- `crates/faber` has no YAML parser dependency,
- unknown fields still fail validation.

## Open Questions

- Should future Faber-owned Markdown metadata outside `explain/*.md` also be required to use TOML front matter, or is this plan only for the explain corpus?
- Should the broader Norma YAML runtime module remain available, be renamed as a separate optional data-format surface, or be removed in a future clean-break plan?
- Should old `---` explain front matter fail immediately, or should there be one temporary migration-only compatibility path? The current recommendation is immediate failure after corpus migration.
