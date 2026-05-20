# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Canonical Repo Shape

Current development is centered on the `radix` Cargo workspace:

```
radix/
├── Cargo.toml              # workspace manifest
├── crates/
│   ├── radix/              # active compiler and CLI
│   └── norma/              # Rust runtime support crate
└── stdlib/
    └── norma/              # Faber stdlib definitions and @ verte metadata

examples/
└── exempla/                # example .fab programs

docs/
└── grammatica/             # language documentation

scripta/                    # current helper scripts only
```

Do not restore archived bootstrap, self-hosting, old reference, proba, or golden trees into this repository. Historical surfaces live in the sibling archive repository.

## Stdlib (norma)

The stdlib source of truth is [`radix/stdlib/norma`](radix/stdlib/norma). These `.fab` files define collection methods, HAL contracts, and target translations:

```fab
# From radix/stdlib/norma/innatum/lista.fab
@ verte ts "push"
@ verte rs "push"
functio adde(T elem) -> vacuum
```

Runtime-backed Rust support lives in [`radix/crates/norma`](radix/crates/norma).

How it works:

1. `radix/stdlib/norma/*.fab` defines stdlib methods and metadata.
2. The compiler loads stdlib translation metadata through the norma registry.
3. Calls such as `myList.adde(x)` lower to target-specific calls like `push`.

## Critical Rules

1. **Type-first syntax**: `textus name` not `name: textus`.
2. **Verify grammar**: Check `EBNF.md` before assuming syntax exists.
3. **No invented syntax**: No `Type?`, no made-up suffixes.
4. **Banned keyword**: `cum` (English homograph).
5. **Nullable params**: Use `ignotum`, not invented patterns.
6. **Run scripts via bun**: `bun run build:radix`, not direct script paths.
7. **Correctness over completion**: Explicit over convenient.
8. **Fix root causes**: Do not paper over upstream missing type information.

## Grammar Rules

- Empty collections need explicit types: `[] innatum lista<T>`, `{} innatum tabula<K,V>`.
- Missing type info in codegen is an upstream bug, not a reason to guess in codegen.
- Prefer `reddit` for single-expression returns: `si cond reddit x`.
- Use Stroustrup brace style.

## Commands

Use the workspace manifest from the repository root:

```bash
bun run check:radix
bun run test:radix
bun run ci
bun run build:radix
```

Equivalent raw Cargo commands:

```bash
cargo fmt --manifest-path radix/Cargo.toml --all -- --check
cargo test --manifest-path radix/Cargo.toml
cargo build --release --manifest-path radix/Cargo.toml -p radix
```

Compiler CLI examples:

```bash
cargo run --manifest-path radix/Cargo.toml -p radix -- targets
cargo run --manifest-path radix/Cargo.toml -p radix -- check examples/exempla/salve-munde.fab
cargo run --manifest-path radix/Cargo.toml -p radix -- build examples/exempla/salve-munde.fab
cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t rust examples/exempla/salve-munde.fab
cargo run --manifest-path radix/Cargo.toml -p radix -- emit -t go examples/exempla/salve-munde.fab
```

Compilation pipeline:

```text
Lex -> Parse -> Collect -> Resolve -> Lower -> Typecheck -> Analysis -> Codegen
```

Use `radix/crates/radix` for all new compiler feature development, semantic analysis, diagnostics, and codegen work.

## Syntax Patterns

### Type Declarations

```fab
# Correct
textus nomen
numerus aetas
functio greet(textus name) -> textus

# Wrong
nomen: textus
functio greet(name: textus): textus
```

### Block Syntax

```text
itera ex        # itera ex items fixum item { }
itera de        # itera de obj fixum key { }
cura            # cura expr fixum h { }
tempta...cape   # tempta { } cape err { }
dum             # dum cond { }
si              # si cond { }
elige           # elige val { casu case { } }
discerne        # discerne val { casu Var { } }
```

### Function Annotations

```fab
functio parse() -> numerus
@ futura
functio fetch() -> textus
@ cursor
functio items() -> numerus
@ futura
@ cursor
functio stream() -> datum
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
