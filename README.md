# Faber Romanus

**The Roman Craftsman** — A Latin programming language.

Write code in Latin, compile to JavaScript or Zig. The compiler teaches Latin grammar through error messages.

## Quick Start

```bash
# Install
bun install

# Compile a .la file
bun run src/cli.ts compile examples/salve.la

# Run directly
bun run src/cli.ts run examples/salve.la

# Check for errors
bun run src/cli.ts check examples/salve.la
```

## Example

```la
// salve.la - Hello World

functio salve(nomen) {
  redde "Salve, " + nomen + "!"
}

fixum nomen = "Mundus"
scribe(salve(nomen))
```

Compiles to:

```javascript
function salve(nomen) {
  return ("Salve, " + nomen + "!");
}
const nomen = "Mundus";
scribe(salve(nomen));
```

## Language Reference

### Variables

```la
esto nomen = "Marcus"      // let (mutable)
fixum PI = 3.14159         // const (immutable)
```

### Functions

```la
functio salve(nomen: Textus) -> Textus {
  redde "Salve, " + nomen
}

futura functio fetch(url: Textus) -> Textus {
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

pro item in lista {
  // for...in
}

pro numero ex numeros {
  // for...of
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
├── lexicon/     # Latin vocabulary (nouns, verbs, keywords)
├── tokenizer/   # Source -> Tokens
├── parser/      # Tokens -> AST
├── codegen/     # AST -> JavaScript or Zig
└── cli.ts       # Command-line interface
```

## Philosophy

- **Compiler as tutor**: Error messages teach Latin grammar
- **Accessibility over purity**: Lower barriers, no gatekeeping
- **Case endings matter**: Latin morphology carries semantic meaning

## License

MIT
