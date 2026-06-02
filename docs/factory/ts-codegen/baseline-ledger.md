# TypeScript Codegen Baseline Ledger

**Date**: 2026-06-02  
**Worktree**: `/Users/ianzepp/work/ianzepp/faber-ts-codegen`  
**Branch**: `factory/ts-codegen`  
**Phase Artifact**: `docs/factory/ts-codegen/phase-1-baseline-delivery.md`  
**Harness**: `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`

## Toolchain Detection

Observed by the TypeScript e2e harness:

- Formatter: skipped, no `prettier` or `deno` on `PATH`.
- Linter: skipped, no `biome` or `eslint` on `PATH`.
- Typechecker: `tsc --noEmit main.ts`.
- Runtime: `tsc main.ts; node main.js`.

The command-line shell also resolved:

- `tsc`: `/Users/ianzepp/.nvm/versions/node/v24.15.0/bin/tsc`
- `node`: `/opt/homebrew/bin/node`

No `prettier`, `deno`, `biome`, or `eslint` executable was found by `command -v`.

## Tier Counts

```text
TypeScript e2e exempla:
  frontend analyzed: 101/101
  TypeScript emitted: 100/101
  formatted: 0/101 (skipped: no prettier or deno)
  linted: 0/101 (skipped: no biome or eslint)
  typecheck-valid: 64/101
  runnable: 63/101
  behavior-checked: 1/101
```

Expected-output checks are available for 1 exemplar file:
`examples/exempla/salve-munde.expected`.

## Failure Clusters

### Unsupported TypeScript Target Shape

- `examples/exempla/ad/ad.fab`

Current result: frontend analysis succeeds, but TypeScript codegen rejects `ad`
with `ad is not supported for TypeScript codegen`.

Likely next phase: capability and host-boundary classification. Either lower to
documented TypeScript runtime helpers or keep an explicit target diagnostic.

### Genus, Method, Constructor, And Interface Shape

Representative failures:

- `abstractus/abstractus.fab`
- `ego/ego.fab`
- `generis/generis.fab`
- `genus/creo.fab`
- `genus/genus.fab`
- `genus/methodi.fab`
- `implet/implet.fab`
- `nexum/nexum.fab`
- `novum/novum.fab`
- `optionalis/optionalis.fab`
- `pactum/pactum.fab`
- `sub/sub.fab`
- `syntaxis/arena-mixta.fab`
- `vocatio/vocatio.fab`

Common symptoms:

- Required class fields have no initializer under strict TypeScript.
- Object literals miss required fields when Faber permits partial construction
  through later hooks or defaults.
- Methods are emitted or resolved as static members where call sites expect
  instance methods.
- Interface implementation checks fail because required methods are not emitted
  on the instance shape.

Likely next phase: declarations and calls, focused on genus field initialization,
`creo` hooks, instance method emission, and interface method shape.

### Tagged Union And Pattern-Matching Shape

Representative failures:

- `discerne/discerne.fab`
- `discretio/discretio.fab`
- `finge/finge.fab`
- `omnia/omnia.fab`
- `ordo/ordo.fab`
- `syntaxis/discerne-insanum.fab`

Common symptoms:

- Faber enum names such as `Event` collide with TypeScript DOM globals.
- Variant constructors such as `Active`, `Pending`, `Move`, and `Loquere` are
  referenced without emitted TypeScript constructor bindings.
- Some expression-valued `discerne` paths emit functions that TypeScript sees as
  missing returns.

Likely next phase: tagged union constructors and target name hygiene, including
DOM global avoidance under the selected TypeScript library.

### Async, Cursor, And Await Lowering

Representative failures:

- `cede/cede.fab`
- `futura/futura.fab`
- `incipiet/incipiet.fab`
- `itera/cursor-iteratio.fab`
- `syntaxis/fluxus-cede.fab`

Common symptoms:

- Async functions return raw `T` instead of `Promise<T>`.
- `await` appears inside non-async functions or generated closures.
- Cursor-like values are treated as scalar `number` values instead of iterable
  or async-iterable TypeScript values.

Likely next phase: async/cursor lowering and runtime entrypoints.

### Expression-Valued Control Flow

Representative failures:

- `elige/ergo-redde.fab`
- `si/ergo-redde.fab`
- `itera/cursor-iteratio.fab`
- `syntaxis/fluxus-cede.fab`

Common symptoms:

- Expression-valued `elige`, `si`, or `discerne` paths lower to function-like
  TypeScript forms where not every branch returns a value.

Likely next phase: control-flow and pattern-matching expression lowering.

### Collections And Stdlib Method Translation

Representative failures:

- `innatum/innatum.fab`
- `morphologia/morphologia.fab`

Common symptoms:

- Empty map values emit `Map<any, any>` while the annotated type is
  `Record<string, number>`.
- Faber stdlib methods such as `primus`, `addita`, and `inverte` are not all
  translated to TypeScript collection operations.
- `toSorted` requires a newer TypeScript lib target than the harness currently
  supplies.

Likely next phase: collection and stdlib method translations, plus deliberate
TypeScript library target policy.

### Strict Literal Narrowing

Representative failures:

- `adfirma/adfirma.fab`
- `binarius/binarius.fab`

Common symptoms:

- TypeScript reports comparisons between narrowed literal types, such as
  `"Marcus"` versus `""` or `10` versus `5`, as unintentional.

Likely next phase: type and value shape, deciding whether generated declarations
should widen primitive literal bindings in executable code.

### Optional Parameter And Nullability Shape

Representative failure:

- `functio/optionalis.fab`

Common symptom: optional TypeScript parameters introduce `undefined`, but Faber
nullable value positions use `nihil`/`null` semantics.

Likely next phase: nullable `T ∪ nihil`, `sponte`, and optional parameter
representation.

### Break And Continue In Generated Closures

Representative failures:

- `perge/perge.fab`
- `rumpe/rumpe.fab`

Common symptom: `break` and `continue` cross generated function boundaries.

Likely next phase: control-flow lowering for expression helper wrappers and loop
body emission.

### Runtime-Only Failure After Typecheck

Representative failure:

- `functio/recursio.fab`

Current result: TypeScript typechecks, transpiles, and starts under Node, then
fails with `RangeError: Maximum call stack size exceeded`.

Likely next phase: inspect generated recursion and conditional return lowering
before classifying this as backend logic versus exemplar behavior.

## Validation Evidence

- `cargo test -p radix codegen::ts -- --nocapture`: passed, 11 tests.
- `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`: passed, with
  the tier counts above.
- `cargo test -p radix`: passed, 487 regular tests, 4 ignored tests, hygiene
  tests, and doctests.
- `./scripta/lint`: passed.

## Next Phase Recommendation

The highest-value next cluster is declarations and calls for `genus`, methods,
constructors, and interface implementation. It affects the largest number of
typecheck failures and should improve representative `genus`, `finge`-adjacent,
`pactum`, and method-call exemplars without changing the harness boundary.

## Phase 2 Update: Genus Instance Methods

**Phase Artifact**: `docs/factory/ts-codegen/phase-2-genus-methods-delivery.md`

Phase 2 changed TypeScript genus lowering so ordinary methods are emitted as
instance methods, `ego` lowers to `this` inside method bodies, uninitialized
class fields use definite-assignment declarations, and struct-target
construction emits `Object.assign(new Type(), { ... })` so generated instances
retain prototype methods and field defaults.

Representative improvements:

- `examples/exempla/genus/methodi.fab` now passes strict `tsc`.
- `examples/exempla/implet/implet.fab` now passes strict `tsc`.

Updated harness result:

```text
TypeScript e2e exempla:
  frontend analyzed: 101/101
  TypeScript emitted: 100/101
  formatted: 0/101 (skipped: no prettier or deno)
  linted: 0/101 (skipped: no biome or eslint)
  typecheck-valid: 76/101
  runnable: 75/101
  behavior-checked: 1/101
```

Remaining failure clusters after Phase 2:

- Unsupported TypeScript target shape: `ad/ad.fab`.
- Strict literal narrowing: `adfirma/adfirma.fab`, `binarius/binarius.fab`.
- Async/cursor lowering: `cede/cede.fab`, `futura/futura.fab`,
  `incipiet/incipiet.fab`, `itera/cursor-iteratio.fab`,
  `syntaxis/fluxus-cede.fab`.
- Tagged union and pattern constructors: `discerne/discerne.fab`,
  `discretio/discretio.fab`, `finge/finge.fab`, `omnia/omnia.fab`,
  `ordo/ordo.fab`, `syntaxis/discerne-insanum.fab`.
- Optional parameter/nullability shape: `functio/optionalis.fab`.
- Collections and stdlib translations: `innatum/innatum.fab`,
  `morphologia/morphologia.fab`.
- Expression-valued control flow: `elige/ergo-redde.fab`,
  `si/ergo-redde.fab`, plus overlap with async/cursor and pattern matching.
- Break/continue across generated closures: `perge/perge.fab`,
  `rumpe/rumpe.fab`.
- Target-specific type mismatches or deliberate diagnostics still needing
  triage: `membrum/membrum.fab`, `mori/mori.fab`, `sub/sub.fab`,
  `vocatio/vocatio.fab`.
- Runtime-only failure after typecheck: `functio/recursio.fab`.

Phase 2 validation evidence:

- `cargo test -p radix codegen::ts -- --nocapture`: passed, 12 tests.
- Direct strict `tsc` for emitted `examples/exempla/genus/methodi.fab`: passed.
- Direct strict `tsc` for emitted `examples/exempla/implet/implet.fab`: passed.
- `cargo test -p radix exempla_ts_e2e -- --ignored --nocapture`: passed, with
  the updated tier counts above.
- `cargo test -p radix`: passed, 488 regular tests, 4 ignored tests, hygiene
  tests, and doctests.
- `./scripta/lint`: passed.

The successful coverage threshold is now met. Continue only if the next cluster
is small and high-value; otherwise this is a valid handoff point for the
TypeScript factory run.
