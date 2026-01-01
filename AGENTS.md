# Faber Romanus

A Latin programming language compiler. "The Roman Craftsman."

## Quick Reference

### Faber CLI

| Command                                   | Purpose                            |
| ----------------------------------------- | ---------------------------------- |
| `bun run faber compile <file.fab>`        | Compile to TypeScript (default)    |
| `bun run faber compile <file.fab> -t py`  | Compile to Python                  |
| `bun run faber compile <file.fab> -t zig` | Compile to Zig                     |
| `bun run faber compile <file.fab> -t rs`  | Compile to Rust                    |
| `bun run faber compile <file.fab> -t cpp` | Compile to C++                     |
| `bun run faber compile <file.fab> -t fab` | Emit canonical Faber source        |
| `bun run faber run <file.fab>`            | Compile and execute (TS only)      |
| `bun run faber check <file.fab>`          | Check for errors without compiling |
| `bun run faber format <file.fab>`         | Format source file                 |

### Development Scripts

| Command                  | Purpose                            |
| ------------------------ | ---------------------------------- |
| `bun test`               | Run all tests                      |
| `bun test -t "pattern"`  | Run tests matching pattern         |
| `bun test --coverage`    | Run tests with line coverage       |
| `bun run sanity`         | Check feature test coverage exists |
| `bun run lint`           | Lint TypeScript source             |
| `bun run lint:fix`       | Lint with auto-fix                 |
| `bun run prettier`       | Format code                        |
| `bun run prettier:check` | Check formatting                   |

### Build & Release

| Command                     | Purpose                                |
| --------------------------- | -------------------------------------- |
| `bun run build`             | Build faber executable to `opus/`      |
| `bun run exempla`           | Compile all `exempla/*.fab` to `opus/` |
| `bun run exempla -- -t all` | Compile exempla to all targets         |
| `bun run verify:exempla`    | Verify exempla compile correctly       |
| `bun run grammar`           | Regenerate `GRAMMAR.md` from parser    |
| `bun run release`           | Release new version                    |

### Misc Tools

| Command                    | Purpose                                        |
| -------------------------- | ---------------------------------------------- |
| `bun run misc:ast`         | Check prettier/tree-sitter cover all AST nodes |
| `bun run misc:tree-sitter` | Regenerate and verify tree-sitter parser       |

## Grammar

See `GRAMMAR.md` for the complete syntax reference. It is auto-generated from parser source comments via `bun run grammar`.

**Never assume Faber syntax.** Before writing `.fab` code:

1. Check `grammatica/*.md` for the relevant grammar rules
2. Look at existing `.fab` files in `exempla/` or `fons-fab/` for patterns
3. Validate with `bun run faber check <file.fab>` before committing

Common pitfalls:

- Don't invent syntax (e.g., `Type?` for nullable) — verify it exists first
- Empty collections need explicit types: `[] qua lista<T>`, `{} qua tabula<K,V>`
- Use `ignotum` for parameters that may be null, not invented suffixes

### Primitive Types

| Faber      | TypeScript | Python  | Zig          | C++            | Rust     |
| ---------- | ---------- | ------- | ------------ | -------------- | -------- |
| `textus`   | `string`   | `str`   | `[]const u8` | `std::string`  | `String` |
| `numerus`  | `number`   | `int`   | `i64`        | `int64_t`      | `i64`    |
| `fractus`  | `number`   | `float` | `f64`        | `double`       | `f64`    |
| `bivalens` | `boolean`  | `bool`  | `bool`       | `bool`         | `bool`   |
| `nihil`    | `null`     | `None`  | `null`       | `std::nullopt` | `None`   |
| `vacuum`   | `void`     | `None`  | `void`       | `void`         | `()`     |

### Block Syntax Patterns

Faber uses a consistent `keyword expr VERB name { body }` pattern:

| Construct       | Syntax                                   | Purpose        |
| --------------- | ---------------------------------------- | -------------- |
| `ex...pro`      | `ex items pro item { }`                  | iterate values |
| `de...pro`      | `de obj pro key { }`                     | iterate keys   |
| `cura...fit`    | `cura resource fit handle { }`           | resource scope |
| `tempta...cape` | `tempta { } cape err { }`                | error handling |
| `dum`           | `dum condition { }`                      | while loop     |
| `si`            | `si condition { }`                       | conditional    |
| `elige`         | `elige value { si case { } }`            | switch         |
| `discerne`      | `discerne value { si Variant ut v { } }` | pattern match  |

**discerne binding style:** Prefer `ut` (whole variant) over `pro` (positional extraction):

```fab
# Preferred: ut binds the whole variant
discerne node {
    si BinaryExpr ut e {
        process(e.left, e.operator, e.right)
    }
}

# Avoid: pro extracts by field position (fragile, obscure)
discerne node {
    si BinaryExpr pro left, op, right {
        process(left, op, right)
    }
}
```

### Return Type Verbs

| Verb    | Async | Generator | Meaning            |
| ------- | :---: | :-------: | ------------------ |
| `fit`   |  no   |    no     | "it becomes"       |
| `fiet`  |  yes  |    no     | "it will become"   |
| `fiunt` |  no   |    yes    | "they become"      |
| `fient` |  yes  |    yes    | "they will become" |

### Parameter Modifiers

Function parameters follow the pattern: `[preposition] [si] [ceteri] type name [ut alias] [vel default]`

| Modifier | Position          | Purpose             | Example                  |
| -------- | ----------------- | ------------------- | ------------------------ |
| `de`     | prefix            | borrowed, read-only | `de textus source`       |
| `in`     | prefix            | mutable borrow      | `in lista<T> items`      |
| `si`     | after preposition | optional parameter  | `si numerus depth`       |
| `ceteri` | after si          | rest/variadic       | `ceteri textus[] args`   |
| `ut`     | after name        | internal alias      | `textus location ut loc` |
| `vel`    | after name/alias  | default value       | `si numerus page vel 1`  |

**Optional parameters (`si`):**

- Without `vel`: type becomes nullable, caller can omit, body receives `nihil`
- With `vel`: parameter has default value, type stays as declared
- Required params cannot follow optional (except `ceteri`)
- Borrowed params (`de`/`in`) cannot have `vel` defaults

```fab
# Optional without default (receives nihil if omitted)
functio greet(textus name, si textus title) -> textus

# Optional with default
functio paginate(si numerus page vel 1, si numerus limit vel 10) -> textus

# Borrowed optional (no default allowed)
functio analyze(textus source, de si numerus depth) -> numerus
```

### String Formatting

Use `scriptum()` for formatted strings (required for Zig, works on all targets):

```fab
fixum greeting = scriptum("Hello, {}!", name)
```

| Target | Output                                             |
| ------ | -------------------------------------------------- |
| TS     | `` `Hello, ${name}!` ``                            |
| Python | `"Hello, {}!".format(name)`                        |
| Rust   | `format!("Hello, {}!", name)`                      |
| C++    | `std::format("Hello, {}!", name)`                  |
| Zig    | `std.fmt.allocPrint(alloc, "Hello, {}!", .{name})` |

Format strings pass through verbatim — use target-appropriate placeholders.

## Directory Structure

- `fons/` — compiler source (lexicon, tokenizer, parser, semantic, codegen)
- `proba/` — tests mirroring fons/ structure
- `exempla/` — example .fab programs
- `consilia/` — design documents (not authoritative)
- `grammatica/` — auto-generated grammar docs

### Codegen Layout

```
fons/codegen/
├── index.ts              # Router: dispatches to target generators
├── types.ts              # Shared types (CodegenTarget, RequiredFeatures)
└── <target>/             # ts, py, rs, cpp, zig, fab
    ├── index.ts          # Public API for target
    ├── generator.ts      # Main generator class
    ├── expressions/      # Expression handlers (binary.ts, call.ts, etc.)
    ├── statements/       # Statement handlers (si.ts, functio.ts, etc.)
    └── norma/            # Standard library helpers (lista.ts, etc.)
```

### Test Layout

See `proba/README.md` for test framework details. Structure mirrors codegen:

- `fons/codegen/<target>/statements/si.ts` → `proba/codegen/statements/si.yaml`

## Syntax Reminder

Faber uses **type-first** syntax, not TypeScript-style `name: Type`:

```fab
# Correct
textus nomen
numerus aetas
functio greet(textus name) -> textus

# Wrong (not Faber syntax)
nomen: textus
aetas: numerus
functio greet(name: textus): textus
```

The colon `:` is used only for default values in genus properties, not for type annotations.

## Banned Keywords

- `cum` — The Latin preposition "with" is permanently banned due to its English homograph.

## Code Standards

**Documentation Tags** (in addition to global): `TARGET:` — target-specific behavior, `GRAMMAR:` — EBNF for parser functions.

**Error Handling**: Never crash on malformed input. Collect errors and continue.

## Design Philosophy

**LLM Readability Goal**

Write Faber code that other LLMs would call "unbreakable." Every token should pull its weight in resolving ambiguity. Patterns should be so consistent that deviation feels like a bug. The test: when an LLM reads the code, it should feel safe — no wobbles, no "wait, what?", no attention spikes. Industrial. Roman. Mechanically certain.

**Human Readability Frame**

When evaluating syntax decisions, adopt the "Latin professor meets gen-z autistic programmer" frame:

- **Latin professor**: Does the grammar follow authentic Latin patterns? Adjective-noun agreement, word order, proper declension. If something is grammatically wrong, it should itch.
- **Gen-z autistic programmer**: Is the syntax consistent and predictable? Does the pattern work everywhere? Are there special cases that break the mental model?

Both must be satisfied. A syntax that's grammatically correct Latin but inconsistent as a programming language fails. A syntax that's perfectly regular but butchers Latin also fails.

Example: Visibility modifiers use postfix position (`genus Foo publicum`) because Latin places adjectives after nouns AND it keeps the declaration keyword in column 0 for predictable parsing. Gender agreement (`publica` for feminine `functio`, `publicum` for neuter `genus`) because it's correct Latin AND the suffix carries semantic signal — like `getUser()` vs `getUsers()`.

## Communication Style

Sporadically include Latin phrases (e.g., "Opus perfectum est", "Bene factum").
.
