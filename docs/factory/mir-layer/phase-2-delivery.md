# Phase 2 Delivery: MIR Inspection Surface

## Interpreted Problem

Phase 1 created the MIR data model, but no compiler path can produce MIR from analyzed Faber source. Phase 2 should add the developer inspection surface and the first lowering boundary without expanding into primitive expression lowering.

The important tightening is that MIR must be library-friendly. A MIR program is a collection of functions. It should not require a `main` function or privilege one entry point, because Faber code should eventually lower as libraries, packages, tests, command-line tools, and standalone programs using the same function model.

## Normalized Spec

- Add an internal MIR lowering entry point from `driver::AnalyzedUnit`.
- Add a compiler-developer command:

```bash
cargo run -p radix --bin radix -- mir <file>
```

- Use the existing deterministic `dump_program` text format for command output.
- Lower top-level Faber functions into `MirFunction` shells.
- If a source file has an executable entry block, lower it as an ordinary synthetic `MirFunction`.
- Do not introduce a special MIR `main` node, `MirEntry` node, or required entry function.
- Permit function-only source files to produce MIR output.
- Preserve source spans where available so unsupported-lowering diagnostics can point at the source.
- Reject unsupported HIR nodes with explicit MIR diagnostics.
- Keep Phase 3 work out of scope: primitive arithmetic, local assignment, calls, expression lowering, and non-trivial bodies should not be required in Phase 2.

## MIR Function Contract

MIR executable units are functions:

```text
MirProgram
  functions: Vec<MirFunction>
```

Function sources:

- A Faber `functio` lowers to a `MirFunction` with `source: Some(DefId)`.
- A top-level executable entry block, if present, lowers to a synthetic `MirFunction` with `source: None`.
- The synthetic entry function is not semantically special inside MIR. It is only another function produced from source shape.
- A library-like source file with only function declarations should still produce a valid `MirProgram`.

Phase 2 should not decide final binary entrypoint policy. Package/runtime layers can later choose which MIR function acts as a binary entry, test harness entry, exported library symbol, or CLI dispatch root.

## Supported Phase 2 Lowering Subset

Supported:

- Function shell lowering:
  - function ID,
  - source `DefId`,
  - optional name symbol,
  - parameters as MIR locals,
  - return type,
  - spans.
- Trivial body lowering:
  - empty `vacuum` function body,
  - empty entry block,
  - explicit no-value return where already trivial.
- Synthetic entry function creation when the HIR has `entry`.

Unsupported and expected to diagnose:

- Primitive arithmetic.
- Local declarations and assignment.
- Direct calls.
- Runtime calls such as `nota`.
- `si`, loops, `itera`, `discerne`.
- Structs, enums, interfaces, methods, closures.
- Optional chains and non-null assertions.
- `iace`, `tempta`, `mori`, and failure flow.
- Collection literals and transforms.

## Stage Graph

1. Add `mir::lower` module with public lowering API.
2. Add MIR-specific error type with message and optional span.
3. Lower top-level function shells and synthetic entry shells.
4. Add `radix mir` CLI command.
5. Add deterministic command/unit tests for function-only and entry-block inputs.
6. Add unsupported-node tests proving non-trivial bodies fail closed.

## Checkpoints

- `radix mir` exists on the developer CLI.
- A function-only source file emits deterministic MIR.
- A source file with an empty entry block emits deterministic MIR.
- Source with a non-trivial body fails with an explicit unsupported-MIR diagnostic.
- No target backend consumes MIR.
- Existing HIR-to-codegen behavior remains unchanged.

## Out Of Scope

- MIR validation beyond basic construction and fail-closed lowering checks.
- Primitive expression lowering.
- Rust backend consumption.
- WASM or native output.
- Package-level binary entrypoint policy.
- Human-readable symbol-name rendering beyond the current deterministic dump format.

## Validation

- Focused unit tests for `mir::lower`.
- CLI/tool test proving `radix mir` appears in the command surface and emits deterministic text for a tiny supported input.
- Negative test for an unsupported non-trivial body.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 2 complete.

## Completion Gate

Phase 2 is complete when MIR can be inspected from source through `radix mir`, function-only/library-like source files work, entry-block source files work without a special `main` concept, and unsupported lowering fails clearly.
