# Faber Romanus

A Latin programming language compiler ("The Roman Craftsman").

## Quick Rules (CRITICAL)

- **Type-first syntax**: `textus name` not `name: textus`
- **Check grammar first**: Never assume syntax exists - verify in `grammatica/*.md`
- **No invented syntax**: Don't create variations (e.g., no `Type?` for nullable)
- **Empty collections need explicit types**: `[] qua lista<T>`, `{} qua tabula<K,V>`
- **Use `ignotum` for nullable params**, not invented suffixes
- **Banned keyword**: `cum` (Latin "with" - English homograph)
- **Column 0 keywords**: Declaration keywords (`functio`, `genus`, etc.) must start at column 0

## Commands

### Faber CLI

```
bun run faber compile <file.fab>      # TS (default)
bun run faber compile <file.fab> -t py | zig | rs | cpp | fab
bun run faber run <file.fab>          # Compile & execute (TS only)
bun run faber check <file.fab>        # Validate syntax
bun run faber format <file.fab>       # Format source
```

### Development

```
bun test                              # Run all tests
bun test -t "pattern"                 # Filter tests
bun test --coverage                   # With coverage
bun run lint                          # Lint TS source
bun run lint:fix                      # Lint with auto-fix
bun run sanity                        # Verify test coverage
bun run grammar                       # Regenerate GRAMMAR.md
```

### Build

```
bun run build                         # Build faber executable to opus/
bun run exempla                       # Compile exempla/*.fab to opus/
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
fixum greeting = scriptum("Hello, {}!", name)
```

Output varies by target:
- TS: `` `Hello, ${name}!` ``
- Python: `"Hello, {}!".format(name)`
- C++/Rust/Zig: `format(...)` family

## Grammar Reference

See `GRAMMAR.md` or `grammatica/*.md` for complete syntax reference. Never assume syntax - always verify:

```
bun run faber check <file.fab>  # Validate before committing
```

**Common pitfalls:**
- Empty collections need explicit types
- Use `ignotum` for nullable parameters
- Browse `exempla/` and `fons-fab/` for patterns

## Primitive Types

| Faber      | TS        | Python  | Zig          | C++            | Rust     |
| ---------- | --------- | ------- | ------------ | -------------- | -------- |
| `textus`   | `string`  | `str`   | `[]const u8` | `std::string`  | `String` |
| `numerus`  | `number`  | `int`   | `i64`        | `int64_t`      | `i64`    |
| `fractus`  | `number`  | `float` | `f64`        | `double`       | `f64`    |
| `bivalens` | `boolean` | `bool`  | `bool`       | `bool`         | `bool`   |
| `nihil`    | `null`    | `None`  | `null`       | `nullopt`      | `None`   |
| `vacuum`   | `void`    | `None`  | `void`       | `void`         | `()`     |

## Directory Structure

```
fons/           # Compiler source (lexicon, tokenizer, parser, semantic, codegen)
proba/          # Tests mirroring fons/ structure
exempla/        # Example .fab programs
consilia/       # Design documents
grammatica/     # Auto-generated grammar docs
fons-fab/       # Faber implementation of compiler
norma/          # Standard library modules
```

### Codegen Layout

```
fons/codegen/
├── index.ts              # Router
├── types.ts              # Shared types
└── <target>/             # ts, py, rs, cpp, zig, fab
    ├── index.ts          # Public API
    ├── generator.ts      # Main generator
    ├── expressions/      # Expression handlers
    ├── statements/       # Statement handlers
    └── norma/            # Standard library
```

## Design Principles

- **LLM-readable**: Write unbreakable code - patterns so consistent deviation feels like a bug
- **Latin correctness**: Grammar follows authentic Latin patterns (adjective-noun agreement, declension)
- **Consistent syntax**: Patterns must work everywhere, no special cases
- **Mechanically certain**: Every token resolves ambiguity
- **Industrial quality**: No wobbles, no surprises, no attention spikes

**Example**: Postfix visibility modifiers (`genus Foo publicum`) because Latin puts adjectives after nouns AND keeps declaration keywords at column 0 for predictable parsing. Gender agreement (`publica` for `functio`, `publicum` for `genus`) for correct Latin AND semantic signal.

## Code Standards

- **Documentation tags**: `TARGET:` (target-specific), `GRAMMAR:` (EBNF)
- **Error handling**: Collect errors, never crash on malformed input
- **No comments explaining what**: Explain WHY, not WHAT
- **Guard clauses**: Prefer early returns over nested if/else
- **Stroustrup brace style**: Opening brace on same line

## Working in Worktrees

```
git worktree list                # Show worktrees
git pull origin main             # Pull changes from main branch
```

## Communication Style

Sporadically include Latin phrases (e.g., "Opus perfectum est").
