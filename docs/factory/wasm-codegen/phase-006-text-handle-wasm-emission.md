# Phase 006: Text Handle Wasm Emission

## Interpreted Problem

After Phase 005, the largest MIR-lowered Wasm-emission cluster was text:
string constants, `textus` locals/returns, text diagnostics, text
concatenation, and format-string runtime calls.

## Normalized Spec

- Keep HIR -> typed HIR -> MIR -> Wasm intact.
- Represent `textus` in the Wasm text probe as an opaque `i32` handle.
- Emit string literals as compile-valid text handles.
- Keep diagnostic imports type-distinct: text diagnostics must use
  `*_text`, not the boolean `*_i32` imports.
- Emit text concatenation through an explicit `faber_text.concat` import.
- Emit format-string calls through arity/signature-specific `faber_text`
  imports.
- Do not claim host/runtime execution; imported text functions require a host
  implementation, so instantiate/run tiers remain separate.

## Repo-Aware Baseline

The Phase 005 harness reported 4/101 compile-valid exemplars. Many
MIR-lowered files stopped at `constant String(Symbol(...))` or
`type Primitive(Textus)`. `redde/redde.fab` also showed that text concatenation
must not fall through numeric Wasm arithmetic.

## Stage Graph

1. Add `TextHandle` to the Wasm probe's internal value classification.
2. Map `textus` and string constants to Wasm `i32` handle values.
3. Add text-specific diagnostic imports.
4. Route text concatenation through `faber_text.concat`.
5. Route format-string runtime calls through signature-specific imports.
6. Validate focused WAT tests and the ignored exemplar harness.

## Checkpoints

- `cargo test -p radix wasm -- --nocapture`
- `cargo test -p radix exempla_wasm_e2e -- --ignored --nocapture`
- `cargo test -p radix mir -- --nocapture`
- `cargo test -p radix`
- `./scripta/lint`

## Gate Plan

The phase is complete when text-heavy exemplars advance to compile-valid,
WAT validation catches no emitted-invalid modules, and instantiate/run remain
reported as host-tier skips rather than compiler passes.
