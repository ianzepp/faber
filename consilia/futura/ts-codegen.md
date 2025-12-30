# Faber Codegen Target

Generate Faber source code (`.fab`) from AST. Primary use case: TS-to-Faber transformer.

## Use Cases

1. **TS → Faber migration**: Parse TypeScript, transform to Faber AST, codegen to `.fab`
2. **Refactoring tools**: Transform Faber AST, output modified source
3. **Code generation**: Programmatically generate Faber code

## Architecture

```
fons/codegen/fab/
├── index.ts          # Public API
├── generator.ts      # Main generator class
├── expressions/      # Expression handlers
└── statements/       # Statement handlers
```

No `norma/` folder needed — stdlib calls (`scribe`, `lista.adde`, etc.) pass through unchanged.

Follows the same patterns as `ts/`, `py/`, `rs/`, `cpp/`, `zig/` targets.

## Style Guide

Output uses canonical Faber style:

### Control Flow

| Construct | Output | Avoid |
|-----------|--------|-------|
| if | `si` | — |
| else-if | `sin` | `aliter si` |
| else | `secus` | `aliter` |

### Operators

| Construct | Output | Avoid |
|-----------|--------|-------|
| logical and | `&&` | `et` |
| logical or | `||` | `aut` |
| logical not | `non` | `!` |

### Functions

| Construct | Output | Avoid |
|-----------|--------|-------|
| return type | `->` | `fit/fiet/fiunt/fient` |
| lambda | `pro x: expr` | `pro x redde expr` |

### Formatting

| Setting | Value |
|---------|-------|
| Indentation | 4 spaces |
| Braces | same line |
| Other | defer to prettier |

## Integration with TS Parser

The TS-to-Faber pipeline:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   TS Source │ ──▶ │  Faber AST  │ ──▶ │  .fab File  │
└─────────────┘     └─────────────┘     └─────────────┘
     ts-parser          transform          fab codegen
```

1. **ts-parser**: Parse TypeScript using `@typescript-eslint/parser` or `ts-morph`
2. **transform**: Convert TS AST nodes to Faber AST nodes (see `ts-gaps.md` for mappings)
3. **fab codegen**: Generate Faber source from AST

## Example Transformations

### TypeScript Input

```ts
class Dog extends Animal {
  protected name: string;

  constructor(name: string) {
    super();
    this.name = name;
  }

  async fetch(): Promise<Ball> {
    const ball = await findBall();
    return ball;
  }
}

export { Dog };
```

### Faber Output

```fab
genus Dog sub Animal {
    protectus textus name

    functio creo(textus name) {
        supra()
        ego.name = name
    }

    futura functio fetch() -> Ball {
        fixum ball = cede findBall()
        redde ball
    }
}

exporta Dog
```

## Implementation Notes

- Reuse AST node types from `fons/parser/types.ts`
- Generator class extends or mirrors `BaseGenerator` pattern
- Each statement/expression type gets a handler function
- Output is valid Faber that can round-trip through the parser

## Test Strategy

Tests in `proba/codegen/fab/` using YAML format:

```yaml
name: genus with inheritance
input: |
  genus Dog sub Animal {
      textus name
  }
expected:
  fab: |
    genus Dog sub Animal {
        textus name
    }
```

Round-trip tests: parse → AST → codegen → parse → compare ASTs.
