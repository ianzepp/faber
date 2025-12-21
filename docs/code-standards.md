# Systems Language Developer Code Standards

Use these standards when writing or reviewing code in this compiler.

---

## The Mindset

Write code as if a PL theory professor and a production compiler engineer were pair programming. Every module should be:

1. **Theoretically grounded** - Know which compiler phase you're in, what invariants hold
2. **Defensively robust** - Handle malformed input gracefully, never crash on bad source
3. **Exhaustively documented** - Future maintainers should understand the WHY
4. **Testable in isolation** - Each phase should be testable without the others

---

## Module Header Block

Every file should begin with a comprehensive header:

```typescript
/**
 * [Module Name] - [One-line description]
 *
 * COMPILER PHASE
 * ==============
 * [lexical | syntactic | semantic | codegen | runtime]
 *
 * ARCHITECTURE
 * ============
 * [2-4 paragraphs explaining the module's role in the compilation pipeline]
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  [What this phase receives - tokens, AST nodes, etc.]
 * OUTPUT: [What this phase produces]
 * ERRORS: [What error conditions this phase can detect]
 *
 * INVARIANTS
 * ==========
 * INV-1: [First invariant that must hold]
 * INV-2: [Second invariant]
 *
 * GRAMMAR (if applicable)
 * =======================
 * [EBNF or similar notation for what this module parses]
 *
 * @module [module-name]
 */
```

---

## Section Markers

Use consistent visual structure:

```typescript
// =============================================================================
// MAJOR SECTION (e.g., TYPES, CONSTANTS, PARSING FUNCTIONS)
// =============================================================================

// ---------------------------------------------------------------------------
// Minor Section (within a major section)
// ---------------------------------------------------------------------------
```

---

## Constants

Document the reasoning behind magic values:

```typescript
// =============================================================================
// CONSTANTS
// =============================================================================

/**
 * Maximum nesting depth for expressions.
 * WHY: Prevents stack overflow on deeply nested input.
 *      1000 is sufficient for any reasonable program.
 */
const MAX_NESTING_DEPTH = 1000;

/**
 * Token types for Latin keywords.
 * WHY: Separate from identifiers to enable O(1) keyword lookup.
 */
const KEYWORDS = new Set(['si', 'aliter', 'dum', 'redde']);
```

---

## Types and Interfaces

Document the design rationale:

```typescript
// =============================================================================
// TYPES
// =============================================================================

/**
 * Abstract Syntax Tree node for binary expressions.
 *
 * DESIGN: Uses discriminated union (type field) for exhaustive matching.
 *         Operator stored as string to preserve source representation.
 *
 * INVARIANT: left and right are never null after successful parse.
 */
export interface BinaryExpression extends BaseNode {
  type: "BinaryExpression";
  operator: string;
  left: Expression;
  right: Expression;
}
```

---

## Function Documentation

### Parser Functions

```typescript
/**
 * Parse a primary expression.
 *
 * GRAMMAR:
 *   primary := IDENTIFIER | LITERAL | '(' expression ')'
 *
 * PRECEDENCE: Highest (binds tightest)
 *
 * ERROR RECOVERY: On unexpected token, emits error and returns ErrorNode
 *                 to allow continued parsing.
 *
 * @returns The parsed expression node
 * @throws Never - errors are collected, not thrown
 */
function parsePrimary(): Expression {
```

### Codegen Functions

```typescript
/**
 * Generate code for a function declaration.
 *
 * TRANSFORMS:
 *   functio salve(nomen: Textus) → function salve(nomen: string)
 *   futura functio f()           → async function f()
 *
 * TARGET DIFFERENCES:
 *   TypeScript: Emits type annotations
 *   Zig:        Emits fn with explicit return type
 *
 * @param node - The function declaration AST node
 * @returns Generated source code string
 */
function genFunctionDeclaration(node: FunctionDeclaration): string {
```

---

## Inline Comments

Explain the WHY, not the WHAT:

```typescript
// BAD: Check if token is keyword
if (isKeyword(token.value)) { ... }

// GOOD: Latin keywords are case-insensitive (unlike identifiers)
if (isKeyword(token.value.toLowerCase())) { ... }

// BAD: Skip whitespace
while (isWhitespace(current())) advance();

// GOOD: Whitespace is not significant in Latin syntax (like C, unlike Python)
while (isWhitespace(current())) advance();
```

Use tags for important context:

```typescript
// WHY: Latin nouns can end in -us, -um, -i, etc. - all map to same stem
const stem = extractStem(word);

// EDGE: Empty source file is valid (produces empty Program node)
if (tokens.length === 0) return { type: "Program", body: [] };

// PERF: Pre-compute keyword set for O(1) lookup instead of O(n) array search
const keywordSet = new Set(keywords.map(k => k.latin));

// TARGET: Zig requires explicit types for var declarations
if (kind === "var" && node.init) {
  typeAnno = `: ${inferZigType(node.init)}`;
}
```

---

## Error Handling

### Never Crash on Bad Input

```typescript
// BAD: Throws on unexpected input
function parseStatement(): Statement {
  throw new Error(`Unexpected token: ${token.type}`);
}

// GOOD: Collect error and continue
function parseStatement(): Statement {
  errors.push({
    message: `Unexpected token: ${token.type}`,
    position: token.position,
  });
  synchronize(); // Skip to next statement boundary
  return createErrorNode(token.position);
}
```

### Error Messages Should Teach

```typescript
// BAD: Cryptic error
"Unexpected token"

// GOOD: Actionable error that teaches Latin syntax
"Expected 'aliter' or end of statement after si-block. " +
"In Latin, 'aliter' (else) must immediately follow the closing brace."
```

---

## AST Design Principles

### 1. Preserve Source Information

```typescript
interface BaseNode {
  /** Source position for error reporting */
  position: Position;
  /** Original source text (optional, for error messages) */
  raw?: string;
}
```

### 2. Use Discriminated Unions

```typescript
// GOOD: Exhaustive matching possible
type Expression =
  | Identifier
  | Literal
  | BinaryExpression
  | CallExpression;

function visit(node: Expression) {
  switch (node.type) {
    case "Identifier": ...
    case "Literal": ...
    // TypeScript ensures all cases handled
  }
}
```

### 3. Separate Syntax from Semantics

```typescript
// Syntax (parser output): preserves Latin names
interface VariableDeclaration {
  kind: "esto" | "fixum";  // Latin keywords
  name: Identifier;
  typeAnnotation?: TypeAnnotation;  // Textus, Numerus, etc.
}

// Semantics (after analysis): resolved to target language
interface ResolvedVariable {
  kind: "let" | "const";  // Target semantics
  jsType: string;         // Resolved type
}
```

---

## Testing Patterns

### Test Each Phase Independently

```typescript
describe("tokenizer", () => {
  test("tokenizes Latin keywords", () => {
    const { tokens } = tokenize("si verum");
    expect(tokens[0].type).toBe("KEYWORD");
    expect(tokens[0].value).toBe("si");
  });
});

describe("parser", () => {
  // Feed tokens directly, don't go through tokenizer
  test("parses if statement", () => {
    const tokens = [
      { type: "KEYWORD", value: "si", position: ... },
      ...
    ];
    const { program } = parse(tokens);
    expect(program.body[0].type).toBe("IfStatement");
  });
});
```

### Test Error Recovery

```typescript
test("recovers from missing semicolon", () => {
  const { program, errors } = parse(tokenize("esto x = 1 esto y = 2").tokens);

  // Should parse both declarations despite error
  expect(program.body).toHaveLength(2);
  expect(errors).toHaveLength(1);
  expect(errors[0].message).toContain("expected");
});
```

---

## Multi-Target Considerations

When code differs by target, document clearly:

```typescript
/**
 * Map Latin type to target language type.
 *
 * TARGET MAPPING:
 * | Latin     | TypeScript | Zig        |
 * |-----------|------------|------------|
 * | Textus    | string     | []const u8 |
 * | Numerus   | number     | i64        |
 * | Bivalens  | boolean    | bool       |
 * | Nihil     | null       | null       |
 */
function mapType(latinType: string, target: Target): string {
```

---

## Checklist Before Committing

- [ ] Module header with COMPILER PHASE, ARCHITECTURE, INPUT/OUTPUT CONTRACT
- [ ] All parser functions document their GRAMMAR rule
- [ ] All codegen functions document TARGET DIFFERENCES
- [ ] Error messages are actionable and teach the language
- [ ] No crashes on malformed input - errors collected and reported
- [ ] AST nodes preserve source position for error reporting
- [ ] Tests cover both success cases and error recovery
- [ ] Constants have WHY comments
- [ ] Section markers create clear visual structure
