# Phase 06: Targets and Runtime Shape

## Goal

Turn the first target-specific implementation into a maintainable multi-target strategy.

## Scope

- Decide whether CLI parsing logic is emitted inline, provided by runtime support crates/packages, or split between both.
- Define target capability expectations for TypeScript, Rust, Go, and future targets.
- Decide how target capability limits surface in `radix targets`.
- Add conformance tests for shared CLI behavior across supported targets.
- Document intentionally unsupported target/feature combinations.

## Out Of Scope

- Adding every target at once.
- Advanced interactive CLI features.
- Shell completion generation.

## Design Questions

- Is a runtime helper acceptable for generated CLI programs, or should generated output stay self-contained?
- Which target should become the reference implementation for behavior?
- How much help formatting should be shared versus target-local?

## Acceptance

- Target support is explicit and discoverable.
- At least two targets share the same CLI IR contract.
- Unsupported target/feature combinations fail before misleading code is emitted.

