# Phase 0 Delivery: Go E2E Harness Truthfulness

## Interpreted Problem

The Go e2e harness printed a real corpus pass count but only asserted that
`salve-munde.fab` passed. That made `91/101` useful as telemetry but too weak as
a gate: unexpected Go failures could remain green, and expected failures that
started passing would not force metadata cleanup.

## Normalized Spec

Make `exempla_go_e2e` truthful like the Rust harness:

- Record the current known Go failing exempla as expected-failure metadata.
- Fail the harness on any unexpected Go e2e failure.
- Fail the harness when an expected-failure exemplar starts passing.
- Preserve the printed pass count and failure reasons.
- Preserve stdout checking for sibling `.expected` files.
- Record the gated baseline in `docs/factory/go-codegen/baseline-ledger.md`.

Out of scope:

- Fixing any backend failure cluster.
- Changing exemplar source files or `.expected` stdout behavior.
- Changing Rust e2e harness behavior.

## Repo-Aware Baseline

Observed current baseline:

```text
Go e2e exempla: 91/101 exempla files pass end-to-end
Expected-output checks enabled for 1 exempla files
```

Current known Go expected failures:

- `ad/ad.fab`
- `functio/optionalis.fab`
- `genus/creo.fab`
- `inter/inter.fab`
- `itera/cursor-iteratio.fab`
- `itera/nidificatus.fab`
- `si/ergo-redde.fab`
- `syntaxis/arena-mixta.fab`
- `syntaxis/destructura-sparsa.fab`
- `syntaxis/fluxus-cede.fab`

Relevant code:

- `crates/radix/src/exempla_e2e_test.rs` contains Rust and Go e2e harnesses.
- The Rust harness already implements strict unexpected-failure and
  expected-pass checks.

## Stage Graph

1. Add Go expected-failure metadata for the ten current failing exempla.
2. Replace the Go harness's `salve-munde.fab`-only assertion with strict
   unexpected-failure and unexpected-pass assertions.
3. Keep pass count, expected-output count, and `[fail]` reason printing.
4. Update the baseline ledger to clarify that `91/101` is a gated expected
   state, not completion evidence.
5. Run the Go ignored e2e harness and `cargo test -p radix`.

## Checkpoints

Focused:

```bash
cargo test -p radix exempla_go_e2e -- --ignored --nocapture
```

Broad:

```bash
cargo test -p radix
```

Gate passes when the focused command exits green only because the current
failure set exactly matches metadata, while still printing the full pass count
and failure reasons.

## Gate Plan

The phase is complete when:

- Go e2e expected failures are explicit in code.
- Unexpected failures fail the test.
- Expected failures that pass fail the test.
- `baseline-ledger.md` records this truthfulness distinction.
- Verification commands pass.

## Open Questions

None.

