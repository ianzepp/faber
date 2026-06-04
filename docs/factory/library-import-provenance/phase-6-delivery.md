# Phase 6 Delivery: Validation Gate

## Objective

Run the focused and broad validation gates for the provider-qualified library
import and provenance migration.

## Focused Checks

```bash
cargo test -p faber library
cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server
cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call
cargo test -p radix runtime
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

Results:

- `cargo test -p faber library`: passed.
- `cargo test -p faber package_fixture_runs_norma_http_hal_against_local_server`: passed for both duplicated Faber test targets.
- `cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call`: passed.
- `cargo test -p radix runtime`: passed.
- `cargo run -p faber -- check stdlib/norma/hal/http.fab`: passed with `ok: stdlib/norma/hal/http.fab`.

Substitution:

- The original Phase 6 plan named
  `cargo test -p radix http_hal_calls_emit_norma_runtime_bridge_and_concrete_response_type`.
  That test was intentionally replaced during Phase 3/4 because its source
  shape proved the unsafe no-provenance bridge. Positive imported HTTP runtime
  bridging is covered by
  `cargo test -p faber aliased_norma_http_import_lowers_by_provider_identity`;
  negative user-defined HTTP behavior is covered by
  `cargo test -p radix user_defined_http_without_provenance_does_not_emit_norma_runtime_call`.

## Broad Checks

```bash
cargo test -p radix
cargo test -p faber
cargo test -p norma
./scripta/test
```

Results:

- `cargo test -p radix`: passed.
- `cargo test -p faber`: passed.
- `cargo test -p norma`: passed.
- `./scripta/test`: passed.

## Gate Result

Phase 6 passed. The validated implementation satisfies the factory checkpoints:

- provider-qualified `norma:...` built-in imports resolve;
- old built-in `norma/...` imports reject with targeted diagnostics;
- relative local imports remain local;
- unknown providers report provider diagnostics;
- alias imports preserve provider identity;
- Rust runtime call bridging uses imported binding provenance;
- Rust runtime type mapping uses imported item provenance;
- user-defined same-name or same-shape HTTP surfaces do not trigger Norma
  runtime call or type mapping.
