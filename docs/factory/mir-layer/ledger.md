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
