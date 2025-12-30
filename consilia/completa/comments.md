# Comment Preservation

**STATUS: IMPLEMENTED**

Preserve comments through the parse → AST → codegen pipeline.

## Current State

Comments are now preserved through the full pipeline:

1. **Tokenizer**: Emits COMMENT tokens with type info (line/block/doc)

    ```ts
    // fons/tokenizer/index.ts - COMMENT tokens now emitted
    addToken('COMMENT', comment.trim(), pos, undefined, 'line');
    ```

2. **AST**: BaseNode includes leadingComments and trailingComments

    ```ts
    // fons/parser/ast.ts
    export interface BaseNode {
        position: Position;
        resolvedType?: SemanticType;
        leadingComments?: Comment[];
        trailingComments?: Comment[];
    }
    ```

3. **Codegen**: All targets emit comments via shared helpers in `fons/codegen/types.ts`

## Use Cases

| Pipeline                  | Priority | Notes                                             |
| ------------------------- | -------- | ------------------------------------------------- |
| TS → Faber                | High     | Preserve developer documentation during migration |
| Faber → Faber (formatter) | High     | Essential for formatting                          |
| Faber → TS/Py/etc.        | Medium   | Nice to have for generated code                   |

## Proposed Design

### Comment Types

```ts
interface Comment {
    type: 'line' | 'block' | 'doc';
    value: string;
    position: Position;
}
```

| Type    | Syntax       | Special Handling                            |
| ------- | ------------ | ------------------------------------------- |
| `line`  | `// ...`     | Single line                                 |
| `block` | `/* ... */`  | Multi-line                                  |
| `doc`   | `/** ... */` | Documentation, may parse structured content |

### AST Changes

Add optional comment arrays to `BaseNode`:

```ts
export interface BaseNode {
    position: Position;
    resolvedType?: SemanticType;
    leadingComments?: Comment[];
    trailingComments?: Comment[];
}
```

### Tokenizer Changes

1. Enable COMMENT token emission (currently commented out)
2. Track comment type (line/block/doc)
3. Preserve original formatting/whitespace

```ts
addToken(
    'COMMENT',
    {
        type: 'line',
        value: comment.trim(),
    },
    pos,
);
```

### Parser Changes

Attach comments to adjacent AST nodes:

```ts
// Leading comments: between previous statement and current
// Trailing comments: on same line after node

function attachComments(node: BaseNode, tokens: Token[]) {
    node.leadingComments = collectLeadingComments(node.position);
    node.trailingComments = collectTrailingComments(node.position);
}
```

### Codegen Changes

Each target generator emits comments in target syntax:

| Target | Line     | Block           |
| ------ | -------- | --------------- |
| Faber  | `// ...` | `/* ... */`     |
| TS     | `// ...` | `/* ... */`     |
| Python | `# ...`  | `""" ... """`   |
| Rust   | `// ...` | `/* ... */`     |
| C++    | `// ...` | `/* ... */`     |
| Zig    | `// ...` | N/A (line only) |

## Implementation Steps

1. **Tokenizer**: Emit COMMENT tokens with type and position
2. **Parser**: Collect comments, attach to nodes during parsing
3. **AST types**: Add `leadingComments`/`trailingComments` to `BaseNode`
4. **Codegen base**: Add `emitLeadingComments()`/`emitTrailingComments()` helpers
5. **Each target**: Call comment helpers before/after node emission
6. **Tests**: Round-trip tests ensuring comments survive

## Edge Cases

- Comments between tokens in complex expressions
- Comments inside parameter lists
- Orphan comments (not adjacent to any node)
- Nested block comments (not supported in most languages)
- Shebang lines (`#!/usr/bin/env node`)

## Example

### Input (Faber)

```fab
// User represents a registered user
genus User {
    textus nomen  // display name

    /*
     * Create a new user with validation
     */
    functio creo(textus nomen) {
        ego.nomen = nomen
    }
}
```

### AST

```ts
{
    type: 'GenusDeclaration',
    name: 'User',
    leadingComments: [
        { type: 'line', value: 'User represents a registered user' }
    ],
    members: [
        {
            type: 'FieldDeclaration',
            name: 'nomen',
            trailingComments: [
                { type: 'line', value: 'display name' }
            ]
        },
        {
            type: 'FunctioDeclaration',
            name: 'creo',
            leadingComments: [
                { type: 'block', value: 'Create a new user with validation' }
            ]
        }
    ]
}
```

### Output (TypeScript)

```ts
// User represents a registered user
class User {
    nomen: string; // display name

    /*
     * Create a new user with validation
     */
    constructor(nomen: string) {
        this.nomen = nomen;
    }
}
```
