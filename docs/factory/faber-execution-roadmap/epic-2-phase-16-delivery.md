# Epic 2 Phase 16 Delivery: Exhaustive Rust Elige

Timestamp: 2026-05-24 11:31:31 EDT

## Objective

Make Rust lowering for `elige` without `ceterum` preserve Faber's do-nothing fallthrough semantics while satisfying Rust's exhaustive `match` requirement.

## Baseline

Latest Epic 2 Rust e2e result: 77/100 exempla pass.

The remaining adjacent `elige` failures are:

- `examples/exempla/elige/elige.fab`: rustc rejects non-exhaustive string and integer matches.
- `examples/exempla/elige/in-functione.fab`: all explicit cases return, so Rust treats following fallback statements as unreachable.

Both come from the same lowering gap: when no `ceterum` arm exists, Rust needs an explicit wildcard arm that does nothing.

## Implementation Plan

1. Detect whether a generated Rust match already has a wildcard arm.
2. Emit `_ => {}` when no wildcard/default arm exists.
3. Add a focused regression for `elige` without `ceterum`.
4. Validate with focused tests and the Rust exempla e2e harness.

## Non-Goals

- Do not change parser or HIR shape for `elige`.
- Do not change `discerne` enum/pattern behavior in this phase.
