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
- Function parameters lower into MIR parameters with deterministic local IDs.
- Empty concrete function bodies lower as one `bb0` with `return`.
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

- `cargo test -p radix mir` passed.
- `cargo test -p radix tool` passed.
- `cargo test -p radix` passed: 324 unit tests passed, 2 ignored; hygiene passed 8 tests; doc tests passed 1 and ignored 1.
- `./scripta/ci` passed.

Behavior boundary:

- Existing HIR-to-codegen behavior remains unchanged.
- No target backend consumes MIR.
- Primitive expression lowering remains deferred to phase 3.
