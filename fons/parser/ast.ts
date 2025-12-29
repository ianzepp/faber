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
    | ImportaDeclaration
    | DestructureDeclaration
    | VariaDeclaration
    | FunctioDeclaration
    | GenusDeclaration
    | PactumDeclaration
    | TypeAliasDeclaration
    | OrdoDeclaration
    | DiscretioDeclaration
    | ExpressionStatement
    | SiStatement
    | DumStatement
    | IteratioStatement
    | InStatement
    | EligeStatement
    | DiscerneStatement
    | CustodiStatement
    | AdfirmaStatement
    | ReddeStatement
    | RumpeStatement
    | PergeStatement
    | BlockStatement
    | IaceStatement
    | TemptaStatement
    | ScribeStatement
    | FacBlockStatement
    | ProbandumStatement
    | ProbaStatement
    | CuraBlock
    | CuraStatement
    | AdStatement;

// ---------------------------------------------------------------------------
// Import/Export Declarations
// ---------------------------------------------------------------------------

/**
 * Import specifier with optional alias.
 *
 * GRAMMAR (in EBNF):
 *   importSpecifier := IDENTIFIER ('ut' IDENTIFIER)?
 *
 * WHY: Separates the imported name from the local binding name.
 *      Used by both ImportaDeclaration and DestructureDeclaration.
 *
 * Examples:
 *   scribe                -> imported=scribe, local=scribe
 *   scribe ut s           -> imported=scribe, local=s
 *   nomen ut n            -> imported=nomen, local=n
 */
export interface ImportSpecifier extends BaseNode {
    type: 'ImportSpecifier';
    imported: Identifier;
    local: Identifier;
    rest?: boolean; // WHY: For ceteri (rest) patterns in destructuring
}

/**
 * Import declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (specifierList | '*')
 *   specifierList := importSpecifier (',' importSpecifier)*
 *   importSpecifier := IDENTIFIER ('ut' IDENTIFIER)?
 *
 * INVARIANT: Either specifiers is non-empty OR wildcard is true.
 * INVARIANT: source is never empty string.
 *
 * Examples:
 *   ex norma importa scribe, lege         -> source="norma", specifiers=[{scribe,scribe}, {lege,lege}]
 *   ex norma importa scribe ut s          -> source="norma", specifiers=[{scribe,s}]
 *   ex "norma/tempus" importa nunc        -> source="norma/tempus", specifiers=[{nunc,nunc}]
 *   ex norma importa *                    -> source="norma", wildcard=true
 */
export interface ImportaDeclaration extends BaseNode {
    type: 'ImportaDeclaration';
    source: string;
    specifiers: ImportSpecifier[];
    wildcard: boolean;
}

/**
 * Destructuring declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   destructDecl := 'ex' expression bindingKeyword specifierList
 *   bindingKeyword := 'fixum' | 'varia' | 'figendum' | 'variandum'
 *   specifierList := importSpecifier (',' importSpecifier)*
 *   importSpecifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
 *
 * WHY: Extracts properties from objects into individual bindings.
 *      Uses same specifier format as imports for consistency.
 *      Async variants (figendum/variandum) imply await on source.
 *
 * Examples:
 *   ex persona fixum nomen, aetas           -> extract nomen, aetas
 *   ex persona fixum nomen ut n, aetas ut a -> extract with aliases
 *   ex persona fixum nomen, ceteri rest     -> extract nomen, collect rest
 *   ex promise figendum result              -> await + extract
 */
export interface DestructureDeclaration extends BaseNode {
    type: 'DestructureDeclaration';
    source: Expression;
    kind: 'fixum' | 'varia' | 'figendum' | 'variandum';
    specifiers: ImportSpecifier[];
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
 * Array destructuring pattern.
 *
 * GRAMMAR (in EBNF):
 *   arrayPattern := '[' arrayPatternElement (',' arrayPatternElement)* ']'
 *   arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
 *
 * WHY: Allows unpacking array elements into variables by position.
 *
 * Examples:
 *   [a, b, c]               -> extract first three elements
 *   [first, ceteri rest]    -> extract first, collect rest
 *   [_, second, _]          -> skip first and third, extract second
 */
export interface ArrayPattern extends BaseNode {
    type: 'ArrayPattern';
    elements: ArrayPatternElement[];
}

/**
 * Single element in an array pattern.
 *
 * name: the variable name to bind (or '_' pseudo-identifier for skip)
 * rest: if true, collects all remaining elements (ceteri pattern)
 * skip: if true, this position is skipped (underscore)
 *
 * Examples:
 *   [a]                      -> name=a, rest=false, skip=false
 *   [_, b]                   -> first: skip=true, second: name=b
 *   [first, ceteri tail]     -> second: name=tail, rest=true
 */
export interface ArrayPatternElement extends BaseNode {
    type: 'ArrayPatternElement';
    name: Identifier;
    rest?: boolean;
    skip?: boolean;
}

/**
 * Variable declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   varDecl := ('varia' | 'fixum' | 'figendum' | 'variandum') typeAnnotation? IDENTIFIER ('=' expression)?
 *   arrayDestruct := ('varia' | 'fixum' | 'figendum' | 'variandum') arrayPattern '=' expression
 *
 * INVARIANT: kind is Latin keyword, not target language (let/const).
 * INVARIANT: Either typeAnnotation or init SHOULD be present (but not enforced by parser).
 *
 * WHY: Preserves Latin keywords for semantic phase to map to target semantics.
 *
 * NOTE: Object destructuring uses DestructureDeclaration with ex-prefix syntax:
 *       ex persona fixum nomen, aetas (NOT fixum { nomen, aetas } = persona)
 *
 * Async bindings (figendum/variandum) imply await without explicit cede:
 *   figendum = "that which will be fixed" (gerundive) -> const x = await ...
 *   variandum = "that which will be varied" (gerundive) -> let x = await ...
 *
 * Target mappings:
 *   varia     → let (TS), var (Zig), assignment (Py)
 *   fixum     → const (TS), const (Zig), assignment (Py)
 *   figendum  → const x = await (TS), assignment (Py), N/A (Zig)
 *   variandum → let x = await (TS), assignment (Py), N/A (Zig)
 *
 * Examples:
 *   varia numerus x = 5
 *   fixum SALVE = "ave"
 *   fixum [a, b, c] = coords
 *   figendum data = fetchData()
 *   variandum result = fetchInitial()
 */
export interface VariaDeclaration extends BaseNode {
    type: 'VariaDeclaration';
    kind: 'varia' | 'fixum' | 'figendum' | 'variandum';
    name: Identifier | ArrayPattern;
    typeAnnotation?: TypeAnnotation;
    init?: Expression;
}

/**
 * Compile-time type parameter for generic functions.
 *
 * GRAMMAR (in EBNF):
 *   typeParamDecl := 'prae' 'typus' IDENTIFIER
 *
 * INVARIANT: name is the type parameter identifier (e.g., T, U).
 *
 * WHY: Latin 'prae' (before) indicates compile-time evaluation.
 *      Combined with 'typus' (type), creates generic type parameters.
 *
 * Target mappings:
 *   prae typus T → <T> (TS), TypeVar (Py), comptime T: type (Zig), <T> (Rust)
 *
 * Examples:
 *   functio max(prae typus T, T a, T b) -> T
 *   functio create(prae typus T) -> T
 */
export interface TypeParameterDeclaration extends BaseNode {
    type: 'TypeParameterDeclaration';
    name: Identifier;
}

/**
 * Function declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   funcDecl := 'futura'? 'functio' IDENTIFIER '(' paramList ')' ('->' typeAnnotation)? blockStmt
 *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
 *
 * INVARIANT: async flag set by presence of 'futura' keyword.
 * INVARIANT: params is always an array (empty if no parameters).
 * INVARIANT: typeParams contains compile-time type parameters (prae typus T).
 *
 * Target mappings:
 *   functio        → function (TS), def (Py), fn (Zig), fn (Rust)
 *   futura functio → async function (TS), async def (Py), fn returning !T (Zig)
 *   cursor functio → function* (TS), generator def (Py), N/A (Zig)
 *
 * Examples:
 *   functio salve(nomen: textus) -> textus { ... }
 *   futura functio cede() { ... }
 *   functio max(prae typus T, T a, T b) -> T { ... }
 */
export interface FunctioDeclaration extends BaseNode {
    type: 'FunctioDeclaration';
    name: Identifier;
    typeParams?: TypeParameterDeclaration[];
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
 *   parameter := ('de' | 'in' | 'ex')? 'ceteri'? (typeAnnotation IDENTIFIER | IDENTIFIER)
 *
 * INVARIANT: preposition is Latin (de/in/ex), not English (from/in/of).
 * INVARIANT: case and preposition enable semantic analysis of parameter roles.
 * INVARIANT: rest is true when 'ceteri' modifier present (variadic/rest parameter).
 *
 * WHY: Latin prepositions indicate semantic roles that map to different constructs
 *      in target languages. For systems targets (Rust/Zig), 'de' = borrowed/read-only
 *      and 'in' = mutable borrow. Note: 'ad' is reserved for statement-level dispatch.
 *
 * Target mappings (prepositions):
 *   de (from)  → param (TS/Py), &T (Rust), []const (Zig) — borrowed/read-only
 *   in (into)  → param (TS/Py), &mut T (Rust), *T (Zig) — mutable borrow
 *   ex (from)  → param (TS/Py), source semantics
 *   ceteri     → ...rest (TS), *args (Py), slice (Zig)
 *
 * Examples:
 *   textus nomen              -> regular param
 *   de textus source          -> borrowed/read-only param
 *   in lista<T> items         -> mutable borrow param
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
 *   typeAliasDecl := 'typus' IDENTIFIER '=' (typeAnnotation | typeofAnnotation)
 *   typeofAnnotation := 'typus' IDENTIFIER
 *
 * INVARIANT: name is the alias identifier.
 * INVARIANT: typeAnnotation is the type being aliased (standard form).
 * INVARIANT: typeofTarget is set when RHS is `typus identifier` (typeof).
 * INVARIANT: Exactly one of typeAnnotation or typeofTarget is set.
 *
 * WHY: Enables creating named type aliases for complex types.
 *      When RHS is `typus identifier`, extracts the type of a value.
 *
 * Examples:
 *   typus ID = textus
 *   typus UserID = numerus<32, Naturalis>
 *   typus ConfigTypus = typus config    // type ConfigTypus = typeof config
 */
export interface TypeAliasDeclaration extends BaseNode {
    type: 'TypeAliasDeclaration';
    name: Identifier;
    typeAnnotation: TypeAnnotation;
    typeofTarget?: Identifier;
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
export interface OrdoMember extends BaseNode {
    type: 'OrdoMember';
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
 * INVARIANT: members is non-empty array of OrdoMember.
 *
 * WHY: "ordo" (order/rank) represents enumerated constants.
 *
 * Examples:
 *   ordo color { rubrum, viridis, caeruleum }
 *   ordo status { pendens = 0, actum = 1, finitum = 2 }
 */
export interface OrdoDeclaration extends BaseNode {
    type: 'OrdoDeclaration';
    name: Identifier;
    members: OrdoMember[];
}

// ---------------------------------------------------------------------------
// Discretio (Tagged Union) Declarations
// ---------------------------------------------------------------------------

/**
 * Variant field declaration within a discretio variant.
 *
 * Uses type-first syntax like genus fields.
 *
 * Examples:
 *   numerus x           -> fieldType=numerus, name=x
 *   textus key          -> fieldType=textus, name=key
 */
export interface VariantField extends BaseNode {
    type: 'VariantField';
    name: Identifier;
    fieldType: TypeAnnotation;
}

/**
 * Variant declaration within a discretio.
 *
 * GRAMMAR (in EBNF):
 *   variant := IDENTIFIER ('{' variantFields '}')?
 *   variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
 *
 * INVARIANT: name is the variant tag (e.g., Click, Keypress, Quit).
 * INVARIANT: fields is empty for unit variants (no payload).
 *
 * Examples:
 *   Click { numerus x, numerus y }  -> name=Click, fields=[x, y]
 *   Keypress { textus key }         -> name=Keypress, fields=[key]
 *   Quit                            -> name=Quit, fields=[]
 */
export interface VariantDeclaration extends BaseNode {
    type: 'VariantDeclaration';
    name: Identifier;
    fields: VariantField[];
}

/**
 * Discretio (tagged union) declaration.
 *
 * GRAMMAR (in EBNF):
 *   discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
 *
 * INVARIANT: name is the union type name (e.g., Event, Option, Result).
 * INVARIANT: variants is non-empty array of VariantDeclaration.
 *
 * WHY: "discretio" (distinction) for tagged unions. Each variant has a
 *      compiler-managed tag for exhaustive pattern matching.
 *
 * Target mappings:
 *   TypeScript: Discriminated union with 'tag' property
 *   Zig:        union(enum)
 *   Rust:       enum with struct variants
 *
 * Examples:
 *   discretio Event {
 *       Click { numerus x, numerus y }
 *       Keypress { textus key }
 *       Quit
 *   }
 *
 *   discretio Option<T> {
 *       Some { T value }
 *       None
 *   }
 */
export interface DiscretioDeclaration extends BaseNode {
    type: 'DiscretioDeclaration';
    name: Identifier;
    typeParameters?: Identifier[];
    variants: VariantDeclaration[];
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
 * INVARIANT: methods contains all method declarations (FunctioDeclaration with implicit ego).
 * INVARIANT: implements lists pactum names this genus fulfills.
 *
 * WHY: Latin 'genus' (kind/type) for data structures with fields and methods.
 *      No inheritance - composition and pactum (interfaces) only.
 *
 * Target mappings:
 *   genus  → class (TS), class (Py), struct (Zig), struct (Rust)
 *   implet → implements (TS), Protocol (Py), comptime duck typing (Zig), impl Trait (Rust)
 *   ego    → this (TS), self (Py), self (Zig), self (Rust)
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
    constructor?: FunctioDeclaration;
    methods: FunctioDeclaration[];
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
 * INVARIANT: alternate can be BlockStatement (else) or SiStatement (else if chain).
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
export interface SiStatement extends BaseNode {
    type: 'SiStatement';
    test: Expression;
    consequent: BlockStatement;
    alternate?: BlockStatement | SiStatement;
    catchClause?: CapeClause;
}

/**
 * While loop statement.
 *
 * GRAMMAR (in EBNF):
 *   whileStmt := 'dum' expression blockStmt ('cape' IDENTIFIER blockStmt)?
 *
 * INVARIANT: catchClause allows error handling within loop iterations.
 */
export interface DumStatement extends BaseNode {
    type: 'DumStatement';
    test: Expression;
    body: BlockStatement;
    catchClause?: CapeClause;
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
 *
 * Target mappings:
 *   ex...pro  → for...of (TS), for...in (Py), for (slice) |x| (Zig)
 *   de...pro  → for...in (TS), for...in keys (Py), iteration over keys (Zig)
 *   ex...fiet → for await...of (TS), async for (Py), N/A (Zig)
 */
/**
 * Collection DSL transform operation.
 *
 * GRAMMAR (in EBNF):
 *   dslTransform := dslVerb dslArg?
 *   dslVerb := 'prima' | 'ultima' | 'summa'
 *   dslArg := expression
 *
 * WHY: DSL transforms provide concise syntax for collection operations.
 *      They desugar to method calls during semantic analysis.
 *
 * Examples:
 *   prima 5        -> .prima(5)
 *   ultima 3       -> .ultima(3)
 *   summa          -> .summa()
 */
export interface CollectionDSLTransform extends BaseNode {
    type: 'CollectionDSLTransform';
    verb: string;
    argument?: Expression;
}

export interface IteratioStatement extends BaseNode {
    type: 'IteratioStatement';
    kind: 'in' | 'ex';
    variable: Identifier;
    iterable: Expression;
    body: BlockStatement;
    async: boolean;
    catchClause?: CapeClause;
    transforms?: CollectionDSLTransform[];
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
export interface InStatement extends BaseNode {
    type: 'InStatement';
    object: Expression;
    body: BlockStatement;
}

/**
 * Switch statement (value matching).
 *
 * GRAMMAR (in EBNF):
 *   eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
 *   eligeCase := 'si' expression (blockStmt | 'ergo' expression)
 *   defaultCase := ('aliter' | 'secus') (blockStmt | statement)
 *
 * WHY: Latin 'elige' (choose) for value-based switch.
 *      Use 'discerne' for variant pattern matching on discretio types.
 *
 * Example:
 *   elige status {
 *       si "pending" { processPending() }
 *       si "active" { processActive() }
 *       aliter { processDefault() }
 *   }
 */
export interface EligeStatement extends BaseNode {
    type: 'EligeStatement';
    discriminant: Expression;
    cases: EligeCasus[];
    defaultCase?: BlockStatement;
    catchClause?: CapeClause;
}

/**
 * Switch case for value matching (part of switch statement).
 */
export interface EligeCasus extends BaseNode {
    type: 'EligeCasus';
    test: Expression;
    consequent: BlockStatement;
}

/**
 * Variant matching statement (for discretio types).
 *
 * GRAMMAR (in EBNF):
 *   discerneStmt := 'discerne' expression '{' variantCase* '}'
 *   variantCase := 'si' IDENTIFIER ('pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
 *
 * WHY: 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
 *      Uses 'si' for conditional match, 'pro' for bindings.
 *
 * Example:
 *   discerne event {
 *       si Click pro x, y { scribe "clicked at " + x + ", " + y }
 *       si Keypress pro key { scribe "pressed " + key }
 *       si Quit { mori "goodbye" }
 *   }
 */
export interface DiscerneStatement extends BaseNode {
    type: 'DiscerneStatement';
    discriminant: Expression;
    cases: VariantCase[];
}

/**
 * Variant case for pattern matching (part of discerne statement).
 *
 * GRAMMAR (in EBNF):
 *   variantCase := 'si' IDENTIFIER ('pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
 *
 * INVARIANT: variant is the variant name to match (e.g., Click, Keypress).
 * INVARIANT: bindings are field names to bind (empty for unit variants).
 * INVARIANT: consequent is the block to execute on match.
 *
 * WHY: 'si' (if) for conditional variant match, 'pro' (for) to introduce bindings.
 *
 * Examples:
 *   si Click pro x, y { ... }  -> variant=Click, bindings=[x, y]
 *   si Keypress pro key { ... } -> variant=Keypress, bindings=[key]
 *   si Quit { ... }             -> variant=Quit, bindings=[]
 */
export interface VariantCase extends BaseNode {
    type: 'VariantCase';
    variant: Identifier;
    bindings: Identifier[];
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
export interface CustodiStatement extends BaseNode {
    type: 'CustodiStatement';
    clauses: CustodiClause[];
}

/**
 * Guard clause (part of guard statement).
 */
export interface CustodiClause extends BaseNode {
    type: 'CustodiClause';
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
export interface AdfirmaStatement extends BaseNode {
    type: 'AdfirmaStatement';
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
export interface ReddeStatement extends BaseNode {
    type: 'ReddeStatement';
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
export interface RumpeStatement extends BaseNode {
    type: 'RumpeStatement';
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
export interface PergeStatement extends BaseNode {
    type: 'PergeStatement';
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
export interface IaceStatement extends BaseNode {
    type: 'IaceStatement';
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
 *
 * Target mappings:
 *   scribe → console.log (TS), print() (Py), std.debug.print (Zig)
 *   vide   → console.debug (TS), print("[DEBUG]") (Py), std.debug.print (Zig)
 *   mone   → console.warn (TS), print("[WARN]") (Py), std.debug.print (Zig)
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
 * Scriptum (format string) expression.
 *
 * GRAMMAR (in EBNF):
 *   scriptumExpr := 'scriptum' '(' STRING (',' expression)* ')'
 *
 * WHY: "scriptum" (that which has been written) is the perfect passive participle
 *      of scribere. While scribe outputs to console, scriptum returns a formatted string.
 *      This is the expression counterpart to the scribe statement.
 *
 * WHY: Format string is passed through verbatim to target. User must use target-appropriate
 *      placeholders ({} for Zig/Rust, %s/%d for C++, etc.). Faber does not translate.
 *
 * Target mappings:
 *   scriptum("Hello, {}", name) →
 *     Zig:  std.fmt.allocPrint(alloc, "Hello, {}", .{name})
 *     Rust: format!("Hello, {}", name)
 *     C++:  std::format("Hello, {}", name)
 *     Py:   "Hello, {}".format(name)
 *     TS:   interpolation or runtime helper
 *
 * Examples:
 *   scriptum("Hello, {}", name)
 *   scriptum("{} + {} = {}", a, b, a + b)
 */
export interface ScriptumExpression extends BaseNode {
    type: 'ScriptumExpression';
    format: Literal; // The format string (must be a string literal)
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
 *
 * Target mappings:
 *   tempta → try (TS/Py), N/A (Zig uses error unions)
 *   cape   → catch (TS), except (Py), catch |err| (Zig)
 *   demum  → finally (TS), finally (Py), defer (Zig)
 */
export interface TemptaStatement extends BaseNode {
    type: 'TemptaStatement';
    block: BlockStatement;
    handler?: CapeClause;
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
 * WHY: Reusable in both TemptaStatement and control flow (SiStatement, loops).
 */
export interface CapeClause extends BaseNode {
    type: 'CapeClause';
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
    catchClause?: CapeClause;
}

// ---------------------------------------------------------------------------
// Test Declarations (Proba)
// ---------------------------------------------------------------------------

/**
 * Test suite declaration (probandum).
 *
 * GRAMMAR (in EBNF):
 *   probandumDecl := 'probandum' STRING '{' probandumBody '}'
 *   probandumBody := (anteBlock | postBlock | probandumDecl | probaStmt)*
 *
 * INVARIANT: name is the suite description string.
 * INVARIANT: body contains setup/teardown blocks, nested suites, and tests.
 *
 * WHY: Latin "probandum" (gerundive of probare) = "that which must be tested".
 *      Analogous to describe() in Jest/Vitest or test module in Zig.
 *
 * Target mappings:
 *   TypeScript: describe("name", () => { ... })
 *   Python:     class TestName: ...
 *   Zig:        test "name" { ... } (flattened with prefix)
 *   Rust:       mod tests { ... } (flattened with prefix)
 *   C++:        void test_name() { ... } (flattened with prefix)
 *
 * Examples:
 *   probandum "Tokenizer" {
 *       ante { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 */
export interface ProbandumStatement extends BaseNode {
    type: 'ProbandumStatement';
    name: string;
    body: (CuraBlock | ProbandumStatement | ProbaStatement)[];
}

/**
 * Test modifier for skipped or todo tests.
 *
 * WHY: Two modifiers:
 *   omitte = skip (imperative: "skip!")
 *   futurum = todo/pending (noun: "the future")
 */
export type ProbaModifier = 'omitte' | 'futurum';

/**
 * Individual test case (proba).
 *
 * GRAMMAR (in EBNF):
 *   probaStmt := 'proba' probaModifier? STRING blockStmt
 *   probaModifier := 'omitte' STRING | 'futurum' STRING
 *
 * INVARIANT: name is the test description string.
 * INVARIANT: modifier is optional (omitte/futurum) with reason string.
 * INVARIANT: body is the test block.
 *
 * WHY: Latin "proba" (imperative of probare) = "test!" / "prove!".
 *      Analogous to test() or it() in Jest/Vitest.
 *
 * Target mappings:
 *   TypeScript: test("name", () => { ... })
 *   Python:     def test_name(): ...
 *   Zig:        test "name" { ... }
 *   Rust:       #[test] fn name() { ... }
 *   C++:        void test_name() { ... }
 *
 * Examples:
 *   proba "parses integers" { adfirma parse("42") est 42 }
 *   proba omitte "blocked by #42" { ... }
 *   proba futurum "needs async support" { ... }
 */
export interface ProbaStatement extends BaseNode {
    type: 'ProbaStatement';
    name: string;
    modifier?: ProbaModifier;
    modifierReason?: string;
    body: BlockStatement;
}

/**
 * Timing for cura blocks in test context.
 */
export type CuraTiming = 'ante' | 'post';

/**
 * Curator kinds for cura statements.
 *
 * WHY: Explicit curator kind declares resource management type.
 *      - arena: Arena allocator (memory freed on scope exit)
 *      - page: Page allocator (memory freed on scope exit)
 *      - (future: curator, liber, transactio, mutex, conexio)
 */
export type CuratorKind = 'arena' | 'page';

/**
 * Resource management / test setup-teardown block.
 *
 * GRAMMAR (in EBNF):
 *   curaBlock := 'cura' ('ante' | 'post') 'omnia'? blockStmt
 *
 * INVARIANT: timing distinguishes setup (ante) vs teardown (post).
 * INVARIANT: omnia flag distinguishes all vs each.
 *
 * WHY: Latin "cura" (care, concern) for resource management.
 *      In test context:
 *        cura ante { } = beforeEach (care before each test)
 *        cura ante omnia { } = beforeAll (care before all tests)
 *        cura post { } = afterEach (care after each test)
 *        cura post omnia { } = afterAll (care after all tests)
 *
 * Target mappings:
 *   TypeScript: beforeEach() / beforeAll() / afterEach() / afterAll()
 *   Python:     @pytest.fixture / setup_module / teardown
 *   Zig:        inlined into each test
 *   Rust:       inlined into each test
 *   C++:        inlined into each test
 *
 * Examples:
 *   cura ante { lexer = init() }
 *   cura ante omnia { db = connect() }
 *   cura post { cleanup() }
 *   cura post omnia { db.close() }
 */
export interface CuraBlock extends BaseNode {
    type: 'CuraBlock';
    timing: CuraTiming;
    omnia: boolean;
    body: BlockStatement;
}

/**
 * Resource management statement.
 *
 * GRAMMAR (in EBNF):
 *   curaStmt := 'cura' curatorKind? expression? ('pro' | 'fit' | 'fiet') typeAnnotation? IDENTIFIER blockStmt catchClause?
 *   curatorKind := 'arena' | 'page'
 *
 * INVARIANT: curatorKind is optional; when present, declares allocator type.
 * INVARIANT: resource is optional for allocator kinds (arena/page create their own).
 * INVARIANT: binding is the identifier that receives the resource/allocator.
 * INVARIANT: typeAnnotation is optional explicit type for the binding.
 * INVARIANT: async flag indicates fiet (async) vs pro/fit (sync).
 * INVARIANT: body is the scoped block where resource is used.
 * INVARIANT: catchClause is optional error handling.
 *
 * WHY: Latin "cura" (care, concern) + binding verb for scoped resources.
 *      - pro: neutral binding ("for")
 *      - fit: sync binding ("it becomes")
 *      - fiet: async binding ("it will become")
 *      Guarantees cleanup via solve() on scope exit.
 *
 * Target mappings:
 *   TypeScript: try { } finally { binding.solve?.(); } (allocators stripped)
 *   Python:     with expr as binding: ... (allocators stripped)
 *   Zig:        ArenaAllocator / PageAllocator with defer deinit()
 *   Rust:       RAII / Drop at scope end
 *
 * Examples:
 *   cura arena fit mem { ... }                    // arena allocator
 *   cura page fit mem { ... }                     // page allocator
 *   cura aperi("data.bin") fit fd { lege(fd) }   // generic resource
 *   cura connect(url) fiet conn { ... }          // async resource
 *   cura aperi("config.json") fit File fd { }    // with type annotation
 */
export interface CuraStatement extends BaseNode {
    type: 'CuraStatement';
    curatorKind?: CuratorKind;
    resource?: Expression;
    binding: Identifier;
    typeAnnotation?: TypeAnnotation;
    async: boolean;
    body: BlockStatement;
    catchClause?: CapeClause;
}

// ---------------------------------------------------------------------------
// Ad (Dispatch) Statement
// ---------------------------------------------------------------------------

/**
 * Binding verb for ad statement result binding.
 *
 * WHY: Mirrors function return type verbs for consistency.
 *      Encodes both sync/async and single/plural semantics.
 *
 * | Verb   | Async | Plural | Meaning              |
 * |--------|-------|--------|----------------------|
 * | fit    | no    | no     | becomes (sync)       |
 * | fiet   | yes   | no     | will become (async)  |
 * | fiunt  | no    | yes    | become (sync plural) |
 * | fient  | yes   | yes    | will become (async)  |
 */
export type AdBindingVerb = 'fit' | 'fiet' | 'fiunt' | 'fient';

/**
 * Binding clause for ad statement result.
 *
 * GRAMMAR (in EBNF):
 *   adBinding := adBindingVerb typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
 *   adBindingVerb := 'fit' | 'fiet' | 'fiunt' | 'fient'
 *
 * INVARIANT: verb encodes sync/async and single/plural.
 * INVARIANT: typeAnnotation is optional (can be inferred from syscall table).
 * INVARIANT: name is the binding variable.
 * INVARIANT: alias is optional 'ut' rename (like import aliases).
 *
 * WHY: 'pro' introduces the binding (consistent with iteration/lambda bindings).
 *      Optional 'ut' provides an alias when the binding name should differ.
 *
 * Examples:
 *   fit textus pro content          -> sync, explicit type
 *   fiet Response pro response      -> async, explicit type
 *   fiunt textus pro lines          -> sync plural (stream)
 *   pro content                     -> type inferred, fit implied
 *   fit textus pro content ut c     -> with alias
 */
export interface AdBinding extends BaseNode {
    type: 'AdBinding';
    verb: AdBindingVerb;
    typeAnnotation?: TypeAnnotation;
    name: Identifier;
    alias?: Identifier;
}

/**
 * Ad (dispatch) statement.
 *
 * GRAMMAR (in EBNF):
 *   adStmt := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
 *   argumentList := (expression (',' expression)*)?
 *
 * INVARIANT: target is a string literal (dispatch endpoint).
 * INVARIANT: arguments is always an array (empty for zero-arg calls).
 * INVARIANT: binding is optional (fire-and-forget if omitted).
 * INVARIANT: body is optional (can bind without block for simple cases).
 * INVARIANT: catchClause is optional error handling.
 *
 * WHY: Latin 'ad' (to/toward) dispatches to named endpoints:
 *      - Stdlib syscalls: "fasciculus:lege", "console:log"
 *      - External packages: "hono/Hono", "hono/app:serve"
 *      - Remote services: "https://api.example.com/users"
 *
 * Target resolution:
 *   - "module:method" -> stdlib dispatch
 *   - "package/export" -> external package
 *   - "http://", "https://" -> routed to caelum:request
 *   - "file://" -> routed to fasciculus:lege
 *   - "ws://", "wss://" -> routed to caelum:websocket
 *
 * Examples:
 *   // Fire-and-forget
 *   ad "console:log" ("hello")
 *
 *   // Sync binding with block
 *   ad "fasciculus:lege" ("file.txt") fit textus pro content {
 *       scribe content
 *   }
 *
 *   // Async binding
 *   ad "http:get" (url) fiet Response pro response {
 *       scribe response.body
 *   }
 *
 *   // Plural/streaming
 *   ad "http:batch" (urls) fient Response[] pro responses {
 *       ex responses pro r { scribe r.status }
 *   }
 *
 *   // With error handling
 *   ad "fasciculus:lege" ("file.txt") fit textus pro content {
 *       scribe content
 *   } cape err {
 *       scribe "Error: " + err.message
 *   }
 */
export interface AdStatement extends BaseNode {
    type: 'AdStatement';
    target: string;
    arguments: (Expression | SpreadElement)[];
    binding?: AdBinding;
    body?: BlockStatement;
    catchClause?: CapeClause;
}

/**
 * Pro expression (lambda/anonymous function).
 *
 * GRAMMAR (in EBNF):
 *   lambdaExpr := 'pro' params? ('->' typeAnnotation)? (':' expression | 'redde' expression | blockStmt)
 *   params := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: params is always an array (empty for zero-arg lambdas).
 * INVARIANT: async inferred from presence of cede in block body.
 * INVARIANT: returnType is optional - required for Zig target.
 *
 * WHY: Latin 'pro' (for) + 'redde' (return) creates lambda syntax.
 *      Expression form: "for x, return x * 2"
 *      Block form: "for x { ... }" for multi-statement bodies
 *      Return type mirrors function syntax: "pro x -> numerus: x * 2"
 *
 * Examples:
 *   pro x redde x * 2          -> (x) => x * 2
 *   pro x, y redde x + y       -> (x, y) => x + y
 *   pro redde 42               -> () => 42
 *   pro x { redde x * 2 }      -> (x) => { return x * 2; }
 *   pro { scribe "hi" }        -> () => { console.log("hi"); }
 *   pro x -> numerus: x * 2    -> (x): number => x * 2 (typed return)
 *   pro -> textus: "hello"     -> (): string => "hello" (typed, zero-param)
 */
export interface LambdaExpression extends BaseNode {
    type: 'LambdaExpression';
    params: Identifier[];
    returnType?: TypeAnnotation;
    body: Expression | BlockStatement;
    async: boolean;
}

/**
 * Compile-time evaluation expression.
 *
 * GRAMMAR (in EBNF):
 *   praefixumExpr := 'praefixum' (blockStmt | '(' expression ')')
 *
 * INVARIANT: body is either a BlockStatement or an Expression.
 *
 * WHY: Latin 'praefixum' (pre-fixed, past participle of praefigere) extends
 *      the 'fixum' vocabulary. Where 'fixum' means "fixed/constant", 'praefixum'
 *      means "pre-fixed" — fixed before runtime (at compile time).
 *
 * TARGET SUPPORT:
 *   Zig:    comptime { ... } or comptime (expr)
 *   C++:    constexpr or template evaluation
 *   Rust:   const (in const context)
 *   TS/Py:  ERROR - not supported (no native compile-time evaluation)
 *
 * Examples:
 *   fixum size = praefixum(256 * 4)           // simple expression
 *   fixum table = praefixum { ... redde x }   // block with computation
 */
export interface PraefixumExpression extends BaseNode {
    type: 'PraefixumExpression';
    body: Expression | BlockStatement;
}

/**
 * Collection DSL expression (ex source transforms... without iteration).
 *
 * GRAMMAR (in EBNF):
 *   dslExpr := 'ex' expression dslTransform (',' dslTransform)*
 *
 * WHY: DSL expressions provide concise syntax for collection pipelines
 *      when used in assignment context (no `pro` iteration binding).
 *
 * Examples:
 *   fixum top5 = ex items prima 5
 *   fixum total = ex prices summa
 *   fixum last3 = ex items ultima 3
 *
 * Target mapping (desugared):
 *   ex items prima 5 -> items.prima(5)
 *   ex prices summa  -> prices.summa()
 */
export interface CollectionDSLExpression extends BaseNode {
    type: 'CollectionDSLExpression';
    source: Expression;
    transforms: CollectionDSLTransform[];
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
    | EgoExpression
    | Literal
    | ArrayExpression
    | ObjectExpression
    | RangeExpression
    | BinaryExpression
    | UnaryExpression
    | EstExpression
    | QuaExpression
    | CallExpression
    | MemberExpression
    | ArrowFunctionExpression
    | AssignmentExpression
    | ConditionalExpression
    | CedeExpression
    | NovumExpression
    | TemplateLiteral
    | LambdaExpression
    | PraefixumExpression
    | CollectionDSLExpression
    | ScriptumExpression;

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
export interface EgoExpression extends BaseNode {
    type: 'EgoExpression';
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
 * Target mappings:
 *   0..10    → Array.from({length: 10}, (_, i) => i) (TS), range(0, 10) (Py), 0..10 (Zig)
 *   0 usque 10 → Array.from({length: 11}, ...) (TS), range(0, 11) (Py), 0..11 (Zig)
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
// Type Check Expression
// ---------------------------------------------------------------------------

/**
 * Type check expression (est/non est with type operand).
 *
 * GRAMMAR (in EBNF):
 *   typeCheckExpr := expression ('est' | 'non' 'est') typeAnnotation
 *
 * INVARIANT: expression is the value being checked.
 * INVARIANT: targetType is the type to check against.
 * INVARIANT: negated is true for 'non est' form.
 *
 * WHY: Latin 'est' (is) for runtime type checking.
 *      For primitive types (textus, numerus, bivalens, functio), generates typeof.
 *      For user-defined types (genus), generates instanceof.
 *
 * Target mappings:
 *   x est textus     -> typeof x === "string"
 *   x est numerus    -> typeof x === "number"
 *   x est bivalens   -> typeof x === "boolean"
 *   x est functio    -> typeof x === "function"
 *   x est persona    -> x instanceof persona
 *   x non est textus -> typeof x !== "string"
 *
 * Examples:
 *   si x est textus { ... }
 *   si x non est numerus { ... }
 *   si obj est persona { ... }
 */
export interface EstExpression extends BaseNode {
    type: 'EstExpression';
    expression: Expression;
    targetType: TypeAnnotation;
    negated: boolean;
}

// ---------------------------------------------------------------------------
// Type Cast Expression
// ---------------------------------------------------------------------------

/**
 * Type cast expression (qua operator).
 *
 * GRAMMAR (in EBNF):
 *   castExpr := call ('qua' typeAnnotation)*
 *
 * INVARIANT: expression is the value being cast.
 * INVARIANT: targetType is the type to cast to.
 *
 * WHY: Latin 'qua' (as, in the capacity of) for type assertions.
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
 *   data qua textus              -> data as string
 *   response.body qua objectum   -> (response.body) as object
 *   x qua A qua B                -> (x as A) as B (left-associative)
 */
export interface QuaExpression extends BaseNode {
    type: 'QuaExpression';
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
 * INVARIANT: params follows same structure as FunctioDeclaration.
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
 *
 * Target mappings:
 *   cede → await (TS), await (Py), try (Zig error union), .await (Rust)
 *   cede (in cursor) → yield (TS), yield (Py), N/A (Zig)
 */
export interface CedeExpression extends BaseNode {
    type: 'CedeExpression';
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
 *
 * Target mappings:
 *   novum Type     → new Type() (TS), Type() (Py), Type.init() (Zig)
 *   novum Type { } → new Type() merged with object (TS/Py), Type{ .field = } (Zig)
 */
export interface NovumExpression extends BaseNode {
    type: 'NovumExpression';
    callee: Identifier;
    arguments: (Expression | SpreadElement)[];
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
 *   typeAnnotation := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets*
 *   typeParams := '<' typeParameter (',' typeParameter)* '>'
 *
 * INVARIANT: name is the base type name (textus, numerus, etc.).
 * INVARIANT: nullable indicates optional type with '?'.
 * INVARIANT: union contains multiple types for union types (unio<A, B>).
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
 *   unio<textus, numerus> -> name="union", union=[{name="textus"}, {name="numerus"}]
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
