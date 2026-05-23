# Phase 01 Delivery: Crate Surface, CLI, Driver, Diagnostics

## Scope

Apply the Radix documentation methodology to the Phase 1 command-surface and
diagnostic-boundary files:

- `crates/radix/src/lib.rs`
- `crates/radix/src/bin/radix.rs`
- `crates/radix/src/cli.rs`
- `crates/radix/src/tool.rs`
- `crates/radix/src/driver/mod.rs`
- `crates/radix/src/driver/source.rs`
- `crates/radix/src/driver/session.rs`
- `crates/radix/src/diagnostics/mod.rs`
- `crates/radix/src/diagnostics/diagnostic.rs`
- `crates/radix/src/diagnostics/catalog.rs`
- `crates/radix/src/diagnostics/render.rs`

Test files are excluded.

## Implementation Plan

1. Use bounded agents on disjoint file groups:
   - CLI/surface: `lib.rs`, `bin/radix.rs`, `cli.rs`, `tool.rs`
   - Driver: `driver/mod.rs`, `driver/source.rs`, `driver/session.rs`
   - Diagnostics: `diagnostics/mod.rs`, `diagnostic.rs`, `catalog.rs`,
     `render.rs`
2. Review all generated comments from the supervisor context and remove any
   decorative, unsupported, or behavior-implying claims that are not grounded in
   the current code.
3. Preserve behavior and formatting; this phase is documentation-only.

## Acceptance Criteria

- File headers explain the role of the public Radix API, CLI IR, driver/session
  ownership, and diagnostic rendering/catalog boundary.
- Public and crate-facing contracts document invariants, error behavior, and
  phase context where names and signatures are not enough.
- Private helpers stay compact unless they encode policy or edge-case handling.
- No test files are modified.
- `cargo fmt --check`, `cargo test -p radix`, and `git diff --check` pass.

## Verification

- `cargo fmt --check` passed.
- `cargo test -p radix` passed: 425 unit tests passed, 3 ignored; binary tests
  passed; 8 hygiene tests passed; doc tests passed with 1 passed and 1 ignored.
- `git diff --check` passed.
- Poker-face completion gate cleared at 94%; the only noted gap was this
  verification section still being a placeholder, now resolved.
