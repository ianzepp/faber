# Faber Romanus

**The Roman Craftsman** — A Latin programming language.

Write code in Latin, compile to TypeScript, Zig, or WebAssembly. The compiler teaches Latin grammar through error messages.

## Quick Start

```bash
# Install
bun install

# Compile a .fab file
bun run src/cli.ts compile examples/salve.fab

# Run directly
bun run src/cli.ts run examples/salve.fab

# Check for errors
bun run src/cli.ts check examples/salve.fab
```

## Compilation Targets

```bash
# TypeScript (default)
bun run src/cli.ts compile hello.fab

# Zig
bun run src/cli.ts compile hello.fab -t zig

# WebAssembly Text Format
bun run src/cli.ts compile hello.fab -t wasm
```

## Example

```la
// salve.fab - Hello World

functio salve(nomen) -> Textus {
  redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe(salve(nomen))
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

Or WASM:

```wat
(module
  (func (export "salve") (param $nomen i64) (result i64)
    ;; ...
  )
  (func (export "_start")
    ;; ...
  )
)
```

## Language Reference

### Variables

```la
esto nomen = "Marcus"      // let (mutable)
fixum PI = 3.14159         // const (immutable)
```

### Functions

```la
functio salve(Textus nomen) -> Textus {
  redde "Salve, " + nomen
}

futura functio fetch(Textus url) -> Textus {
  // async function
  redde exspecta getData(url)
}
```

### Control Flow

```la
si conditio {
  // if
}
aliter si alia {
  // else if
}
aliter {
  // else
}

dum conditio {
  // while
}

in lista pro item {
  // for...in (iterate keys)
}

ex numeros pro numero {
  // for...of (iterate values)
}
```

### Error Handling

Any block can have a `cape` (catch) clause:

```la
si riskyCall() {
  process()
}
cape erratum {
  handleError(erratum)
}
```

Explicit try/catch/finally:

```la
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

### Operators

| Latin | JavaScript | Meaning |
|-------|------------|---------|
| `et`  | `&&`       | and     |
| `aut` | `\|\|`     | or      |
| `!`   | `!`        | not     |

### Types

```la
Textus          // String
Numerus         // Number
Bivalens        // Boolean (verum/falsum)
Lista<T>        // Array
Tabula<K, V>    // Map
Copia<T>        // Set
Promissum<T>    // Promise
```

### Keywords Reference

| Latin | JavaScript |
|-------|------------|
| `esto` | `let` |
| `fixum` | `const` |
| `functio` | `function` |
| `futura` | `async` |
| `redde` | `return` |
| `si` | `if` |
| `aliter` | `else` |
| `dum` | `while` |
| `pro` | `for` |
| `tempta` | `try` |
| `cape` | `catch` |
| `demum` | `finally` |
| `iace` | `throw` |
| `exspecta` | `await` |
| `novum` | `new` |
| `verum` | `true` |
| `falsum` | `false` |
| `nihil` | `null` |

## Development

```bash
# Run tests
bun test

# 119 tests covering lexicon, tokenizer, parser, and codegen
```

## Architecture

```
src/
├── lexicon/     # Latin vocabulary (nouns, verbs, keywords, types)
├── tokenizer/   # Source -> Tokens
├── parser/      # Tokens -> AST
├── codegen/     # AST -> TypeScript, Zig, or WASM
│   ├── ts.ts    # TypeScript generator
│   ├── zig.ts   # Zig generator
│   └── wasm.ts  # WebAssembly Text Format generator
└── cli.ts       # Command-line interface

editors/
└── zed/         # Zed IDE extension (Tree-sitter grammar)
```

## Philosophy

- **Compiler as tutor**: Error messages teach Latin grammar
- **Accessibility over purity**: Lower barriers, no gatekeeping
- **Case endings matter**: Latin morphology carries semantic meaning

## License

MIT
