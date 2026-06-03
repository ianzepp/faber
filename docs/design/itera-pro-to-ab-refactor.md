# Approved refactor: `itera pro` → `itera ab` (range iteration)

**Status:** Implemented
**Decision basis:** Pre-release source-surface refinement; no external codebase compatibility contract
**Last updated:** 2026-06-03

---

## Summary

Replace range-loop syntax:

```fab
itera pro 0‥10 fixum i { ... }
```

with

```fab
itera ab 0‥10 fixum i { ... }
```

Collection iteration stays unchanged:

```fab
itera ex items fixum item { ... }   # values
itera de tabula fixum key { ... }   # keys / indices
```

The numeric span (`0‥10`, `0 ante 5`, `0 usque 3`, `0…5`, optional `per 2` **inside** the range expression) is **not** part of this rename.

This is a hard cut. Faber is still pre-release, has no customer corpus, and should not carry a compatibility alias for this change.

---

## Motivation

### Why change at all?

1. **Latin fit:** `ab` (from / away from a starting point) matches interval iteration (“from 0 through the span”) better than treating range loops as a third peer of `ex`/`de` under a purpose word (`pro` = for).
2. **Clearer taxonomy:** `ex` / `de` are **collection projections** (values vs keys/indices). `ab` would mark **interval iteration** only, without overloading `de` (“from” on maps vs “from 0”).
3. **Keyword slot:** Pipeline collection DSL **`ab`** is retired ([`docs/factory/remove-ab-dsl/`](../factory/remove-ab-dsl/goal.md), [`explain/ab.legacy.md`](../../explain/ab.legacy.md)). `ab` is not in the active lexer today. Reuse is approved with an explicit break narrative.
4. **Avoid bad alternatives considered in discussion:**
   - **`itera in 0‥10`** — collides with mutable-borrow `in` on parameters/types and with English “for x in collection.”
   - **`itera de 0‥10`** — operand-driven overload on `de` (keys/indices on collections vs range on intervals).
   - **`itera ad 0‥10`** — wrong Latin role (toward/to) and clashes with live **`ad`** capability-call syntax.

### Why not keep `pro`?

`itera pro` is defensible Latin (“for each index in the span”) and is already shipped in exempla, MIR/WASM notes, and docs. Keeping it is the **zero-migration** default. This decision trades that internal migration cost for a source surface that looks and reads better before release.

---

## Current language contract (baseline)

### Grammar (`EBNF.md`)

```ebnf
iteraStmt := 'itera' (('ex' | 'de') expression | 'pro' expression ('per' expression)?)
           ('fixum' | 'varia') IDENTIFIER (blockStmt | ergoToken statement) catchClause?
```

Note: optional `per` on the `itera` line in EBNF is **not** implemented in the parser today; step is written inside the range expression (`0‥10 per 2`).

### Effective semantics today

| Form | Operand | Binding / behavior |
|------|---------|-------------------|
| `itera ex` | collection | element values |
| `itera de` | array/lista or map/tabula | indices for arrays/lists; keys for maps |
| `itera pro` | **range** (`Intervallum`) | numeric loop variable along span |

MIR lowering (`crates/radix/src/mir/lower/control.rs`) requires `HirIteraMode::Pro` sources to be `Intervallum`; other shapes diagnose *"itera pro source before range MIR lowering"*.

Typecheck has legacy branches for `Pro` on array/map types, but **codegen/MIR path is range-only** in practice.

### Where `pro` does **not** appear

| Surface | Uses `pro`? |
|---------|-------------|
| Range subexpressions (`0‥10`, `ante`, `usque`, `per`) | No |
| String slicing `"text"[0‥5]` | No |
| `ad` success bindings | **Rejected** — `→ pro name` errors; use `→ Type name` |
| Retired `ab` collection pipeline | Dead syntax (not lexer keyword) |

**Implication:** Renaming is **`itera pro` → `itera ab` only**, not a range-expression rewrite.

---

## Approved contract

### Syntax

```ebnf
iteraStmt := 'itera' (('ex' | 'de') expression | 'ab' expression)
           ('fixum' | 'varia') IDENTIFIER (blockStmt | ergoToken statement) catchClause?
```

`ab` is the source spelling for range iteration only. Prefer implementing it as a contextual `itera` mode rather than reserving `ab` as a global user identifier, unless the implementation deliberately chooses global reservation and updates identifier rules/tests accordingly.

Examples (same ranges as today):

```fab
itera ab 0‥5 fixum i { nota i }
itera ab 0 ante 5 fixum i { nota i }
itera ab 0…5 fixum i { nota i }
itera ab 0‥10 per 2 fixum i { nota i }
itera ab 10‥0 per -1 fixum i { nota i }
```

### Reader-facing rule

- **`itera ex` / `itera de`** — choose how to walk a **collection** (values vs key/index channel).
- **`itera ab`** — walk a **numeric interval** introduced by a range expression.
- **Not the retired pipeline:** `ab users …` filter DSL is gone; loop `ab` is interval-from only.

### Disambiguation vs `ad`

| Token | Role |
|-------|------|
| `itera ab …` | Interval iteration (always two-word phrase) |
| `ad "cap:verb" (…) { }` | Host capability call |

Operational risk: **`ab` / `ad` typos** in editors and LLM output. Mitigation: docs, `faber explain`, and diagnostics that mention the distinction when parsing fails near `ad`/`ab`.

---

## Implementation scope

### Compiler (`crates/radix`)

- **Lexer / keyword policy:** make `ab` available for `itera ab`. Keep it contextual if practical so ordinary `ab` identifiers remain valid outside the loop-mode slot. Do not revive retired collection-pipeline parsing.
- **Parser:** `parse_itera_stmt` accepts `ab` instead of `pro`; error text becomes *expected 'ex', 'de', or 'ab'*. Do **not** keep `itera pro` as a parse alias.
- **AST / HIR:** replace source-level `IteraMode::Pro`. Prefer semantic internal names such as `Range` or `Interval` over `Ab`; `Ab` is source syntax, not the compiler invariant.
- **Semantic / borrow / typecheck:** update mode matches and diagnostic strings. Tighten typecheck so range mode requires an `Intervallum`-shaped source instead of preserving legacy array/map `Pro` branches.
- **MIR:** range-iteration lowering follows the renamed semantic mode; unsupported-source diagnostics should say `itera ab`.
- **Codegen (Rust, Go, TS, Faber):** update iteration mode matches. Faber round-trip emission must print `itera ab`.
- **Tests:** update live parser, semantic, MIR, codegen, driver, and e2e source strings from `itera pro` to `itera ab`. Keep or update negative tests for rejected `ad ... → pro name`; that syntax remains invalid but is independent of range iteration.

### Faber CLI / explain

- Regenerate or hand-update embedded explain: retire [`explain/pro.md`](../../explain/pro.md) or replace with `explain/ab.md` (interval iteration).
- Keep [`explain/ab.legacy.md`](../../explain/ab.legacy.md) as legacy pipeline; cross-link “not loop `ab`.”
- Update [`explain/itera.md`](../../explain/itera.md), [`explain/per.md`](../../explain/per.md) examples.
- Ensure `faber explain ab` resolves to the new interval-iteration entry while still preserving a discoverable legacy note for the removed pipeline.

### Exempla and harnesses

All live `itera pro` exempla must be changed mechanically to `itera ab` so compilation and e2e harnesses continue to pass. Known live areas include `examples/exempla/itera/`, `examples/exempla/ante/`, `examples/exempla/per/`, `examples/exempla/usque/`, and syntax examples that compile as part of the active corpus.

E2E / MIR tests referencing `itera pro` in source strings.

Historical design, factory, release, and legacy website references may continue to mention `itera pro` when they are explicitly historical. Do not spend implementation effort rewriting old release notes or archived website content unless they are used as live generated docs.

### User-facing docs

- [`README.md`](../../README.md) iteration lines
- [`EBNF.md`](../../EBNF.md)
- [`AGENTS.md`](../../AGENTS.md) if it documents `itera pro`
- [`hosts/macos-arm64/*.md`](../../hosts/macos-arm64/) — fix stale `→ pro` on `ad` **independently** of this refactor (documentation drift today)

### Website

- Curated content / grammar pages if synced from repo sources. Ignore legacy imported website content unless it is republished as current documentation.

---

## Migration decision

| Strategy | Pros | Cons |
|----------|------|------|
| **Hard cut** | One canonical form; no compatibility residue | Requires internal exempla/tests update in same change |
| **Parse alias** (`pro` → `ab` with deprecation diagnostic) | Softer upgrade | Carries legacy keyword longer |
| **Dual accept, emit `ab`** | Round-trip pretty-print teaches new form | More parser/HIR complexity |

Decision: **hard cut**. There is no external compatibility requirement, and the repo should converge on the canonical spelling now. `itera pro` should fail after the implementation lands.

---

## Alternatives recorded (not chosen yet)

1. **Keep `itera pro`** — status quo; document ex/de vs pro as collection-pair vs interval-head.
2. **`itera de` on ranges** — overload `de` by operand type; fewer keywords, harder spec.
3. **`itera ab`** — approved decision.
4. **New word `inter` / `per` as loop head** — `per` already means step inside ranges.

---

## Resolved review questions

1. **Is reusing `ab` worth reviving confusion with retired pipeline `ab` and neighbor `ad`?** Yes, before release. The syntax is always the two-word phrase `itera ab`, and docs/explain must distinguish it from retired pipeline `ab`.
2. **Hard cut vs deprecation alias?** Hard cut. No external corpus exists.
3. **Should internal HIR stay `Pro` or rename to `Ab`?** Rename away from `Pro`, but prefer semantic `Range`/`Interval` naming over source-level `Ab`.
4. **Explain corpus:** one canonical `ab` interval entry plus preserved `ab.legacy` pipeline note is enough.
5. **Any downstream book/tooling outside this repo that hard-codes `itera pro`?** None known; historical in-repo references can remain historical.

---

## Implementation checklist

- [x] Add/enable contextual `ab` loop mode without reviving pipeline `ab`.
- [x] Replace parser acceptance of `itera pro` with `itera ab`.
- [x] Rename AST/HIR/MIR/codegen mode away from `Pro`; prefer `Range`/`Interval`.
- [x] Tighten typecheck so interval mode accepts only range expressions.
- [x] Update live `.fab` exempla and active test source strings.
- [x] Update `EBNF.md`, `README.md`, `explain/`, and current website-generated docs.
- [x] Preserve or update negative coverage for `ad ... → pro name` as an unrelated invalid syntax.
- [x] Run `./scripta/test` or a narrower justified test set plus affected e2e checks.

---

## References

- Grammar: [`EBNF.md`](../../EBNF.md) — `iteraStmt`, range (`‥`, `ante`, `usque`, `per`)
- Retired pipeline: [`docs/factory/remove-ab-dsl/goal.md`](../factory/remove-ab-dsl/goal.md)
- Exempla ranges: [`examples/exempla/itera/intervallum.fab`](../../examples/exempla/itera/intervallum.fab)
- MIR lowering: `crates/radix/src/mir/lower/control.rs` (`HirIteraMode::Range`)
- Related design note: [`tla-radix-notes.md`](tla-radix-notes.md) (unrelated mechanically; same `docs/design/` home)

---

## Decision log

| Date | Outcome |
|------|---------|
| 2026-06-03 | Document created; **no implementation** |
| 2026-06-03 | Approved for hard-cut implementation before release. |
| 2026-06-03 | Implemented contextual `itera ab`, semantic `Range` mode, live docs/examples updates, and hard-cut tests. |
