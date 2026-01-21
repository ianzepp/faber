# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Grammar Reference

See `EBNF.md` for the formal specification, `fons/grammatica/*.md` for prose tutorials, and `fons/exempla/` or `fons/rivus/` for working examples.

## Stdlib (norma)

**The stdlib is fully implemented via `fons/norma/*.fab`.** These files define all collection methods (`lista.adde`, `tabula.pone`, `textus.longitudo`, etc.) with `@ verte` annotations that specify translations per target:

```fab
# From fons/norma/lista.fab
@ verte ts "push"
@ verte py "append"
@ verte rs "push"
@ verte zig (ego, elem, alloc) -> "§0.adde(§2, §1)"
functio adde(T elem) -> vacuum
```

**Build pipeline:**
1. `fons/norma/*.fab` → `bun run build:norma` → `norma.<target>.gen.ts` (for faber) and `norma.gen.fab` (for rivus)
2. Codegen calls `getNormaTranslation(target, type, method)` to get the translation
3. Method calls like `myList.adde(x)` become `myList.push(x)` in TypeScript output

**Runtime libraries** for targets that need them live in `fons/subsidia/`:
- `subsidia/zig/` - `Lista`, `Tabula`, `Copia` wrappers with Latin method names
- `subsidia/cpp/` - Helper functions for complex operations
- `subsidia/rs/` - Helper functions for complex operations

## Project Layout

```
fons/                   # Source code ("fons" = source/spring)
├── faber-ts/           # Reference compiler (TypeScript)
│   ├── codegen/        # Code generators by target
│   │   ├── norma.<target>.gen.ts  # Generated stdlib (per target)
│   │   ├── shared/     # Shared codegen utilities
│   │   ├── ts/         # TypeScript codegen
│   │   │   ├── expressions/
│   │   │   ├── statements/
│   │   │   └── preamble/
│   │   └── ...         # py, rs, cpp, zig, fab, go
│   ├── lexicon/        # Lexer definitions
│   ├── tokenizer/      # Tokenizer implementation
│   ├── parser/         # Parser and AST
│   └── semantic/       # Type checking and analysis
├── rivus/              # Bootstrap compiler (Faber source)
│   ├── ast/            # AST type definitions (.fab)
│   ├── cli/            # CLI entry point (.fab)
│   ├── lexicon/        # Lexer modules (.fab)
│   ├── lexor/          # Lexer implementation (.fab)
│   ├── parser/         # Parser modules (.fab)
│   ├── semantic/       # Semantic analysis (.fab)
│   └── codegen/        # Code generation (.fab)
│       ├── norma.gen.fab  # Generated stdlib
│       └── ts/, go/    # Target-specific codegen
├── proba/              # Shared test suite (YAML specs)
│   ├── codegen/        # Codegen tests by target
│   ├── capabilities/   # Feature capability tests
│   ├── harness/        # Test runner infrastructure
│   └── norma/          # Stdlib tests
├── exempla/            # Example .fab programs
├── grammatica/         # Language documentation (prose tutorials)
├── norma/              # Standard library definitions (@ verte annotations)
└── subsidia/           # Runtime helpers (Zig, C++, Rust)

opus/                   # Build outputs ("opus" = work/product)
├── bin/                # Compiled executables (faber, rivus)
├── exempla/            # Compiled examples by target
├── proba/              # Test results database
└── rivus/              # Compiled rivus by target

agents/                 # Agent prompt definitions
scripta/                # Build and utility scripts
```

## Agents

Standalone agent runner at `~/.local/bin/agent`. Runs agents as separate Claude processes in worktrees.

```bash
agent personas list                  # Show available personas
agent jobs list                      # Show running/completed jobs
agent run "goal"                     # Run with default persona
agent run --persona opifex "goal"    # Run with specific persona
agent run -i 102 "fix the bug"       # Work on GitHub issue
agent run -m opus "goal"             # Use specific model
```

### Available Personas

| Persona | Purpose | When to use |
|---------|---------|-------------|
| `opifex` | Issue worker | Well-defined GitHub issues with clear deliverables |
| `diogenes` | Explorer | Recon before implementation, codebase exploration |
| `columbo` | Investigator | Root cause diagnosis when things break |
| `cato` | PR reviewer | Review before merge |
| `seneca` | Design reviewer | Review planning docs, find gaps |
| `ego` | General purpose | Research tasks, anything that doesn't fit others |
| `titus` | TypeScript fixer | Type errors specifically |
| `galen` | Test diagnostician | Classify test failures |
| `augur` | Consequence analyst | Trace what will break from a change |
| `manager` | Coordinator | Multi-phase tasks, delegation |

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

### Building Compilers

Binaries must be built before use:

```
bun run build                         # Build faber + rivus (default)
bun run build --no-rivus              # Build faber only
bun run build --no-faber --artifex    # Build artifex (rivus compiling itself)
```

### Rivus CLI (Primary Compiler)

The self-hosted compiler at `opus/bin/rivus`. **Use this for all new development.**

```
./opus/bin/rivus compile <file.fab>         # Compile to TypeScript (default)
./opus/bin/rivus compile <file.fab> -o out.ts
bun run test:rivus                          # Run tests against rivus
```

**When to use Rivus:**

- All new feature development
- Daily compilation tasks
- Language evolution

**Known Issues:**

- Parser has infinite loop on some inputs - investigation needed
- Tests may hang (use Ctrl+C to interrupt)

### Faber CLI (Bootstrap Compiler)

The TypeScript implementation at `opus/bin/faber`. **Primary purpose: compile rivus.**

```
./opus/bin/faber compile <file.fab>         # TS (default)
./opus/bin/faber run <file.fab>             # Compile & execute (TS only)
./opus/bin/faber check <file.fab>           # Validate syntax
```

**When to use Faber:**

- Building rivus (`bun run build:rivus` uses faber internally)
- Fallback when rivus has bugs

**Language backports to faber should be limited** - new features go in rivus first.

### Development

```
bun run test:rivus                    # Run tests against rivus (primary)
bun test                              # Run faber unit tests
bun test -t "pattern"                 # Filter tests
bun run test:report                   # Run tests with DB tracking + feature matrix
bun run test:report -- --compiler rivus --targets ts
bun run test:report -- --verify       # With target verification (compile/run)
bun run test:report -- --feature si   # Filter by feature name
bun run lint                          # Lint TS source (fons/faber-ts)
bun run lint:fix                      # Lint with auto-fix
bun run sanity                        # Verify test coverage
```

**Test Reports (`test:report`)**

Runs the test harness with SQLite recording and generates a feature support matrix showing pass/fail status for each feature across all targets. The database is recreated on each run (not for long-term tracking, just result summarization).

Output includes:
- Feature matrix showing ✓ (all pass), ✗ (all fail), or `n/m` (partial pass) per target
- Total counts at bottom
- List of failed tests with error messages

Available options (via `--`):
- `--compiler <faber|rivus|artifex>` - Which compiler to test (default: rivus)
- `--targets <ts,py,cpp,rs,zig>` - Comma-separated target list (default: all)
- `--verify` - Compile and execute generated code (slower but thorough)
- `--feature <pattern>` - Filter tests by feature name
- `--verbose` / `-v` - Show detailed progress

Database location: `opus/proba/results.db` (recreated each run)

### Build

```
bun run build:faber-ts                   # Build faber executable to opus/bin/faber
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
elige           # elige val { casu case { } }     - switch
discerne        # discerne val { casu Var { } }  - pattern match
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
