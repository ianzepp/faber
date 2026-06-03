# Proposed refactor: `itera pro` → `itera ab` (range iteration)

**Status:** Proposed — not approved for implementation  
**Author intent:** Capture design reasoning for review and a second opinion before any grammar or compiler work  
**Last updated:** 2026-06-03

---

## Summary

Replace range-loop syntax

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

---

## Motivation

### Why change at all?

1. **Latin fit:** `ab` (from / away from a starting point) matches interval iteration (“from 0 through the span”) better than treating range loops as a third peer of `ex`/`de` under a purpose word (`pro` = for).
2. **Clearer taxonomy:** `ex` / `de` are **collection projections** (values vs keys/indices). `ab` would mark **interval iteration** only, without overloading `de` (“from” on maps vs “from 0”).
3. **Keyword slot:** Pipeline collection DSL **`ab`** is retired ([`docs/factory/remove-ab-dsl/`](../factory/remove-ab-dsl/goal.md), [`explain/ab.legacy.md`](../../explain/ab.legacy.md)). `ab` is not in the active lexer. Reuse is possible with an explicit break narrative.
4. **Avoid bad alternatives considered in discussion:**
   - **`itera in 0‥10`** — collides with mutable-borrow `in` on parameters/types and with English “for x in collection.”
   - **`itera de 0‥10`** — operand-driven overload on `de` (keys/indices on collections vs range on intervals).
   - **`itera ad 0‥10`** — wrong Latin role (toward/to) and clashes with live **`ad`** capability-call syntax.

### Why not keep `pro`?

`itera pro` is defensible Latin (“for each index in the span”) and is already shipped in exempla, MIR/WASM notes, and docs. Keeping it is the **zero-migration** default. This proposal trades stability for a clearer interval marker.

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
| `itera de` | array/lista | indices; on map/tabula | keys |
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

## Proposed contract

### Syntax

```ebnf
iteraStmt := 'itera' (('ex' | 'de') expression | 'ab' expression)
           ('fixum' | 'varia') IDENTIFIER (blockStmt | ergoToken statement) catchClause?
```

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

## Implementation blast radius (if approved)

### Compiler (`crates/radix`)

- **Lexer:** add `ab` keyword → `TokenKind::Ab` (or reuse name); remove or alias `pro` on `itera` path.
- **Parser:** `parse_itera_stmt` — `ab` instead of `pro`; error text *expected 'ex', 'de', or 'ab'*.
- **HIR:** `HirIteraMode::Pro` → `Ab` (or keep internal `Pro` as alias during transition).
- **Semantic / borrow / typecheck:** update mode matches and diagnostic strings.
- **MIR:** `lower_itera` Pro branch → Ab; error messages.
- **Codegen (Rust, Go, TS, Faber):** iteration emission labels / comments only if they print source modes.
- **Tests:** broad string replace in `itera pro` fixtures; negative tests for retired `→ pro` on `ad` unchanged.

### Faber CLI / explain

- Regenerate or hand-update embedded explain: retire [`explain/pro.md`](../../explain/pro.md) or replace with `explain/ab.md` (interval iteration).
- Keep [`explain/ab.legacy.md`](../../explain/ab.legacy.md) as legacy pipeline; cross-link “not loop `ab`.”
- Update [`explain/itera.md`](../../explain/itera.md), [`explain/per.md`](../../explain/per.md) examples.

### Exempla and harnesses

All current `itera pro` exempla (under `examples/exempla/itera/`, `ante/`, `per/`, `usque/`, etc.) — mechanical update.

E2E / MIR tests referencing `itera pro` in source strings.

Wasm factory docs: e.g. `phase-015-range-itera-pro-wasm.md` naming (rename or add forward pointer).

### User-facing docs

- [`README.md`](../../README.md) iteration lines
- [`EBNF.md`](../../EBNF.md)
- [`AGENTS.md`](../../AGENTS.md) if it documents `itera pro`
- [`hosts/macos-arm64/*.md`](../../hosts/macos-arm64/) — fix stale `→ pro` on `ad` **independently** of this refactor (documentation drift today)

### Website

- Curated content / grammar pages if synced from repo sources.

---

## Migration strategies

| Strategy | Pros | Cons |
|----------|------|------|
| **Hard cut** | One canonical form | Breaks existing `.fab` until updated |
| **Parse alias** (`pro` → `ab` with deprecation diagnostic) | Softer upgrade | Carries legacy keyword longer |
| **Dual accept, emit `ab`** | Round-trip pretty-print teaches new form | More parser/HIR complexity |

Recommendation if the refactor lands: **one release with parse alias + deprecation warning**, then remove `pro` on `itera` in the following release. Internal enums can rename immediately.

---

## Alternatives recorded (not chosen yet)

1. **Keep `itera pro`** — status quo; document ex/de vs pro as collection-pair vs interval-head.
2. **`itera de` on ranges** — overload `de` by operand type; fewer keywords, harder spec.
3. **`itera ab`** — this proposal.
4. **New word `inter` / `per` as loop head** — `per` already means step inside ranges.

---

## Open questions for reviewers

1. **Is reusing `ab` worth reviving confusion with retired pipeline `ab` and neighbor `ad`?**
2. **Hard cut vs deprecation alias** — what is the expected external corpus size (user packages, archive repo)?
3. **Should internal HIR stay `Pro` (comment: interval) or rename to `Ab` for maintainers?**
4. **Explain corpus:** one canonical `ab` term vs `ab` (interval) + `ab.legacy` (pipeline) — enough for LLMs?
5. **Any downstream book/tooling** outside this repo that hard-codes `itera pro`?

---

## Suggested review checklist

- [ ] Latin / pedagogy: interval head = `ab` vs `pro` vs overloaded `de`
- [ ] Operational: `ab`/`ad` typo cost acceptable?
- [ ] Compiler: MIR range path only — confirm no hidden `itera pro collection` reliance
- [ ] Docs: plan for stale `ad → pro` host examples
- [ ] Migration: alias period length and policy
- [ ] Second opinion sign-off before lexer/token churn

---

## References

- Grammar: [`EBNF.md`](../../EBNF.md) — `iteraStmt`, range (`‥`, `ante`, `usque`, `per`)
- Retired pipeline: [`docs/factory/remove-ab-dsl/goal.md`](../factory/remove-ab-dsl/goal.md)
- Exempla ranges: [`examples/exempla/itera/intervallum.fab`](../../examples/exempla/itera/intervallum.fab)
- MIR lowering: `crates/radix/src/mir/lower/control.rs` (`HirIteraMode::Pro`)
- Related design note: [`tla-radix-notes.md`](tla-radix-notes.md) (unrelated mechanically; same `docs/design/` home)

---

## Decision log

| Date | Outcome |
|------|---------|
| 2026-06-03 | Document created; **no implementation** |

When decided, append here: **Approved** / **Rejected** / **Deferred**, with reviewer and follow-up issue or PR link.