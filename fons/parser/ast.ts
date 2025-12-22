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
 * 3. Latin keywords are preserved as literals (varia, fixum) for semantic analysis
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

import type { Position } from '../tokenizer/types';
import type { Case, Number as GramNumber } from '../lexicon/types';
import type { SemanticType } from '../semantic/types';

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
    position: Position;
    resolvedType?: SemanticType;
}

/**
 * Program is the root node of the AST.
 *
 * INVARIANT: Body is always an array, never null (empty source = empty array).
 */
export interface Program extends BaseNode {
    type: 'Program';
    body: Statement[];
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
    | GenusDeclaration
    | TypeAliasDeclaration
    | ExpressionStatement
    | IfStatement
    | WhileStatement
    | ForStatement
    | WithStatement
    | SwitchStatement
    | GuardStatement
    | AssertStatement
    | ReturnStatement
    | BlockStatement
    | ThrowStatement
    | TryStatement
    | ScribeStatement;

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
    type: 'ImportDeclaration';
    source: string;
    specifiers: Identifier[];
    wildcard: boolean;
}

// ---------------------------------------------------------------------------
// Variable and Function Declarations
// ---------------------------------------------------------------------------

/**
 * Object destructuring pattern.
 *
 * GRAMMAR (in EBNF):
 *   objectPattern := '{' patternProperty (',' patternProperty)* '}'
 *   patternProperty := IDENTIFIER (':' IDENTIFIER)?
 *
 * WHY: Allows unpacking object properties into variables.
 *
 * Examples:
 *   { nomen, aetas }           -> extract nomen and aetas
 *   { nomen: localName }       -> extract nomen as localName
 */
export interface ObjectPattern extends BaseNode {
    type: 'ObjectPattern';
    properties: ObjectPatternProperty[];
}

/**
 * Single property in an object pattern.
 *
 * key: the property name to extract from the object
 * value: the variable name to bind (same as key if not renamed)
 */
export interface ObjectPatternProperty extends BaseNode {
    type: 'ObjectPatternProperty';
    key: Identifier;
    value: Identifier;
}

/**
 * Variable declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   varDecl := ('varia' | 'fixum') (IDENTIFIER | objectPattern) (':' typeAnnotation)? ('=' expression)?
 *
 * INVARIANT: kind is Latin keyword (varia/fixum), not target language (let/const).
 * INVARIANT: Either typeAnnotation or init SHOULD be present (but not enforced by parser).
 *
 * WHY: Preserves Latin keywords for semantic phase to map to target semantics.
 *
 * Examples:
 *   varia x: numerus = 5
 *   fixum SALVE = "ave"
 *   fixum { nomen, aetas } = persona
 *   fixum { nomen: localName } = persona
 */
export interface VariableDeclaration extends BaseNode {
    type: 'VariableDeclaration';
    kind: 'varia' | 'fixum';
    name: Identifier | ObjectPattern;
    typeAnnotation?: TypeAnnotation;
    init?: Expression;
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
 *   functio salve(nomen: textus) -> textus { ... }
 *   futura functio cede() { ... }
 */
export interface FunctionDeclaration extends BaseNode {
    type: 'FunctionDeclaration';
    name: Identifier;
    params: Parameter[];
    returnType?: TypeAnnotation;
    body: BlockStatement;
    async: boolean;
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
    type: 'Parameter';
    name: Identifier;
    typeAnnotation?: TypeAnnotation;
    case?: Case;
    preposition?: string;
}

/**
 * Type alias declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
 *
 * INVARIANT: name is the alias identifier.
 * INVARIANT: typeAnnotation is the type being aliased.
 *
 * WHY: Enables creating named type aliases for complex types.
 *
 * Examples:
 *   typus ID = textus
 *   typus UserID = numerus<32, Naturalis>
 */
export interface TypeAliasDeclaration extends BaseNode {
    type: 'TypeAliasDeclaration';
    name: Identifier;
    typeAnnotation: TypeAnnotation;
}

// ---------------------------------------------------------------------------
// Genus (Struct) Declarations
// ---------------------------------------------------------------------------

/**
 * Field declaration within a genus.
 *
 * GRAMMAR (in EBNF):
 *   fieldDecl := 'publicus'? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
 *
 * INVARIANT: typeAnnotation uses Latin word order (type before name).
 * INVARIANT: isPublic defaults to false (private by default).
 * INVARIANT: isStatic is true when 'generis' modifier present.
 *
 * WHY: Latin word order places type before name (e.g., "textus nomen" not "nomen: textus").
 * WHY: Field defaults use ':' (declarative "has value") not '=' (imperative "assign").
 *      This aligns with object literal syntax: { nomen: "Marcus" }
 *
 * Examples:
 *   textus nomen                    -> private field
 *   publicus textus nomen           -> public field
 *   numerus aetas: 0                -> field with default
 *   generis fixum PI: 3.14159       -> static constant
 */
export interface FieldDeclaration extends BaseNode {
    type: 'FieldDeclaration';
    name: Identifier;
    fieldType: TypeAnnotation;
    init?: Expression;
    isPublic: boolean;
    isStatic: boolean;
}

/**
 * Genus (struct/class) declaration.
 *
 * GRAMMAR (in EBNF):
 *   genusDecl := 'genus' IDENTIFIER typeParams? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
 *   genusMember := fieldDecl | methodDecl
 *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
 *
 * INVARIANT: name is the type name (lowercase by convention).
 * INVARIANT: fields contains all field declarations.
 * INVARIANT: methods contains all method declarations (FunctionDeclaration with implicit ego).
 * INVARIANT: implements lists pactum names this genus fulfills.
 *
 * WHY: Latin 'genus' (kind/type) for data structures with fields and methods.
 *      No inheritance - composition and pactum (interfaces) only.
 *
 * Examples:
 *   genus persona {
 *       textus nomen
 *       numerus aetas
 *   }
 *
 *   genus persona implet iterabilis {
 *       textus nomen
 *       functio sequens() -> textus? { ... }
 *   }
 */
export interface GenusDeclaration extends BaseNode {
    type: 'GenusDeclaration';
    name: Identifier;
    typeParameters?: Identifier[];
    implements?: Identifier[];
    fields: FieldDeclaration[];
    methods: FunctionDeclaration[];
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
    type: 'ExpressionStatement';
    expression: Expression;
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
    type: 'IfStatement';
    test: Expression;
    consequent: BlockStatement;
    alternate?: BlockStatement | IfStatement;
    catchClause?: CatchClause;
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
    type: 'WhileStatement';
    test: Expression;
    body: BlockStatement;
    catchClause?: CatchClause;
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
    type: 'ForStatement';
    kind: 'in' | 'ex';
    variable: Identifier;
    iterable: Expression;
    body: BlockStatement;
    catchClause?: CatchClause;
}

/**
 * With statement (context block).
 *
 * GRAMMAR (in EBNF):
 *   withStmt := 'cum' expression blockStmt
 *
 * WHY: Latin 'cum' (with) establishes context for property access.
 *      Inside the block, bare identifiers in assignments refer to
 *      properties of the context object.
 *
 * Example:
 *   cum user {
 *       nomen = "Marcus"
 *       email = "marcus@roma.it"
 *   }
 *   // Compiles to: user.nomen = "Marcus"; user.email = "marcus@roma.it";
 */
export interface WithStatement extends BaseNode {
    type: 'WithStatement';
    object: Expression;
    body: BlockStatement;
}

/**
 * Switch statement.
 *
 * GRAMMAR (in EBNF):
 *   switchStmt := 'elige' expression '{' switchCase* defaultCase? '}' catchClause?
 *   switchCase := 'si' expression blockStmt
 *   defaultCase := 'aliter' blockStmt
 *
 * WHY: Latin 'elige' (choose) for switch, 'si' (if) for cases.
 *      Uses 'aliter' (otherwise) for default case.
 *
 * Example:
 *   elige status {
 *       si "pending" { processPending() }
 *       si "active" { processActive() }
 *       aliter { processDefault() }
 *   }
 */
export interface SwitchStatement extends BaseNode {
    type: 'SwitchStatement';
    discriminant: Expression;
    cases: SwitchCase[];
    defaultCase?: BlockStatement;
    catchClause?: CatchClause;
}

/**
 * Switch case (part of switch statement).
 */
export interface SwitchCase extends BaseNode {
    type: 'SwitchCase';
    test: Expression;
    consequent: BlockStatement;
}

/**
 * Guard statement (grouped early-exit checks).
 *
 * GRAMMAR (in EBNF):
 *   guardStmt := 'custodi' '{' guardClause+ '}'
 *   guardClause := 'si' expression blockStmt
 *
 * WHY: Latin 'custodi' (guard!) groups early-exit conditions.
 *      Each clause should contain an early exit (redde, iace, rumpe, perge).
 *
 * Example:
 *   custodi {
 *       si user == nihil { redde nihil }
 *       si useri age < 0 { iace "Invalid age" }
 *   }
 */
export interface GuardStatement extends BaseNode {
    type: 'GuardStatement';
    clauses: GuardClause[];
}

/**
 * Guard clause (part of guard statement).
 */
export interface GuardClause extends BaseNode {
    type: 'GuardClause';
    test: Expression;
    consequent: BlockStatement;
}

/**
 * Assert statement.
 *
 * GRAMMAR (in EBNF):
 *   assertStmt := 'adfirma' expression (',' STRING)?
 *
 * WHY: Latin 'adfirma' (affirm/assert) for runtime invariant checks.
 *      Always-on runtime checks - if condition is false, throws an error.
 *      Optional message for custom error text; otherwise auto-generated.
 *
 * Example:
 *   adfirma x > 0
 *   adfirma x > 0, "x must be positive"
 */
export interface AssertStatement extends BaseNode {
    type: 'AssertStatement';
    test: Expression;
    message?: Expression;
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
    type: 'ReturnStatement';
    argument?: Expression;
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
    type: 'BlockStatement';
    body: Statement[];
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
    type: 'ThrowStatement';
    argument: Expression;
}

/**
 * Scribe (print) statement.
 *
 * GRAMMAR (in EBNF):
 *   scribeStmt := 'scribe' expression (',' expression)*
 *
 * WHY: Latin 'scribe' (write!) as a statement keyword, not a function call.
 *      Supports printf-style format strings passed directly to target runtime.
 *
 * Examples:
 *   scribe "hello"
 *   scribe "%s: %d", name, count
 */
export interface ScribeStatement extends BaseNode {
    type: 'ScribeStatement';
    arguments: Expression[];
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
    type: 'TryStatement';
    block: BlockStatement;
    handler?: CatchClause;
    finalizer?: BlockStatement;
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
    type: 'CatchClause';
    param: Identifier;
    body: BlockStatement;
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
    | ArrayExpression
    | ObjectExpression
    | RangeExpression
    | BinaryExpression
    | UnaryExpression
    | CallExpression
    | MemberExpression
    | ArrowFunctionExpression
    | AssignmentExpression
    | ConditionalExpression
    | AwaitExpression
    | NewExpression
    | TemplateLiteral;

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
    type: 'Identifier';
    name: string;
    morphology?: {
        stem: string;
        case?: Case;
        number?: GramNumber;
    };
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
    type: 'Literal';
    value: string | number | boolean | null;
    raw: string;
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
    type: 'TemplateLiteral';
    raw: string;
}

/**
 * Array literal expression.
 *
 * GRAMMAR (in EBNF):
 *   arrayExpr := '[' (expression (',' expression)*)? ']'
 *
 * INVARIANT: elements is always an array (empty array = empty elements).
 *
 * Examples:
 *   []           -> elements=[]
 *   [1, 2, 3]    -> elements=[Literal, Literal, Literal]
 *   [[1], [2]]   -> elements=[ArrayExpression, ArrayExpression]
 */
export interface ArrayExpression extends BaseNode {
    type: 'ArrayExpression';
    elements: Expression[];
}

/**
 * Object literal expression.
 *
 * GRAMMAR (in EBNF):
 *   objectExpr := '{' (objectProperty (',' objectProperty)*)? '}'
 *   objectProperty := (IDENTIFIER | STRING) ':' expression
 *
 * WHY: Object literals are the primary way to create structured data.
 *
 * Examples:
 *   {}                           -> empty object
 *   { nomen: "Marcus" }          -> single property
 *   { nomen: "Marcus", aetas: 30 } -> multiple properties
 */
export interface ObjectExpression extends BaseNode {
    type: 'ObjectExpression';
    properties: ObjectProperty[];
}

/**
 * Single property in an object literal.
 *
 * key: property name (identifier or string)
 * value: property value expression
 */
export interface ObjectProperty extends BaseNode {
    type: 'ObjectProperty';
    key: Identifier | Literal;
    value: Expression;
}

/**
 * Range expression for iteration bounds.
 *
 * GRAMMAR (in EBNF):
 *   rangeExpr := expression '..' expression ('per' expression)?
 *
 * WHY: Provides concise syntax for numeric iteration ranges.
 *      End is inclusive (0..10 includes 10).
 *      Optional step via 'per' keyword.
 *
 * Examples:
 *   0..10           -> start=0, end=10, step=undefined (default 1)
 *   0..10 per 2     -> start=0, end=10, step=2
 *   10..0 per -1    -> start=10, end=0, step=-1 (countdown)
 */
export interface RangeExpression extends BaseNode {
    type: 'RangeExpression';
    start: Expression;
    end: Expression;
    step?: Expression;
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
    type: 'BinaryExpression';
    operator: string;
    left: Expression;
    right: Expression;
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
    type: 'UnaryExpression';
    operator: string;
    argument: Expression;
    prefix: boolean;
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
    type: 'CallExpression';
    callee: Expression;
    arguments: Expression[];
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
    type: 'MemberExpression';
    object: Expression;
    property: Identifier;
    computed: boolean;
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
    type: 'ArrowFunctionExpression';
    params: Parameter[];
    body: Expression | BlockStatement;
    async: boolean;
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
    type: 'AssignmentExpression';
    operator: string;
    left: Identifier | MemberExpression;
    right: Expression;
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
    type: 'ConditionalExpression';
    test: Expression;
    consequent: Expression;
    alternate: Expression;
}

// ---------------------------------------------------------------------------
// Async and Object Creation
// ---------------------------------------------------------------------------

/**
 * Await expression.
 *
 * GRAMMAR (in EBNF):
 *   awaitExpr := 'cede' expression
 *
 * INVARIANT: argument is never null.
 *
 * WHY: Latin 'cede' (to wait for) for async/await.
 */
export interface AwaitExpression extends BaseNode {
    type: 'AwaitExpression';
    argument: Expression;
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
    type: 'NewExpression';
    callee: Identifier;
    arguments: Expression[];
}

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

/**
 * Type parameter for parameterized types.
 *
 * DESIGN: Union type allows different parameter kinds:
 *         - TypeAnnotation: Generic type params (lista<textus>)
 *         - Literal: Numeric params (numerus<32>) or size specs
 *         - ModifierParameter: Type modifiers (numerus<Naturalis>)
 *
 * WHY: Type parameters aren't always types - they can be size constraints
 *      or ownership/mutability modifiers.
 */
export type TypeParameter = TypeAnnotation | Literal | ModifierParameter;

/**
 * Modifier parameter for type annotations.
 *
 * GRAMMAR (in EBNF):
 *   modifierParam := 'Naturalis' | 'Proprius' | 'Alienus' | 'Mutabilis'
 *
 * INVARIANT: name is one of the four supported modifiers.
 *
 * WHY: Type modifiers control numeric signedness, ownership semantics, etc.
 *      - Naturalis: unsigned/natural numbers
 *      - Proprius: owned (move semantics)
 *      - Alienus: borrowed (reference semantics)
 *      - Mutabilis: mutable
 *
 * Examples:
 *   numerus<Naturalis> -> unsigned integer
 *   textus<Alienus> -> borrowed string reference
 */
export interface ModifierParameter extends BaseNode {
    type: 'ModifierParameter';
    name: 'Naturalis' | 'Proprius' | 'Alienus' | 'Mutabilis';
}

/**
 * Type annotation for variables, parameters, and return types.
 *
 * GRAMMAR (in EBNF):
 *   typeAnnotation := IDENTIFIER typeParams? '?'? ('|' typeAnnotation)*
 *   typeParams := '<' typeParameter (',' typeParameter)* '>'
 *
 * INVARIANT: name is the base type name (textus, numerus, etc.).
 * INVARIANT: nullable indicates optional type with '?'.
 * INVARIANT: union contains multiple types for union types (A | B).
 * INVARIANT: typeParameters can contain types, literals, or modifiers.
 *
 * Examples:
 *   textus -> name="textus"
 *   numerus? -> name="numerus", nullable=true
 *   lista<textus> -> name="lista", typeParameters=[TypeAnnotation]
 *   numerus<32> -> name="numerus", typeParameters=[Literal{value=32}]
 *   numerus<Naturalis> -> name="numerus", typeParameters=[ModifierParameter]
 *   textus | numerus -> name="union", union=[{name="textus"}, {name="numerus"}]
 */
export interface TypeAnnotation extends BaseNode {
    type: 'TypeAnnotation';
    name: string;
    typeParameters?: TypeParameter[];
    nullable?: boolean;
    union?: TypeAnnotation[];
    arrayShorthand?: boolean; // true if parsed from [] syntax (e.g., numerus[] vs lista<numerus>)
}
