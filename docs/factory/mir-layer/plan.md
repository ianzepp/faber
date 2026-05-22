# MIR Layer Factory Plan

**Status**: in-progress (Phases 0–6A complete; 6B–12 pending)
**Created**: 2026-05-22
**Target Repo**: `/Users/ianzepp/work/ianzepp/faber`
**Factory Artifact Dir**: `docs/factory/mir-layer/`
**Depends On**: typed HIR, `TypeTable`, Rust backend, package build layout, `norma` runtime boundary
**Commit Policy**: Commit after each completed phase and validation gate pass

## Interpreted Problem

Faber currently compiles from parsed source through semantic analysis into typed HIR, then each backend lowers HIR directly into target source. That is still workable for source-oriented targets, but it makes every backend re-solve the same hard language-lowering problems:

- expression-valued control flow,
- option/null behavior,
- `tempta` / `iace` failure flow,
- string formatting,
- method and stdlib call lowering,
- pattern matching,
- collection literals and transforms,
- runtime-backed primitives,
- entrypoint and package shape.

The missing layer is a Faber-owned middle IR below HIR and above target emitters. This MIR should be execution-shaped rather than source-shaped. It should normalize Faber semantics once, validate the normalized form, and give Rust, WASM, and eventual native backends a smaller contract to implement.

The goal is not to replace the Rust backend in one jump. The goal is to create a stable backend layer that first improves inspection and Rust codegen, then becomes the launch point for WASM and native experiments.

## Proposed Design

Introduce a typed, non-SSA MIR:

```text
AST
  -> HIR
  -> typed HIR
  -> MIR
  -> backend-specific output
       -> Rust source
       -> future WASM
       -> future Cranelift/native
```

The first MIR should be intentionally modest:

- explicit functions,
- explicit locals and temporaries,
- explicit basic blocks,
- explicit terminators,
- explicit call forms,
- explicit runtime intrinsics,
- explicit aggregate construction,
- explicit return/error exits,
- enough type information to validate every operand and produced value.

It should not start as full SSA. SSA can be a later lowering from MIR once there is a real optimizer or native backend that earns it.

## Non-Negotiable Design Rules

- HIR remains the source of Faber language meaning.
- MIR commits to execution mechanics.
- MIR must be target-neutral; it must not contain Rust syntax, Cargo metadata, WASM imports, native linker flags, or Cranelift concepts.
- Runtime and stdlib operations should appear as target-neutral intrinsics or resolved provider calls.
- Missing type information at MIR lowering time is an upstream compiler bug, not a reason to guess in MIR or codegen.
- The existing Rust backend remains the primary correctness gate until a MIR-backed Rust path can prove equivalent behavior.
- MIR adoption must be incremental and inspectable; do not flip all codegen to MIR in one phase.

## Locked Phase 1 Decisions

- MIR starts as a typed, non-SSA, control-flow/block IR.
- MIR should reference semantic types through a MIR-owned wrapper instead of exposing raw `TypeId` everywhere.
- Recoverable failure is represented as control flow, not as a baked-in Rust-like `Result<T, E>` value.
- The first developer inspection surface should be `radix mir`.
- MIR lowering should be canonical and target-neutral.
- Any initial `MirConfig` should be limited to lowering/debug policy, not target-specific semantics.
- Real memory-management operations such as `drop`, `retain`, `release`, and `free` are deferred until Faber has a concrete runtime memory model.

## Initial MIR Shape

Candidate module:

```text
crates/radix/src/mir/
├── mod.rs
├── nodes.rs
├── lower.rs
├── validate.rs
├── dump.rs
└── tests...
```

Core concepts:

```text
MirProgram
MirFunction
MirBlock
MirStmt
MirTerminator
MirValue
MirPlace
MirType
MirIntrinsic
MirRuntimeCall
```

Representative terminators:

```text
return value
return_error value
goto block
branch condition then_block else_block
switch value cases default
unreachable
```

Representative statements:

```text
local place: type
assign place value
call dest = callee(args)
runtime_call dest = intrinsic(args)
construct dest = aggregate(args)
drop value
```

## Break Boundary

### In Scope

- Add a MIR module and data model.
- Lower selected typed HIR into MIR.
- Add MIR validation.
- Add MIR dump/inspection support for compiler development.
- Add tests proving lowered MIR shape for selected language constructs.
- Add a MIR-backed Rust codegen path only after MIR can represent a meaningful subset.
- Preserve the existing HIR-to-Rust path until the MIR path is proven.

### Out of Scope

- Direct native binary generation.
- LLVM or Cranelift integration.
- Full SSA.
- MIR optimization passes beyond simple validation and cleanup.
- Replacing all existing codegen targets in one phase.
- Changing Faber syntax to make MIR easier.
- Reworking package management except where needed to expose MIR inspection commands.

## Current Evidence

- The driver already creates an `AnalyzedUnit` containing `HirProgram`, `TypeTable`, and `Interner`.
- Semantic analysis already runs collection, resolution, HIR lowering, typecheck, borrow analysis for Rust, exhaustiveness, and linting.
- HIR nodes already carry `HirId`, `DefId`, spans, and optional resolved types on expressions.
- The current `Codegen` trait consumes HIR directly.
- Package builds currently emit a generated Rust crate under `target/faber/` and invoke Cargo.
- Target-neutral library/provider metadata is already identified as important by the standard-library data-format plan.

## Stage Graph

| Phase | Name | Goal | Checkpoint |
| ----- | ---- | ---- | ---------- |
| 0 | Baseline and invariants | Record current HIR/codegen behavior and MIR design constraints. | Ledger captures representative examples, known backend quirks, and the first MIR subset. |
| 1 | MIR data model | Add MIR node types, IDs, type references, block structure, and debug formatting. | `cargo test -p radix mir` or equivalent focused tests compile the model without lowering behavior. |
| 2 | MIR inspection surface | Add an internal dump path for MIR from a single source file. | A simple program can be parsed, checked, lowered to placeholder/subset MIR, and printed deterministically. |
| 3 | Primitive expression lowering | Lower literals, locals, arithmetic, assignment, simple calls, and returns. | MIR tests prove primitives and function bodies lower without target codegen. |
| 4 | Control-flow normalization | Lower blocks, `si`, `dum`, `rumpe`, `perge`, and expression returns into explicit blocks and terminators; keep `itera` deferred. | MIR has no expression-valued control flow for the supported subset. |
| 5A | Alternate-exit surface | Add the typed alternate-exit glyph contract with `→ Success ⇥ Error` through lexing, parsing, HIR, and semantic signatures. | `radix check` accepts explicit failable signatures and rejects `iace` where no alternate exit is declared. |
| 5B | Alternate-exit MIR lowering | Lower `iace` and `mori` through the new function contract into explicit MIR exits. | MIR distinguishes normal `return` from recoverable `return_error` without Rust `Result` syntax. |
| 5C | Structured `cape` handling surface | Replace `tempta` as the canonical local handler model by attaching `cape` to structured statements and conditional arms. | `fac`, `dum`, `si`, `sin`, and `secus` can consume local `iace` and failable-call alternate exits without dynamic exception search. |
| 6A | Aggregate, option, and runtime MIR contract | Define the shared MIR node vocabulary for aggregate payloads, projections, option/null operations, runtime intrinsics, and provider identity. | Phase 6B and Phase 7 can implement against one stable target-neutral MIR contract. |
| 6B | Aggregate and option lowering | Represent structs, enums, tuples, arrays, maps, option/null, optional chain, and non-null assertion using the Phase 6A contract. | MIR uses explicit construction/projection/runtime operations; no high-level optional-chain nodes remain for the supported subset. |
| 7 | Runtime intrinsic boundary | Define and lower target-neutral intrinsics for printing, string formatting, collection operations, conversions, and stdlib-backed calls using the Phase 6A contract. | MIR references runtime/provider operations without Rust module paths or Cargo details. |
| 8 | MIR validation | Add validation for block termination, type presence, operand compatibility, def-use sanity, and unresolved placeholders. | Invalid MIR is rejected before any backend sees it; diagnostics point back to source spans where possible. |
| 9 | Rust backend vertical slice | Add a MIR-to-Rust backend behind an explicit experimental path for the supported subset. | Selected examples generate Rust from MIR and compile/run through Cargo with behavior matching the existing backend. |
| 10 | Rust backend migration | Move stable lowering responsibilities from HIR-to-Rust into MIR where proven. | Existing Rust backend tests pass with MIR enabled for selected constructs; fallback remains for unported constructs. |
| 11 | WASM readiness slice | Use MIR to scope a minimal WASM backend or WASM text/object experiment. | A primitive `incipit` program can lower from MIR to WASM-oriented output without changing HIR semantics. |
| 12 | Native readiness review | Decide whether Cranelift/native is justified based on MIR completeness, runtime ABI, and WASM evidence. | Review produces a separate factory plan for Cranelift/native or explicitly defers it. |

## Phase Details

### Phase 0: Baseline and Invariants

Steps:

- Inspect `git status --short`.
- Capture representative HIR and Rust output for:
  - primitive arithmetic,
  - function call,
  - `si` expression,
  - loop,
  - option/null,
  - string formatting,
  - simple struct construction,
  - `iace` / `tempta`,
  - `nota`.
- Identify Rust backend behaviors that are semantic lowering rather than Rust emission.
- Decide the first MIR subset and explicitly defer the rest.
- Create `docs/factory/mir-layer/ledger.md` when implementation starts.

Checkpoint:

- The implementation has a factual baseline.
- First subset is small enough for a vertical slice.
- No source behavior changed.

### Phase 1: MIR Data Model

Steps:

- Add `crates/radix/src/mir`.
- Define IDs for MIR functions, blocks, locals, temporaries, and values.
- Represent MIR types with a small MIR-owned wrapper around semantic `TypeId`.
- Leave room in that wrapper for later ABI/layout information without deciding runtime representation in Phase 1.
- Define block and terminator structures.
- Add deterministic debug rendering for test snapshots.

Checkpoint:

- The MIR model compiles.
- Unit tests prove stable formatting and basic construction.
- No lowering or target behavior changes.

### Phase 2: MIR Inspection Surface

Steps:

- Add an internal lowering entry point from `AnalyzedUnit`.
- Lower top-level Faber functions into `MirFunction` shells.
- Treat MIR as a library-friendly collection of functions; do not require or privilege a `main` entry point.
- If a source file has an executable entry block, lower it as an ordinary synthetic `MirFunction` rather than a special MIR root.
- Add `radix mir` as the compiler-developer inspection command.
- Add test helpers for deterministic MIR dump output.
- Keep output deterministic and low-noise.
- Reject unsupported HIR nodes with explicit unsupported-MIR diagnostics.

Checkpoint:

- A simple function-only `.fab` file can produce MIR inspection output.
- A simple entry-block `.fab` file can produce MIR inspection output without introducing a special `main` concept.
- Unsupported constructs fail clearly rather than silently lowering incorrectly.

### Phase 3: Primitive Expression Lowering

Steps:

- Lower literals into MIR constants.
- Lower paths into MIR value/place references.
- Lower local declarations and assignments.
- Lower primitive unary and binary operations.
- Lower direct function calls where the callee is a resolved `DefId`.
- Lower explicit returns.

Checkpoint:

- Focused MIR tests cover primitive functions.
- Every lowered expression has an available type.
- No backend consumes MIR yet.

### Phase 4: Control-Flow Normalization

Steps:

- Lower Faber `si` into MIR branch terminators and join blocks.
- Lower block expressions into explicit temporaries.
- Lower `dum` into block cycles.
- Lower `rumpe` and `perge` with loop target tracking.
- Keep `itera` deferred until collection/range/runtime semantics are explicit.

Checkpoint:

- MIR for supported constructs has explicit blocks and terminators.
- There are no nested Faber `si` / `dum` expression-control nodes in MIR for the supported subset.

### Phase 5A: Alternate-Exit Surface

Steps:

- Add the alternate-exit glyph token for `⇥`.
- Extend function declarations from `→ Success` to `→ Success ⇥ Error`.
- Extend function types from `(A) → B` to `(A) → B ⇥ E`.
- Carry the optional error type through AST, HIR, semantic function signatures, and developer inspection surfaces.
- Typecheck `iace expr` against the current function's declared alternate-exit type.
- Reject `iace` in functions without `⇥ Error`, except where a later local handler explicitly consumes it.
- Do not trace `iace` up the dynamic call stack; failure propagation must be explicit at call sites.
- Reject failable calls in ordinary expression position until caller-side propagation/handling syntax is defined.
- Keep existing target codegen behavior unchanged for existing programs; new failable signatures may be accepted by `check` but should fail clearly if emitted by a backend that does not support them yet.

Checkpoint:

- `radix check` accepts a function shaped like `functio divide(...) → numerus ⇥ textus`.
- `radix check` rejects `iace` in non-failable functions.
- Semantic function types preserve both normal and alternate-exit types.
- No MIR lowering behavior depends on the new surface yet.

### Phase 5B: Alternate-Exit MIR Lowering

Steps:

- Add `error_ty: Option<MirType>` to `MirFunction`.
- Render the alternate-exit type deterministically in MIR dumps.
- Lower `iace expr` to `MirTerminatorKind::ReturnError`.
- Type-preserve the `iace` operand the same way `redde` operands are type-preserved.
- Lower `mori expr` to a target-neutral fatal operation followed by `Unreachable`.
- Keep `tempta`, `cape`, `demum`, and failable-call propagation deferred.
- Treat `tempta` as a legacy local-handler surface that Phase 5C will remove or replace.
- Do not deepen `tempta` support or lower inside `tempta` bodies during Phase 5B.
- Preserve enough source span information for useful diagnostics.

Checkpoint:

- MIR for failable functions distinguishes normal `return` from recoverable `return_error`.
- `iace` requires a declared alternate-exit type and emits a typed `return_error` operand.
- `mori` is fatal and not catchable.
- Existing Rust behavior remains unchanged unless explicitly using a later experimental MIR backend.

### Phase 5C: Structured `cape` Handling Surface

Steps:

- Define `cape` as an optional handler attached to structured statements and conditional arms, not arbitrary bare blocks.
- Make `fac { ... } cape err { ... }` the canonical one-shot local error boundary.
- Preserve existing `dum ... cape` and `fac ... cape` grammar intent while tightening semantics.
- Extend `secus` so `secus { ... } cape err { ... }` is a valid arm-scoped handler.
- Keep `si` and `sin` handlers arm-scoped.
- Reject bare `{ ... } cape err { ... }`.
- Remove `tempta` from the canonical grammar or make it fail with a direct migration diagnostic to `fac { ... } cape`.
- Keep `demum` cleanup/finally semantics deferred.
- Lower handled local alternate exits to explicit MIR handler edges or equivalent target-neutral control flow.
- Allow failable calls inside active lexical `cape` boundaries and lower their alternate exits to the handler edge.
- Infer a conservative single handler error type from caught alternate exits; defer broad union synthesis if needed.
- Ensure handler fallthrough rejoins after the handled construct.
- Ensure `dum ... cape` exits the loop after handler fallthrough rather than resuming the loop.
- Preserve `mori` as fatal and not catchable.

Checkpoint:

- `fac { iace ... } cape err { ... }` is accepted without requiring the enclosing function to declare `⇥`.
- `iace` without a local handler and without an enclosing `⇥` remains rejected.
- Failable calls inside a local handler boundary are consumed locally; failable calls outside a handler or propagation syntax remain rejected.
- `dum`, `si`, `sin`, and `secus` handlers parse and scope independently.
- `tempta` is no longer treated as the canonical local handler surface.
- MIR makes local handler flow explicit without dynamic stack search.

### Phase 6A: Aggregate, Option, and Runtime MIR Contract

Steps:

- Define aggregate construction payloads that preserve tuple/list/set order, struct field names, map keys, and enum variant identity.
- Tighten projection/index representation so Phase 6B can lower field and index access without fabricating unstable value IDs.
- Define option/null MIR operations or control-flow forms for nil wrapping, nil tests, unwrap/assertion, optional chain, and `vel`.
- Define structured runtime intrinsic payloads for diagnostics, string formatting, conversions, collection operations, and provider calls.
- Keep provider identity separate from target linkage and `@ verte` target translation strings.
- Update deterministic MIR dump formatting for each new contract shape.
- Add MIR node/dump tests for aggregate payloads, projections, option operations, runtime intrinsics, and provider identity.
- Keep broad HIR lowering fail-closed.

Checkpoint:

- MIR has one shared target-neutral contract that Phase 6B and Phase 7 can both consume.
- The contract can represent named struct fields, keyed map entries, operand-friendly index access, nullable operations, and structured runtime/provider identity.
- No backend consumes the new MIR shapes.

### Phase 6B: Aggregate and Option Lowering

Steps:

- Lower struct construction and field access using the Phase 6A aggregate/projection contract.
- Represent enum variants and pattern-match inputs where Phase 6A supports their payload shape.
- Lower tuple, list, map, and set literals using explicit aggregate payloads or Phase 6A-approved runtime operations.
- Represent `T ∪ nihil` with the Phase 6A option operation model.
- Lower optional chain and non-null assertion.
- Lower `vel` / coalesce according to Faber nullable semantics, not target truthiness.
- Extend assignment lowering for supported field/index places.
- Keep runtime-backed collection methods and stdlib/provider calls deferred to Phase 7.
- Reject unsupported aggregate shapes with clear MIR-lowering diagnostics.

Checkpoint:

- MIR expresses aggregate and optional behavior explicitly.
- Unsupported aggregate/option shapes produce clear MIR-lowering diagnostics.
- No high-level optional-chain nodes remain for the supported subset.

### Phase 7: Runtime Intrinsic Boundary

Steps:

- Consume the Phase 6A runtime/provider MIR contract.
- Lower HIR runtime-backed operations into target-neutral intrinsic/provider calls such as:
  - `nota`,
  - string formatting,
  - numeric/text conversions,
  - collection append/index/length,
  - stdlib provider calls.
- Keep provider identity separate from target linkage.
- Avoid Rust module paths, Cargo dependency specs, WASM imports, and native object names in MIR.
- Keep aggregate/option construction semantics owned by Phase 6B except where an operation is explicitly runtime-backed.

Checkpoint:

- MIR can represent runtime-backed operations without target-specific strings.
- Rust/WASM/native linkage decisions remain backend-owned.

### Phase 8: MIR Validation

Steps:

- Validate every block has one terminator.
- Validate used values have definitions.
- Validate required types exist.
- Validate operand types match operation contracts.
- Validate unsupported placeholders do not reach backend lowering.
- Add tests for deliberately invalid MIR.

Checkpoint:

- MIR validation fails closed.
- Backends can assume validated MIR invariants.

### Phase 9: Rust Backend Vertical Slice

Steps:

- Add an experimental MIR-to-Rust emitter.
- Start with the primitive subset from Phases 3 and 4.
- Compile generated Rust through existing package or test helpers.
- Compare behavior against the current HIR-to-Rust backend.

Checkpoint:

- Selected examples compile and run through MIR-backed Rust.
- Existing HIR-to-Rust remains available and is still the default unless explicitly changed.

### Phase 10: Rust Backend Migration

Steps:

- Move proven semantic lowering out of Rust codegen and into MIR.
- Keep Rust codegen focused on Rust syntax, imports, and target conventions.
- Port constructs one coherent group at a time.
- Delete old duplicated lowering only after tests prove parity.

Checkpoint:

- Existing Rust backend tests pass.
- MIR-backed constructs do not regress package builds.
- Remaining HIR-direct paths are documented as unported.

### Phase 11: WASM Readiness Slice

Steps:

- Choose a minimal WASM output strategy:
  - WAT text for inspection,
  - `wasm-encoder`,
  - or another small Rust-native library.
- Lower a tiny primitive MIR subset to WASM-oriented output.
- Stub or define imports for runtime operations such as `nota`.

Checkpoint:

- WASM work consumes MIR, not HIR.
- Any missing runtime ABI decisions are recorded as blockers.

### Phase 12: Native Readiness Review

Steps:

- Evaluate MIR coverage against native needs:
  - data layout,
  - calling convention,
  - memory management,
  - runtime ABI,
  - linking,
  - debug/diagnostic story.
- Decide whether Cranelift is the right first native backend.
- Produce a separate native factory plan if justified.

Checkpoint:

- Native work either has a grounded plan or is explicitly deferred.
- The MIR layer remains useful even if native waits.

## Validation Strategy

Use layered validation rather than waiting for final executable output:

- MIR unit tests for construction and validation.
- MIR lowering tests for selected HIR constructs.
- Snapshot-style tests for deterministic MIR dump output.
- Existing Rust codegen tests for behavior parity.
- Package-level Cargo tests only after the MIR-to-Rust vertical slice exists.
- `./scripta/ci` before marking a phase complete.

## Resolved Design Questions

- MIR will use a MIR-owned type wrapper around semantic `TypeId` in the first implementation. A richer `MirType` / representation layer is deferred until runtime layout decisions are concrete.
- Recoverable failure will be represented as control flow through explicit error exits or error edges. Rust `Result<T, String>` remains a Rust backend rendering choice, not a MIR semantic primitive.
- Local recoverable-failure handling will use lexical `cape` attachment on structured statements, with `fac { ... } cape` as the canonical one-shot boundary. `tempta` is not the future core handler model.
- `cape` attaches by grammar production, not to the nearest arbitrary block token.
- MIR will preserve Faber-level ownership and mutability facts such as parameter mode, receiver mode, local mutability, assignability, and addressable places. It will not carry Rust lifetimes or Rust borrow-checker internals.
- Early MIR will not include meaningful `drop`, `retain`, `release`, or `free` operations. Storage lifetime markers may be added later, but only when they serve a concrete validation or backend need.
- MIR inspection will start as `radix mir`, matching the existing compiler-developer command surface.
- MIR lowering will be target-neutral. If `MirConfig` exists early, it should only control lowering/debug policy such as unsupported-node handling, not target-specific language semantics.

## Deferred Questions

- Exact ABI/layout representation for strings, lists, maps, options, structs, enums, closures, and runtime values.
- Whether MIR needs a distinct layout-aware `MirType` after the first semantic-type wrapper.
- Whether and when MIR should lower to SSA.
- Whether storage lifetime markers should become mandatory before WASM or native work.
- Whether MIR validation should grow a formal ownership/borrow layer independent of Rust borrow analysis.
- Whether future backend profiles such as `Portable`, `RustBridge`, or `WasmSubset` are useful after canonical MIR exists.
- Whether a cleanup construct replaces `demum`, and how it interacts with future resource-lifetime rules.

## Deferred Native Questions

Direct native codegen must wait until MIR can answer or expose:

- concrete layout for strings, lists, maps, options, structs, and enums,
- calling convention for Faber functions and runtime functions,
- memory ownership and destruction rules,
- runtime library artifact shape,
- linker invocation strategy,
- platform-specific packaging,
- debug and source-map equivalent story.

These are not MIR Phase 1 problems, but MIR should avoid choices that make them impossible.

---

*This plan treats MIR as the next backend layer, not as a synonym for native codegen. Bene currit when Rust remains the correctness gate while MIR becomes the shared execution contract.*
