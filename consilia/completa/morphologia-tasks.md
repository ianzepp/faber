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
    - If the receiver type is not morphology-enabled, the call must be treated as an ordinary method call.
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
- **Forms**: `imperativus` and `perfectum` only

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
    - If a call parses as morphology (via `parseMethodum`), only treat it as morphology if receiver type is morphology-enabled.
    - If receiver is enabled but stem or form is not declared, produce a semantic error.

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
- Morphology dispatch occurs only when the receiver is morphology-enabled.
- No emitted code contains hidden `await`.
- A non-morphology receiver with a similarly named method is not hijacked.
- At least one test asserts the above.

## Status (Milestone 1)

Completed.

- AST: Added `MorphologiaDeclaratio` and per-call `MorphologiaInvocatio` for receiver-bound dispatch.
- Parser: Centralized line-based annotation parsing and `@ radix` form validation.
- Semantics: Registry keyed by receiver+stem+forms; call validation annotates eligible calls and emits diagnostics.
- Codegen: Dispatch uses semantic annotation; removed hidden `await` in morphology generators.
- Tests: Added Rivus morphology cases in `fons/proba/codegen/expressions/morphologia.yaml`.

Build/test notes:
- `bun run build:rivus` succeeds.
- `bun run test:rivus` runs; morphology tests pass; existing unrelated Rivus failures remain (see `tmp/test-rivus.log`).

## Expansion Plan (After Milestone 1)

1. Add `futurum_indicativum` and `futurum_activum` (async forms) with explicit consumption (no auto-await).
2. Add generator forms (if desired) once the type model for iterators is clear.
3. Extend from `lista` to `tabula`/`copia`, then IO types (`solum`, `caelum`, `arca`, `nucleus`).
