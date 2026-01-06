---
status: archived
updated: 2026-01-06
note: Design for hypothetical TS→Faber transpiler. The fab codegen target exists (Faber→Faber formatter), but TS parser was never built. Rivus was written from scratch instead.
---

# TypeScript to Faber Parser

This document describes the architecture for a TypeScript-to-Faber transpiler that uses the existing Faber AST and codegen infrastructure.

## Motivation

Converting TypeScript codebases to Faber manually (or via LLM) is tedious and error-prone. An automated transpiler provides:

- **Consistent output**: Same TS input always produces same Faber output.
- **Clear error reporting**: Unsupported constructs are flagged with locations.
- **Incremental migration**: Convert file-by-file, fix errors, continue.

## Architecture

```
TypeScript Source (.ts)
        │
        ▼
┌───────────────────┐
│   TS Compiler     │  (typescript package)
│   API / Parser    │
└───────────────────┘
        │
        ▼
    TS AST (ts.Node tree)
        │
        ▼
┌───────────────────┐
│   AST Transformer │  (new module: fons/ts-parser/)
│                   │
│  - Node mapping   │
│  - Keyword xlat   │
│  - Error collect  │
└───────────────────┘
        │
        ▼
    Faber AST (existing types from fons/parser/ast.ts)
        │
        ▼
┌───────────────────┐
│  Faber Codegen    │  (existing: fons/codegen/)
│  Target: fab      │
└───────────────────┘
        │
        ▼
  Faber Source (.fab)
```

## Dependencies

```json
{
  "dependencies": {
    "typescript": "^5.x"
  }
}
```

The TypeScript compiler API provides:
- Full parsing with type information
- Source maps for error reporting
- Handles all TS syntax edge cases

Alternative: `ts-morph` for a higher-level API, but adds another dependency.

## Module Structure

```
fons/ts-parser/
├── index.ts              # Public API: parseTypeScript(source) → Program
├── transformer.ts        # Main visitor: ts.Node → AST node
├── statements.ts         # Statement node transforms
├── expressions.ts        # Expression node transforms
├── types.ts              # Type annotation transforms
├── errors.ts             # Error collection and reporting
└── mappings.ts           # Keyword/type lookup tables
```

## Transformer Design

### Core Pattern

```typescript
import ts from 'typescript';
import type { Statement, Expression, Program } from '../parser/ast';

export function transformSourceFile(sourceFile: ts.SourceFile): Program {
    const errors: TransformError[] = [];
    const body = sourceFile.statements.map(stmt => transformStatement(stmt, errors));

    if (errors.length > 0) {
        // Report all errors, don't fail on first
    }

    return {
        type: 'Program',
        body,
        position: positionFrom(sourceFile),
    };
}

function transformStatement(node: ts.Statement, errors: TransformError[]): Statement {
    switch (node.kind) {
        case ts.SyntaxKind.VariableStatement:
            return transformVariableStatement(node as ts.VariableStatement, errors);
        case ts.SyntaxKind.FunctionDeclaration:
            return transformFunctionDeclaration(node as ts.FunctionDeclaration, errors);
        case ts.SyntaxKind.ClassDeclaration:
            return transformClassDeclaration(node as ts.ClassDeclaration, errors);
        // ... etc
        default:
            errors.push({ node, message: `Unsupported statement: ${ts.SyntaxKind[node.kind]}` });
            return createErrorPlaceholder(node);
    }
}
```

### Keyword Mappings

```typescript
// fons/ts-parser/mappings.ts

export const VAR_KIND_MAP = {
    [ts.SyntaxKind.ConstKeyword]: 'fixum',
    [ts.SyntaxKind.LetKeyword]: 'varia',
    [ts.SyntaxKind.VarKeyword]: 'varia', // treat var as mutable
} as const;

export const TYPE_MAP: Record<string, string> = {
    'string': 'textus',
    'number': 'numerus',
    'boolean': 'bivalens',
    'null': 'nihil',
    'undefined': 'nihil',
    'void': 'vacuum',
    'never': 'numquam',
    'unknown': 'ignotum',
    'any': 'ignotum',
    'bigint': 'magnus',
    'object': 'objectum',
};

export const COLLECTION_MAP: Record<string, string> = {
    'Array': 'lista',
    'Map': 'tabula',
    'Set': 'copia',
    'Promise': 'promissum',
    'Iterator': 'cursor',
    'Iterable': 'cursor',
};

export const OPERATOR_MAP: Record<string, string> = {
    // Optional: convert to Latin operators
    // '&&': 'et',
    // '||': 'aut',
    // '!': 'non',
    // Keep as-is for now (both are valid Faber)
};
```

### Node Transform Examples

#### Variable Declaration

```typescript
// const x: string = "hello"  →  fixum textus x = "hello"

function transformVariableStatement(
    node: ts.VariableStatement,
    errors: TransformError[]
): VariaDeclaration[] {
    return node.declarationList.declarations.map(decl => {
        const kind = VAR_KIND_MAP[node.declarationList.flags & ts.NodeFlags.Const ?
            ts.SyntaxKind.ConstKeyword : ts.SyntaxKind.LetKeyword];

        return {
            type: 'VariaDeclaration',
            kind,
            name: transformIdentifier(decl.name),
            typeAnnotation: decl.type ? transformType(decl.type, errors) : undefined,
            init: decl.initializer ? transformExpression(decl.initializer, errors) : undefined,
            position: positionFrom(decl),
        };
    });
}
```

#### Function Declaration

```typescript
// async function fetch(url: string): Promise<Response>
// →  futura functio fetch(textus url) fiet Response

function transformFunctionDeclaration(
    node: ts.FunctionDeclaration,
    errors: TransformError[]
): FunctioDeclaration {
    const isAsync = node.modifiers?.some(m => m.kind === ts.SyntaxKind.AsyncKeyword) ?? false;
    const isGenerator = !!node.asteriskToken;

    return {
        type: 'FunctioDeclaration',
        name: transformIdentifier(node.name!),
        params: node.parameters.map(p => transformParameter(p, errors)),
        returnType: node.type ? transformType(node.type, errors) : undefined,
        body: transformBlock(node.body!, errors),
        async: isAsync,
        generator: isGenerator,
        position: positionFrom(node),
    };
}
```

#### Class Declaration

```typescript
// class User { name: string; constructor(name: string) { this.name = name; } }
// →  genus User { textus name  functio creo(textus name) { ego.name = name } }

function transformClassDeclaration(
    node: ts.ClassDeclaration,
    errors: TransformError[]
): GenusDeclaration {
    // Check for unsupported features
    if (node.heritageClauses?.some(h => h.token === ts.SyntaxKind.ExtendsKeyword)) {
        errors.push({ node, message: 'Class inheritance (extends) is not supported in Faber' });
    }

    const fields: FieldDeclaration[] = [];
    const methods: FunctioDeclaration[] = [];
    let constructor: FunctioDeclaration | undefined;

    for (const member of node.members) {
        if (ts.isPropertyDeclaration(member)) {
            fields.push(transformPropertyDeclaration(member, errors));
        }
        else if (ts.isMethodDeclaration(member)) {
            methods.push(transformMethodDeclaration(member, errors));
        }
        else if (ts.isConstructorDeclaration(member)) {
            constructor = transformConstructor(member, errors);
        }
        // ... decorators, getters, setters → errors
    }

    return {
        type: 'GenusDeclaration',
        name: transformIdentifier(node.name!),
        fields,
        methods,
        constructor,
        implements: extractImplements(node),
        position: positionFrom(node),
    };
}
```

### Error Handling

```typescript
// fons/ts-parser/errors.ts

export interface TransformError {
    node: ts.Node;
    message: string;
    severity: 'error' | 'warning';
}

export function formatErrors(errors: TransformError[], sourceFile: ts.SourceFile): string {
    return errors.map(err => {
        const { line, character } = sourceFile.getLineAndCharacterOfPosition(err.node.getStart());
        return `${sourceFile.fileName}:${line + 1}:${character + 1}: ${err.severity}: ${err.message}`;
    }).join('\n');
}
```

## Faber Codegen Target

Add a new codegen target that emits Faber source:

```
fons/codegen/fab/
├── index.ts
├── generator.ts
├── expressions/
└── statements/
```

This is straightforward — essentially pretty-printing the AST back to Faber syntax.

### Example Output

```typescript
// generator.ts
class FaberGenerator extends BaseGenerator {
    visitVariaDeclaration(node: VariaDeclaration): string {
        const type = node.typeAnnotation ? this.visitType(node.typeAnnotation) + ' ' : '';
        const init = node.init ? ' = ' + this.visitExpression(node.init) : '';
        return `${node.kind} ${type}${node.name.name}${init}`;
    }

    visitSiStatement(node: SiStatement): string {
        const test = this.visitExpression(node.test);
        const consequent = this.visitBlock(node.consequent);
        const alternate = node.alternate ? ` secus ${this.visitStatement(node.alternate)}` : '';
        return `si ${test} ${consequent}${alternate}`;
    }

    // ... etc
}
```

## CLI Integration

```bash
# Convert single file
bun run faber convert src/user.ts -o src/user.fab

# Convert directory
bun run faber convert src/ -o faber-src/

# Check mode (report errors only)
bun run faber convert src/ --check

# With error output
bun run faber convert src/ 2> errors.txt
```

## Implementation Phases

### Phase 1: Core Statements
- Variable declarations
- Function declarations
- If/else, while, for-of, for-in
- Return, throw, try/catch
- Basic expressions

### Phase 2: Types and Classes
- Type annotations
- Type aliases
- Interfaces → pactum
- Classes → genus
- Enums → ordo

### Phase 3: Advanced
- Destructuring
- Spread operators
- Arrow functions
- Async/await
- Generators

### Phase 4: Polish
- Source maps
- Formatting options
- Incremental conversion
- Watch mode

## Testing Strategy

```
proba/ts-parser/
├── statements/
│   ├── variable.test.ts
│   ├── function.test.ts
│   └── class.test.ts
├── expressions/
│   ├── binary.test.ts
│   └── call.test.ts
└── fixtures/
    ├── simple.ts
    ├── simple.expected.fab
    ├── complex.ts
    └── complex.expected.fab
```

Each test:
1. Parse TS fixture
2. Transform to Faber AST
3. Generate Faber source
4. Compare to expected output

## Open Questions

1. **Formatting**: Should output match `faber format` style, or preserve some TS formatting hints?

2. **Comments**: Preserve TS comments in output? Requires comment extraction from TS AST.

3. **Type inference**: When TS has inferred types (no explicit annotation), should we:
   - Omit type (let Faber infer)
   - Use TS compiler API to get inferred type and emit it

4. **Import paths**: Transform `./user` to `./user` or `"./user"`? Handle `.js` extensions?

5. **Async binding**: Detect `const x = await ...` and convert to `figendum x = ...`?
