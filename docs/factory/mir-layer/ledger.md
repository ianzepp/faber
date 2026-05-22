# MIR Layer Factory Ledger

## Phase 0 Baseline

Status: complete as a factual baseline.

Worktree at phase start:

- `git status --short` produced no tracked or untracked changes.

Representative commands:

- `cargo run -q -p radix --bin radix -- hir <example>`
- `cargo run -q -p radix --bin radix -- emit -t rust <example>`

Representative examples inspected:

| Construct | Example | HIR surface | Rust surface |
| --- | --- | --- | --- |
| Primitive arithmetic | `examples/exempla/binarius/binarius.fab` | `success: true`, `items: 1` | arithmetic and comparison lower directly to Rust expressions such as `(10 + 5)`, `(flags & mask)`, and chained boolean expressions |
| Function call | `examples/exempla/functio/functio.fab` | `success: true`, `items: 4` | top-level `functio` lowers to Rust `fn`; calls lower directly by resolved names |
| `si` expression/statement | `examples/exempla/si/secus.fab` | `success: true`, `items: 0` | block `si` lowers to nested Rust `if`; expression-valued branches in arithmetic examples lower to Rust `if` expressions |
| Loop | `examples/exempla/dum/dum.fab` | `success: true`, `items: 0` | `dum` lowers to Rust `while`; mutable locals become `let mut` |
| Option/null | `examples/exempla/optionalis/optionalis.fab` | `success: true`, `items: 2` | voluntary fields lower to `Option<T>`; optional chains lower to chained `as_ref().map(...)`; `vel` lowers to `unwrap_or(...)` |
| String formatting | `examples/exempla/scriptum/scriptum.fab` | `success: true`, `items: 0` | string-template application lowers to `format!(...)` |
| Struct construction | `examples/exempla/genus/creo.fab` | `success: true`, `items: 2` | `genus` lowers to Rust `struct` plus `impl`; object construction lowers to struct literals |
| `iace` / `tempta` | `examples/exempla/tempta/tempta.fab` | `success: true`, `items: 0` | Rust target rejects `tempta`, `cape`, and `iace` before emission |
| `nota` | `examples/exempla/nota/nota.fab` | `success: true`, `items: 0` | diagnostic output lowers to `println!`, including multi-argument formatting |

Current HIR inspection limitation:

- The `radix hir` command is useful for phase reachability and top-level shape, but it does not currently print expression-level HIR or typed expression details. Phase 2 should make `radix mir` the deterministic inspection surface for execution-shaped lowering instead of expanding phase 1.

Rust backend semantic-lowering responsibilities observed:

- Chooses Rust control-flow constructs for Faber `si`, `dum`, and related expression forms.
- Chooses `Option<T>` representation for nullable or voluntary slots.
- Lowers optional chain and `vel` behavior into Rust combinators.
- Lowers diagnostic `nota` into `println!`.
- Lowers string-template application into `format!`.
- Lowers struct declarations, object literals, and methods into Rust `struct` and `impl` shapes.
- Applies Rust-target policy rejection for unsupported failure-flow constructs before backend emission.

First MIR subset:

- Include functions, entry-like function bodies, locals, temporaries, constants, simple assignments, direct calls, primitive unary/binary operations, simple aggregate construction, explicit blocks, explicit `goto`/`branch`/`return` terminators, and target-neutral runtime/intrinsic call placeholders.

Explicit deferrals:

- HIR lowering.
- CLI inspection command.
- MIR validation beyond construction-time data shape.
- Rust backend consumption of MIR.
- Full failure flow, pattern matching, optional-chain normalization, collection transforms, package layout, ABI/layout, memory-management operations, SSA, and optimization.

## Phase 1 Baseline

Status: complete.

Rules for phase 1:

- Add model and deterministic rendering only.
- Do not lower from HIR.
- Do not add `radix mir`.
- Do not alter existing target codegen behavior.

Implemented artifacts:

- `crates/radix/src/mir/mod.rs`
- `crates/radix/src/mir/nodes.rs`
- `crates/radix/src/mir/dump.rs`
- `crates/radix/src/mir/nodes_test.rs`

Model coverage:

- MIR IDs for functions, blocks, locals, temporaries, values, and future layout metadata.
- `MirType` wraps semantic `TypeId` and reserves an optional layout slot.
- Program, function, parameter, local, temporary, block, statement, terminator, value, operand, place, intrinsic, runtime call, and aggregate data structures.
- Deterministic text rendering through `dump_program`.

Verification:

- `cargo test -p radix mir` passed.
- `cargo test -p radix` passed: 318 unit tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- Verification audit on 2026-05-22 reran `cargo test -p radix mir`, `cargo test -p radix`, and `./scripta/ci`; all passed after applying `cargo fmt --all`.

Behavior boundary:

- No HIR lowering code was added.
- No CLI command was added.
- No codegen backend consumes MIR.
- Existing target behavior remains on the HIR-to-codegen path.

## Phase 2 Baseline

Status: complete.

Implemented artifacts:

- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/lower_test.rs`
- `radix mir` command dispatch in `crates/radix/src/bin/radix.rs`
- `cmd_mir` command implementation in `crates/radix/src/tool.rs`

Lowering subset:

- Top-level Faber `functio` items lower into `MirFunction` shells.
- Function parameters lower into MIR parameters and matching MIR local slots with deterministic local IDs.
- Empty concrete function bodies lower as one `bb0` with `return`.
- Explicit no-value `redde` in an otherwise trivial function body lowers as one `bb0` with `return`.
- Empty executable entry blocks lower as ordinary synthetic `MirFunction` values with `source: None`; MIR still has no special root, `main`, or `MirEntry` node.
- Function-only source files can produce MIR output.

Unsupported diagnostics:

- Non-empty function bodies fail with `unsupported MIR lowering in phase 2`.
- Non-empty entry blocks fail with `unsupported MIR lowering in phase 2`.
- Unsupported top-level HIR items, including structs, fail explicitly.
- CLI program-specific MIR lowering is rejected for this phase.

Manual checkpoint output:

```text
$ printf 'functio saluta() {}\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#5 {
  bb0:
    return
}
```

```text
$ printf 'incipit {}\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#5 {
  bb0:
    return
}
```

```text
$ printf 'incipit { nota "salve" }\n' | cargo run -q -p radix --bin radix -- mir -
error: <stdin>:1:9: unsupported MIR lowering in phase 2: non-empty entry blocks before primitive expression lowering
```

Verification:

- `cargo test -p radix mir` passed, including tests for function shells, params as MIR locals, explicit no-value `redde`, empty entry blocks, unsupported top-level items, and unsupported non-empty entry blocks.
- `cargo test -p radix tool` passed, including a deterministic MIR output assertion for a tiny valid source.
- `cargo test -p radix` passed: 327 unit tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Existing HIR-to-codegen behavior remains unchanged.
- No target backend consumes MIR.
- Primitive expression lowering remains deferred to phase 3.

## Phase 3 Baseline

Status: complete.

Implemented commit:

- `8a3104ef` - `Implement MIR primitive expression lowering`

Implemented artifacts:

- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/lower_test.rs`

Lowering subset:

- Straight-line `functio` bodies lower into typed MIR statements and terminators.
- Primitive literals lower into MIR constants for `numerus`, `fractus`, `textus`, `bivalens`, and `nihil`.
- Bare constant `redde` operands materialize into typed temporaries so return operands preserve `MirType`.
- Function parameters lower as MIR parameters plus matching immutable MIR locals.
- `fixum` and `varia` declarations with initializers create MIR locals before emitting initializer assignments.
- Parameter and local reads lower to `MirOperand::Place`.
- Local reassignment emits assignment to an existing `MirPlace`.
- Primitive unary and binary expressions materialize into fresh typed temporaries.
- Direct calls whose callee is a resolved path lower through `MirStmtKind::Call` with `MirCallee::Definition(DefId)`.
- Vacuum direct calls emit calls without destinations.
- Explicit `redde expr` and no-value `redde` lower to MIR return terminators.

Unsupported diagnostics:

- `si` and `dum` fail with explicit control-flow MIR-lowering diagnostics.
- Diagnostic verbs such as `nota` fail with explicit runtime-intrinsic MIR-lowering diagnostics.
- Assignment targets that are not local places fail before lowering an arbitrary expression as a destination.
- Other out-of-scope constructs remain fail-closed through explicit unsupported-MIR diagnostics.

Manual checkpoint output:

```text
$ printf 'functio computa() → numerus { varia numerus x ← 1 x ← x + 2 redde x }\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#1 {
  locals:
    var _0: ty#1
  temps:
    %0: ty#1
  bb0:
    _0 = const int 1: ty#1
    %0 = _0 + const int 2: ty#1
    _0 = %0: ty#1
    return _0
}
```

```text
$ printf 'functio duplex(numerus n) → numerus { redde n * 2 } functio usa() → numerus { redde duplex(4) }\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
  temps:
    %0: ty#1
  bb0:
    %0 = _0 * const int 2: ty#1
    return %0
}

function f1 -> ty#1 {
  temps:
    %0: ty#1
  bb0:
    %0 = call def#0(const int 4)
    return %0
}
```

Verification:

- `cargo test -p radix mir` passed: 22 tests passed.
- `cargo test -p radix` passed: 337 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Existing HIR-to-codegen behavior remains unchanged.
- No target backend consumes MIR.
- Control-flow normalization remains deferred to phase 4.

## Phase 4 Baseline

Status: complete.

Implemented artifacts:

- `crates/radix/src/mir/lower.rs`
- `crates/radix/src/mir/lower_test.rs`
- `crates/radix/src/tool_test.rs`

Lowering subset:

- MIR lowering now uses an internal open-block builder with explicit current-block tracking.
- Finished MIR still emits only complete `MirBlock` values with explicit terminators.
- Statement-shaped `si` lowers to `Branch` terminators plus then/else/join blocks.
- Expression-valued `si` lowers by destination-passing into a shared local or temporary before joining.
- `dum` lowers to start-to-condition, condition/body/after block cycles.
- `rumpe` lowers to the active loop exit block.
- `perge` lowers to the active loop condition/repeat block.
- `redde` inside nested `si` and `dum` bodies seals the current block with `Return`.
- Branch arms closed by `redde`, `rumpe`, or `perge` do not emit spurious `goto` edges to joins.
- Nested blocks lower inline for the supported Phase 4 subset.

Unsupported diagnostics:

- `itera` fails with an explicit iterator MIR-lowering diagnostic.
- `discerne` fails with an explicit switch MIR-lowering diagnostic.
- `tempta` fails with an explicit error-flow MIR-lowering diagnostic.
- Diagnostic verbs and other out-of-scope constructs remain fail-closed through explicit unsupported-MIR diagnostics.

Manual checkpoint output:

```text
$ printf 'functio signum(numerus n) → numerus { si n > 0 ergo redde n redde 0 }\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
  temps:
    %0: ty#3
    %1: ty#1
  bb0:
    %0 = _0 > const int 0: ty#3
    branch %0 bb1 bb2
  bb1:
    return _0
  bb2:
    %1 = const int 0: ty#1
    return %1
}
```

```text
$ printf 'functio primus(numerus n) → numerus { varia numerus i ← 0 dum i < n { i ← i + 1 si i < 3 ergo perge rumpe } redde i }\n' | cargo run -q -p radix --bin radix -- mir -
function f0 -> ty#1 {
  params:
    _0: ty#1
  locals:
    let _0: ty#1
    var _1: ty#1
  temps:
    %0: ty#3
    %1: ty#1
    %2: ty#3
  bb0:
    _1 = const int 0: ty#1
    goto bb1
  bb1:
    %0 = _1 < _0: ty#3
    branch %0 bb2 bb3
  bb2:
    %1 = _1 + const int 1: ty#1
    _1 = %1: ty#1
    %2 = _1 < const int 3: ty#3
    branch %2 bb4 bb5
  bb3:
    return _1
  bb4:
    goto bb1
  bb5:
    goto bb3
}
```

Verification:

- `cargo test -p radix mir` passed: 28 tests passed.
- `cargo test -p radix` passed: 343 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Existing HIR-to-codegen behavior remains unchanged.
- No target backend consumes MIR.
- Failure-flow lowering remains deferred to phase 5.

## Phase 5A Baseline

Status: complete.

Implemented artifacts:

- `TokenKind::ExitArrow` and lexer support for `⇥`.
- AST function declarations, pactum methods, and function type expressions now carry optional alternate-exit types.
- HIR functions, interface methods, and semantic `FuncSig` now preserve optional alternate-exit `TypeId`s.
- Typechecking tracks the current function alternate-exit type and checks `iace` values against it.
- Unhandled `iace` is rejected when no `⇥` contract or local `tempta`/`cape` handler is present.
- Failable calls are rejected in ordinary expression position until explicit caller handling or propagation syntax exists.
- Faber inspection/codegen output renders `→ Success ⇥ Error` signatures.

Surface examples:

```fab
functio divide(numerus a, numerus b) → numerus ⇥ textus {
    si b = 0 ergo iace "division by zero"
    redde a / b
}
```

```fab
typus Op = (numerus) → numerus ⇥ textus
```

Verification:

- `cargo test -p radix` passed: 352 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed after `cargo fmt`.

Behavior boundary:

- MIR lowering of `iace` and alternate-exit function signatures remains deferred to Phase 5B.
- Caller-side propagation and explicit handler syntax remain deferred.
- Existing backend lowering paths do not consume MIR and were not converted to typed recoverable failure lowering in this phase.

## Phase 5B Baseline

Status: complete.

Implemented artifacts:

- `MirFunction` now carries `error_ty: Option<MirType>` alongside unchanged `return_ty`.
- `radix mir` renders failable function headers as `function fN -> ty#S ⇥ ty#E`.
- HIR function `err_ty` from Phase 5A threads into MIR function lowering.
- `HirExprKind::Throw` lowers to `MirTerminatorKind::ReturnError`.
- Constant and value `iace` operands are materialized into typed temporaries using the same transfer-operand discipline as `redde`.
- Fabricated HIR with `iace` but no function alternate-exit type fails during MIR lowering.
- `HirExprKind::Panic` lowers to a target-neutral `panic` runtime call with `numquam` return type and seals the block with `Unreachable`.
- `tempta` remains fail-closed as a deferred legacy local-handler surface; MIR lowering does not descend into its body to recover nested `iace`.

Representative MIR shape:

```text
function f0 -> ty#1 ⇥ ty#0 {
  temps:
    %0: ty#0
  bb0:
    %0 = const string sym#N: ty#0
    return_error %0
}
```

Verification:

- `cargo test -p radix mir` passed: 34 tests passed.
- `cargo test -p radix` passed: 358 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- Verification audit on 2026-05-22 reran `cargo test -p radix mir`, `cargo test -p radix`, and `./scripta/ci`; all passed.
- Manual `radix mir` probes confirmed failable functions render `⇥`, `iace` lowers to `return_error`, `mori` lowers to `runtime panic(...) -> numquam` followed by `unreachable`, `tempta` fails closed with the Phase 5C diagnostic, and unhandled failable calls are rejected before MIR.

Behavior boundary:

- `tempta`, `cape`, and `demum` lowering remains out of scope.
- Caller-side propagation and local handling remain deferred to Phase 5C or later.
- No `TryCall` or failable-call control-flow terminator was introduced.
- No target backend consumes MIR.

## Phase 5B/5C Planning Update

Status: planned.

Decision captured:

- Phase 5B remains narrowly scoped to function-level alternate-exit MIR lowering.
- Phase 5B must lower declared `iace` to `return_error` and `mori` to fatal unreachable flow.
- Phase 5B must not harden `tempta` as the local handling primitive.
- Phase 5C becomes the structured local handling phase.
- `cape` attaches to structured statements or conditional arms by grammar production, not to the nearest arbitrary block.
- `fac { ... } cape err { ... }` is the canonical one-shot local handler boundary.
- Failable calls inside an active lexical `cape` boundary are locally consumed by that handler; failable calls outside a handler or propagation syntax remain rejected.
- Handler error binding starts with a conservative single inferred error type for all caught alternate exits.
- `dum ... cape` is statement-scoped loop handling; handler fallthrough exits the loop rather than resuming it.
- `si` / `sin` / `secus` handlers are arm-scoped.
- Bare `{ ... } cape err { ... }` is rejected.
- `tempta` should be removed from the canonical source surface or rejected with a migration diagnostic to `fac { ... } cape`.
- `demum` cleanup/finally behavior remains deferred.

Planning artifacts:

- `docs/factory/mir-layer/phase-5b-delivery.md` tightened around the no-`tempta` hardening boundary.
- `docs/factory/mir-layer/phase-5c-delivery.md` added for structured `cape` handling.
- `docs/factory/mir-layer/plan.md` updated to include Phase 5C.

## Phase 5C Baseline

Status: complete.

Implemented artifacts:

- `EBNF.md` and `docs/grammatica/errores.md` now describe structured `cape` attachment and reject `tempta` as legacy syntax.
- `secus { ... } cape err { ... }` now parses as an arm-scoped handler.
- Bare `{ ... } cape err { ... }` remains rejected.
- Parser diagnostics reject `tempta` with a migration note to `fac { ... } cape`.
- HIR now carries explicit `HirCape` metadata and structured `Handled` expressions instead of lowering `fac ... cape` through `Tempta`.
- Typechecking distinguishes function alternate-exit sinks from local handler sinks.
- Local handlers accept `iace` without requiring the enclosing function to declare `⇥`.
- Local handlers accept failable direct calls and infer the handler binding type from the caught alternate-exit type.
- Failable calls outside local handlers remain rejected until propagation syntax is designed.
- MIR lowers handled `iace` by assigning the payload into the handler binding local and jumping to the handler block.
- MIR lowers handled failable direct calls through an explicit `try_call` terminator with success and error edges.
- `mori` remains fatal and does not route through local handlers.

Representative MIR shapes:

```text
%0 = const string sym#N: ty#0
_0 = %0: ty#0
goto bb_handler
```

```text
%0 = try_call def#0() ok bb_success error _0 -> bb_handler
```

Validation:

- `cargo test -p radix cape` passed: 11 tests passed.
- `cargo test -p radix mir` passed: 37 tests passed.
- `cargo test -p radix` passed: 366 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.
- Verification audit on 2026-05-22 reran `cargo test -p radix cape`, `cargo test -p radix mir`, `cargo test -p radix`, and `./scripta/ci`; all passed.
- Manual CLI probes confirmed `fac { iace ... } cape` lowers without `return_error`, handled failable calls lower to `try_call`, `si`/`sin`/`secus` handlers parse and lower independently, `dum ... cape` exits to the post-loop path after handler fallthrough, bare-block `cape` is rejected, legacy `tempta` is rejected with the migration diagnostic, and unhandled failable calls remain rejected.
- The verification pass fixed `EBNF.md` so the top-level `statement` production lists `facBlockStmt`, matching the now-canonical `fac { ... } cape` boundary.

Behavior boundary:

- Handler binding inference remains conservative: one assignable caught error type per handler, with `ignotum` for handlers that catch no typed alternate exit.
- General propagation syntax remains out of scope.
- `demum` cleanup/finally semantics remain deferred.
- Target backends still do not consume MIR. Structured `cape` codegen remains out of scope except for existing target rejection/fallback behavior.

## Phase 6 Split Planning Update

Status: planned.

Decision captured:

- Former Phase 6 is split into Phase 6A and Phase 6B.
- Phase 6A owns the shared MIR contract for aggregate payloads, projections, option/null operations, runtime intrinsics, and provider identity.
- Phase 6B owns aggregate and option lowering after the Phase 6A contract is in place.
- Phase 7 remains the runtime intrinsic boundary, but now consumes the Phase 6A contract rather than inventing its own runtime/provider shapes.
- Current delivery sequencing starts Phase 7 only after Phase 6B completion, so runtime intrinsic lowering consumes both the Phase 6A contract and the Phase 6B aggregate/option lowering baseline.

Planning artifacts:

- `docs/factory/mir-layer/phase-6a-delivery.md` added for the shared MIR contract.
- `docs/factory/mir-layer/phase-6b-delivery.md` added for aggregate/option lowering.
- `docs/factory/mir-layer/phase-7-delivery.md` added for runtime intrinsic/provider lowering after the contract.
- `docs/factory/mir-layer/plan.md` updated to replace Phase 6 with Phase 6A and Phase 6B and to clarify Phase 7's dependency on Phase 6A.

## Phase 6A Baseline

Status: complete.

Implemented artifacts:

- `MirStmtKind::Construct` now carries a single `MirAggregate` payload instead of a positional field list.
- `MirAggregate` now includes ordered, named, or keyed payload fields.
- Struct construction can preserve `Symbol -> operand` field names.
- Map construction can preserve key/value operand pairs.
- `MirProjection::Index` now carries `MirOperand` instead of `MirValueId`, so index projections can use the operand shape Phase 6B lowering will naturally have.
- `MirValueKind::Option` and `MirOptionOp` define explicit option/null operations for none/some wrapping, nil checks, unwrap, coalesce, and optional chain links.
- `MirIntrinsic` now carries structured target-neutral identity for diagnostics, string formatting, conversions, collection operations, panic, and provider calls.
- Provider identity is represented as Faber/stdlib symbols, separate from target linkage or `@ verte` translation strings.
- Existing `mori` lowering now emits the structured panic runtime intrinsic through the updated runtime-call contract.
- Broad aggregate, option, diagnostic, formatting, conversion, collection, and provider HIR lowering remains fail-closed for Phase 6B/7.

Representative MIR dump shapes:

```text
_0 = construct struct def#7: ty#0 {sym#11: const string sym#12, sym#13: const int 36}
_1 = construct map: ty#1 {const string sym#14 => const int 1}
return _1[const string sym#14]
```

```text
_0 = option some(const string sym#9): ty#0
_1 = option chain(_0, .sym#10): ty#0
_2 = option coalesce(_1, const string sym#11): ty#0
```

```text
runtime diagnostic mone(const string sym#20) -> ty#5
_0 = runtime format_string template sym#21(const string sym#22) -> ty#0
_1 = runtime convert runtime -> ty#1 fallback const int 0(_0) -> ty#1
_1 = runtime collection length(_0) -> ty#1
_0 = runtime provider sym#30/sym#31::sym#32() -> ty#0
```

Validation:

- `cargo test -p radix mir` passed: 40 tests passed.
- `cargo test -p radix` passed: 369 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Phase 6A is contract-only and does not lower broad aggregate, option, or runtime-backed HIR constructs.
- No target backend consumes MIR.
- Aggregate/option lowering remains Phase 6B.
- Runtime/provider lowering remains Phase 7.

## Phase 6B Baseline

Status: complete.

Implemented artifacts:

- Ordered aggregate payloads now distinguish ordinary operands from spread operands.
- `radix mir` lowers list/array literals, including `sparge` spread elements, to ordered aggregate construction.
- `radix mir` lowers object-to-struct construction to named aggregate fields, including omitted fields with HIR-backed defaults.
- `radix mir` lowers object-to-map construction to keyed aggregate entries.
- `radix mir` lowers set construction from array-like `⇢ copia<T>` sources to ordered set aggregates.
- `radix mir` lowers enum variant construction through `finge ... ⇢ Enum` to enum-variant aggregate construction.
- Top-level type metadata items (`genus`, `discretio`, `pactum`, aliases, imports) no longer block MIR lowering.
- Field reads and index reads lower to MIR place projections.
- Field and index assignments lower for addressable local-backed places.
- Optional chain lowers to `MirOptionOp::Chain`.
- Non-null member/index assertion syntax (`!.`, `![`) now parses and lowers through `MirOptionOp::Unwrap { mode: Assert }` plus projection.
- `vel` / coalesce lowers to `MirOptionOp::Coalesce`.
- Map spread remains fail-closed with an explicit MIR diagnostic.
- Runtime-backed collection methods, diagnostics, string formatting, conversions, and provider calls remain deferred to Phase 7.

Representative MIR dump shapes:

```text
%0 = construct struct def#0: ty#12 {sym#2: const string sym#6, sym#4: const int 36}
_0 = %0: ty#11
return _0.sym#2
```

```text
%1 = construct array: ty#20 [const int 0, ..._0]
return _1[const int 0]
```

```text
%0 = construct map: ty#13 {const string sym#5 => const int 1, const string sym#6 => const int 2}
%0 = construct set: ty#12 [const int 1, const int 2]
%0 = construct variant def#1: ty#12 {sym#3: const string sym#5}
```

```text
%0 = option chain(_0, .sym#2): ty#16
%0 = option unwrap_assert(_0): ty#11
%0 = option coalesce(_0, const string sym#3): ty#0
```

Validation:

- `cargo test -p radix mir` passed: 48 tests passed.
- `cargo test -p radix` passed: 377 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Phase 6B supports the aggregate and option/null subset covered by the focused MIR fixtures.
- Struct field defaults with HIR initializers are expanded into struct aggregate fields when omitted at construction; semantic analysis still owns required/default checks.
- Map/object spread remains unsupported in MIR and fails clearly.
- Runtime/provider lowering remains Phase 7.
- No target backend consumes MIR.

## Phase 7 Baseline

Status: complete.

Implemented artifacts:

- Diagnostic verbs lower to `MirIntrinsic::Diagnostic` runtime calls.
- String-template application lowers to `MirIntrinsic::FormatString` with template symbol and evaluated arguments.
- Runtime conversion (`⇒`) lowers to `MirIntrinsic::Convert` with runtime flavor, target type, hint symbols, source value, and optional fallback.
- Selected collection methods lower to `MirIntrinsic::Collection`: append, immutable append, index/read, length, and contains.
- Imported provider/module calls lower to `MirIntrinsic::Provider` using source import identity and method/function symbols.
- Unsupported method shapes remain fail-closed with an explicit MIR diagnostic.
- Target backends remain unchanged and do not consume MIR.

Representative MIR dump shapes:

```text
runtime diagnostic nota(const string sym#1) -> ty#5
runtime diagnostic vide(_0) -> ty#5
runtime diagnostic mone(const string sym#2) -> ty#5
```

```text
%0 = runtime format_string template sym#2(_0) -> ty#0
%0 = runtime convert runtime -> ty#1 hints [sym#3, sym#4] fallback const int 0(_0) -> ty#1
%0 = runtime collection length(_0) -> ty#1
%0 = runtime provider sym#1/sym#2::sym#3() -> ty#7
```

Validation:

- `cargo test -p radix mir` passed: 50 tests passed.
- `cargo test -p radix` passed: 379 tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Phase 7 covers the runtime operation classes represented by the focused MIR fixtures.
- Runtime operation identity is target-neutral; MIR does not encode Rust, Go, TypeScript, WASM, native linkage, Cargo dependency, or `@ verte` target translation strings for these operations.
- Collection pipelines, closure-backed methods, runtime ABI/linkage, backend consumption, and MIR validation remain deferred.
