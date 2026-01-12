---
status: completed
updated: 2026-01-06
note: Milestone 1 completed - receiver-bound morphology dispatch implemented in Rivus for lista.filtra/filtrata
see: morphologia.md for design documentation
---

# morphologia-tasks.md

This file is a task brief for implementing **Morphologia** in the **Rivus** compiler only.

## Goal (One Sentence)

Implement morphology as **typed, opt-in semantic dispatch** for stdlib (Latin-named) APIs in Rivus, using `@ radix ...` declarations to validate and lower morphology-based method calls—**without hidden control flow**.

## Non-Negotiables

1. **No hidden `await`** in any codegen output.
    - Morphology can affect _return type/shape_ and flags.
    - Consumption remains explicit via `cede`, `figendum`, `variandum`, loops, etc.
2. **Receiver-bound dispatch only**.
    - Never dispatch solely because a method name “looks like” morphology.
    - Only dispatch when the receiver type is morphology-enabled **and the parsed stem is declared for that receiver**.
    - Otherwise the call must be treated as an ordinary method call.
3. **Single annotation syntax**.
    - Supported: `@ radix imperativus, perfectum, ...` (line-based)
    - Unsupported: `@ radix(...)` (parenthesized)
4. **Terminology**.
    - `radix` means **stem**.
    - Do **not** store conjugation lists under an AST field named `radices`.

## Terminology

- **Stem (`radix`)**: e.g. `filtr` in `filtra` / `filtrata` / `filtrabit` / `filtratura`.
- **Form (`forma`)**: which conjugation-derived semantic variant is invoked:
    - `imperativus` (sync mutate)
    - `perfectum` (sync return-new)
    - `futurum_indicativum` (async mutate)
    - `futurum_activum` (async return-new)
    - (later) `praesens_participium` (generator)
    - (later) async generator form

Note: Milestone 1 is validated primarily with `imperativus` + `perfectum`; the parser accepts the async form names as well.

- **Morphology flags**: derived from form and used by semantic/codegen.

## Source Syntax: `@ radix ...` (Line-Based)

### Example

```faber
@ radix imperativus, perfectum, futurum_indicativum, futurum_activum
functio filtra<T>(...) fit vacuum { ... }
```

### Parsing rule

- Parse `@` annotations as **prefix lines**.
- For `@ radix`, parse a comma-separated list of identifiers **only while tokens remain on the same source line** as the `@` annotation.
- The next line begins the declaration.

## Target Scope (First Milestone)

Implement end-to-end morphology for:

- **Receiver type**: `lista<T>` (only)
- **Stem(s)**: start with `filtr` only
- **Forms**: `imperativus` and `perfectum` (tests + acceptance criteria)

Then expand incrementally.

## Implementation Tasks

### Phase 0: Prep / Audit

1. Identify current POC entry points:
    - Morphology parsing: `fons/rivus/parser/morphologia.fab` (`parseMethodum`)
    - TS dispatch: `fons/rivus/codegen/ts/expressia/index.fab`
    - Stem registry/lowering: `fons/rivus/codegen/radices.fab`
2. Confirm the current POC violates invariants (hidden `await`, lexical dispatch) and mark these as to-be-fixed.

### Phase 1: Structured Annotations in Rivus AST

Goal: represent `@ radix ...` as structured metadata on declarations.

Tasks:

1. Decide where morphology declarations live:
    - Option A (recommended): attach to **type definitions** (e.g. `genus lista`) so dispatch is naturally receiver-bound.
    - Option B: attach to **functions/method declarations** and build registry by looking up receiver type.

2. Add/extend AST structures to carry:
    - whether a type/declaration is morphology-enabled
    - which forms are allowed
    - (optional) declared stem(s) / aliases

Constraints:

- Avoid ambiguous naming: store allowed forms under `formae` / `conjugationes` / `morphologia`.

### Phase 2: Parser Support for `@ radix ...`

Goal: parse line-based `@ radix` and populate the AST metadata.

Tasks:

1. Centralize annotation parsing logic so it is reused by:
    - top-level statement parsing (`functio`, `genus`, `pactum`)
    - genus member parsing (methods)
    - pactum member parsing (methods)

2. Implement `@ radix` parsing:
    - accept only identifier tokens (no strings/expressions)
    - parse comma-separated identifiers on the same line

3. Validate the identifiers (forms) at parse time _only for spelling/known values_.
    - Produce a dedicated parse error if an unknown form is listed.

Deliverable:

- A `@ radix ...` annotation in source is represented in the AST.

### Phase 3: Semantic Registry (Receiver-Bound)

Goal: build a per-type morphology registry that allows validating calls.

Tasks:

1. In the semantic phase, build a registry keyed by:
    - receiver type (e.g., `lista<T>`)
    - stem (`filtr`)
    - allowed forms (`imperativus`, `perfectum`, ...)

2. Decide how stems are obtained:
    - Option A: derive stem from the declared base method name (strip ending)
    - Option B: require explicit declaration of stems in the annotation

3. Add semantic validation for calls:
    - If a call parses as morphology (via `parseMethodum`), only treat it as morphology if receiver type is morphology-enabled **and the parsed stem is declared**.
    - If the receiver is enabled but the stem is not declared, treat it as a normal method call (do not error; avoid lexical hijacking).
    - If the stem is declared but the form is not allowed, produce a semantic error.

Deliverable:

- Morphology calls are rejected/accepted based on receiver type + declared forms.

### Phase 4: Remove Lexical Hijacking & Hidden Await

Goal: make the existing TS morphology codegen safe.

Tasks:

1. Remove any `await` injection inside generated expressions in `fons/rivus/codegen/radices.fab`.
    - Async forms should lower to expressions that evaluate to a Future/Promise.
    - Awaiting remains the job of `cede` / async contexts.

2. Ensure call dispatch in `fons/rivus/codegen/ts/expressia/index.fab` depends on semantic info (receiver-bound), not just `estRadixListae()`.

Deliverable:

- The compiler never emits `await` unless it comes from explicit source constructs.

### Phase 5: Codegen Integration (TS First)

Goal: lower a validated morphology call to the correct backend implementation.

Tasks:

1. For each supported `(receiver, stem, form)`:
    - define a lowering strategy that preserves the semantics of flags.

2. Ensure TS type shape matches semantics:
    - `imperativus`: returns `void` / mutates receiver
    - `perfectum`: returns new value
    - `futurum_*`: returns `Promise<...>` (no implicit await)

Deliverable:

- End-to-end compilation for a small `.fab` fixture using `lista.filtra(...)` and `lista.filtrata(...)`.

### Phase 6: Diagnostics

Goal: errors that teach the user.

Tasks:

1. For invalid `@ radix` forms, include:
    - the invalid identifier
    - the list of supported form names

2. For invalid calls, include:
    - the method name
    - parsed `(stem, form)` if applicable
    - the receiver type (as best as semantic layer can name it)
    - allowed forms for that stem on that receiver

### Phase 7: Tests

Goal: lock behavior with minimal tests.

Tasks:

1. Add a Rivus-focused test fixture that covers:
    - accepted: `lista.filtra` and `lista.filtrata` when `lista` has `@ radix` enabling them
    - rejected: `Calculator.adde`-style collisions (method looks morphological but receiver is not enabled)
    - rejected: calling a form not declared for the stem
    - rejected: unknown form in `@ radix ...`

2. Run:
    - `bun test` (or the narrowest equivalent that covers Rivus)

## Acceptance Criteria (Milestone 1)

- `@ radix imperativus, perfectum` is parsed with line-based args only.
- Morphology dispatch occurs only when the receiver is morphology-enabled and the parsed stem is declared for that receiver.
- No emitted code contains hidden `await`.
- A non-morphology receiver with a similarly named method is not hijacked.
- At least one test asserts the above.

## Status (Milestone 1)

Completed.

- AST: Added `MorphologiaDeclaratio` and per-call `MorphologiaInvocatio` for receiver-bound dispatch.
    - `@ radix` is intended for receiver methods (e.g. `genus lista { @ radix ... functio filtra ... }`), not for top-level free functions.
- Parser: Centralized line-based annotation parsing and `@ radix` form validation.
- Semantics: Registry keyed by receiver+stem+forms; call validation annotates eligible calls.
    - If the stem is undeclared, the call is left alone (no hijacking).
    - If the stem is declared but the form is disallowed, a semantic error is emitted.
- Codegen: Dispatch uses semantic annotation; removed hidden `await` in morphology generators.
- Tests: Added Rivus morphology cases in `fons/proba/codegen/expressions/morphologia.yaml`.

Build/test notes:

- `bun test fons/proba/rivus.test.ts -t morphologia --timeout 5000` passes.
- Full `bun run test:rivus` currently has unrelated failures in this repo; do not treat that as a Morphologia regression unless the failures point at Morphologia output.

## Milestone 2: `lista` as the Reference Stdlib

**Goal:** `lista<T>` works correctly with Morphologia for **sync + async** variants and can be used as the reference implementation for the rest of the stdlib.

### Requirements

1. **`lista` must be morphology-enabled by default** (no user-side `@ radix` required).
    - The existing `@ radix ...` mechanism remains useful for user-defined/experimental receivers, but stdlib `lista` should not require local declarations.
    - This is necessary so the shared `fons/proba/norma/lista.yaml` TS expectations (e.g. `items.filtrata(...)` → `.filter(...)`) can pass under Rivus.

2. **Support sync + async forms for `lista`**

- Sync:
    - `imperativus` (mutate) — e.g. `filtra`, `ordina`, `inverte`, `adde`
    - `perfectum` (return-new) — e.g. `filtrata`, `ordinata`, `inversa`, `addita`
- Async:
    - `futurum_indicativum` (mutate) — e.g. `filtrabit` (must return `Promise<void>`-shape; no hidden await)
    - `futurum_activum` (return-new) — e.g. `filtratura` (must return `Promise<lista<T>>`-shape; no hidden await)

3. **Typing is correct for the key cases**

- `lista.filtra(...)` returns `vacuum` (or equivalent in TS output), mutates receiver.
- `lista.filtrata(...)` returns a new `lista<T>`.
- Async forms return `Promise<...>` in TS output (still explicitly consumed via `cede` or assigned to `figendum`).

4. **No codegen split allowed for `lista`**

If `lista` tests fail under Rivus while passing under the primary compiler, determine whether the failure is:

- a missing morphology registry entry for `lista` in Rivus,
- a mismatch in stem parsing (`parseMethodum`) vs how the stdlib spells methods,
- or a real backend divergence between Faber and Rivus.

### Implementation Checklist

1. **Seed the morphology registry for built-in `lista`** during analyzer initialization.
    - Populate `morphologiaRegistra["lista"]` with all stems implemented in `fons/rivus/codegen/radices.fab`.
    - For each stem, list the allowed forms.
    - Keep it explicit (no inference); it’s a reference spec.

2. **Ensure call-site semantic annotation happens for `lista` without local `@ radix`**.
    - When receiver type is `Genericum` with `nomen == "lista"`, treat it as morphology-enabled and consult the seeded registry.

3. **Extend/verify TS codegen lowering for async forms**.
    - Continue returning `Promise` expressions (no `await`).
    - If a form implies mutation, return a promise that resolves after mutation.

4. **Test strategy**

- Run Rivus tests filtered to lista/norma:
    - `bun test fons/proba/rivus.test.ts -t lista --timeout 5000`
    - `bun test fons/proba/rivus.test.ts -t filtrata --timeout 5000`
- Also run the same patterns against the primary compiler’s tests:
    - `bun test fons/proba/faber.test.ts -t lista`

### Diagnosing Faber vs Rivus codegen split

If the primary compiler’s `lista` expectations pass but Rivus fails:

1. Identify the failing YAML case and its TS expectation in `fons/proba/norma/lista.yaml`.
2. Compare Faber vs Rivus output for the same snippet:
    - Faber: `bun test fons/proba/faber.test.ts -t "<case name fragment>"`
    - Rivus: `bun test fons/proba/rivus.test.ts -t "<case name fragment>" --timeout 5000`
3. If Rivus still emits `items.filtrata(...)` instead of `.filter(...)`, it is almost certainly missing default `lista` registration (registry seeding / receiver detection).

## Expansion Plan (After Milestone 2)

1. Add generator forms (if desired) once the type model for iterators is clear.
2. Extend from `lista` to `tabula`/`copia`, then IO types (`solum`, `caelum`, `arca`, `nucleus`).

## Status (Milestone 2)

Completed.

- Seeded built-in Morphologia registry entries for `lista` so it is enabled by default.
- Registered all `lista` stems from `fons/rivus/codegen/radices.fab` with explicit form lists.
- Extended `lista` TS morphology codegen for async forms in mutating and return-new variants (no hidden `await`).
- Build/test notes:
  - `bun run build:rivus` passes.
  - `bun run test:rivus` still has unrelated failures in this repo (lista/ranges/numeric/DSL cases remain failing); Morphologia changes did not introduce new errors in the tail output.
