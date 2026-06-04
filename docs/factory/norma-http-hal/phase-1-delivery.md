# Phase 1 Delivery Spec: Interface Tightening

## Scope

Tighten the Faber-facing HTTP HAL interface without adding runtime or compiler
bridge behavior.

## Required Changes

- Change JSON-related HTTP signatures from `quidlibet` to `valor`.
- Rename response/request byte and JSON helpers to snake_case names that match
  Rust bridge normalization:
  - `corpusOcteti` to `corpus_octeti`
  - `corpusJson` to `corpus_json`
- Keep server-side declarations present but clearly marked as deferred, since
  server runtime support is outside the first HTTP HAL slice.
- Preserve type-first grammar and existing pactum structure.

## Checkpoint

Run:

```bash
cargo run -p faber -- check stdlib/norma/hal/http.fab
```

The checkpoint passes when the interface checks and no runtime/codegen behavior
has been changed.
