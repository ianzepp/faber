/**
 * AST Node Definitions - Abstract Syntax Tree type definitions
 *
 * COMPILER PHASE
 * ==============
 * syntactic
 *
 * ARCHITECTURE
 * ============
 * This module defines the Abstract Syntax Tree (AST) node types produced by the
 * parser. The AST is a structured representation of Latin source code that preserves
 * syntactic information while abstracting away lexical details like whitespace.
 *
 * The AST design follows several key principles:
 * 1. All nodes extend BaseNode to carry source position for error reporting
 * 2. Discriminated unions (via 'type' field) enable exhaustive pattern matching
 * 3. Latin keywords are preserved as literals (esto, fixum) for semantic analysis
 * 4. Optional morphology info on Identifiers enables case-aware transformations
 *
 * This AST sits between the parser and semantic analyzer in the pipeline. It
 * deliberately preserves Latin-specific syntax (like prepositional parameters)
 * that will be transformed into target language constructs during code generation.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (type definitions only)
 * OUTPUT: Type definitions exported for parser and codegen phases
 * ERRORS: N/A (compile-time type checking only)
 *
 * INVARIANTS
 * ==========
 * INV-1: All AST nodes MUST include a position field for error reporting
 * INV-2: Discriminated union types MUST have unique 'type' string literals
 * INV-3: Optional fields use ? notation, never null/undefined unions
 * INV-4: Node types preserve Latin syntax, NOT target language semantics
 *
 * @module parser/ast
 */

import type { Position } from "../tokenizer/types"
import type { Case, Number as GramNumber } from "../lexicon/types"
import type { SemanticType } from "../semantic/types"

// =============================================================================
// BASE TYPES
// =============================================================================

/**
 * Base node with position for error reporting.
 *
 * INVARIANT: Every AST node extends this to ensure source tracking.
 *
 * The resolvedType field is populated by the semantic analyzer and used by
 * code generators to make type-aware decisions.
 */
export interface BaseNode {
  position: Position
  resolvedType?: SemanticType
}

/**
 * Program is the root node of the AST.
 *
 * INVARIANT: Body is always an array, never null (empty source = empty array).
 */
export interface Program extends BaseNode {
  type: "Program"
  body: Statement[]
}

// =============================================================================
// STATEMENT TYPES
// =============================================================================

/**
 * Discriminated union of all statement types.
 *
 * DESIGN: TypeScript discriminated union enables exhaustive switch statements
 *         in visitors and transformers.
 */
export type Statement =
  | ImportDeclaration
  | VariableDeclaration
  | FunctionDeclaration
  | ExpressionStatement
  | IfStatement
  | WhileStatement
  | ForStatement
  | ReturnStatement
  | BlockStatement
  | ThrowStatement
  | TryStatement

// ---------------------------------------------------------------------------
// Import/Export Declarations
// ---------------------------------------------------------------------------

/**
 * Import declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   importDecl := 'ex' IDENTIFIER 'importa' (importSpec | '*')
 *   importSpec := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: Either specifiers is non-empty OR wildcard is true.
 * INVARIANT: source is never empty string.
 *
 * Examples:
 *   ex norma importa scribe, lege  -> source="norma", specifiers=[scribe, lege]
 *   ex norma importa *             -> source="norma", wildcard=true
 */
export interface ImportDeclaration extends BaseNode {
  type: "ImportDeclaration"
  source: string
  specifiers: Identifier[]
  wildcard: boolean
}

// ---------------------------------------------------------------------------
// Variable and Function Declarations
// ---------------------------------------------------------------------------

/**
 * Variable declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   varDecl := ('esto' | 'fixum') IDENTIFIER (':' typeAnnotation)? ('=' expression)?
 *
 * INVARIANT: kind is Latin keyword (esto/fixum), not target language (let/const).
 * INVARIANT: Either typeAnnotation or init SHOULD be present (but not enforced by parser).
 *
 * WHY: Preserves Latin keywords for semantic phase to map to target semantics.
 *
 * Examples:
 *   esto x: Numerus = 5
 *   fixum SALVE = "ave"
 */
export interface VariableDeclaration extends BaseNode {
  type: "VariableDeclaration"
  kind: "esto" | "fixum"
  name: Identifier
  typeAnnotation?: TypeAnnotation
  init?: Expression
}

/**
 * Function declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   funcDecl := 'futura'? 'functio' IDENTIFIER '(' paramList ')' ('->' typeAnnotation)? blockStmt
 *   paramList := (parameter (',' parameter)*)?
 *
 * INVARIANT: async flag set by presence of 'futura' keyword.
 * INVARIANT: params is always an array (empty if no parameters).
 *
 * Examples:
 *   functio salve(nomen: Textus) -> Textus { ... }
 *   futura functio exspecta() { ... }
 */
export interface FunctionDeclaration extends BaseNode {
  type: "FunctionDeclaration"
  name: Identifier
  params: Parameter[]
  returnType?: TypeAnnotation
  body: BlockStatement
  async: boolean
}

/**
 * Function parameter.
 *
 * GRAMMAR (in EBNF):
 *   parameter := ('ad' | 'cum' | 'in' | 'ex')? IDENTIFIER (':' typeAnnotation)?
 *
 * INVARIANT: preposition is Latin (ad/cum/in/ex), not English (to/with/in/from).
 * INVARIANT: case and preposition enable semantic analysis of parameter roles.
 *
 * WHY: Latin prepositions indicate semantic roles that map to different constructs
 *      in target languages (e.g., 'ad' might indicate a callback parameter).
 */
export interface Parameter extends BaseNode {
  type: "Parameter"
  name: Identifier
  typeAnnotation?: TypeAnnotation
  case?: Case
  preposition?: string
}

// ---------------------------------------------------------------------------
// Control Flow Statements
// ---------------------------------------------------------------------------

/**
 * Expression statement (expression used as statement).
 *
 * GRAMMAR (in EBNF):
 *   exprStmt := expression
 *
 * INVARIANT: expression is never null.
 */
export interface ExpressionStatement extends BaseNode {
  type: "ExpressionStatement"
  expression: Expression
}

/**
 * Conditional (if) statement.
 *
 * GRAMMAR (in EBNF):
 *   ifStmt := 'si' expression blockStmt ('cape' IDENTIFIER blockStmt)? ('aliter' (ifStmt | blockStmt))?
 *
 * INVARIANT: catchClause is unique to Latin - allows error handling within conditionals.
 * INVARIANT: alternate can be BlockStatement (else) or IfStatement (else if chain).
 *
 * WHY: Latin 'cape' clause enables localized error handling in conditionals,
 *      not found in most target languages.
 *
 * Examples:
 *   si x > 0 { ... }
 *   si x > 0 { ... } cape erratum { ... }
 *   si x > 0 { ... } aliter { ... }
 *   si x > 0 { ... } aliter si x < 0 { ... } aliter { ... }
 */
export interface IfStatement extends BaseNode {
  type: "IfStatement"
  test: Expression
  consequent: BlockStatement
  alternate?: BlockStatement | IfStatement
  catchClause?: CatchClause
}

/**
 * While loop statement.
 *
 * GRAMMAR (in EBNF):
 *   whileStmt := 'dum' expression blockStmt ('cape' IDENTIFIER blockStmt)?
 *
 * INVARIANT: catchClause allows error handling within loop iterations.
 */
export interface WhileStatement extends BaseNode {
  type: "WhileStatement"
  test: Expression
  body: BlockStatement
  catchClause?: CatchClause
}

/**
 * For loop statement.
 *
 * GRAMMAR (in EBNF):
 *   forStmt := 'pro' IDENTIFIER ('in' | 'ex') expression blockStmt ('cape' IDENTIFIER blockStmt)?
 *
 * INVARIANT: kind is 'in' (for...in) or 'ex' (for...of).
 *
 * WHY: Latin distinguishes iteration kinds with different prepositions:
 *      'in' = iterate over keys/indices
 *      'ex' = iterate over values (from/out of collection)
 */
export interface ForStatement extends BaseNode {
  type: "ForStatement"
  kind: "in" | "ex"
  variable: Identifier
  iterable: Expression
  body: BlockStatement
  catchClause?: CatchClause
}

/**
 * Return statement.
 *
 * GRAMMAR (in EBNF):
 *   returnStmt := 'redde' expression?
 *
 * INVARIANT: argument is optional (void return).
 */
export interface ReturnStatement extends BaseNode {
  type: "ReturnStatement"
  argument?: Expression
}

/**
 * Block statement (sequence of statements in braces).
 *
 * GRAMMAR (in EBNF):
 *   blockStmt := '{' statement* '}'
 *
 * INVARIANT: body is always an array (empty block = empty array).
 */
export interface BlockStatement extends BaseNode {
  type: "BlockStatement"
  body: Statement[]
}

// ---------------------------------------------------------------------------
// Exception Handling
// ---------------------------------------------------------------------------

/**
 * Throw statement.
 *
 * GRAMMAR (in EBNF):
 *   throwStmt := 'iace' expression
 *
 * INVARIANT: argument is never null.
 *
 * WHY: Latin 'iace' (to throw/hurl) for throwing exceptions.
 */
export interface ThrowStatement extends BaseNode {
  type: "ThrowStatement"
  argument: Expression
}

/**
 * Try-catch-finally statement.
 *
 * GRAMMAR (in EBNF):
 *   tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
 *
 * INVARIANT: At least one of handler or finalizer SHOULD be present.
 *
 * WHY: Latin keywords:
 *      tempta = try (attempt)
 *      cape = catch (seize/capture)
 *      demum = finally (at last)
 */
export interface TryStatement extends BaseNode {
  type: "TryStatement"
  block: BlockStatement
  handler?: CatchClause
  finalizer?: BlockStatement
}

/**
 * Catch clause (part of try or control flow statements).
 *
 * GRAMMAR (in EBNF):
 *   catchClause := 'cape' IDENTIFIER blockStmt
 *
 * INVARIANT: param is the error variable name.
 *
 * WHY: Reusable in both TryStatement and control flow (IfStatement, loops).
 */
export interface CatchClause extends BaseNode {
  type: "CatchClause"
  param: Identifier
  body: BlockStatement
}

// =============================================================================
// EXPRESSION TYPES
// =============================================================================

/**
 * Discriminated union of all expression types.
 *
 * DESIGN: Expressions produce values, statements perform actions.
 */
export type Expression =
  | Identifier
  | Literal
  | BinaryExpression
  | UnaryExpression
  | CallExpression
  | MemberExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | ConditionalExpression
  | AwaitExpression
  | NewExpression
  | TemplateLiteral

// ---------------------------------------------------------------------------
// Primary Expressions
// ---------------------------------------------------------------------------

/**
 * Identifier (variable/function name).
 *
 * GRAMMAR (in EBNF):
 *   identifier := IDENTIFIER
 *
 * INVARIANT: name is the raw identifier string from source.
 * INVARIANT: morphology is optional - populated by lexicon if Latin word recognized.
 *
 * WHY: morphology enables case-aware semantic analysis but is not required
 *      for parsing (allows non-Latin identifiers like API names).
 */
export interface Identifier extends BaseNode {
  type: "Identifier"
  name: string
  morphology?: {
    stem: string
    case?: Case
    number?: GramNumber
  }
}

/**
 * Literal value (string, number, boolean, null).
 *
 * GRAMMAR (in EBNF):
 *   literal := STRING | NUMBER | 'verum' | 'falsum' | 'nihil'
 *
 * INVARIANT: value type matches the literal kind.
 * INVARIANT: raw preserves original source text for error messages.
 *
 * Examples:
 *   "hello" -> value="hello", raw='"hello"'
 *   42      -> value=42, raw='42'
 *   verum   -> value=true, raw='verum'
 *   nihil   -> value=null, raw='nihil'
 */
export interface Literal extends BaseNode {
  type: "Literal"
  value: string | number | boolean | null
  raw: string
}

/**
 * Template literal (template string).
 *
 * GRAMMAR (in EBNF):
 *   templateLiteral := '`' templateChar* '`'
 *
 * INVARIANT: raw includes the backticks.
 *
 * WHY: For now stores as raw string. Full implementation would parse
 *      embedded expressions, but that requires template expression tokens.
 */
export interface TemplateLiteral extends BaseNode {
  type: "TemplateLiteral"
  raw: string
}

// ---------------------------------------------------------------------------
// Binary and Unary Expressions
// ---------------------------------------------------------------------------

/**
 * Binary expression (two operands with infix operator).
 *
 * GRAMMAR (in EBNF):
 *   binaryExpr := expression operator expression
 *   operator   := '+' | '-' | '*' | '/' | '%' | '==' | '!=' | '<' | '>' | '<=' | '>=' | '&&' | '||'
 *
 * INVARIANT: left and right are never null after successful parse.
 * INVARIANT: operator is stored as string to preserve source representation.
 *
 * DESIGN: Operator precedence is handled during parsing, not stored in AST.
 */
export interface BinaryExpression extends BaseNode {
  type: "BinaryExpression"
  operator: string
  left: Expression
  right: Expression
}

/**
 * Unary expression (single operand with prefix or postfix operator).
 *
 * GRAMMAR (in EBNF):
 *   unaryExpr := operator expression | expression operator
 *   operator  := '!' | '-' | 'non'
 *
 * INVARIANT: prefix indicates operator position (true = prefix, false = postfix).
 * INVARIANT: argument is never null.
 *
 * WHY: Latin 'non' keyword supported alongside '!' for negation.
 */
export interface UnaryExpression extends BaseNode {
  type: "UnaryExpression"
  operator: string
  argument: Expression
  prefix: boolean
}

// ---------------------------------------------------------------------------
// Call and Member Access
// ---------------------------------------------------------------------------

/**
 * Function call expression.
 *
 * GRAMMAR (in EBNF):
 *   callExpr := expression '(' argumentList ')'
 *   argumentList := (expression (',' expression)*)?
 *
 * INVARIANT: callee can be any expression (Identifier, MemberExpression, etc.).
 * INVARIANT: arguments is always an array (empty for zero-arg calls).
 */
export interface CallExpression extends BaseNode {
  type: "CallExpression"
  callee: Expression
  arguments: Expression[]
}

/**
 * Member access expression.
 *
 * GRAMMAR (in EBNF):
 *   memberExpr := expression '.' IDENTIFIER | expression '[' expression ']'
 *
 * INVARIANT: computed=false means dot notation (obj.prop).
 * INVARIANT: computed=true means bracket notation (obj[prop]).
 *
 * WHY: computed flag enables different code generation strategies.
 */
export interface MemberExpression extends BaseNode {
  type: "MemberExpression"
  object: Expression
  property: Identifier
  computed: boolean
}

// ---------------------------------------------------------------------------
// Function Expressions
// ---------------------------------------------------------------------------

/**
 * Arrow function expression.
 *
 * GRAMMAR (in EBNF):
 *   arrowFuncExpr := '(' paramList ')' '=>' (expression | blockStmt)
 *
 * INVARIANT: body is Expression for concise form, BlockStatement for full form.
 * INVARIANT: params follows same structure as FunctionDeclaration.
 *
 * Examples:
 *   (x) => x + 1
 *   (x, y) => { redde x + y }
 */
export interface ArrowFunctionExpression extends BaseNode {
  type: "ArrowFunctionExpression"
  params: Parameter[]
  body: Expression | BlockStatement
  async: boolean
}

// ---------------------------------------------------------------------------
// Assignment and Conditional
// ---------------------------------------------------------------------------

/**
 * Assignment expression.
 *
 * GRAMMAR (in EBNF):
 *   assignExpr := (IDENTIFIER | memberExpr) '=' expression
 *
 * INVARIANT: left must be Identifier or MemberExpression (lvalue).
 * INVARIANT: operator is '=' for now (compound assignments not yet supported).
 */
export interface AssignmentExpression extends BaseNode {
  type: "AssignmentExpression"
  operator: string
  left: Identifier | MemberExpression
  right: Expression
}

/**
 * Conditional (ternary) expression.
 *
 * GRAMMAR (in EBNF):
 *   conditionalExpr := expression '?' expression ':' expression
 *
 * INVARIANT: test, consequent, and alternate are all required.
 *
 * WHY: Currently not implemented in parser, but defined for future use.
 */
export interface ConditionalExpression extends BaseNode {
  type: "ConditionalExpression"
  test: Expression
  consequent: Expression
  alternate: Expression
}

// ---------------------------------------------------------------------------
// Async and Object Creation
// ---------------------------------------------------------------------------

/**
 * Await expression.
 *
 * GRAMMAR (in EBNF):
 *   awaitExpr := 'exspecta' expression
 *
 * INVARIANT: argument is never null.
 *
 * WHY: Latin 'exspecta' (to wait for) for async/await.
 */
export interface AwaitExpression extends BaseNode {
  type: "AwaitExpression"
  argument: Expression
}

/**
 * New expression (object construction).
 *
 * GRAMMAR (in EBNF):
 *   newExpr := 'novum' IDENTIFIER '(' argumentList ')'
 *
 * INVARIANT: callee is Identifier (constructor name).
 * INVARIANT: arguments is always an array.
 *
 * WHY: Latin 'novum' (new) for object construction.
 */
export interface NewExpression extends BaseNode {
  type: "NewExpression"
  callee: Identifier
  arguments: Expression[]
}

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

/**
 * Type annotation for variables, parameters, and return types.
 *
 * GRAMMAR (in EBNF):
 *   typeAnnotation := IDENTIFIER typeParams? '?'? ('|' typeAnnotation)*
 *   typeParams := '<' typeAnnotation (',' typeAnnotation)* '>'
 *
 * INVARIANT: name is the base type name (Textus, Numerus, etc.).
 * INVARIANT: nullable indicates optional type with '?'.
 * INVARIANT: union contains multiple types for union types (A | B).
 * INVARIANT: typeParameters contains generic type arguments (Array<T>).
 *
 * Examples:
 *   Textus -> name="Textus"
 *   Numerus? -> name="Numerus", nullable=true
 *   Array<Textus> -> name="Array", typeParameters=[{name="Textus"}]
 *   Textus | Numerus -> name="union", union=[{name="Textus"}, {name="Numerus"}]
 */
export interface TypeAnnotation extends BaseNode {
  type: "TypeAnnotation"
  name: string
  typeParameters?: TypeAnnotation[]
  nullable?: boolean
  union?: TypeAnnotation[]
}
