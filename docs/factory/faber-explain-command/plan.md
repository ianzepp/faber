# Faber Explain Command Factory Plan

**Status**: planned
**Created**: 2026-05-21
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/faber-explain-command/`
**Primary Owner**: `crates/faber` build/project tool
**Explicit Non-Owner**: `crates/radix` compiler front end
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber is moving toward glyph-first, Latin-keyword syntax. That makes the language expressive, but it also creates a discoverability problem for humans and future agent/LLM workflows. If an agent sees `≡`, `→`, `proba`, `custodi`, or `solum_in` and does not know what it means, it should be able to ask the installed Faber tool for a short, grounded explanation.

The desired command is a Faber-tool feature, not compiler behavior:

```bash
faber explain ≡
faber explain proba
faber explain →
faber explain --json custodi
```

The source material should be Markdown with TOML frontmatter:

```text
explain/
├── ≡.md
├── →.md
├── proba.md
├── probandum.md
└── adfirma.md
```

Those files should be compiled into the `faber` binary so the installed tool does not need runtime access to the repo's docs directory.

## Boundary

Faber owns:

- the `faber explain <term>` command,
- the `explain/` Markdown corpus,
- frontmatter parsing and lookup behavior,
- user-facing plain text output,
- optional machine-readable JSON output,
- redirects from legacy/non-canonical terms to canonical entries.

Radix should not own:

- language explanation copy,
- agent-facing docs lookup,
- command dispatch for `faber explain`,
- any runtime dependency on the explain corpus.

Radix remains the compiler implementation. `faber explain` is the shipped user/agent interface for language grammar and meaning, so entries must be self-contained and must not require users to have `EBNF.md` or the source tree.

## Authoring Inputs

Explain entries should be authored from current repo sources, but the shipped entry must stand alone:

- `docs/grammatica/*.md` for user-facing language docs.
- `docs/grammatica/verba.md` for keyword inventory.
- `docs/grammatica/operatores.md` for operators.
- `examples/exempla/**` for runnable examples.
- `crates/radix/src/lexer/token.rs` and parser code only as verification inputs, not as prose source.

The `explain/` corpus becomes the public grammar interface compiled into the Faber build tool. Entries should not point end users back to source-tree-only grammar files for required understanding. If existing docs and compiler behavior disagree during authoring, the implementation session should record the disagreement and make the explain entry explicit.

## Corpus Format

Each entry is a Markdown file with TOML frontmatter (delimited by `+++`) followed by a short Markdown body.

Example operator entry:

```markdown
+++
term = "≡"
kind = "operator"
category = "comparison"
canonical = true
summary = "Compares two values for equality and returns bivalens."
syntax = "<expression> ≡ <expression>"
examples = ["examples/exempla/proba/proba.fab"]
aliases = ["equals", "equality"]
legacy = ["=="]
related = ["≠", "est", "non est"]
+++

Use `≡` when an expression should be true only when both sides compare equal.

```fab
adfirma 1 + 1 ≡ 2
```
```

Example keyword entry:

```markdown
+++
term = "proba"
kind = "keyword"
category = "testing"
canonical = true
summary = "Defines a single test case."
syntax = "proba <name> <block>"
examples = ["examples/exempla/proba/proba.fab"]
aliases = ["test"]
related = ["probandum", "adfirma", "omitte", "futurum"]
+++

`proba` introduces one test case. The body is normal Faber code and usually contains one or more `adfirma` assertions.

```fab
proba "arithmetic passes" {
    adfirma 1 + 1 ≡ 2
}
```
```

Example legacy redirect entry:

```markdown
+++
term = "=="
kind = "legacy"
category = "comparison"
canonical = false
canonical_term = "≡"
summary = "Legacy equality spelling. Use ≡ in Faber source."
syntax = "<expression> ≡ <expression>"
related = ["≡"]
+++

`==` is not canonical Faber syntax. Use `≡`.

```fab
adfirma left ≡ right
```
```

## Frontmatter Schema

Initial fields:

| Field | Required | Type | Meaning |
| ----- | -------- | ---- | ------- |
| `term` | yes | string | Exact lookup term displayed by the CLI. |
| `kind` | yes | string enum | `keyword`, `operator`, `annotation`, `type`, `modifier`, `legacy`, or `concept`. |
| `category` | yes | string | Broad grouping such as `testing`, `comparison`, `function`, `control-flow`. |
| `canonical` | yes | bool | Whether this term is valid canonical Faber source. |
| `summary` | yes | string | One-sentence definition. |
| `syntax` | yes | string | Self-contained usage pattern for this term. |
| `examples` | no | string list | Repo example files that demonstrate this term. |
| `aliases` | no | string list | Lookup synonyms such as `equals`, `test`, or `return arrow`. |
| `legacy` | no | string list | Deprecated spellings that should redirect here. |
| `canonical_term` | no | string | Required for `canonical = false` legacy entries. |
| `related` | no | string list | Other terms to suggest. |

The schema should be validated by tests. Unknown frontmatter fields should fail validation unless a phase deliberately expands the schema.

## Command Shape

Primary command:

```bash
faber explain <term>
```

Options:

```bash
faber explain --json <term>
faber explain --list
faber explain --category testing
```

Phase 1 may implement only:

```bash
faber explain <term>
faber explain --json <term>
```

Plain output should be short:

```text
≡
Kind: operator
Category: comparison
Meaning: Compares two values for equality and returns bivalens.

Example:
adfirma 1 + 1 ≡ 2

Syntax: <expression> ≡ <expression>
Examples: examples/exempla/proba/proba.fab
Related: ≠, est, non est
```

Legacy output should correct the user:

```text
==
Legacy: not canonical Faber source.
Use: ≡

Example:
adfirma left ≡ right
```

JSON output should be stable enough for tooling:

```json
{
  "term": "≡",
  "kind": "operator",
  "category": "comparison",
  "canonical": true,
  "summary": "Compares two values for equality and returns bivalens.",
  "syntax": "<expression> ≡ <expression>",
  "examples": ["examples/exempla/proba/proba.fab"],
  "aliases": ["equals", "equality"],
  "legacy": ["=="],
  "related": ["≠", "est", "non est"],
  "body": "Use `≡` when..."
}
```

## Embedding Strategy

Preferred implementation:

- Put source files at repo root under `explain/`.
- Add a Faber crate build script or compile-time include strategy.
- Compile the corpus into `crates/faber`.
- Keep runtime lookup independent of the working directory.

Two acceptable approaches:

1. `build.rs` scans `../../explain` and generates a Rust source file under `OUT_DIR`.
2. Use a compile-time embedding crate if already acceptable for project dependencies.

Prefer `build.rs` plus standard library code unless the implementation becomes unnecessarily complex. This keeps the dependency footprint small.

The generated Rust should contain static strings and/or a static table. It should not parse the filesystem at runtime.

## Initial Corpus Scope

The first implementation should avoid trying to document the whole language. Seed a small, high-value set:

Operators and glyphs:

- `≡`
- `≠`
- `≤`
- `≥`
- `→`
- `←`
- `⊕`
- `⊖`
- `⊛`
- `⊘`

Testing:

- `proba`
- `probandum`
- `adfirma`
- `omitte`
- `futurum`

Control flow:

- `si`
- `sin`
- `secus`
- `custodi`
- `reddit`

Functions and packages:

- `functio`
- `incipit`
- `importa`
- `fixum`
- `varia`

Legacy redirect entries:

- `==` -> `≡`
- `!=` -> `≠`
- `<=` -> `≤`
- `>=` -> `≥`
- `->` -> `→`

Each entry must have:

- frontmatter,
- one short body paragraph,
- one canonical Faber example,
- a self-contained syntax pattern,
- at least one related term where obvious.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline and corpus scaffold | Create corpus directory, define schema, select initial entries from grammar/docs/examples. | No command yet; entries are valid Markdown/frontmatter. |
| 1 | Compile-time embedding | Embed `explain/` entries into `crates/faber` without runtime filesystem dependency. | Unit test proves entries are available from the binary crate. |
| 2 | Lookup registry | Parse frontmatter/body and resolve term, alias, and legacy redirects. | Lookup tests pass for canonical, alias, and legacy queries. |
| 3 | CLI command | Add `faber explain <term>` and `--json`. | Command works for `≡`, `proba`, and `==`. |
| 4 | Corpus expansion and validation | Add initial corpus entries and schema validation tests. | Every seed entry validates and has self-contained syntax and examples where available. |
| 5 | Docs and guardrails | Document the command and add drift checks. | README/docs mention explain; validation catches bad entries. |

## Phase Details

### Phase 0: Baseline and Corpus Scaffold

Steps:

- Inspect `git status --short`.
- Create root `explain/`.
- Define frontmatter schema in the plan or implementation docs.
- Seed a few hand-written entries:
  - `≡.md`,
  - `→.md`,
  - `proba.md`.
- Verify each entry against:
  - current grammar behavior,
  - one `docs/grammatica/*.md` source when available,
  - one `examples/exempla/**/*.fab` source when available.

Checkpoint:

- Corpus files exist.
- No Faber binary behavior changes yet.

### Phase 1: Compile-Time Embedding

Steps:

- Add `crates/faber/build.rs` or another compile-time include mechanism.
- If using `build.rs`, scan root `explain/*.md` deterministically.
- Generate a Rust table containing:
  - source filename,
  - raw Markdown source.
- Ensure Cargo rebuilds when `explain/*.md` changes.
- Keep generated code out of the repository.

Checkpoint:

- `cargo test -p faber` can assert that embedded raw entries include `≡`, `→`, and `proba`.
- Running the binary from another directory still has access to the corpus.

### Phase 2: Lookup Registry

Steps:

- Add a small frontmatter parser in `crates/faber`.
- Reuse existing dependencies if already available; avoid adding a large docs framework.
- Parse body Markdown separately from frontmatter.
- Build lookup indexes for:
  - `term`,
  - `aliases`,
  - `legacy`,
  - `canonical_term`.
- Validate required fields and unknown fields.

Checkpoint:

- Lookup by `≡` returns the equality entry.
- Lookup by `equals` returns or redirects to `≡`.
- Lookup by `==` returns a legacy correction pointing to `≡`.
- Invalid entries fail tests.

### Phase 3: CLI Command

Steps:

- Add `Explain(ExplainArgs)` to `crates/faber/src/main.rs`.
- Implement:

```bash
faber explain <term>
faber explain --json <term>
```

- Keep output concise and deterministic.
- Print a helpful error for unknown terms:

```text
error: no explanation found for 'foo'
hint: run `faber explain --list`
```

The `--list` hint may exist before the command is implemented, but the better phase boundary is to implement `--list` in Phase 5.

Checkpoint:

- `cargo run -p faber -- explain ≡` works.
- `cargo run -p faber -- explain proba` works.
- `cargo run -p faber -- explain ==` returns a canonical correction.
- `cargo run -p faber -- explain --json ≡` emits valid JSON.

### Phase 4: Corpus Expansion and Validation

Steps:

- Add all initial corpus entries listed above.
- Add tests that every entry:
  - parses,
  - has required frontmatter fields,
  - has a non-empty body,
  - has a self-contained syntax pattern,
  - uses canonical Faber syntax in code examples,
  - has valid related references or explicitly allowed missing related terms.
- Add at least one snapshot-like assertion for text output and JSON output.

Checkpoint:

- All seed entries are available through the command.
- Legacy redirects exist for old glyph-clean-break forms.

### Phase 5: Docs and Guardrails

Steps:

- Update `README.md`.
- Add or update `docs/grammatica/` docs to mention `faber explain`.
- Document the corpus format for maintainers.
- Add `faber explain --list`.
- Add `faber explain --category <name>` if low-risk.
- Consider a hygiene test that fails if `explain/*.md` has invalid frontmatter or broken related references.

Checkpoint:

- Users can discover available terms.
- Maintainers know how to add entries.
- Final validation passes.

## Epic Candidates And Scopable Issues

### Issue A: Corpus Format and Seed Entries

Create `explain/` and seed enough entries to prove the model.

Acceptance:

- entries are Markdown with frontmatter,
- entries contain self-contained syntax patterns,
- examples use documented Faber syntax,
- entries are reviewable as normal docs.

### Issue B: Compile-Time Embedding

Compile the corpus into `crates/faber`.

Acceptance:

- no runtime filesystem dependency,
- changes to `explain/*.md` trigger rebuilds,
- embedded raw text is testable.

### Issue C: Registry and Lookup

Parse and index entries.

Acceptance:

- exact lookup,
- alias lookup,
- legacy correction lookup,
- unknown term error.

### Issue D: CLI Output

Expose the command.

Acceptance:

- plain text output is short,
- JSON output is valid and stable,
- command belongs only to `faber`.

### Issue E: Docs and Expansion

Make it maintainable.

Acceptance:

- README/docs mention the feature,
- maintainer instructions exist,
- initial corpus covers high-value glyphs and keywords.

## Checkpoints

Commit after each completed phase:

- corpus scaffold,
- embedding,
- registry,
- CLI command,
- corpus expansion,
- docs/validation.

Do not block Phase 1 on documenting the full language. The command is useful as soon as a small seed set works.

## Companion Skill Plan

- Use `delivery` for implementation planning and phase specs.
- Use `zombie-docs` after corpus expansion to catch entries that contradict current grammar behavior or `docs/grammatica`.
- Use `clean-break` when adding legacy entries so they clearly point away from removed old syntax instead of re-legitimizing it.
- Use `poker-face` after Phase 3 to prevent overclaiming coverage.

## Gate Plan

Required final validation:

```bash
cargo fmt --all -- --check
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release -p faber
cargo build --release -p radix
```

Required command smoke:

```bash
cargo run -p faber -- explain ≡
cargo run -p faber -- explain →
cargo run -p faber -- explain proba
cargo run -p faber -- explain ==
cargo run -p faber -- explain --json ≡
```

Required architectural proof:

- `crates/radix` does not depend on the explain corpus.
- `faber explain` works outside the repo root.
- `explain/*.md` files are embedded at build time.

## Open Questions

- Should the corpus live at repo root `explain/` or under `docs/explain/`? This plan recommends root `explain/` because it is product data, not only docs.
- Should `--list` and `--category` ship in Phase 3 or wait until Phase 5?
- Should Markdown body be rendered as plain text, lightly stripped, or printed as Markdown?
- Should JSON include the raw Markdown body or a stripped text body?
- Should entries be localized later, or is English-only correct for now?
