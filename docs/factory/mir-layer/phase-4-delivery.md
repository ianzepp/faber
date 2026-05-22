# Phase 4 Delivery: `si` / `dum` Control-Flow Normalization

## Interpreted Problem

After Phase 3, MIR can lower straight-line primitive `functio` bodies. Phase 4 should make MIR represent structured Faber control flow as an explicit block graph.

The purpose is to lower Faber `si`, `dum`, `rumpe`, `perge`, and `redde` into `MirBlock` sequences and `MirTerminator` edges without introducing SSA, runtime intrinsics, pattern matching, `itera`, or backend consumption.

## Normalized Spec

- Add control-flow graph construction to `mir::lower`.
- Lower Faber `si` into MIR `Branch` terminators and join blocks.
- Lower expression-valued `si` with destination-passing: each branch writes its result into a shared destination before joining.
- Lower `dum` into condition, body, and after blocks.
- Resolve `rumpe` to the current loop's exit target.
- Resolve `perge` to the current loop's repeat target.
- Preserve explicit `redde` inside nested `si` / `dum` bodies as MIR `Return` terminators.
- Track the current insertion block and prevent appending statements after a terminator.
- Keep `radix mir` deterministic for supported control-flow shapes.
- Keep unsupported Faber constructs as explicit MIR-lowering diagnostics.

## Faber-To-MIR Terminology

Use Faber terms for source constructs and MIR terms for internal nodes:

| Faber source | MIR concept |
| --- | --- |
| `functio` | `MirFunction` |
| `si` / `sin` / `secus` | `MirTerminatorKind::Branch` plus join blocks |
| `dum` | condition/body/after `MirBlock` cycle |
| `rumpe` | `Goto` to loop exit target |
| `perge` | `Goto` to loop repeat target |
| `redde` | `MirTerminatorKind::Return` |

Do not describe the Faber surface as `if`, `while`, `break`, `continue`, or `return` in phase artifacts unless explicitly comparing generic compiler terminology.

## Block Builder Contract

Phase 4 should introduce a block-building discipline:

- The lowerer should use an internal builder representation where a block can be open until it is sealed with a terminator.
- Finished MIR still contains only complete `MirBlock` values with explicit `MirTerminator`s.
- The builder has a current `MirBlockId`.
- A statement can append only to an open current block.
- A terminator seals the current block.
- Lowering can create fresh blocks for branches, joins, loop conditions, loop bodies, and loop exits.
- Every supported path should end in an explicit terminator.
- If source flow continues after a terminated block, lowering should continue from a reachable join/after block, not append to the closed block.
- If a `si` arm closes with `redde`, that arm must not also emit a `goto` to the join block.

Full MIR validation remains Phase 8, but Phase 4 must stop generating malformed block graphs for its supported subset.

## Destination-Passing Contract

Expression-valued `si` should lower through a destination-passing helper rather than ad hoc temporary patching:

```text
lower_expr_to_destination(expr, destination)
```

For Phase 4:

- The destination can be a fresh temporary or an existing `fixum` / `varia` local place chosen by the surrounding lowering context.
- Each reachable `si` arm writes to the same destination.
- Reachable arms flow to the join block.
- Arms closed by `redde`, `rumpe`, or `perge` do not flow to the join block.
- The joined expression result is the shared destination operand.

This keeps non-SSA MIR regular without introducing phi nodes.

## `si` Contract

Statement-shaped `si` lowers to:

```text
bb0:
  branch cond bb_then bb_else

bb_then:
  ...
  goto bb_join

bb_else:
  ...
  goto bb_join

bb_join:
  ...
```

If there is no `secus`, the else target is the join block.

Expression-valued `si` lowers by destination-passing into a shared destination:

```text
%0: ty#N
bb0:
  branch cond bb_then bb_else

bb_then:
  %0 = then_value
  goto bb_join

bb_else:
  %0 = else_value
  goto bb_join

bb_join:
  ...
```

This is the non-SSA answer for Phase 4. Do not add phi nodes.

## `dum` / `rumpe` / `perge` Contract

`dum` lowers to:

```text
bb_start:
  goto bb_cond

bb_cond:
  branch cond bb_body bb_after

bb_body:
  ...
  goto bb_cond

bb_after:
  ...
```

The lowerer should maintain a loop target stack:

```text
LoopContext {
    perge_target: MirBlockId,
    rumpe_target: MirBlockId,
}
```

- `perge` lowers to `goto perge_target`.
- `rumpe` lowers to `goto rumpe_target`.
- `rumpe` / `perge` outside a loop should already be rejected by semantic analysis; MIR lowering should still fail clearly if it sees one without a loop context.

## Supported Phase 4 Lowering Subset

Supported:

- Phase 3 straight-line primitive bodies.
- Nested blocks.
- `si` with `secus`.
- `si` without `secus`.
- `sin` chains if HIR has already normalized or exposes them plainly enough to lower.
- Expression-valued `si` where both branches produce a Phase 3-supported value.
- `dum` with Phase 3-supported conditions and bodies.
- `rumpe` and `perge` inside supported `dum`.
- `redde` from inside supported `si` / `dum` bodies.

Unsupported and expected to diagnose:

- `itera`, including `itera ex`, `itera de`, and `itera pro`.
- `elige` and `discerne`.
- `tempta`, `cape`, `iace`, and `mori`.
- Diagnostic verbs such as `nota`, `mone`, `vide`, and `scribe`.
- Method calls.
- Struct, enum, tuple, array, map, and set construction.
- Optional chain and non-null assertion.
- Closures, `cede`, async/generator shapes, and collection transforms.

## Stage Graph

1. Add block builder helpers for fresh blocks, current block tracking, open-block state, and terminator sealing.
2. Add loop context stack for `dum`, `rumpe`, and `perge`.
3. Lower statement-shaped `si`.
4. Add `lower_expr_to_destination` for expression-valued `si`.
5. Lower `dum` into condition/body/after blocks.
6. Lower `rumpe` and `perge`.
7. Add tests for arms that close with `redde` and therefore do not flow to the join.
8. Add deterministic MIR dump tests for each supported shape.
9. Add negative tests for deferred `itera`, `elige`, `discerne`, and `tempta`.

## Checkpoints

- `radix mir` emits explicit block graphs for supported `si` and `dum`.
- Expression-valued `si` materializes into a shared destination before the join.
- `rumpe` and `perge` target the correct loop blocks.
- `redde` inside nested control flow closes the current MIR block.
- Arms closed by `redde`, `rumpe`, or `perge` do not emit spurious join edges.
- Unsupported `itera`, `discerne`, and `tempta` fail with explicit unsupported-MIR diagnostics.
- No target backend consumes MIR.
- Existing HIR-to-codegen behavior remains unchanged.

## Fixture Candidates

`si` with `redde`:

```fab
functio signum(numerus n) → numerus {
    si n > 0 ergo redde n
    redde 0
}
```

Expression-valued `si`:

```fab
functio positum(numerus n) → numerus {
    fixum numerus x ← n > 0 sic 1 secus 0
    redde x
}
```

`dum`:

```fab
functio summa(numerus n) → numerus {
    varia numerus i ← 0
    varia numerus total ← 0

    dum i < n {
        total ← total + i
        i ← i + 1
    }

    redde total
}
```

`rumpe` / `perge`:

```fab
functio primus(numerus n) → numerus {
    varia numerus i ← 0

    dum i < n {
        i ← i + 1
        si i < 3 ergo perge
        rumpe
    }

    redde i
}
```

Deferred `itera`:

```fab
functio summa(lista<numerus> nums) → numerus {
    varia numerus total ← 0
    itera ex nums fixum n {
        total ← total + n
    }
    redde total
}
```

## Out Of Scope

- `itera` lowering.
- `elige` / `discerne` lowering.
- Pattern matching.
- Runtime intrinsic boundary.
- Aggregate and option representation.
- Failure-flow lowering.
- SSA / phi nodes.
- Rust backend consumption.
- WASM or native output.

## Validation

- Focused unit tests for `si` lowering.
- Focused unit tests for expression-valued `si` materialization.
- Focused unit tests for `dum`, `rumpe`, and `perge`.
- Focused unit tests for closed `si` arms that should not branch to the join.
- Negative tests for deferred `itera`, `discerne`, and `tempta`.
- CLI/tool test proving `radix mir` emits deterministic text for supported control-flow input.
- `cargo test -p radix mir`.
- `cargo test -p radix`.
- `./scripta/ci` before marking Phase 4 complete.

## Completion Gate

Phase 4 is complete when supported Faber `si` / `dum` control flow lowers into explicit MIR blocks and terminators, `rumpe` / `perge` resolve through loop context, expression-valued `si` works without SSA, deferred constructs fail clearly, and no backend behavior changes.
