# TLA+ for Radix-RS — Design Notes

Status: **Design thinking** (not yet implemented)

## Motivation

Ensure the radix-rs compiler handles the entire Faber language correctly for all cases — every valid program compiles correctly, every invalid program produces meaningful diagnostics, nothing is silently dropped or mishandled.

## Two Distinct Ideas

### 1. TLA+ as an Emit Target (for user programs)

Emit TLA+ specifications from Faber source code. The fundamental challenge: Faber is imperative, TLA+ models state machines. A direct translation of most Faber code has no meaningful TLA+ equivalent.

**Where it could work:** stateful structs, enum-based state machines, concurrent/async code, protocol-like patterns.

**Practical approaches (ranked):**

- **Annotation-driven spec extraction** — Add `@ tla` annotations for invariants, state variables, temporal properties. The emitter extracts annotated items into `.tla` specs. Most practical.
- **State machine pattern recognition** — Detect enum+match patterns that look like state machines and auto-generate TLA+ specs. Narrow but doable.
- **Separate DSL mode** — Faber syntax with TLA+ semantics. Cleanest separation but largest scope.

**Mechanically easy** — adding a new `Codegen` backend is straightforward (add `Target::Tla`, implement trait, use `CodeWriter`). The semantic mapping is the hard part.

### 2. TLA+ for the Compiler Itself (higher value)

Model-check radix-rs's own invariants. The compiler is a stateful pipeline with well-defined phases and data transformations where things can go subtly wrong.

**High-value specs:**

| Spec | What it verifies |
|------|-----------------|
| `DefIdLifecycle.tla` | Every DefId created during parsing is resolvable during codegen. No dangling references across phases. |
| `ErrorCollection.tla` | Errors accumulate without loss. Codegen is unreachable when errors exist. No silent drops. Recovery doesn't corrupt subsequent nodes. |
| `TypeResolution.tla` | Every TypeId assigned during type checking exists in the TypeTable. Assignment and lookup lifecycle is complete. |
| `PipelinePhases.tla` | Lex -> Parse -> HIR Lower -> Semantic -> Codegen ordering is enforced. No phase skipped or entered with stale data. (Lower value — already enforced structurally.) |
| `FailablePropagation.tla` | Transitive closure of failable functions terminates and is correct. (Interesting but narrow.) |

**Best candidates to start with:** DefId lifecycle and error collection — subtle bugs, large state space, silent wrong output rather than crashes.

**Key limitation:** TLA+ specs model the *design*, not the implementation. Specs and Rust code can drift apart. Nothing forces them to stay in sync.

## Complementary Approaches

TLA+ verifies the design is sound. These approaches verify the implementation covers the design:

| Approach | What it catches |
|----------|----------------|
| **Grammar-driven fuzz testing** | Generate random valid Faber from EBNF, feed through radix-rs. Catches panics and invalid output. Like TLA+ exhaustive exploration but against the real compiler. |
| **AST/HIR exhaustiveness audit** | Verify every AST variant is handled in lowering, every HirExprKind in every codegen backend, every Type in type checking. Catch-all `_ =>` arms hide gaps. |
| **Proba-to-EBNF coverage mapping** | Map every EBNF production to at least one proba test. Find untested productions. Most direct answer to "what's missing?" |
| **Property-based testing** | Generators for valid Faber ASTs with properties: lowering never panics, codegen output parses in target language, FaberCodegen round-trip is identity. |

## When to Use What

- **Design might be wrong** (type inference completeness, error recovery soundness, borrow model correctness) -> TLA+
- **Implementation might be incomplete** (missing cases, untested productions) -> exhaustiveness audits + proba coverage
- **Implementation might be buggy** (edge cases, unexpected input) -> fuzz testing + property-based testing

## Next Steps

1. Audit proba coverage against EBNF to find untested language features (most immediate value)
2. Write `DefIdLifecycle.tla` as a proof-of-concept spec (most bounded TLA+ starting point)
3. Audit `_ =>` catch-all match arms in lowering and codegen for hidden gaps
