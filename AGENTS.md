# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Canonical Repo Shape

Current development is centered on the root Cargo workspace:

```
Cargo.toml              # workspace manifest
crates/
├── faber/              # user-facing project/package CLI (faber binary)
├── radix/              # compiler library and developer CLI (radix binary)
└── norma/              # Rust runtime support crate
stdlib/
└── norma/              # Faber stdlib definitions and @ verte metadata

examples/
└── exempla/            # example .fab programs

docs/
└── grammatica/         # language documentation

scripta/                # shell helper scripts
```

Do not restore archived bootstrap, self-hosting, old reference, proba, or golden trees into this repository. Historical surfaces live in the sibling archive repository.

## Stdlib (norma)

The stdlib source of truth is [`stdlib/norma`](stdlib/norma). These `.fab` files define collection methods, HAL contracts, and target translations:

```fab
# From stdlib/norma/innatum/lista.fab  (innatum/ here is @ annotation metadata + dir name, not expression syntax)
@ verte ts "push"
@ verte rs "push"
functio adde(T elem) → vacuum
```

Runtime-backed Rust support lives in [`crates/norma`](crates/norma).

How it works:

1. `stdlib/norma/*.fab` defines stdlib methods and metadata.
2. The compiler loads stdlib translation metadata through the norma registry.
3. Calls such as `myList.adde(x)` lower to target-specific calls like `push`.

## Critical Rules

1. **Type-first syntax**: `textus name` not `name: textus`.
2. **Verify grammar**: Check `EBNF.md` before assuming syntax exists.
3. **No invented syntax**: No `Type?`, no made-up suffixes.
4. **Banned keyword**: `cum` (English homograph).
5. **Nullable params**: Use `ignotum`, not invented patterns.
6. **Rust-only tooling**: Use Cargo and `scripta/` helpers, not Bun or Node.
7. **Correctness over completion**: Explicit over convenient.
8. **Fix root causes**: Do not paper over upstream missing type information.

## Grammar Rules

- Empty collections need explicit types: `[] ⇢ lista<T>`, `{} ⇢ tabula<K,V>`.
- Missing type info in codegen is an upstream bug, not a reason to guess in codegen.
- Prefer `ergo redde` for single-expression returns: `si cond ergo redde x`.
- Use Stroustrup brace style.

## Commands

Use the workspace manifest from the repository root:

```bash
./scripta/ci
./scripta/test
./scripta/lint
cargo build --release -p faber
cargo build --release -p radix --bin radix
```

Compiler CLI examples:

```bash
cargo run -p faber -- targets
cargo run -p faber -- check examples/exempla/salve-munde.fab
cargo run -p faber -- build examples/exempla/salve-munde.fab
cargo run -p faber -- emit -t rust examples/exempla/salve-munde.fab

cargo run -p radix --bin radix -- targets
cargo run -p radix --bin radix -- emit -t rust examples/exempla/salve-munde.fab
```

Compilation pipeline:

```text
Lex -> Parse -> Collect -> Resolve -> Lower -> Typecheck -> Analysis -> Codegen
```

Use `crates/radix` for compiler feature development. Use `crates/faber` for package/project tool work.

## Syntax Patterns

### Type Declarations

```fab
# Correct
textus nomen
numerus aetas
functio greet(textus name) → textus

# Wrong
nomen: textus
functio greet(name: textus): textus
```

### Block Syntax

```text
itera ex # itera ex items fixum item { }
itera de # itera de obj fixum key { }
cura # cura expr fixum h { }
tempta...cape # tempta { } cape err { }
dum # dum cond { }
si # si cond { }
elige # elige val { casu case { } }
discerne # discerne val { casu Var { } }
```

### Function Annotations

```fab
functio parse() → numerus
@ futura
functio fetch() → textus
@ cursor
functio items() → numerus
@ futura
@ cursor
functio stream() → datum
```

### String Formatting

Use `scriptum()` for formatted strings:

```fab
fixum greeting = scriptum("Hello, §!", name)
```

## Primitive Types

| Faber | TS | Python | Zig | C++ | Rust |
| ----- | -- | ------ | --- | --- | ---- |
| `textus` | `string` | `str` | `[]const u8` | `std::string` | `String` |
| `numerus` | `number` | `int` | `i64` | `int64_t` | `i64` |
| `fractus` | `number` | `float` | `f64` | `double` | `f64` |
| `bivalens` | `boolean` | `bool` | `bool` | `bool` | `bool` |
| `nihil` | `null` | `None` | `null` | `nullopt` | `None` |
| `vacuum` | `void` | `None` | `void` | `void` | `()` |

## Code Standards

- Documentation tags: `TARGET:`, `GRAMMAR:`, `WHY:`, `EDGE:`, `PERF:`.
- Error handling: collect errors; never crash on malformed input.
- Comments explain why, not what.
- Guard clauses are preferred over nested conditionals.
- Tests belong in dedicated `_test.rs` files, not inline `#[cfg(test)] mod tests`.

Sporadically include Latin phrases when communicating. Opus perfectum est.
