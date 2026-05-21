# Norma Standard Library Verb Naming Review & Migration-Safe Conjugation Plan

**Status**: Draft  
**Date**: 2026-05-21  
**Context**: Pre-Rust implementation review of `stdlib/norma/*.fab` interfaces  
**Goal**: Ensure current stdlib method names are already the intended future user-facing conjugated call forms, so later root-form declarations can land without forcing user code renames.

---

## Context and Objectives

The long-term vision for Faber’s standard library is that **verb conjugation itself** should communicate execution semantics. Today, stdlib interfaces spell full method names and use explicit annotations such as `@ futura` and `@ cursor`. Later, those annotations should become unnecessary: a stdlib declaration may define the root operation, and the compiler can derive the requested wrapper behavior from the call-site conjugation.

That means the names in `stdlib/norma/*.fab` are not disposable temporary names. They are intended to be the stable call-site surface:

```fab
# Today: full annotated declarations may exist.
fixum rows = cede conn.quaeret(sql, params)

# Later: the declaration may collapse to a root-backed operation, but user code
# should still call the same conjugated form.
fixum rows = cede conn.quaeret(sql, params)
```

Possible wrapper dimensions include:

- Imperative / base forms (`-a`, `-e`, `-i`): synchronous
- Future indicative forms (`-et`, `-ebit`, `-abit`): asynchronous (`promissum<T>`)
- Future plural forms: async generators (`cursor<T>`)
- Present plural forms: synchronous generators (`cursor<T>`)

Resolved policy: generators use the real present plural form of the declared root; async generators use the real future plural form.

Compiler caveat: the active `radix-rs` compiler can parse these stdlib interface files, but future root-form declaration lowering and morphology-based wrapper synthesis are not implemented yet. This review is about freezing the intended user-facing call forms before the runtime/API surface hardens.

This document reviews the current interfaces and proposes targeted renames **before** significant Rust implementation work begins in `crates/norma/`.

### Principles

1. **Conjugation should be predictable** — once a root is known, the async/generator form should be obvious.
2. **Call-site names are migration contracts** — current full method names should remain valid when future stdlib declarations collapse to roots.
3. **Keep one root for one operation** — do not split a root merely because two conjugated forms look similar.
4. **Favor distinct roots for distinct operations** — use a new root when the semantic difference is lifecycle, ownership, cardinality, or side effects, not merely wrapper behavior.
5. **Consistency across similar operations** — read/write, query/mutate, publish/subscribe should follow the same root and conjugation policy.
6. **Minimize churn for future users** — changes now are cheap; changes after Rust crates ship are expensive.

---

## Review Methodology

- Reviewed all `pactum` declarations in `stdlib/norma/`.
- Focused on files with `@ futura` and/or `@ cursor` annotations.
- Evaluated each method against:
  - Documented conjugation intent in the file’s own comments.
  - Consistency with peer methods in the same module.
  - Clarity of the root + ending pattern.
  - Suitability for a future root-based + conjugation model.
  - Whether the current name can remain stable after annotation removal.

---

## Current State Assessment

### Strengths

- Many I/O modules (`solum`, `consolum`, `processus`) already follow a clear pattern:
  - Base/imperative → sync
  - `-et` / `-ebit` future → async
- Some modules deliberately chose distinct roots for different semantics (`genera` vs `dimitte`, response builders `replica`/`scribe`/`funde`).
- The documentation in most HAL files is honest about the intended conjugation model.

### Weaknesses / Open Policy

- Generator spelling is now resolved as real present plural for sync generators and real future plural for async generators. Companion docs must stay aligned with that policy.
- Several modules only define one side of what should eventually be a pair.
- Some verb choices feel strained when trying to force conjugation to carry too much meaning (especially around streaming vs single-value results).
- A few roots need explicit policy decisions before they become public API.

---

## Module-by-Module Findings & Recommendations

### 1. `solum.fab` (Filesystem) — Line Collection Resolved

**Current Pattern**:
- `lege` / `leget`
- `hauri` / `hauriet`
- `carpe` / `carpiet`
- `scribe` / `scribet`
- `funde` / `fundet`
- etc.

**Assessment**: Strong. The `-et` / `-iet` future forms are used consistently for async file operations.

**Recommendations**:
- Keep `carpe` / `carpiet` as sync/async collect-all-lines forms returning `lista<textus>`.
- If line streaming is added later, use the real plural forms of `carpere`: `carpunt` for sync generator, `carpent` for async generator.

---

### 2. `consolum.fab` (Console) — Excellent

**Current Pattern**:
- `funde` / `fundet`
- `scribe` / `scribet`
- `dic` / `dicet`
- `mone` / `monet`
- `vide` / `videbit`
- `hauri` / `hauriet`
- `lege` / `leget`

**Assessment**: One of the cleanest modules. The distinction between `scribe` (with newline) and `dic` (without) is well-motivated.

**Recommendations**:
- Keep current naming.
- Keep `videbit` unless the broader morphology policy deliberately rejects classical irregularity. `videet` would be artificially regular and less Latin.

---

### 3. `processus.fab` — Mostly Good

**Current**:
- `exsequi` / `exsequetur` (shell execution)
- `genera` (attached spawn)
- `dimitte` (detached spawn)

**Assessment**: Using distinct roots for `genera` vs `dimitte` is the right call — these are semantically very different lifecycles.

**Recommendations**:
- `exsequi` / `exsequetur` is solid.
- Consider adding a generator/streaming variant for process output in the future. Prefer the same root if it is the same execution operation with streaming output; use a distinct root only if the lifecycle or ownership model changes.

---

### 4. `arca.fab` + `Transactio` (Database) — Query Forms Resolved

This is the key test case for migration-safe conjugation. The root `quaer-` remains shared because `quaeret` and `quaerent` are the same database query operation with different wrapper semantics.

**Current**:
- `quaerent` (`@ cursor`) — stream rows
- `quaeret` — return list
- `capiet` — return first or nihil
- `exsequetur` — mutation
- `inseret` — insert returning ID
- `incipiet`, `committet`, `revertet`

**Problems**:
- `quaerent` / `quaeret` are visually similar, but that is not by itself a reason to split roots. Compiler morphology validation should catch invalid forms.
- `quaerent` is correct under the resolved policy that async generators use the real future plural form.
- `capiet` uses a different root for "return first row." This is correct: single-row retrieval is a distinct operation family, not just a wrapper over query-all/query-stream.

**Recommendations**:

| Current      | Recommendation | Rationale |
|--------------|----------------|-----------|
| `quaerent`   | Keep. | Real future plural async-generator form of `quaerere`; query streaming and query collection remain conjugations of the same operation. |
| `quaeret`    | Keep. | Good future-form call site for async query returning collected rows. |
| `capiet`     | Keep as a separate root. | Single-row retrieval returns one object/row and can have its own sync, async, or at-most-one generator forms. |
| `exsequetur` | Keep. | Mutation execution is separate from query result production. |
| `inseret`    | Keep. | Insert returning ID is a distinct database operation. |

**Resolved**:
`quaerent` is the correct async-generator form for the query root. `capiet` is a separate single-result retrieval root. Do not solve visual similarity by switching either operation to an unrelated root.

---

### 5. `thesaurus.fab` (Cache + Pub/Sub) — Pub/Sub Stream Resolved

**Problems**:
- Heavy use of future forms for almost everything (reasonable for a remote cache).
- `nuntient()` on `Subscriptio` for the message stream used artificial suffix regularity instead of the real future plural of `nuntiare`.
- Many short TTL / existence methods feel like they should stay simple.

**Recommendations**:
- Rename `nuntient()` to `nuntiabunt()` for the async message stream.
- Keep the message/announcement root because that is the conceptual operation of delivered messages.
- Do not switch to `auscultet` merely to signal streaming; `auscultabit` already means subscribe/listen and returns the subscription object.

---

### 6. `http.fab` — Acceptable

All client/server operations are async-only, so only future forms are defined (`petet`, `mittet`, `exspectabit`, etc.).

**Recommendations**:
- The response builder methods (`replica`, `scribe`, `funde`) are sync helpers inside handlers — correct.
- If sync HTTP clients are ever added, they should use non-future forms of the same roots (`petere`, `mittere`, etc.), which may feel slightly awkward. Consider documenting this.

---

### 7. Other Modules (Brief)

- **`tempus.fab`**: `dormiet` for sleep is fine.
- **`crypta.fab`**: Async hash functions are intentionally slow — future forms are correct.
- **`caelum.fab`**, **`nuncius.fab`**, **`pressura.fab`**: Mostly future-only networking/messaging. Review for real Latin conjugation of each root rather than suffix uniformity.
- Data format modules (`json`, `toml`, `yaml`): Currently only define base forms. When async versions are added, they should use the real future form for each root rather than forcing an artificial `-et` pattern.

---

## Cross-Cutting Recommendations

### 1. Standardize Generator/Streaming Signaling

Resolved policy: generators use real plural conjugation of the declared root.

Examples:
- `quaerere` -> `quaerunt` for sync query stream.
- `quaerere` -> `quaerent` for async query stream.
- `nuntiare` -> `nuntiant` for sync message stream.
- `nuntiare` -> `nuntiabunt` for async message stream.

### 2. Prefer Distinct Roots for Large Semantic Differences

Examples already done well (`genera`/`dimitte`, response builders):
- Continue this pattern when the difference is about lifecycle, ownership, cardinality, or side effects rather than just wrapper behavior.
- Keep the same root when the difference is collect vs stream, sync vs async, or both, assuming the underlying operation is conceptually the same.

### 3. Prefer Real Latin Conjugation

Resolved policy: prefer real Latin conjugation over artificial suffix regularity.

Rationale:
- LLMs and Latin-aware users are more likely to generate and recognize real conjugated forms.
- Predictability should come from declaring each root and conjugation class, not from forcing every verb into one visible suffix pattern.
- Forms such as `videbit` should stay. Invented regularizations such as `videet` should be avoided.

Implications:
- `quaerent` is correct because the intended root is `quaerere`.
- `nuntient` became `nuntiabunt` because the intended root is `nuntiare`.

### 4. Document the Conjugation Contract Explicitly

Create (or expand) a document that defines the expected conjugation patterns for new stdlib contributions. It should explicitly state that current full stdlib method names are migration contracts for future root-form declarations.

---

## Prioritized Change Proposals

### High Priority (before heavy Rust implementation)

| Module     | Current Method     | Suggested Change                  | Priority | Rationale |
|------------|--------------------|-----------------------------------|----------|---------|
| Cross-cutting | Generator form policy | Use real present plural for sync generators and real future plural for async generators | Done | Stabilizes stream call-site forms |
| `arca`     | `quaerent`         | Keep | Done | Real future plural for `quaerere`; same query operation remains one root |
| `arca`     | `capiet`           | Keep as separate root | Done | Single-row retrieval has distinct result shape and wrapper possibilities |
| `solum`    | `carpiet`          | Keep as async line collection | Done | Streaming should add `carpunt` / `carpent`, not rename collected-list forms |
| `thesaurus`| `nuntient`         | Rename to `nuntiabunt` | Done | Real future plural for `nuntiare`; keeps subscribe/listen separate from delivered messages |

### Medium Priority

- Audit all remaining future forms across `caelum`, `nuncius`, `pressura` for real conjugation.
- Keep `morphologia.md` aligned with the resolved present-plural generator and future-plural async-generator policy.
- Ensure data-format modules (`json`/`toml`/`yaml`) have placeholder future forms documented even if not yet implemented.

### Low Priority / Future

- Revisit whether any "sync-only" methods should eventually grow async twins using the same root.

---

## Next Steps

1. Review this document with the project owner.
2. Continue resolving module-specific edge cases, especially first-row database retrieval.
3. Apply only the renames required by resolved policies while the Rust `norma` crates are still early.
4. Add a short "Verb Conjugation Guidelines" section to the Norma contributor docs.

---

**Origin**: Initial Grok draft, revised after owner clarification that current names are migration contracts for future root-form declarations.  
**Status**: Ready for edge-case decisions.
