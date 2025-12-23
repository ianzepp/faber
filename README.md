# Faber Romanus

**The Roman Craftsman** — A Latin programming language.

Write code in Latin, compile to TypeScript or Zig. The compiler teaches Latin grammar through error messages.

## Quick Start

```bash
# Install
bun install

# Compile a .fab file
bun run fons/cli.ts compile exempla/salve.fab

# Run directly
bun run fons/cli.ts run exempla/salve.fab

# Check for errors
bun run fons/cli.ts check exempla/salve.fab
```

## Compilation Targets

```bash
# TypeScript (default)
bun run fons/cli.ts compile hello.fab

# Zig
bun run fons/cli.ts compile hello.fab -t zig
```

## Example

```
// salve.fab - Hello World

functio salve(nomen) -> textus {
  redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe salve(nomen)
```

Compiles to TypeScript:

```typescript
function salve(nomen): string {
  return (("Salve, " + nomen) + "!");
}
const nomen = "Mundus";
console.log(salve(nomen));
```

Or Zig:

```zig
const std = @import("std");

fn salve(nomen: []const u8) []const u8 {
    // ...
}

pub fn main() void {
    const nomen = "Mundus";
    std.debug.print("{s}\n", .{salve(nomen)});
}
```

## Language Reference

### Variables

```
varia nomen = "Marcus"      // let (mutable)
fixum PI = 3.14159         // const (immutable)

// Boolean and null literals
fixum active = verum       // true
fixum done = falsum        // false
fixum empty = nihil        // null
```

### Functions

```
functio salve(nomen) -> textus {
  redde "Salve, " + nomen
}

// With typed parameters (type-first syntax)
functio adde(numerus a, numerus b) -> numerus {
  redde a + b
}

// Async function
futura functio fetch(url) -> textus {
  redde cede getData(url)
}
```

### Output

`scribe` is a statement keyword (like `iace` for throw):

```
scribe "Hello"
scribe "Name:", name, "Age:", age
scribe result
```

### Control Flow

```
// If statement
si x > 5 {
  scribe "big"
}
aliter {
  scribe "small"
}

// One-liner with ergo (then)
si x > 5 ergo scribe "big" aliter scribe "small"

// While loop
dum x > 0 {
  x = x - 1
}

// While one-liner
dum x > 0 ergo x = x - 1

// For-each (source-first: ex collection pro item)
ex items pro item {
  scribe item
}

// For-each one-liner
ex items pro item ergo scribe item

// Range expression
ex 0..10 pro i {
  scribe i
}

// Range with step
ex 0..10 per 2 pro i {
  scribe i
}
```

### Switch Statement

```
elige status {
  si "pending" ergo scribe "waiting"
  si "active" ergo scribe "running"
  si "done" {
    scribe "finished"
    cleanup()
  }
  aliter scribe "unknown"
}
```

### Guard Clauses

```
functio validate(x) -> numerus {
  custodi {
    si x < 0 { redde -1 }
    si x > 100 { redde -1 }
  }
  redde x
}
```

### Assert Statement

```
adfirma x > 0, "x must be positive"
adfirma valid
```

### Objects

```
// Object literal
fixum persona = {
  nomen: "Marcus",
  aetas: 30
}

scribe persona.nomen

// Destructuring
fixum { nomen, aetas } = persona

// Destructuring with rename
fixum { nomen: userName } = persona

// With block - set properties in context
varia config = { host: "", port: 0 }
cum config {
  host = "localhost"
  port = 8080
}
```

### Error Handling

Any control block can have a `cape` (catch) clause:

```
si riskyCall() {
  process()
}
cape erratum {
  handleError(erratum)
}
```

Explicit try/catch/finally:

```
tempta {
  dangerousCode()
}
cape erratum {
  handleError()
}
demum {
  cleanup()
}
```

Throw errors:

```
iace "Something went wrong"
iace novum Error("message")
```

### Operators

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `et`  | `&&`       | and |
| `aut` | `\|\|`     | or |
| `non` | `!`        | not |
| `nulla` | — | is null/empty |
| `nonnulla` | — | is non-null/non-empty |
| `sic` | `?` | then (ternary) |
| `secus` | `:` | otherwise (ternary) |

### Ternary Expressions

```
// Symbolic style (familiar)
varia x = verum ? 1 : 0

// Latin style (sic = thus, secus = otherwise)
varia x = verum sic 1 secus 0

// With conditions
varia status = aetas >= 18 sic "adultus" secus "puer"
```

Both forms compile to identical output. Pick one style per expression—no mixing.

Empty/non-empty checks:

```
si nonnulla items {
  scribe "has items"
}

si nulla data {
  scribe "no data"
}
```

### Types

```
textus          // string
numerus         // number
bivalens        // boolean (verum/falsum)
nihil           // null
vacuum          // void
lista<T>        // Array<T>
tabula<K, V>    // Map<K, V>
copia<T>        // Set<T>
promissum<T>    // Promise<T>
```

Type annotations use type-first syntax:

```
fixum numerus x = 42
fixum textus name = "Marcus"
fixum lista<numerus> items = [1, 2, 3]
```

### Keywords Reference

| Latin | JavaScript | Category |
|-------|------------|----------|
| `varia` | `let` | declaration |
| `fixum` | `const` | declaration |
| `functio` | `function` | declaration |
| `futura` | `async` | modifier |
| `redde` | `return` | control |
| `si` | `if` | control |
| `aliter` | `else` | control |
| `ergo` | (then) | control |
| `dum` | `while` | control |
| `ex...pro` | `for...of` | control |
| `in...pro` | `for...in` | control |
| `elige` | `switch` | control |
| `custodi` | (guard) | control |
| `adfirma` | (assert) | control |
| `tempta` | `try` | control |
| `cape` | `catch` | control |
| `demum` | `finally` | control |
| `iace` | `throw` | control |
| `scribe` | `console.log` | I/O |
| `cede` | `await` | async |
| `novum` | `new` | expression |
| `cum` | (with) | block |
| `verum` | `true` | value |
| `falsum` | `false` | value |
| `nihil` | `null` | value |
| `et` | `&&` | operator |
| `aut` | `\|\|` | operator |
| `non` | `!` | operator |
| `nulla` | — | operator |
| `nonnulla` | — | operator |
| `sic` | `?` | ternary |
| `secus` | `:` | ternary |

## Development

```bash
# Run tests
bun test

# 210 tests covering lexicon, tokenizer, parser, semantic analyzer, and codegen
```

## Architecture

```
fons/            # "source" - compiler source code
├── lexicon/     # Latin vocabulary (keywords, types)
├── tokenizer/   # Source -> Tokens
├── parser/      # Tokens -> AST
├── semantic/    # Type checking and validation
├── codegen/     # AST -> Target language
│   ├── ts.ts    # TypeScript generator
│   └── zig.ts   # Zig generator
└── cli.ts       # Command-line interface

exempla/         # "examples" - example .fab programs
opus/            # "work" - build output (future)
```

## Philosophy

- **Compiler as tutor**: Error messages teach Latin grammar
- **Accessibility over purity**: Lower barriers, no gatekeeping
- **Source-first syntax**: `ex items pro item` reads naturally in Latin

## License

MIT
