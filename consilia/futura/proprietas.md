# proprietas - Property-Style Access for Zero-Arg Methods

**Status:** Future consideration (specialized use case)

**Concern:** Risk of making Faber feel like "TypeScript-with-a-toga." Use sparingly if at all.

---

## Problem

LLMs consistently write `items.longitudo` instead of `items.longitudo()` because they're trained on `arr.length` (JavaScript/TypeScript). For an LLM-first language, this creates first-attempt failures.

However, bending the language to match LLM expectations risks losing Faber's distinctive character. The preferred approach is helpful error messages that teach the correct pattern.

---

## Proposed Feature (If Needed)

`@ proprietas` annotation marks zero-arg methods as property-style access (no parentheses).

```faber
@ proprietas
@ verte ts (ego) -> "§.length"
@ verte py (ego) -> "len(§)"
@ verte rs (ego) -> "§.len()"
@ externa
functio longitudo() fit numerus
```

Usage:
```faber
scribe items.longitudo    # Works (no parens)
scribe items.longitudo()  # Error: property, not method
```

---

## Design Decisions

### 1. Zero-arg only

Properties by definition have no arguments. Methods with optional args stay as methods.

### 2. Mutually exclusive

A method is EITHER callable with parens OR accessible without. Not both.

- `@ proprietas` method: `items.longitudo` valid, `items.longitudo()` error
- Regular method: `items.adde(x)` valid, `items.adde` is function reference

### 3. No parser changes

`items.longitudo` already parses as MemberExpression. Only semantic analysis and codegen need updates.

### 4. Linguistic alignment

| Type | Examples | Access |
|------|----------|--------|
| Nouns/Adjectives | `longitudo`, `vacua`, `primus` | Property |
| Verbs | `adde`, `filtra`, `ordina` | Method |

Latin nouns/adjectives describe static qualities. Verbs describe actions. This maps to property vs method.

---

## Target Language Handling

The `@ verte` template handles variance:

| Target | `longitudo` Translation |
|--------|------------------------|
| TypeScript | `§.length` (property) |
| Python | `len(§)` (function) |
| Rust | `§.len()` (method call) |
| C++ | `§.size()` (method call) |
| Zig | `§.longitudo()` (wrapper) |

---

## Implementation Outline

### Phase 1: Annotation Parsing
- Add `@ proprietas` to annotation grammar (trivial - generic annotations exist)
- Store `proprietas: boolean` flag on NormaMethod

### Phase 2: Semantic Analysis
- When MemberExpression references a `@ proprietas` method, mark as valid
- When CallExpression calls a `@ proprietas` method, emit error

### Phase 3: Codegen
- In `genMemberExpression`, check norma registry for `@ proprietas` methods
- Apply template translation (same pattern as CallExpression)

### Phase 4: Stdlib Migration (if adopted)
Candidates: `longitudo`, `vacua`, `primus`, `ultimus`, `minimus`, `maximus`

---

## Alternative: Better Error Messages

Instead of implementing `@ proprietas`, provide helpful errors:

```
Error: 'longitudo' is a method, not a property. Use 'items.longitudo()'.
```

This maintains language consistency while teaching the correct pattern. LLMs adapt quickly to consistent error feedback.

---

## Recommendation

**Default:** Use helpful error messages. Keep parens-for-everything.

**Exception:** If a specific stdlib method has overwhelming LLM failure rates AND maps cleanly to properties in all target languages, consider `@ proprietas` for that method only.

The feature exists as an escape hatch, not a default pattern.

---

## Naming

`@ proprietas` (Latin: "property, characteristic quality") is the correct term. Alternatives considered:
- `@ attributum` - "attribute" (too generic)
- `@ nomen` - "noun" (doesn't cover adjectives)
- `@ qualitas` - "quality" (philosophical, awkward)

---

## Design Review (Augur Analysis)

*Reviewed: 2026-01-09*

### Consequence Chain

1. **Annotation parsing and storage** — Add `@ proprietas` to grammar (trivial), extend `NormaMethod` interface to store `proprietas: boolean` flag
   → 2. **Norma registry build changes** — `build:norma` script must parse `@ proprietas`, populate flag in generated `norma-registry.gen.ts`, synchronize to `norma-registry.gen.fab` for rivus
     → 3. **Semantic analysis divergence** — `resolveMemberExpression` must distinguish property-methods from callable-methods; `resolveCallExpression` must reject calls to property-methods
       → 4. **Codegen per-target complexity** — Each target (ts/py/rs/cpp/zig) must check `@ proprietas` flag in `genMemberExpression`, apply norma templates there instead of `genCallExpression`
         → 5. **Breaking change risk** — Existing code calling `longitudo()` with parens breaks when annotation added; migration path unclear
           → 6. **Test coverage explosion** — Every target needs tests for: property access success, parenthesized call rejection, optional chaining (`items?.longitudo`), error messages, interaction with norma templates

### Impact Assessment

| Area | Impact | Notes |
|------|--------|-------|
| Lexer | None | No token changes; grammar already accepts member access |
| Parser | None | `obj.prop` already parses as MemberExpression; no AST changes needed |
| Semantic | **High** | Must bifurcate method access logic: property-methods resolve in `resolveMemberExpression`, reject in `resolveCallExpression`; error messages must guide users |
| Codegen | **High** | All 5 targets must check `proprietas` flag in both `genMemberExpression` and `genCallExpression`; template application moves from call-site to member-access |
| Tests | **Medium** | Need per-target tests for property-method access, rejection of call syntax, optional chaining; rivus tests also affected |
| User code | **Medium-High** | Breaking change: code currently using `items.longitudo()` breaks when stdlib migrates to `@ proprietas`; no deprecation path exists |

### Concerns

1. **Semantic analyzer complexity** — Currently, `resolveMemberExpression` returns method types (FunctionType), and `resolveCallExpression` invokes them. With `@ proprietas`, member access must apply templates and return result types directly, duplicating codegen logic in semantic phase. This violates separation of concerns.

2. **Breaking change without migration path** — Annotating `longitudo` with `@ proprietas` instantly breaks all existing `items.longitudo()` calls. No deprecation period, no warning phase, no automated migration tool.

3. **Asymmetric method semantics** — Faber currently has uniform method call syntax: all methods require `()`. Introducing property-methods fragments this into "verbs need parens, nouns don't" — cognitive load when scanning code.

4. **Template application in two places** — Norma templates currently apply exclusively in `genCallExpression`. With `@ proprietas`, they must also apply in `genMemberExpression`. This duplicates template logic.

5. **Insufficient justification** — The design claims LLMs "consistently write `items.longitudo` instead of `items.longitudo()`" but provides no data. Is this a 90% failure rate or 10%? Does clear error feedback fix the issue in subsequent turns?

6. **Linguistic alignment claim is weak** — The document argues nouns/adjectives map to properties, verbs to methods. But Latin doesn't enforce this: `longitudo` (noun) could still be a method returning length. The distinction is aesthetic, not grammatical.

### Questions

- What is the actual LLM failure rate for `longitudo()` vs `longitudo` in trials?
- How should migration work? Maintain both forms for transition, emit warnings, then hard error?
- Why not restrict `@ proprietas` to targets where it maps to native properties (TS `.length`)?
- Can semantic return a placeholder "PropertyMethodType" that codegen resolves, preserving separation of concerns?

### Recommendations

1. **Defer until LLM trial data justifies it.** Measure failure rate and retry overhead with current error messages.

2. **If implemented, start TS-only.** Restrict `@ proprietas` to targets where the native type actually has properties.

3. **Add migration support.** Warning mode, lint rule for auto-fix, gradual rollout.

4. **Separate semantic from codegen.** Introduce `PropertyMethodType` wrapper to avoid duplicating transformation logic.

5. **Defer until Nucleus async generators stabilize.** The recently consolidated runtime design may change how stdlib methods interact with types.
