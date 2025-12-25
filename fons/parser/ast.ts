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
    | PactumDeclaration
    | TypeAliasDeclaration
    | EnumDeclaration
    | ExpressionStatement
    | IfStatement
    | WhileStatement
    | ForStatement
    | WithStatement
    | SwitchStatement
    | GuardStatement
    | AssertStatement
    | ReturnStatement
    | BreakStatement
    | ContinueStatement
    | BlockStatement
    | ThrowStatement
    | TryStatement
    | ScribeStatement
    | FacBlockStatement;

// ---------------------------------------------------------------------------
// Import/Export Declarations
// ---------------------------------------------------------------------------

/**
 * Import declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (importSpec | '*')
 *   importSpec := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: Either specifiers is non-empty OR wildcard is true.
 * INVARIANT: source is never empty string.
 *
 * Examples:
 *   ex norma importa scribe, lege       -> source="norma", specifiers=[scribe, lege]
 *   ex "norma/tempus" importa nunc      -> source="norma/tempus", specifiers=[nunc]
 *   ex norma importa *                  -> source="norma", wildcard=true
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
 * rest: if true, collects all remaining properties (ceteri pattern)
 *
 * Examples:
 *   { nomen }                -> key=nomen, value=nomen, rest=false
 *   { nomen: n }             -> key=nomen, value=n, rest=false
 *   { nomen, ceteri rest }   -> second prop: key=rest, value=rest, rest=true
 */
export interface ObjectPatternProperty extends BaseNode {
    type: 'ObjectPatternProperty';
    key: Identifier;
    value: Identifier;
    rest?: boolean;
}

/**
 * Variable declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   varDecl := ('varia' | 'fixum' | 'figendum' | 'variandum') (IDENTIFIER | objectPattern) (':' typeAnnotation)? ('=' expression)?
 *
 * INVARIANT: kind is Latin keyword, not target language (let/const).
 * INVARIANT: Either typeAnnotation or init SHOULD be present (but not enforced by parser).
 *
 * WHY: Preserves Latin keywords for semantic phase to map to target semantics.
 *
 * Async bindings (figendum/variandum) imply await without explicit cede:
 *   figendum = "that which will be fixed" (gerundive) -> const x = await ...
 *   variandum = "that which will be varied" (gerundive) -> let x = await ...
 *
 * Examples:
 *   varia x: numerus = 5
 *   fixum SALVE = "ave"
 *   fixum { nomen, aetas } = persona
 *   figendum data = fetchData()
 *   variandum result = fetchInitial()
 */
export interface VariableDeclaration extends BaseNode {
    type: 'VariableDeclaration';
    kind: 'varia' | 'fixum' | 'figendum' | 'variandum';
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
    generator: boolean;
    isConstructor?: boolean;
}

/**
 * Function parameter.
 *
 * GRAMMAR (in EBNF):
 *   parameter := ('ad' | 'de' | 'in' | 'ex')? 'ceteri'? (typeAnnotation IDENTIFIER | IDENTIFIER)
 *
 * INVARIANT: preposition is Latin (ad/de/in/ex), not English (to/from/in/of).
 * INVARIANT: case and preposition enable semantic analysis of parameter roles.
 * INVARIANT: rest is true when 'ceteri' modifier present (variadic/rest parameter).
 *
 * WHY: Latin prepositions indicate semantic roles that map to different constructs
 *      in target languages. For systems targets (Rust/Zig), 'de' = borrowed/read-only
 *      and 'in' = mutable borrow.
 *
 * Examples:
 *   nomen: textus             -> regular param
 *   ceteri lista<textus> args -> rest param (...args: string[])
 */
export interface Parameter extends BaseNode {
    type: 'Parameter';
    name: Identifier;
    typeAnnotation?: TypeAnnotation;
    case?: Case;
    preposition?: string;
    rest?: boolean;
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
// Enum Declarations
// ---------------------------------------------------------------------------

/**
 * Enum member within an ordo declaration.
 *
 * GRAMMAR (in EBNF):
 *   enumMember := IDENTIFIER ('=' (NUMBER | STRING))?
 *
 * INVARIANT: name is the member identifier.
 * INVARIANT: value is optional; if omitted, auto-increments from previous.
 *
 * Examples:
 *   rubrum           -> auto value
 *   actum = 1        -> explicit numeric
 *   septentrio = "north"  -> string enum
 */
export interface EnumMember extends BaseNode {
    type: 'EnumMember';
    name: Identifier;
    value?: Literal;
}

/**
 * Enum declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   enumDecl := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
 *
 * INVARIANT: name is lowercase (Latin convention).
 * INVARIANT: members is non-empty array of EnumMember.
 *
 * WHY: "ordo" (order/rank) represents enumerated constants.
 *
 * Examples:
 *   ordo color { rubrum, viridis, caeruleum }
 *   ordo status { pendens = 0, actum = 1, finitum = 2 }
 */
export interface EnumDeclaration extends BaseNode {
    type: 'EnumDeclaration';
    name: Identifier;
    members: EnumMember[];
}

// ---------------------------------------------------------------------------
// Genus (Struct) Declarations
// ---------------------------------------------------------------------------

/**
 * Field declaration within a genus.
 *
 * GRAMMAR (in EBNF):
 *   fieldDecl := 'privatus'? 'generis'? 'nexum'? typeAnnotation IDENTIFIER (':' expression)?
 *
 * INVARIANT: typeAnnotation uses Latin word order (type before name).
 * INVARIANT: isPrivate defaults to false (public by default, struct semantics).
 * INVARIANT: isStatic is true when 'generis' modifier present.
 * INVARIANT: isReactive is true when 'nexum' modifier present.
 *
 * WHY: Latin word order places type before name (e.g., "textus nomen" not "nomen: textus").
 * WHY: Field defaults use ':' (declarative "has value") not '=' (imperative "assign").
 *      This aligns with object literal syntax: { nomen: "Marcus" }
 * WHY: Public by default follows struct semantics - genus is a data structure, not a class.
 *
 * Examples:
 *   textus nomen                    -> public field (default)
 *   privatus textus nomen           -> private field
 *   numerus aetas: 0                -> field with default
 *   generis fixum PI: 3.14159       -> static constant
 *   nexum numerus count: 0          -> reactive field (triggers pingo on change)
 */
export interface FieldDeclaration extends BaseNode {
    type: 'FieldDeclaration';
    name: Identifier;
    fieldType: TypeAnnotation;
    init?: Expression;
    isPrivate: boolean;
    isStatic: boolean;
    isReactive: boolean; // nexum fields trigger re-render on change
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
    constructor?: FunctionDeclaration;
    methods: FunctionDeclaration[];
}

// ---------------------------------------------------------------------------
// Pactum (Interface) Declarations
// ---------------------------------------------------------------------------

/**
 * Pactum declaration (interface/protocol contract).
 */
export interface PactumDeclaration extends BaseNode {
    type: 'PactumDeclaration';
    name: Identifier;
    typeParameters?: Identifier[];
    methods: PactumMethod[];
}

/**
 * Pactum method signature (no body, contract only).
 */
export interface PactumMethod extends BaseNode {
    type: 'PactumMethod';
    name: Identifier;
    params: Parameter[];
    returnType?: TypeAnnotation;
    async: boolean;
    generator: boolean;
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
 *   forStmt := ('ex' | 'in') expression ('pro' | 'fit' | 'fiet') IDENTIFIER blockStmt
 *              ('cape' IDENTIFIER blockStmt)?
 *
 * INVARIANT: kind is 'in' (for...in) or 'ex' (for...of).
 *
 * WHY: Latin distinguishes iteration kinds with different prepositions:
 *      'in' = iterate over keys/indices
 *      'ex' = iterate over values (from/out of collection)
 *
 * The binding keyword encodes sync/async:
 *      'pro' = sync iteration (traditional)
 *      'fit' = sync iteration (verb form: "becomes")
 *      'fiet' = async iteration (verb form: "will become")
 */
export interface ForStatement extends BaseNode {
    type: 'ForStatement';
    kind: 'in' | 'ex';
    variable: Identifier;
    iterable: Expression;
    body: BlockStatement;
    async: boolean;
    catchClause?: CatchClause;
}

/**
 * With statement (mutation block).
 *
 * GRAMMAR (in EBNF):
 *   withStmt := 'in' expression blockStmt
 *
 * WHY: Latin 'in' (into) establishes context for property mutation.
 *      Inside the block, bare identifiers in assignments refer to
 *      properties of the context object.
 *
 * Example:
 *   in user {
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
 * Break statement (loop exit).
 *
 * GRAMMAR (in EBNF):
 *   breakStmt := 'rumpe'
 *
 * WHY: Latin 'rumpe' (break!) exits the innermost loop.
 *      Used within dum, ex...pro, and de...pro loops.
 *
 * Example:
 *   dum verum {
 *       si found { rumpe }
 *   }
 */
export interface BreakStatement extends BaseNode {
    type: 'BreakStatement';
}

/**
 * Continue statement (loop skip).
 *
 * GRAMMAR (in EBNF):
 *   continueStmt := 'perge'
 *
 * WHY: Latin 'perge' (continue/proceed!) skips to next iteration.
 *      Used within dum, ex...pro, and de...pro loops.
 *
 * Example:
 *   ex items pro item {
 *       si item.skip { perge }
 *       process(item)
 *   }
 */
export interface ContinueStatement extends BaseNode {
    type: 'ContinueStatement';
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
 * Throw/panic statement.
 *
 * GRAMMAR (in EBNF):
 *   throwStmt := ('iace' | 'mori') expression
 *
 * INVARIANT: argument is never null.
 *
 * WHY: Latin error keywords for two severity levels:
 *   iace (throw!) → recoverable error, can be caught
 *   mori (die!)   → fatal/panic, unrecoverable
 *
 * Target mappings:
 *   iace → throw (TS/Py), return error.X (Zig), return Err (Rust)
 *   mori → throw (TS/Py), @panic (Zig), panic! (Rust)
 */
export interface ThrowStatement extends BaseNode {
    type: 'ThrowStatement';
    fatal: boolean;
    argument: Expression;
}

/**
 * Output level for scribe/vide/mone statements.
 *
 * WHY: Latin has three output keywords mapping to different console levels:
 *   scribe (write!) → console.log  - normal output
 *   vide (see!)     → console.debug - developer/debug output
 *   mone (warn!)    → console.warn  - warning output
 */
export type OutputLevel = 'log' | 'debug' | 'warn';

/**
 * Scribe (print) statement.
 *
 * GRAMMAR (in EBNF):
 *   scribeStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
 *
 * WHY: Latin output keywords as statement forms, not function calls.
 *   scribe (write!) → console.log  - normal output
 *   vide (see!)     → console.debug - developer/debug output
 *   mone (warn!)    → console.warn  - warning output
 *
 * Examples:
 *   scribe "hello"
 *   vide "debugging:", value
 *   mone "warning: value is", x
 */
export interface ScribeStatement extends BaseNode {
    type: 'ScribeStatement';
    level: OutputLevel;
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

// ---------------------------------------------------------------------------
// Fac (Do) Block and Lambda
// ---------------------------------------------------------------------------

/**
 * Fac block statement (explicit scope block).
 *
 * GRAMMAR (in EBNF):
 *   facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)?
 *
 * INVARIANT: body is always a BlockStatement.
 * INVARIANT: catchClause is optional - for error handling.
 *
 * WHY: Latin 'fac' (do!) creates an explicit scope boundary.
 *      Unlike `si verum { }`, this communicates intent clearly.
 *      When paired with 'cape', provides error boundary semantics.
 *
 * Examples:
 *   fac { riskyOperation() }
 *   fac { riskyOperation() } cape err { handleError(err) }
 */
export interface FacBlockStatement extends BaseNode {
    type: 'FacBlockStatement';
    body: BlockStatement;
    catchClause?: CatchClause;
}

/**
 * Pro expression (lambda/anonymous function).
 *
 * GRAMMAR (in EBNF):
 *   lambdaExpr := 'pro' params? ('redde' expression | blockStmt)
 *   params := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: params is always an array (empty for zero-arg lambdas).
 * INVARIANT: async inferred from presence of cede in block body.
 *
 * WHY: Latin 'pro' (for) + 'redde' (return) creates lambda syntax.
 *      Expression form: "for x, return x * 2"
 *      Block form: "for x { ... }" for multi-statement bodies
 *
 * Examples:
 *   pro x redde x * 2     -> (x) => x * 2
 *   pro x, y redde x + y  -> (x, y) => x + y
 *   pro redde 42          -> () => 42
 *   pro x { redde x * 2 } -> (x) => { return x * 2; }
 *   pro { scribe "hi" }   -> () => { console.log("hi"); }
 */
export interface LambdaExpression extends BaseNode {
    type: 'LambdaExpression';
    params: Identifier[];
    body: Expression | BlockStatement;
    async: boolean;
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
    | ThisExpression
    | Literal
    | ArrayExpression
    | ObjectExpression
    | RangeExpression
    | BinaryExpression
    | UnaryExpression
    | TypeCastExpression
    | CallExpression
    | MemberExpression
    | ArrowFunctionExpression
    | AssignmentExpression
    | ConditionalExpression
    | AwaitExpression
    | NewExpression
    | TemplateLiteral
    | LambdaExpression;

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
 * `ego` self-reference expression (like `this`).
 */
export interface ThisExpression extends BaseNode {
    type: 'ThisExpression';
}

/**
 * Literal value (string, number, bigint, boolean, null).
 *
 * GRAMMAR (in EBNF):
 *   literal := STRING | NUMBER | BIGINT | 'verum' | 'falsum' | 'nihil'
 *
 * INVARIANT: value type matches the literal kind.
 * INVARIANT: raw preserves original source text for error messages.
 *
 * Examples:
 *   "hello" -> value="hello", raw='"hello"'
 *   42      -> value=42, raw='42'
 *   123n     -> value=123n, raw='123n'
 *   verum   -> value=true, raw='verum'
 *   nihil   -> value=null, raw='nihil'
 */
export interface Literal extends BaseNode {
    type: 'Literal';
    value: string | number | bigint | boolean | null;
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
 *   arrayExpr := '[' (arrayElement (',' arrayElement)*)? ']'
 *   arrayElement := 'sparge' expression | expression
 *
 * INVARIANT: elements is always an array (empty array = empty elements).
 * INVARIANT: elements can contain SpreadElement for spread syntax.
 *
 * Examples:
 *   []                    -> elements=[]
 *   [1, 2, 3]             -> elements=[Literal, Literal, Literal]
 *   [sparge a, sparge b]  -> elements=[SpreadElement, SpreadElement]
 */
export interface ArrayExpression extends BaseNode {
    type: 'ArrayExpression';
    elements: (Expression | SpreadElement)[];
}

/**
 * Spread element (sparge) for arrays, objects, and function calls.
 *
 * GRAMMAR (in EBNF):
 *   spreadElement := 'sparge' expression
 *
 * WHY: Latin 'sparge' (scatter/spread) for spreading elements.
 *      Used in arrays: [sparge a, sparge b]
 *      Used in objects: { sparge o }
 *      Used in calls: fn(sparge args)
 *
 * Examples:
 *   sparge a   -> spread array a into containing array
 *   sparge obj -> spread object properties into containing object
 */
export interface SpreadElement extends BaseNode {
    type: 'SpreadElement';
    argument: Expression;
}

/**
 * Object literal expression.
 *
 * GRAMMAR (in EBNF):
 *   objectExpr := '{' (objectMember (',' objectMember)*)? '}'
 *   objectMember := 'sparge' expression | (IDENTIFIER | STRING) ':' expression
 *
 * WHY: Object literals are the primary way to create structured data.
 * INVARIANT: properties can contain SpreadElement for object spread.
 *
 * Examples:
 *   {}                           -> empty object
 *   { nomen: "Marcus" }          -> single property
 *   { sparge defaults, x: 1 }    -> spread + property
 */
export interface ObjectExpression extends BaseNode {
    type: 'ObjectExpression';
    properties: (ObjectProperty | SpreadElement)[];
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
 *   rangeExpr := expression ('..' | 'ante' | 'usque') expression ('per' expression)?
 *
 * WHY: Provides concise syntax for numeric iteration ranges.
 *      Three operators with different end semantics:
 *      - '..' and 'ante': exclusive (0..10 / 0 ante 10 = 0-9)
 *      - 'usque': inclusive (0 usque 10 = 0-10)
 *
 * Examples:
 *   0..10           -> exclusive, produces 0-9
 *   0 ante 10       -> exclusive, produces 0-9 (explicit)
 *   0 usque 10      -> inclusive, produces 0-10
 *   0..10 per 2     -> exclusive with step
 */
export interface RangeExpression extends BaseNode {
    type: 'RangeExpression';
    start: Expression;
    end: Expression;
    step?: Expression;
    inclusive?: boolean;
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
// Type Cast Expression
// ---------------------------------------------------------------------------

/**
 * Type cast expression (ut operator).
 *
 * GRAMMAR (in EBNF):
 *   castExpr := call ('ut' typeAnnotation)*
 *
 * INVARIANT: expression is the value being cast.
 * INVARIANT: targetType is the type to cast to.
 *
 * WHY: Latin 'ut' (as, in the capacity of) for type assertions.
 *      Compile-time only — no runtime overhead or checking.
 *      Use 'est' first when possible for safe narrowing.
 *
 * Target mappings:
 *   TypeScript: x as T
 *   Python:     x (cast ignored, dynamic typing)
 *   Zig:        @as(T, x)
 *   Rust:       x as T
 *   C++:        static_cast<T>(x)
 *
 * Examples:
 *   data ut textus              -> data as string
 *   response.body ut objectum   -> (response.body) as object
 *   x ut A ut B                 -> (x as A) as B (left-associative)
 */
export interface TypeCastExpression extends BaseNode {
    type: 'TypeCastExpression';
    expression: Expression;
    targetType: TypeAnnotation;
}

// ---------------------------------------------------------------------------
// Call and Member Access
// ---------------------------------------------------------------------------

/**
 * Function call expression.
 *
 * GRAMMAR (in EBNF):
 *   callExpr := expression '(' argumentList ')'
 *            | expression '?(' argumentList ')'   // optional call
 *            | expression '!(' argumentList ')'   // non-null assert call
 *   argumentList := (argument (',' argument)*)?
 *   argument := 'sparge' expression | expression
 *
 * INVARIANT: callee can be any expression (Identifier, MemberExpression, etc.).
 * INVARIANT: arguments is always an array (empty for zero-arg calls).
 * INVARIANT: arguments can contain SpreadElement for spread in calls.
 *
 * Examples:
 *   f()              -> args=[]
 *   f(a, b)          -> args=[a, b]
 *   f(sparge nums)   -> args=[SpreadElement]
 *   callback?()      -> optional=true
 *   handler!()       -> nonNull=true
 */
export interface CallExpression extends BaseNode {
    type: 'CallExpression';
    callee: Expression;
    arguments: (Expression | SpreadElement)[];
    optional?: boolean;
    nonNull?: boolean;
}

/**
 * Member access expression.
 *
 * GRAMMAR (in EBNF):
 *   memberExpr := expression '.' IDENTIFIER
 *              | expression '[' expression ']'
 *              | expression '?.' IDENTIFIER      // optional property
 *              | expression '?[' expression ']'  // optional computed
 *              | expression '!.' IDENTIFIER      // non-null property
 *              | expression '![' expression ']'  // non-null computed
 *
 * INVARIANT: computed=false means dot notation (obj.prop).
 * INVARIANT: computed=true means bracket notation (obj[prop]).
 *
 * WHY: computed flag enables different code generation strategies.
 *
 * Examples:
 *   user.name        -> computed=false
 *   items[0]         -> computed=true
 *   user?.name       -> computed=false, optional=true
 *   items?[0]        -> computed=true, optional=true
 *   user!.name       -> computed=false, nonNull=true
 *   items![0]        -> computed=true, nonNull=true
 */
export interface MemberExpression extends BaseNode {
    type: 'MemberExpression';
    object: Expression;
    property: Expression;
    computed: boolean;
    optional?: boolean;
    nonNull?: boolean;
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
 *   newExpr := 'novum' IDENTIFIER ('(' argumentList ')')? (objectLiteral | 'de' expression)?
 *
 * INVARIANT: callee is Identifier (constructor name).
 * INVARIANT: arguments is always an array.
 *
 * WHY: Latin 'novum' (new) for object construction.
 *      Two forms for overrides:
 *      - Inline: `novum Persona { nomen: "Marcus" }`
 *      - From expression: `novum Persona de props`
 */
export interface NewExpression extends BaseNode {
    type: 'NewExpression';
    callee: Identifier;
    arguments: Expression[];
    withExpression?: Expression;
}

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

/**
 * Type parameter for parameterized types.
 *
 * DESIGN: Union type allows different parameter kinds:
 *         - TypeAnnotation: Generic type params (lista<textus>), or size specs (i32, u64)
 *         - Literal: Numeric params (numerus<32>)
 *
 * Examples:
 *   lista<textus> -> TypeAnnotation
 *   numerus<32> -> Literal (size in bits)
 *   numerus<i32> -> TypeAnnotation (explicit signed 32-bit)
 *   numerus<u64> -> TypeAnnotation (explicit unsigned 64-bit)
 */
export type TypeParameter = TypeAnnotation | Literal;

/**
 * Type annotation for variables, parameters, and return types.
 *
 * GRAMMAR (in EBNF):
 *   typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? ('|' typeAnnotation)*
 *   typeParams := '<' typeParameter (',' typeParameter)* '>'
 *
 * INVARIANT: name is the base type name (textus, numerus, etc.).
 * INVARIANT: nullable indicates optional type with '?'.
 * INVARIANT: union contains multiple types for union types (A | B).
 * INVARIANT: typeParameters can contain types or literals.
 * INVARIANT: preposition encodes ownership for systems targets (Rust/Zig):
 *            de = borrowed/read-only (&T, []const u8)
 *            in = mutable borrow (&mut T, *T)
 *
 * Examples:
 *   textus -> name="textus"
 *   numerus? -> name="numerus", nullable=true
 *   lista<textus> -> name="lista", typeParameters=[TypeAnnotation]
 *   numerus<32> -> name="numerus", typeParameters=[Literal{value=32}]
 *   numerus<i32> -> name="numerus", typeParameters=[TypeAnnotation{name="i32"}]
 *   textus | numerus -> name="union", union=[{name="textus"}, {name="numerus"}]
 *   de textus -> name="textus", preposition="de" (borrowed)
 *   in textus -> name="textus", preposition="in" (mutable borrow)
 */
export interface TypeAnnotation extends BaseNode {
    type: 'TypeAnnotation';
    name: string;
    typeParameters?: TypeParameter[];
    nullable?: boolean;
    union?: TypeAnnotation[];
    arrayShorthand?: boolean; // true if parsed from [] syntax (e.g., numerus[] vs lista<numerus>)
    preposition?: string; // 'de' (borrowed) or 'in' (mutable) for systems targets
}
