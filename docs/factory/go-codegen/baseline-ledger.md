# Go Codegen Baseline Ledger

**Goal**: `docs/factory/go-codegen/goal.md`  
**Worktree**: `/Users/ianzepp/work/ianzepp/faber-go-codegen`  
**Branch**: `factory/go-codegen`  
**Baseline date**: 2026-06-02  
**Base commit observed**: `32bc7819`  
**Harness truthfulness status**: strict expected-failure gate added in Phase 0; Go vet findings report separately from pass/fail  
**Go toolchain**: `go version go1.26.3 darwin/arm64`

## Required Baseline Command

```bash
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
```

Result:

```text
Go e2e exempla: 94/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

The ignored harness now gates the full expected corpus state. The pass count is
lower than total because the seven failures below are recorded in Go
expected-failure metadata. The test fails on any unexpected Go e2e failure and
also fails when one of these expected-failure exempla starts passing, so the
metadata must be removed promptly after backend fixes.

At the original observed baseline commit `32bc7819`, the same command printed
`91/101` and exited green while only asserting that `salve-munde.fab` passed.
That earlier green result was not a strict corpus gate and must not be treated
as completion evidence for Go codegen.

## Failure Inventory

| Exemplar | Failure kind | Probable root cause | Likely module |
| --- | --- | --- | --- |
| `examples/exempla/ad/ad.fab` | Go codegen diagnostic | `ad` expressions are rejected for Go targets. | Go driver/codegen target checks |
| `examples/exempla/inter/inter.fab` | `go run` compile failure | Faber membership/interval-like `inter` syntax lowers to Go `&&` between scalar and collection operands. | Go expression lowering for `inter` |
| `examples/exempla/itera/cursor-iteratio.fab` | `go run` compile failure | Cursor functions are emitted as scalar `int` returns instead of iterable values; loop variables degrade to `any`. | Go cursor/iterator lowering |
| `examples/exempla/itera/nidificatus.fab` | `go run` compile failure | Range iterator variables degrade to `any`, making arithmetic invalid. | Go iteration variable typing |
| `examples/exempla/syntaxis/arena-mixta.fab` | `go run` compile failure | Mixed arena/control-flow expression and loop values degrade to `any` without assertions/conversions. | Go expression-valued block lowering; iteration typing |
| `examples/exempla/syntaxis/destructura-sparsa.fab` | `go run` compile failure | Expression-valued block returns `any` where typed `int` is required. | Go expression-valued block lowering |
| `examples/exempla/syntaxis/fluxus-cede.fab` | `go run` compile failure | Cursor/range emission returns scalar `int` and loop variable typing degrades to `any`. | Go cursor/iterator lowering; iteration typing |

## Failure Clusters

### Optional And Nullable Values

Resolved in Phase 1:

- `functio/optionalis.fab`
- `si/ergo-redde.fab`

The Go backend now emits no-default optional parameters as pointer-shaped Go
values, expands omitted direct-call defaults, and wraps non-`nihil` explicit
returns for nullable return slots.

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

Resolved in Phase 2:

- `genus/creo.fab`

Assignments into known typed destinations now use context-aware expression
emission, and expected-`fractus` arithmetic promotes `numerus` operands to
`float64` where the HIR proves the destination is fractional.

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

The truthful baseline now records `94/101` passing as an expected corpus state,
not as a completed backend target. The next most valuable implementation
cluster should be selected from the remaining failures above.
