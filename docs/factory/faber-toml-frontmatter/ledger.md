# Faber TOML Front Matter - Factory Ledger

**Phase Set Source**: `docs/factory/faber-toml-frontmatter/plan.md`
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-toml-frontmatter/`
**Commit Policy**: Commit after each completed phase and validation gate pass
**Created**: 2026-05-21
**Mode**: clean-break for front matter format + parser replacement

## Current Phase
0 - Baseline and inventory (in progress)

## Completed Phases
(none yet)

## Final Validation
(none yet — will be populated in Phase 5)

## Baseline Captures (Phase 0)

### Git Status (start of session)
```
On branch main
Your branch is ahead of 'origin/main' by 2 commits.

nothing to commit, working tree clean
```

### Corpus Inventory
- `find explain -maxdepth 1 -name '*.md' | wc -l` → **167** Markdown explain entries.
- All currently begin with `---` YAML-style front matter (verified by sampling functio.md, proba.md, etc.).
- `explain/coverage.toml` exists and is TOML (already using toml for manifest).
- No `.fab` sources or other metadata use the explain front matter format.

### Current Front Matter Parser (crates/faber/src/explain.rs)
- `parse_entry` (line ~436): requires first line `---`, scans for closing `---`, collects lines, calls `parse_frontmatter`.
- `parse_frontmatter` (line ~495): hand-rolled line parser for tiny YAML subset:
  - `key: "value"` or `key: value` → Scalar
  - `key:` then indented `  - "item"` → List
  - `allowed` list hard-coded; unknown fields error; duplicate fields error.
- `FrontValue` enum: Scalar(String) | List(Vec<String>)
- Helpers: `required_string`, `optional_list`, `required_bool`, `parse_scalar` (strips outer quotes), `parse_kind`.
- Validation: required fields (term,kind,category,canonical,summary,syntax), body must have ```fab example, summary ends with '.', syntax non-empty.
- No serde used for front matter yet (toml is used elsewhere for faber.toml manifests and coverage.toml).

### Current Test Surfaces
- `crates/faber/src/explain_test.rs`:
  - `frontmatter_value` helper (line ~267) hard-codes `---` and `key: value` parsing for filename assertions in coverage test.
  - `unknown_frontmatter_fields_fail` test embeds a `---` bad fixture.
  - All 9 explain tests pass on current corpus.

### Baseline Command Behavior (verified)
- `cargo run -p faber -- explain functio` : renders correctly.
- `cargo run -p faber -- explain --json functio` : JSON shape stable.
- `cargo run -p faber -- explain --list` : works.
- `cargo test -p faber explain_test` : 9/9 PASS.

### Docs Inventory (mentions of explain front matter)
- `docs/grammatica/explain.md`: shows `---` YAML example (lines 31-42).
- `docs/factory/faber-explain-command/plan.md`: repeatedly calls format "YAML-like frontmatter", shows `---` examples.
- `docs/factory/faber-explain-coverage-completion/plan.md`: shows YAML-style examples, refers to "frontmatter".
- `docs/factory/faber-explain-coverage-completion/inventory.md`: mentions "frontmatter".
- `docs/factory/faber-toml-frontmatter/plan.md` itself (this work).
- Many other `---` are Markdown tables, rules, or unrelated prose (e.g. grammatica/*.md table separators); not front matter instructions.
- No current user-facing prose outside factory/grammatica describes the internal parser as "YAML parser".

### YAML Runtime Surfaces (explicitly OUT OF SCOPE per plan §1.2)
- `crates/norma/Cargo.toml`: depends on `serde_yaml`.
- `crates/norma/yaml.rs` and `stdlib/norma/yaml.fab`: Norma YAML data-format support.
- `docs/factory/stdlib-data-formats/plan.md` tracks the broader question.
- `crates/faber` has **no** yaml/serde_yaml dependency today — confirmed clean for this change.

### Parser Dependency Baseline
- `crates/faber/Cargo.toml`: `toml = "1.1.2"` present and used for `FaberManifest` (package.rs) and `CoverageManifest` (explain_test.rs via toml::from_str).
- serde is already a dependency (used for Entry serialization).

### Phase 0 Checkpoint Status
- Ledger created at `docs/factory/faber-toml-frontmatter/ledger.md`.
- No behavior changes performed.
- YAML runtime module explicitly classified out of scope.
- Corpus count, parser locations, test helpers, and docs references recorded.
- All baseline commands and tests PASS.

## Phase 0 Artifacts
- This ledger.md (created during Phase 0).
- todo list initialized for phase tracking.
- No code changes in Phase 0 (inventory only).

## Notes for Next Phases
- Phase 1 should introduce TOML parser + `+++` requirement + rejection of `---` before touching corpus.
- Prefer deserializing directly to a struct with serde + toml for robustness (unknown fields via deny_unknown_fields? or manual post-validation to match current error style).
- Keep error messages similar or improve clarity.
- Commit only after each phase's checkpoint passes.

## Phase 1: TOML Parser Support (completed)

**Date**: 2026-05-21
**Changes**:
- Updated `crates/faber/src/explain.rs`:
  - Added `FrontMatter` struct with `#[derive(Deserialize)]` + `#[serde(deny_unknown_fields)]` (lines ~46-64).
  - Rewrote `parse_entry` (now ~459) to require first line exactly `+++`, collect inter-delimiter text, `toml::from_str::<FrontMatter>`, map directly to Entry.
  - Deleted obsolete `parse_frontmatter`, `FrontValue` enum, `parse_scalar`, `required_string`, `optional_string`, `optional_list`, `required_bool` (no longer referenced; early residue removal).
  - Error messages now come from toml/serde prefixed with "{filename}: ", e.g. "missing field `term`", "unknown field `surprise`...", "invalid type: integer `1`, expected a string".
- Updated `crates/faber/src/explain_test.rs`:
  - Converted `unknown_frontmatter_fields_fail` fixture to TOML `+++` syntax; relaxed assert to check for "surprise" + "unknown field".
  - Added 3 new explicit negative tests covering plan requirements:
    - `old_yaml_frontmatter_delimiters_fail`
    - `missing_or_unterminated_toml_frontmatter_fails`
    - `toml_frontmatter_type_errors_reported` (scalar-for-array + non-string array item)

**Verification**:
- `cargo check -p faber` : PASS
- Specific parser tests (4): all PASS (old --- rejected; TOML good paths and type/unknown/unterm/missing errors reported cleanly).
- Manual: loading any old corpus entry now fails fast at first file with clear "missing frontmatter (expected opening +++ ...)" — exactly the intended Phase 1 behavior.
- No YAML added to crates/faber; toml already present.

**Checkpoint**:
- TOML-frontmatter fixtures parse successfully.
- YAML-style `---` explain entries fail fast with clear error.
- No explain *behavior* changes for valid TOML entries (shape, validation, rendering identical).
- Old hand-rolled YAML parser removed in this phase (frontmatter logic is now TOML-only).
- Ready for corpus migration in Phase 2 (parser committed first per plan guidance).

**Artifacts**:
- Code edits in explain.rs + explain_test.rs
- This ledger updated
- (No doc changes yet)

## Current Phase
1 - TOML parser support (complete; committed)
2 - Corpus migration (next)
