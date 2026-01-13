# Parser Modularization Plan

Split `fons/faber/parser/index.ts` (5778 lines, 109 functions) into manageable modules following the pattern established in `fons/rivus/parser/`.

## Problem

The monolithic parser causes issues for autonomous agents that hit context limits when reading the file. The rivus parser solved this with a `Resolvitor` interface pattern.

## Architecture

### Current (faber)

```
parser/
  index.ts         5778 lines - everything
  ast.ts           2587 lines - AST types
  errors.ts              - error codes
```

### Target (following rivus)

```
parser/
  index.ts         ~150 lines - entry point, Resolver impl
  context.ts       ~400 lines - ParserContext class, token helpers
  resolver.ts       ~50 lines - Resolver interface
  errors.ts              - (unchanged)
  ast.ts                 - (unchanged)
  types.ts         ~200 lines - type annotation parsing
  statements/
    index.ts       ~150 lines - statement dispatcher
    declarations.ts ~800 lines - functio, genus, pactum, ordo, discretio, typus
    control.ts     ~400 lines - si, dum, elige, discerne
    loops.ts       ~300 lines - ex, de, in iteration
    variables.ts   ~250 lines - varia, fixum, patterns
    imports.ts     ~200 lines - importa, specifiers
    actions.ts     ~150 lines - scribe, adfirma, redde, rumpe, perge, iace
    errors.ts      ~200 lines - tempta/cape, custodi
    testing.ts     ~200 lines - probandum, proba, praepara
    blocks.ts      ~150 lines - block, fac block
  expressions/
    index.ts        ~50 lines - expression entry point
    binary.ts      ~400 lines - precedence climbing (assignment through multiplicative)
    unary.ts       ~150 lines - unary, praefixum
    primary.ts     ~600 lines - literals, identifiers, calls, novum, finge, lambda
    dsl.ts         ~300 lines - collection DSL, ab expressions
```

## Resolver Interface

```typescript
// parser/resolver.ts
export interface Resolver {
    ctx(): ParserContext;
    expression(): Expression;
    statement(): Statement;
    block(): BlockStatement;
    typeAnnotation(): TypeAnnotation;
}
```

## ParserContext Class

```typescript
// parser/context.ts
export class ParserContext {
    readonly tokens: Token[];
    current: number = 0;
    readonly errors: ParserError[] = [];
    private uniqueIdCounter: number = 0;
    private pendingComments: Comment[] = [];

    // Token navigation
    peek(offset?: number): Token;
    advance(): Token;
    isAtEnd(): boolean;
    check(type: TokenType): boolean;
    checkKeyword(keyword: string): boolean;
    match(type: TokenType): boolean;
    matchKeyword(keyword: string): boolean;

    // Comment handling
    collectComments(): void;
    consumePendingComments(): Comment[] | undefined;
    collectTrailingComment(nodeLine: number): Comment[] | undefined;

    // Error handling
    reportError(code: ParserErrorCode, context?: string): void;
    expect(type: TokenType, code: ParserErrorCode): Token;
    expectKeyword(keyword: string, code: ParserErrorCode): Token;

    // Recovery
    synchronize(): void;
    synchronizeGenusMember(): void;

    // Helpers
    genUniqueId(prefix: string): string;
    isTypeName(token: Token): boolean;
    isPreposition(token: Token): boolean;
}
```

## Function Mapping

### statements/declarations.ts
- `parseFunctioDeclaration`
- `parseGenusDeclaration`, `parseGenusMember`
- `parsePactumDeclaration`, `parsePactumMethod`
- `parseOrdoDeclaration`
- `parseDiscretioDeclaration`, `parseVariantDeclaration`
- `parseTypeAliasDeclaration`

### statements/control.ts
- `parseSiStatement`
- `parseDumStatement`
- `parseEligeStatement`
- `parseDiscerneStatement`, `parseVariantPattern`

### statements/loops.ts
- `parseExStatement`
- `parseDeStatement`
- `parseInStatement`
- `parseCuraStatement`
- `parseAdStatement`
- `parseIncipitStatement`, `parseIncipietStatement`

### statements/variables.ts
- `parseVariaDeclaration`
- `parseObjectPattern`
- `parseArrayPattern`

### statements/imports.ts
- `parseImportaDeclaration`
- `parseSpecifier`

### statements/actions.ts
- `parseScribeStatement`
- `parseAdfirmaStatement`
- `parseReddeStatement`
- `parseRumpeStatement`
- `parsePergeStatement`
- `parseIaceStatement`

### statements/errors.ts
- `parseTemptaStatement`
- `parseCapeClause`
- `parseCustodiStatement`

### statements/testing.ts
- `parseProbandumStatement`
- `parseProbaStatement`
- `parsePraeparaBlock`

### statements/blocks.ts
- `parseBlockStatement`
- `parseFacBlockStatement`

### statements/index.ts
- `parseStatement`
- `parseStatementWithoutComments`
- `parseStatementCore`
- `attachComments`
- `parseAnnotation`, `parseAnnotations` (or separate annotations.ts)

### expressions/binary.ts
- `parseAssignment`
- `parseOr`, `parseAnd`
- `parseBitwiseOr`, `parseBitwiseXor`, `parseBitwiseAnd`
- `parseEquality`, `parseComparison`
- `parseRange`
- `parseAdditive`, `parseMultiplicative`

### expressions/unary.ts
- `parseUnary`
- `parsePraefixumExpression`

### expressions/primary.ts
- `parsePrimary`
- `parseCall`
- `parseIdentifier`, `parseIdentifierOrKeyword`
- `parseNovumExpression`
- `parseFingeExpression`
- `parseLambdaExpression`
- `parseQuaExpression`
- `parseLegeExpression`
- `parseScriptumExpression`
- `parseRegexLiteral`
- `parseArgumentList`
- `parseTernary`

### expressions/dsl.ts
- `parseCollectionDSLExpression`
- `parseDSLTransforms`
- `parseAbExpression`
- `isDSLVerb`

### types.ts
- `parseTypeAnnotation`
- `parseTypeAndParameterList`
- `parseParameterList`
- `parseParameter`

## Execution Plan

### Phase 1: Infrastructure
1. Create `parser/resolver.ts` with Resolver interface
2. Create `parser/context.ts` with ParserContext class
3. Extract token helpers and state management

### Phase 2: Type Parsing
4. Create `parser/types.ts` - smallest, fewest dependencies

### Phase 3: Expression Parsing
5. Create `parser/expressions/binary.ts`
6. Create `parser/expressions/unary.ts`
7. Create `parser/expressions/primary.ts`
8. Create `parser/expressions/dsl.ts`
9. Create `parser/expressions/index.ts`

### Phase 4: Statement Parsing
10. Create `parser/statements/blocks.ts`
11. Create `parser/statements/actions.ts`
12. Create `parser/statements/variables.ts`
13. Create `parser/statements/imports.ts`
14. Create `parser/statements/errors.ts`
15. Create `parser/statements/testing.ts`
16. Create `parser/statements/control.ts`
17. Create `parser/statements/loops.ts`
18. Create `parser/statements/declarations.ts`
19. Create `parser/statements/index.ts`

### Phase 5: Wiring
20. Update `parser/index.ts` with Resolver implementation
21. Run tests, fix any issues
22. Delete dead code from original index.ts

## Verification

After each phase:
```bash
bun test fons/faber/parser/index.test.ts
bun run typecheck
```

## Notes

- Keep public API unchanged: `parse(tokens: Token[]): ParserResult`
- Each parsing function signature: `(r: Resolver) => ASTNode`
- ParserContext accessed via `r.ctx()`
- Cross-module recursion via `r.expression()`, `r.statement()`, etc.
- Annotations could stay in statements/index.ts or get their own file
