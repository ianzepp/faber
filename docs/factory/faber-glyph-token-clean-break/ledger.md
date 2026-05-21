# Faber Glyph Token Clean-Break - Factory Ledger

**Phase Set Source**: `docs/factory/faber-glyph-token-clean-break/plan.md`
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Delivery Spec Directory**: `docs/factory/faber-glyph-token-clean-break/`
**Commit Policy**: Commit after each completed phase and validation gate pass
**Created**: 2026-05-21
**Mode**: clean-break / full edit (per plan)

## Current Phase
(all phases complete — clean break delivered)

## Completed Phases
0 - Inventory and baseline (ledger + full classified inventory of old ASCII compounds)
1 - Front-end break (lexer removal of == != <= >= -> += -= *= /= === !== in scan.rs)
2 - Compiler tests migration (driver/mod_test, parser/mod_test, lexer/scan_test + explicit rejection tests)
3 - Examples migration (automation/commands/*.fab updated to canonical glyphs)
4 - Grammar and explain docs migration (all grammatica/*.md + 15+ canonical explain/*.md updated; legacy explain entries preserved)
5 - Guardrails and validation (rejection tests, full ci gates, residue search clean)

## Final Validation
- Lexer no longer emits compound tokens for old ASCII forms.
- Negative tests (rejects_old_ascii_compound_operators + parse checks) pass and prove rejection.
- examples/**/*.fab clean.
- `cargo fmt --all -- --check`, `cargo test --all`, `cargo clippy -D warnings`, release builds all PASS.
- Residue search shows only intentional legacy explain entries, target-language tables, and historical prose.
- Semantic proofs hold (≡ / → accepted; old forms error at lex or parse).
- Bookkeeping closed (plan status + this ledger updated).

## Baseline Captures (Phase 0)

### Git Status (start of session)
```
On branch main
Your branch is ahead of 'origin/main' by 14 commits.

nothing to commit, working tree clean
```

### Explain Coverage Prerequisite Verification
- `cargo test -p faber coverage_manifest_matches_registry` : PASS
- `cargo run -p faber -- explain '=='` : Returns legacy entry correctly worded:
  - "STATUS: Legacy. Not canonical Faber source."
  - "USE INSTEAD: ≡"
  - Example uses `≡`
- Similar for `->`, `!=`, `<=`, `>=`, `===`, `!==`
- Legacy filenames: `eq-eq.legacy.md`, `arrow.legacy.md`, `bang-eq.legacy.md`, `lt-eq.legacy.md`, `gt-eq.legacy.md`
- Coverage manifest (explain/coverage.toml) lists exactly the 7 legacy_terms; no compound assign legacy entries yet (they use glyphs in explain directly via plus-eq.md etc.)

### Canonical Glyph vs Old ASCII Acceptance (Pre-Break)
Confirmed in `crates/radix/src/lexer/scan.rs:172` (scan_operator):
- Both forms produce identical TokenKind:
  - `==` / `≡` → EqEq
  - `!=` / `≠` → BangEq
  - `<=` / `≤` → LtEq
  - `>=` / `≥` → GtEq
  - `->` / `→` → Arrow
  - `+=` / `⊕` → PlusEq
  - `-=` / `⊖` → MinusEq
  - `*=` / `⊛` → StarEq
  - `/=` / `⊘` → SlashEq
- Also accepts `===`/`!==` (EqEqEq / BangEqEq) — no glyph equivalent kept; these are legacy strict forms to remove.
- Glyph-only forms already present and will be preserved: `≡ ≠ ≤ ≥ → ⊕ ⊖ ⊛ ⊘`
- Single-char operators (`=`, `<`, `>`, `+`, `-`, `*`, `/`, `!` for non-null/paren forms) remain; only the multi-char ASCII compounds for the above are excised.

`cargo test -p radix lexes_operator_tokens_consistently` : PASS (pre-break, accepts mixed).

### Inventory of Old-Token Occurrences (Classified)

#### 1. `.fab` Examples / Fixtures (MUST CHANGE - Faber source)
- `examples/automation/commands/runner.fab:6` : `functio dryRun() argumenta args -> vacuum {`
- `examples/automation/commands/inventory.fab:5,16,27` : three `functio ... -> vacuum {`
  - Total old-arrow occurrences: 4
- No `== != <= >= += etc` found in any `examples/**/*.fab` (some canonical glyphs already present in exempla/).
- Classification: **Faber source to change** in Phase 3.
- Note: `examples/automation/` appears to be package-style fixtures from prior test-runner evolution; treat as first-class examples.

#### 2. Compiler Tests - Embedded Faber Source Snippets (MUST CHANGE)
Primary file: `crates/radix/src/driver/mod_test.rs`
- ~30+ instances of `->` in `r#" ... functio ... -> <type>` and bare function decls inside test sources (e.g. lines 261,268,337,342,423,456,478,683,701,740,853,974,1061,1118,1156,1165,1398 context,1449,1471,1547,1569,1577,1603,1656,1689,1715,1790,1809,1904,1955,1962,1966,2127).
- Old comparisons inside fab sources:
  - 1398: `adfirma 1 + 1 == 2`
  - 1449: `adfirma 1 == 1`
  - 1548: `si index < 0 aut index >= items.longitudo() {`
- Other files:
  - `crates/radix/src/lexer/scan_test.rs:52` : massive mixed operator test string containing `-> → == ≡ === != ≠ !== <= ≤ >= ≥ += ...` (tests both paths).
  - `crates/radix/src/parser/mod_test.rs:517,601,606,813` : a few `->` in parse test snippets.
- Generated output assertions (Rust/TS/Go codegen strings containing `fn foo() -> ...`, `<=` in TS etc.): **LEAVE ALONE** per plan (target-language).
- Rust implementation `==` in test code, CLI strings, etc.: **LEAVE ALONE**.
- Classification: **Faber source to change** in Phase 2. Add explicit rejection tests for old forms post-break.

#### 3. Lexer / Parser / Compiler Source (MUST CHANGE - logic + comments)
- `crates/radix/src/lexer/scan.rs:191-268` (scan_operator): the `+ =` , `- >`/`=` , `/ =` , `= =`/`=` , `! =`/`=` , `< =` , `> =` branches for ASCII compounds. Glyph branches (183-186,271-275) stay.
- `crates/radix/src/parser/expr.rs`:
  - Comments: "Equality (==, !=, est, non est)", "Comparison (<, >, <=, >=, intra, inter)"
  - Grammar note line ~711: `['->' type]`
- `crates/radix/src/parser/decl.rs`, `types.rs`, `stmt.rs` (to be audited in Phase 1): likely similar comment/grammar references.
- `crates/radix/src/parser/mod_test.rs` and driver tests as above.
- Codegen (e.g. `crates/radix/src/codegen/faber/ops.rs`): already emits canonical glyphs for roundtrips — no change needed.
- Classification: **Remove old acceptance** (Phase 1). Update comments that document removed source forms.

#### 4. Grammar Docs (MUST CHANGE - teach canonical)
- `docs/grammatica/operatores.md`: heavy use of `<= >= == === != !== && || ? :` and `->` in code blocks + prose. Explicitly teaches old forms as valid.
- `docs/grammatica/fundamenta.md`, `typi.md`, `targets.md`, `cli.md`: many `->` in function signature examples.
- `docs/grammatica/verba.md`: tables mapping `et` to `&&`, `est` to `===`, null checks to `==`/`!=` (these are target comparisons; clarify vs. source).
- `docs/grammatica/regimen.md`, `importa.md`, `errores.md` (minor).
- `EBNF.md:523`: one `>=` example (comparison filter).
- Classification: **Rewrite Faber source examples to glyphs** (Phase 4). Leave target-lang comparisons labeled as such. Add historical note only if plan allows (prefer explain legacy).

#### 5. Explain / Legacy Redirects (PRESERVE + MINOR ALIGN IF NEEDED)
- All `explain/*.legacy.md` (eq-eq, bang-eq, lt-eq, gt-eq, arrow, est, non-est, plus-eq? etc.): already correctly state "not canonical", "use <glyph>", provide glyph example. **DO NOT DELETE**.
- `explain/*.md` (canonical): use glyphs, good.
- `explain/coverage.toml`: legacy_terms list is the contract; may need no edit unless we add more (e.g. compound assigns not currently listed as legacy).
- Classification: **Intentional legacy to preserve** (Phase 4 review only for wording if break changes semantics).

#### 6. Factory Plans & Historical Docs (CLASSIFY PER CONTEXT)
- `docs/factory/faber-glyph-token-clean-break/plan.md` (this): contains mapping tables — **historical, leave**.
- `docs/factory/faber-explain-coverage-completion/{plan.md,inventory.md}`: documents the `==`->`≡` etc mappings and file renames — **historical reference, leave** (prose comparison).
- Other factory/*.md, docs/epics/*, docs/release/* : many `->` in Rust sigs, shell, or old snapshots. 
  - Rust fn sigs / cargo output: leave.
  - Embedded old Faber snippets that look like "valid code": review case-by-case in Phase 4; prefer minimal change or label.
- `AGENTS.md`: contains syntax examples with `->` (wrong per current canonical) and correct type-first; update the wrong ones? (AGENTS is project rule — truth requires sync).
- `README.md`, `DEVELOPER.md`, `ANALYSIS.md`, `TLA.md`: audit for source examples.
- Classification: **Prose/historical comparisons leave**; update any that present removed forms as current valid Faber source.

#### 7. Stdlib (norma/*.fab) — OUT OF SCOPE FOR THIS CLEAN-BREAK (per explicit plan boundary)
- ~1000+ `->` in `functio ... -> <type>` decls + `@ verte ... (x) -> "..."` mappings across 20+ files.
- Also some `===` `==` inside @ verte translation strings (target JS/TS/Python etc.).
- **Decision**: Do not edit in this pass. 
  - Rationale: Plan boundary lists only "examples and fixtures"; stdlib is declarative contract surface synced with `crates/norma/` Rust impl. No evidence that `radix` or `faber` lex/parse these .fab files during normal `cargo test/build` (stdlib contracts are Rust-encoded).
  - If a future `faber check stdlib/...` or doc-gen step is added, it will surface then.
  - @ verte mappings: the `->` is part of the annotation mapping syntax (left=param mapping, right=template), but treated as metadata not user program source for this break.
- Re-evaluate only if validation in Phase 5 shows breakage or user expands scope.

#### 8. Other (Out of Scope)
- All Rust source `==`, `!=`, `->` (fn returns, matches, etc.): **leave** (implementation language).
- Shell/CLI examples (`--release`, `cargo run -p ...`): leave.
- Generated code assertions in tests (`.rs`/`.ts`/`.go` strings): leave.
- Target template strings in @ verte or codegen: leave.
- `node_modules/`, `target/`, `dist/`: ignore.

### Old Token Set to Remove in Phase 1
Per plan "Canonical Source Token Set":
Primary:
- `==` (use `≡`)
- `!=` (use `≠`)
- `<=` (use `≤`)
- `>=` (use `≥`)
- `->` (use `→`)
- `+=` (use `⊕`)
- `-=` (use `⊖`)
- `*=` (use `⊛`)
- `/=` (use `⊘`)

Also remove:
- `===` , `!==` (no single glyph; users should have migrated to `est` / `non est` or `≡` / `≠`)

Open questions from plan (to resolve or defer during impl):
- `&&` / `||` : already keyword-preferred (`et`/`aut`); confirm no lexer path remains.
- `!` logical: keep for `!.` `![` `!(` non-null; do not remove `!` as logical-not without separate design.
- `? :` ternary: deferred (canonical is `sic ... secus`, multi-word).
- `<` `>` simple: stay (single-char math; not compounds).
- No arithmetic `+ - * / %` touched.

### Pre-Break Validation State
- All current `cargo test -p radix` (lexer/parser/driver) pass with dual acceptance.
- Example checks would pass (mixed usage).
- `faber explain` legacy surface already complete and accurate.
- No behavior changed in Phase 0.

## Phase 0 Checkpoint
- Inventory complete and classified.
- Explain baseline confirmed landed and correct.
- Old token set for removal named.
- No edits to compiler behavior yet.
- Ledger written.

**Next**: Mark Phase 0 complete, commit, then Phase 1 lexer excision (smallest auditable change).

## Notes / Decisions
- Using clean-break skill stance throughout: excise the compatibility branches; one canonical source form.
- Will use poker-face after Phase 2 and 5; zombie-docs after Phase 4.
- All edits manual / targeted (no broad regex on Rust source).
- After lexer break, tests will fail exactly on stale fab snippets — that's the signal to migrate (plan intent).
- Commit after ledger (this file) + any minor baseline captures.
