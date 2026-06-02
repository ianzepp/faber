# Go Codegen Baseline Ledger

**Goal**: `docs/factory/go-codegen/goal.md`  
**Worktree**: `/Users/ianzepp/work/ianzepp/faber-go-codegen`  
**Branch**: `factory/go-codegen`  
**Baseline date**: 2026-06-02  
**Base commit observed**: `32bc7819`  
**Go toolchain**: `go version go1.26.3 darwin/arm64`

## Required Baseline Command

```bash
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
```

Result:

```text
Go e2e exempla: 91/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

The ignored harness test itself passed. The pass count is lower than total
because the harness records individual exemplar failures and reports them while
still leaving the Rust test green.

## Failure Inventory

| Exemplar | Failure kind | Probable root cause | Likely module |
| --- | --- | --- | --- |
| `examples/exempla/ad/ad.fab` | Go codegen diagnostic | `ad` expressions are rejected for Go targets. | Go driver/codegen target checks |
| `examples/exempla/functio/optionalis.fab` | `go run` compile failure | Optional/default parameter lowering emits non-nullable Go primitives, nil comparisons, and calls without default argument expansion. | Go function/call lowering; HIR optional parameter metadata |
| `examples/exempla/genus/creo.fab` | `go run` compile failure | Float literal is emitted into an integer-shaped field/value path. | Go type/value shape lowering for `fractus` vs `numerus` |
| `examples/exempla/inter/inter.fab` | `go run` compile failure | Faber membership/interval-like `inter` syntax lowers to Go `&&` between scalar and collection operands. | Go expression lowering for `inter` |
| `examples/exempla/itera/cursor-iteratio.fab` | `go run` compile failure | Cursor functions are emitted as scalar `int` returns instead of iterable values; loop variables degrade to `any`. | Go cursor/iterator lowering |
| `examples/exempla/itera/nidificatus.fab` | `go run` compile failure | Range iterator variables degrade to `any`, making arithmetic invalid. | Go iteration variable typing |
| `examples/exempla/si/ergo-redde.fab` | `go run` compile failure | Nullable return paths use `*int`, but non-nil expression branches return raw `int`. | Go nullable return conversion |
| `examples/exempla/syntaxis/arena-mixta.fab` | `go run` compile failure | Mixed arena/control-flow expression and loop values degrade to `any` without assertions/conversions. | Go expression-valued block lowering; iteration typing |
| `examples/exempla/syntaxis/destructura-sparsa.fab` | `go run` compile failure | Expression-valued block returns `any` where typed `int` is required. | Go expression-valued block lowering |
| `examples/exempla/syntaxis/fluxus-cede.fab` | `go run` compile failure | Cursor/range emission returns scalar `int` and loop variable typing degrades to `any`. | Go cursor/iterator lowering; iteration typing |

## Failure Clusters

### Optional And Nullable Values

- `functio/optionalis.fab`
- `si/ergo-redde.fab`

Symptoms:

- Go primitives are compared with `nil`.
- Calls omit optional/default parameters instead of applying defaults.
- Nullable returns expect pointer values but expression branches return raw
  primitives.

Recommended next phase if chosen: implement a focused optional/nullable Go
lowering slice that preserves pointer shapes for optional parameter slots,
expands default arguments at call sites, and wraps non-nil nullable return
expressions.

### Cursor And Iteration Typing

- `itera/cursor-iteratio.fab`
- `itera/nidificatus.fab`
- `syntaxis/fluxus-cede.fab`
- part of `syntaxis/arena-mixta.fab`

Symptoms:

- Cursor functions return scalar values instead of iterable containers.
- Range loops over scalar cursor results fail Go compile.
- Loop variables are typed as `any`, breaking arithmetic and causing unused
  variables/missing returns.

Recommended next phase if chosen: focus on Go cursor lowering and iteration
variable typing, with representative tests for range/cursor loops.

### Expression-Valued Blocks And Type Assertions

- `syntaxis/arena-mixta.fab`
- `syntaxis/destructura-sparsa.fab`

Symptoms:

- Generated helper closures return `any` where the surrounding function expects
  `string` or `int`.

Recommended next phase if chosen: thread expected expression types into Go
expression-valued block lowering or emit safe typed conversions/assertions where
the HIR type proves the concrete value.

### Primitive Numeric Shape

- `genus/creo.fab`

Symptoms:

- `3.14159` is emitted in an integer context.

Recommended next phase if chosen: trace the constructor/field type path and fix
the upstream type classification or Go type mapping rather than coercing in
codegen.

### Target-Specific Unsupported Syntax

- `ad/ad.fab`
- `inter/inter.fab`

Symptoms:

- `ad` already fails explicitly for Go, which is honest but counts as an e2e
  failure unless expected-failure metadata is added.
- `inter` emits invalid Go instead of a supported lowering or explicit target
  diagnostic.

Recommended next phase if chosen: either implement Go lowering for `inter` where
semantics are clear, or make unsupported `inter` forms fail with a useful Go
target diagnostic. Treat `ad` as a harness-honesty question unless Go semantics
are planned.

## Phase Selection Evidence

The baseline already exceeds the factory goal's first practical target range
with `91/101` passing. The most valuable next implementation cluster is
optional and nullable values because it touches common function-call and return
semantics and has two concrete failing exempla with clear Go compile errors.

Recommended first implementation phase:

```text
Phase 1: Optional/default parameters and nullable Go return wrapping
```

