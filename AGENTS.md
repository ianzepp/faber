# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Project Layout

```
fons/                   # Source code ("fons" = source/spring)
├── faber/              # Reference compiler (TypeScript)
│   │                   # Uses mixed Latin/English identifiers
│   ├── codegen/        # Code generators by target
│   │   ├── norma-registry.gen.ts  # Generated from fons/norma/*.fab
│   │   └── <target>/   # ts, py, rs, cpp, zig, fab
│   │       ├── index.ts
│   │       ├── generator.ts
│   │       ├── expressions/
│   │       ├── statements/
│   │       ├── preamble/   # Target-specific preamble generation
│   │       └── norma/      # Legacy stdlib codegen (partial)
│   ├── lexicon/        # Lexer/tokenizer
│   ├── parser/         # Parser and AST
│   ├── semantic/       # Type checking and analysis
│   └── shared/         # Shared utilities
├── rivus/              # Bootstrap compiler (Faber source)
│   │                   # Uses Latin exclusively - compiler written in Faber
│   ├── ast/            # AST type definitions (.fab)
│   ├── lexicon/        # Lexer modules (.fab)
│   ├── lexor/          # Lexer implementation (.fab)
│   ├── parser/         # Parser modules (.fab)
│   ├── semantic/       # Semantic analysis (.fab)
│   └── codegen/        # Code generation (.fab)
│       ├── norma-registry.gen.fab  # Generated from fons/norma/*.fab
│       ├── ts/         # TypeScript codegen
│       └── zig/        # Zig codegen (partial)
├── proba/              # Shared test suite for both compilers
│   │                   # Maintains feature sync between faber and rivus
│   ├── parser/         # Parser tests
│   ├── semantic/       # Semantic analysis tests
│   └── codegen/        # Codegen tests by target
├── exempla/            # Example .fab programs
├── grammatica/         # Language documentation (prose tutorials)
├── norma/              # Standard library definitions with codegen annotations
└── subsidia/           # Helper utilities (e.g., Zig runtime)

opus/                   # Build outputs ("opus" = work/product)
├── bin/                # Compiled executables (faber, rivus)
└── rivus/fons/         # Compiled rivus source by target
    ├── ts/             # TypeScript output
    └── zig/            # Zig output

consilia/               # Design documents ("consilia" = plans/advice)
├── futura/             # Proposed features (not yet implemented)
├── completa/           # Implemented features (reference docs)
├── archived/           # Superseded or rejected proposals
└── cleanup/            # Refactoring notes

probationes/            # LLM research harness (learnability trials)
scripta/                # Build and utility scripts
editors/                # Editor integrations (syntax highlighting, etc.)
archivum/               # Historical/archived materials
```

## Agents

Specialized sub-agents for delegating tasks. Invoke by name.

| Name | Role | When to Use |
|------|------|-------------|
| **Columbo** | Root cause investigator | Trace failures, debug complex issues, file issues |
| **Cicero** | Language designer | Syntax design, grammar rules, Latin↔code mappings |
| **Galen** | Test fixer | Diagnose/fix failing tests (test-side only) |
| **Titus** | TypeScript fixer | Bulk TS/lint error resolution |
| **Curie** | Trials researcher | LLM experiment design, results analysis |

Agent files: `.claude/agents/`

## CRITICAL RULES

1. **Type-first syntax**: `textus name` not `name: textus`
2. **Verify grammar**: Check `EBNF.md` before assuming syntax exists
3. **No invented syntax**: No `Type?`, no made-up suffixes
4. **Banned keyword**: `cum` (English homograph)
5. **Nullable params**: Use `ignotum`, not invented patterns
6. **Run scripts via bun**: `bun run faber` not `./scripta/faber`
7. **Correctness over completion**: Explicit over convenient
8. **Fix root causes**: Don't paper over problems with workarounds

## GRAMMAR RULES

- Empty collections need explicit types: `[] innatum lista<T>`, `{} innatum tabula<K,V>`
- No fallback guessing in codegen: Missing type info = upstream bug to fix

## Commands

### Faber CLI (Primary Compiler)

The TypeScript implementation in `fons/faber/` - use this for all daily development:

```
bun run faber compile <file.fab>      # TS (default)
bun run faber compile <file.fab> -t py | zig | rs | cpp | fab
bun run faber run <file.fab>          # Compile & execute (TS only)
bun run faber check <file.fab>        # Validate syntax
bun run faber format <file.fab>       # Format source
```

### Rivus CLI (Bootstrap Compiler)

The Faber implementation in `fons/rivus/` - Faber compiler written in Faber itself.
Must be built first with `bun run build:rivus` before use:

```
bun run build:rivus                   # Build bootstrap compiler to opus/rivus/fons/ts/
bun run build:rivus -- -t zig         # Build to opus/rivus/fons/zig/
bun run rivus compile <file.fab>      # Compile using bootstrap (TS only)
bun run rivus compile <file.fab> -o out.ts
bun run test:rivus                    # Run tests against bootstrap compiler
```

**When to use Rivus:**

- Testing that Faber can compile itself
- Verifying bootstrap compiler correctness
- Dogfooding language features

**When to use Faber (default):**

- All normal development
- Multi-target compilation (py, zig, rs, cpp)
- Faster iteration (no rebuild needed)

**Known Issues:**

- Parser has infinite loop on some inputs - investigation needed
- Tests may hang (use Ctrl+C to interrupt)

### Development

```
bun test                              # Run all tests (primary compiler)
bun test -t "pattern"                 # Filter tests
bun test --coverage                   # With coverage
bun run test:rivus                    # Run tests against bootstrap compiler
bun run lint                          # Lint TS source (fons/faber)
bun run lint:fix                      # Lint with auto-fix
bun run sanity                        # Verify test coverage
bun run trial                         # Run LLM learnability trials
```

### Build

```
bun run build:faber                   # Build faber executable to opus/bin/faber
bun run build:rivus                   # Build rivus (bootstrap) to opus/rivus/fons/ts/
bun run build:rivus -- -t zig         # Build rivus to opus/rivus/fons/zig/
bun run exempla                       # Compile fons/exempla/*.fab to opus/
bun run exempla -- -t all             # Compile to all targets
bun run release                       # Release new version
```

### Tools

```
bun run misc:ast                      # Check AST node coverage
bun run misc:tree-sitter              # Regenerate tree-sitter parser
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
ex...pro        # ex items pro item { }         - iterate values
de...pro        # de obj pro key { }            - iterate keys
cura...fit      # cura r fit h { }              - resource scope
tempta...cape   # tempta { } cape err { }       - error handling
dum             # dum cond { }                  - while loop
si              # si cond { }                   - conditional
elige           # elige val { si case { } }     - switch
discerne        # discern val { si Var { } }    - pattern match
```

### Return Type Verbs

```
fit    # becomes (sync)
fiet   # will become (async)
fiunt  # become (sync generator)
fient  # will become (async generator)
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

## Grammar Reference

See `EBNF.md` for the formal specification, `fons/grammatica/*.md` for prose tutorials, and `fons/exempla/` or `fons/rivus/` for working examples.

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

- **Documentation tags**: `TARGET:` (target-specific), `GRAMMAR:` (EBNF)
- **Error handling**: Collect errors, never crash on malformed input
- **No comments explaining what**: Explain WHY, not WHAT
- **Guard clauses**: Prefer early returns over nested if/else
- **Prefer `reddit` for single-line returns**: Use `si cond reddit x` and `casu k reddit v` over `{ redde ... }` when the body is a single expression
- **Stroustrup brace style**: Opening brace on same line

## Working in Worktrees

```
git worktree list                # Show worktrees
git pull origin main             # Pull changes from main branch
```

## Communication Style

Sporadically include Latin phrases (e.g., "Opus perfectum est").
