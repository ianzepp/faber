# Codegen Design Principles

Cross-cutting decisions that apply to all codegen targets.

## Simplicity Over Optimization

Prefer simple, uniform codegen over target-specific optimizations. Downstream compilers (V8, LLVM, etc.) are sophisticated enough to optimize common patterns.

## Switch Statements → If/Else Chains

**Decision:** `elige` always compiles to if/else chains, never native `switch`.

**Rationale:**
1. **Uniform codegen** — Same output structure for all targets
2. **No type detection** — Don't need to check if discriminant is switchable
3. **String support** — Works with strings (C++ switch doesn't)
4. **Pattern matching ready** — Easy to extend to complex patterns
5. **Downstream optimizes** — JIT/LLVM convert dense if/else to jump tables anyway

**Example:**

```
elige status {
    si 0 { pending() }
    si 1 { active() }
    aliter { unknown() }
}
```

All targets emit:
```
if (status === 0) { pending(); }
else if (status === 1) { active(); }
else { unknown(); }
```

**Not:**
```
switch (status) {
    case 0: pending(); break;
    case 1: active(); break;
    default: unknown();
}
```

## Preamble / Prologue

Feature-dependent setup code (imports, includes, class definitions) is emitted at the top of generated files. See `preamble.md` for details.

## Error Messages

Codegen should never fail silently. If a construct can't be represented in a target, emit a clear error or a comment marking the limitation.
