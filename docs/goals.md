# Faber Romanus: Project Goals

## Vision

A Latin programming language that compiles to multiple targets. Latin's case system expresses semantic roles that map naturally to different runtime semantics.

## Why Latin?

Latin grammar carries meaning in word endings (morphology), not just word order:

| Case | Grammatical Role | Programming Concept |
|------|-----------------|---------------------|
| Nominative | subject | the caller, the actor |
| Accusative | direct object | the argument, the payload |
| Dative | indirect object | the recipient, the destination |
| Genitive | possession | property access, ownership |
| Ablative | instrument/means | borrowed reference, context |

A single Latin sentence structure can express what different languages achieve through different syntax.

## Design Principles

### 1. Case endings as semantic hints

Latin morphology provides semantic information that targets interpret according to their memory model:

```
scribe(librō)  // ablative: "using the book"
```

- **JavaScript**: Just a parameter. GC handles memory.
- **Zig**: A pointer/slice. Manual memory, no ownership transfer.
- **Rust** (future): A borrow (`&libro`). Compiler-enforced lifetime.

The programmer writes Latin. Each backend decides what the case means.

### 2. Dual-target from day one

Supporting multiple targets (JS + Zig) from the start forces honest abstraction:

- The AST must be truly target-agnostic
- Language features can't assume GC or manual memory
- Design errors surface early when a construct doesn't translate

### 3. Compiler as tutor

Error messages teach Latin grammar. When you make a morphological mistake, the compiler explains what you wrote vs. what you likely meant.

### 4. Accessibility over purity

Classical Latin purists may object to simplified grammar. That's fine. The goal is a usable programming language, not a Latin course. We simplify where it aids clarity.

## Target Comparison

| Faber Construct | JavaScript | Zig |
|-----------------|------------|-----|
| `esto x = 1` | `let x = 1;` | `var x: i32 = 1;` |
| `fixum x = 1` | `const x = 1;` | `const x: i32 = 1;` |
| `functio f() { }` | `function f() { }` | `fn f() void { }` |
| `redde x` | `return x;` | `return x;` |
| `si ... aliter` | `if ... else` | `if ... else` |
| `dum conditio { }` | `while (conditio) { }` | `while (conditio) { }` |
| `scribe(x)` | `console.log(x);` | `std.debug.print("{}\n", .{x});` |
| `verum / falsum` | `true / false` | `true / false` |
| `nihil` | `null` | `null` |

## Future Directions

### Verb tense as async semantics

Latin verbs conjugate for tense. This could map to execution timing:

| Tense | Meaning | Target Semantics |
|-------|---------|------------------|
| Present | immediate | synchronous execution |
| Future | deferred | async/promise/future |
| Perfect | completed | already resolved |

### Additional targets

- **Rust**: Ownership/borrowing from case endings
- **WASM**: Direct compilation for web/edge
- **LLVM IR**: Maximum portability

## Non-Goals

- Teaching Classical Latin (we simplify where needed)
- 100% coverage of any target language (subset is fine)
- Performance parity with hand-written code (clarity first)
