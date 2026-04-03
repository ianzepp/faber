# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Grammar Reference

See `EBNF.md` for the formal specification, `docs/grammatica/*.md` for prose tutorials, and `examples/exempla/` or `compilers/rivus/` for working examples.

## Stdlib (norma)

**The stdlib is fully implemented via `stdlib/norma/*.fab`.** These files define all collection methods (`lista.adde`, `tabula.pone`, `textus.longitudo`, etc.) with `@ verte` annotations that specify translations per target:

```fab
# From stdlib/norma/lista.fab
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte zig (ego, elem, alloc) -> "§0.adde(§2, §1)"
functio adde(T elem) -> vacuum
```

**How it works:**
1. `stdlib/norma/*.fab` define stdlib methods with `@ verte` annotations
2. Codegen looks up translations via the norma registry
3. Method calls like `myList.adde(x)` become `myList.push(x)` in TypeScript output

**Runtime libraries** for targets that need them live in `runtimes/norma-{ts,go,py,rs}/`:
- `norma-ts/` - TypeScript runtime (codex, json, toml, yaml, hal)
- `norma-go/` - Go runtime (json, toml, yaml, hal)
- `norma-py/` - Python runtime (json, toml, yaml, hal)
- `norma-rs/` - Rust runtime (json, toml, yaml, hal)

## Project Layout

```
fons/                   # Source code ("fons" = source/spring)
├── radix-rs/           # Primary compiler (Rust, ~17k LOC)
│   └── src/
│       ├── lexer/      # Tokenization (Unicode XID, NFC)
│       ├── parser/     # Recursive descent parser
│       ├── syntax/     # AST definitions
│       ├── hir/        # High-level IR + lowering
│       ├── semantic/   # Type checking, borrow analysis, linting
│       ├── codegen/    # Rust and Faber emitters
│       ├── diagnostics/# Error reporting (ariadne)
│       └── driver/     # Compilation orchestration
├── nanus-ts/           # Minimal bootstrap compiler (TypeScript)
├── nanus-go/           # Minimal bootstrap compiler (Go)
├── nanus-py/           # Minimal bootstrap compiler (Python)
├── nanus-rs/           # Minimal bootstrap compiler (Rust)
├── rivus/              # Self-hosted compiler (Faber source, deprioritized)
│   ├── ast/            # AST type definitions (.fab)
│   ├── cli/            # CLI entry point (.fab)
│   ├── lexicon/        # Lexer modules (.fab)
│   ├── lexor/          # Lexer implementation (.fab)
│   ├── parser/         # Parser modules (.fab)
│   ├── semantic/       # Semantic analysis (.fab)
│   └── codegen/        # Code generation (.fab)
│       ├── norma.gen.fab  # Generated stdlib
│       └── ts/, go/    # Target-specific codegen
├── rivus-cli/          # Rivus CLI entry point
├── proba/              # Shared test suite (YAML specs, ~75 files)
│   ├── codegen/        # Codegen tests by target
│   ├── capabilities/   # Feature capability tests
│   ├── harness/        # Test runner infrastructure
│   └── norma/          # Stdlib tests
├── exempla/            # Example .fab programs (~90 files)
├── golden/             # Golden test outputs
├── grammatica/         # Language documentation (prose tutorials)
├── norma/              # Standard library definitions (@ verte annotations)
├── norma-ts/           # TypeScript stdlib runtime
├── norma-go/           # Go stdlib runtime
├── norma-py/           # Python stdlib runtime
└── norma-rs/           # Rust stdlib runtime

opus/                   # Build outputs ("opus" = work/product)
├── bin/                # Compiled executables (nanus-*, radix-rs)
├── exempla/            # Compiled examples by target
├── proba/              # Test results database
└── rivus/              # Compiled rivus by target

scripta/                # Build and utility scripts
```

## CRITICAL RULES

1. **Type-first syntax**: `textus name` not `name: textus`
2. **Verify grammar**: Check `EBNF.md` before assuming syntax exists
3. **No invented syntax**: No `Type?`, no made-up suffixes
4. **Banned keyword**: `cum` (English homograph)
5. **Nullable params**: Use `ignotum`, not invented patterns
6. **Run scripts via bun**: `bun run build` not `./scripta/build`
7. **Correctness over completion**: Explicit over convenient
8. **Fix root causes**: Don't paper over problems with workarounds

## GRAMMAR RULES

- Empty collections need explicit types: `[] innatum lista<T>`, `{} innatum tabula<K,V>`
- No fallback guessing in codegen: Missing type info = upstream bug to fix

## Commands

### radix-rs (Primary Compiler)

The production compiler at `compilers/radix-rs/`. **Use this for all new development.**

```
cd compilers/radix-rs && cargo build --release             # Build
cd compilers/radix-rs && cargo test                        # Run tests (39 tests)
cd compilers/radix-rs && cargo run -- emit <file.fab>      # Emit Rust
cd compilers/radix-rs && cargo run -- parse <file.fab>     # Parse only
cd compilers/radix-rs && cargo run -- check <file.fab>     # Semantic analysis
```

**Compilation pipeline:** Lex → Parse → Collect → Resolve → Lower → Typecheck → Analysis → Codegen

**When to use radix-rs:**
- All new feature development
- Semantic analysis work (type checking, borrow analysis, linting)
- Rust target codegen

### Nanus CLI (Bootstrap Compilers)

Minimal compilers in multiple languages at `opus/bin/nanus-*`. **Primary purpose: compile rivus and other targets.**

```
./opus/bin/nanus-ts emit < file.fab         # Emit TypeScript
./opus/bin/nanus-go compile file.fab -t ts  # Emit TypeScript
./opus/bin/nanus-go compile file.fab -t go  # Emit Go
./opus/bin/nanus-py emit < file.fab         # Emit Python
./opus/bin/nanus-rs emit < file.fab         # Emit Rust
```

**When to use Nanus:**
- Building rivus (`bun run build:rivus` uses nanus-ts internally)
- Targeting TS/Go/Python (radix-rs currently only emits Rust/Faber)
- Fallback when radix-rs doesn't support a feature yet

**Nanus compilers are intentionally minimal** — new features go in radix-rs.

### Rivus CLI (Self-Hosted Compiler, Deprioritized)

The self-hosted compiler at `opus/bin/rivus`. **Not actively developed — has parser bugs.**

```
./opus/bin/rivus compile <file.fab>         # Compile to TypeScript (default)
bun run test:rivus                          # Run tests against rivus
```

**Known Issues:**
- Parser has infinite loop on some inputs
- Tests may hang (use Ctrl+C to interrupt)

### Building

```
bun run build                         # Build all nanus-* compilers
bun run build:nanus-ts                # Build nanus-ts executable to opus/bin/nanus-ts
bun run build:rivus                   # Build rivus (via nanus-ts) to opus/rivus/fons/ts/
bun run build:rivus -- -t zig         # Build rivus to opus/rivus/fons/zig/
bun run exempla                       # Compile examples/exempla/*.fab to opus/
bun run exempla -- -t all             # Compile to all targets
```

### Testing

```
cd compilers/radix-rs && cargo test                   # radix-rs tests (primary)
bun test                              # nanus-ts unit tests
bun test -t "pattern"                 # Filter tests
bun run test:rivus                    # Run tests against rivus
bun run test:report                   # Run tests with DB tracking + feature matrix
bun run test:report -- --compiler rivus --targets ts
bun run test:report -- --verify       # With target verification (compile/run)
bun run test:report -- --feature si   # Filter by feature name
```

**Test Reports (`test:report`)**

Runs the test harness with SQLite recording and generates a feature support matrix showing pass/fail status for each feature across all targets. The database is recreated on each run (not for long-term tracking, just result summarization).

Output includes:
- Feature matrix showing ✓ (all pass), ✗ (all fail), or `n/m` (partial pass) per target
- Total counts at bottom
- List of failed tests with error messages

Available options (via `--`):
- `--compiler <faber|rivus>` - Which compiler to test (default: rivus)
- `--targets <ts,py,cpp,rs,zig>` - Comma-separated target list (default: all)
- `--verify` - Compile and execute generated code (slower but thorough)
- `--feature <pattern>` - Filter tests by feature name
- `--verbose` / `-v` - Show detailed progress

Database location: `opus/proba/results.db` (recreated each run)

### Tools

```
bun run misc:ast                      # Check AST node coverage
bun run misc:tree-sitter              # Regenerate tree-sitter parser
bun run lint                          # Lint TS source (compilers/nanus-ts)
bun run lint:fix                      # Lint with auto-fix
bun run sanity                        # Verify test coverage
```

## Syntax Patterns

### Type Declarations

```fab
# Correct
textus nomen
numerus aetas
functio greet(textus name) -> textus

# Wrong (not Faber)
nomen: textus
functio greet(name: textus): textus

# Colon used only for defaults in genus
genus Persona
    textus nomen: "Anonymous"
```

### Block Syntax

```
itera ex        # itera ex items fixum item { }  - iterate values
itera de        # itera de obj fixum key { }    - iterate keys
cura            # cura expr fixum h { }         - resource scope
tempta...cape   # tempta { } cape err { }       - error handling
dum             # dum cond { }                  - while loop
si              # si cond { }                   - conditional
elige           # elige val { casu case { } }     - switch
discerne        # discerne val { casu Var { } }  - pattern match
```

### Function Annotations

```fab
functio parse() -> numerus                        # sync function
@ futura
functio fetch() -> textus                         # async function
@ cursor
functio items() -> numerus                        # generator function
@ futura
@ cursor
functio stream() -> datum                         # async generator
```

### String Formatting

Use `scriptum()` for formatted strings (required for Zig, works everywhere):

```fab
fixum greeting = scriptum("Hello, §!", name)
```

Output varies by target:

- TS: `` `Hello, ${name}!` ``
- Python: `"Hello, {}!".format(name)`
- C++/Rust/Zig: `format(...)` family

## Primitive Types

| Faber      | TS        | Python  | Zig          | C++           | Rust     |
| ---------- | --------- | ------- | ------------ | ------------- | -------- |
| `textus`   | `string`  | `str`   | `[]const u8` | `std::string` | `String` |
| `numerus`  | `number`  | `int`   | `i64`        | `int64_t`     | `i64`    |
| `fractus`  | `number`  | `float` | `f64`        | `double`      | `f64`    |
| `bivalens` | `boolean` | `bool`  | `bool`       | `bool`        | `bool`   |
| `nihil`    | `null`    | `None`  | `null`       | `nullopt`     | `None`   |
| `vacuum`   | `void`    | `None`  | `void`       | `void`        | `()`     |

## Design Principles

- **LLM-readable**: Patterns so consistent that deviation feels like a bug
- **Latin correctness**: Authentic Latin grammar (adjective-noun agreement, declension)
- **Mechanically certain**: Every token resolves ambiguity, no special cases

## Code Standards

- **Documentation tags**: `TARGET:` (target-specific), `GRAMMAR:` (EBNF), `WHY:`, `EDGE:`, `PERF:`
- **Error handling**: Collect errors, never crash on malformed input
- **No comments explaining what**: Explain WHY, not WHAT
- **Guard clauses**: Prefer early returns over nested if/else
- **Prefer `reddit` for single-line returns**: Use `si cond reddit x` and `casu k reddit v` over `{ redde ... }` when the body is a single expression
- **Stroustrup brace style**: Opening brace on same line
- **Tests in dedicated files**: Use `_test.rs` files (e.g., `collect_test.rs`), not inline `#[cfg(test)] mod tests`

## Communication Style

Sporadically include Latin phrases (e.g., "Opus perfectum est").
