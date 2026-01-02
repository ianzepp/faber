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
// COMMENT TYPES
// =============================================================================

/**
 * Comment type discriminator.
 *
 * Faber uses # for all comments. Only 'line' type exists.
 */
export type CommentType = 'line';

/**
 * Comment node attached to AST nodes.
 *
 * INVARIANT: value contains comment text without delimiters.
 * INVARIANT: position points to the start of the comment in source.
 *
 * WHY: Comments are preserved through the pipeline for:
 *      - Source-to-source transformation (formatting)
 *      - Code generation with documentation
 *      - IDE tooling and documentation extraction
 */
export interface Comment {
    type: CommentType;
    value: string;
    position: Position;
}

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
 *
 * Comments are attached during parsing:
 * - leadingComments: comments appearing before this node (on previous lines)
 * - trailingComments: comments on the same line after this node
 */
export interface BaseNode {
    position: Position;
    resolvedType?: SemanticType;
    leadingComments?: Comment[];
    trailingComments?: Comment[];
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
    | PraeparaBlock
    | CuraStatement
    | AdStatement
    | IncipitStatement
    | IncipietStatement;

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
 *   ex "crypto" importa * ut crypto       -> source="crypto", wildcard=true, wildcardAlias="crypto"
 */
export interface ImportaDeclaration extends BaseNode {
    type: 'ImportaDeclaration';
    source: string;
    specifiers: ImportSpecifier[];
    wildcard: boolean;
    wildcardAlias?: Identifier;
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
    annotations?: Annotation[];
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
 * Return type verb used in function declaration.
 *
 * WHY: Distinguishes between direct return (`->`) and stream protocol (`fit`/`fiet`/`fiunt`/`fient`).
 *      - `arrow`: Direct return, no protocol overhead
 *      - `fit`: Sync single-value stream (flumina protocol)
 *      - `fiet`: Async single-value stream
 *      - `fiunt`: Sync multi-value stream (generator)
 *      - `fient`: Async multi-value stream (async generator)
 */
export type ReturnVerb = 'arrow' | 'fit' | 'fiet' | 'fiunt' | 'fient';

/**
 * Function declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   funcDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
 *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
 *   funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
 *   returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
 *
 * INVARIANT: async flag set by presence of 'futura' modifier or fiet/fient verb.
 * INVARIANT: generator flag set by presence of 'cursor' modifier or fiunt/fient verb.
 * INVARIANT: params is always an array (empty if no parameters).
 * INVARIANT: typeParams contains compile-time type parameters (prae typus T).
 * INVARIANT: isAbstract is true for abstract methods (no body).
 * INVARIANT: body is optional only when isAbstract is true.
 * INVARIANT: curatorName is set when 'curata NAME' modifier present.
 *
 * Target mappings:
 *   functio                    → function (TS), def (Py), fn (Zig), fn (Rust)
 *   functio() futura           → async function (TS), async def (Py), fn returning !T (Zig)
 *   functio() cursor           → function* (TS), generator def (Py), N/A (Zig)
 *   functio() curata alloc     → function with allocator context (Zig)
 *   abstractus functio         → abstract method (TS), @abstractmethod (Py), ERROR (Zig/Rust)
 *
 * Examples:
 *   functio salve(textus nomen) -> textus { ... }
 *   functio fetch(textus url) futura -> Response { ... }
 *   functio range(numerus n) cursor -> numerus { ... }
 *   functio alloc(textus s) curata alloc -> T { ... }
 *   functio max(prae typus T, T a, T b) -> T { ... }
 *   abstractus functio speak() -> textus
 */
export interface FunctioDeclaration extends BaseNode {
    type: 'FunctioDeclaration';
    name: Identifier;
    typeParams?: TypeParameterDeclaration[];
    params: Parameter[];
    returnType?: TypeAnnotation;
    body?: BlockStatement;
    async: boolean;
    generator: boolean;
    curatorName?: string; // WHY: Allocator binding name from 'curata NAME' modifier
    isConstructor?: boolean;
    isAbstract?: boolean;
    visibility?: Visibility;
    returnVerb?: ReturnVerb; // WHY: Tracks syntax used for return type (-> vs fit/fiet/fiunt/fient)
    annotations?: Annotation[];
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
 * Dual naming (Swift-style external/internal):
 *   'ut' introduces an internal alias: name is external (callsite), alias is internal (body).
 *   textus location ut loc    -> caller uses 'location', body uses 'loc'
 *
 * Default values:
 *   'vel' introduces a default value expression.
 *   textus name vel "World"   -> defaults to "World" if not provided
 *   NOTE: Defaults are invalid with de/in prepositions (borrowed params can't have defaults)
 *
 * Optional parameters:
 *   'si' marks a parameter as optional. Without 'vel', the type becomes ignotum<T>.
 *   With 'vel', the parameter has a default value and the type stays T.
 *
 * Examples:
 *   textus nomen              -> regular param
 *   de textus source          -> borrowed/read-only param
 *   in lista<T> items         -> mutable borrow param
 *   ceteri lista<textus> args -> rest param (...args: string[])
 *   textus location ut loc    -> dual naming (external: location, internal: loc)
 *   si numerus aetas          -> optional param (type becomes ignotum<numerus>)
 *   si numerus aetas vel 18   -> optional with default (type stays numerus)
 *   de si numerus depth vel 3 -> borrowed, optional, with default
 */
export interface Parameter extends BaseNode {
    type: 'Parameter';
    name: Identifier;
    alias?: Identifier;
    defaultValue?: Expression;
    typeAnnotation?: TypeAnnotation;
    case?: Case;
    preposition?: string;
    rest?: boolean;
    optional?: boolean; // WHY: 'si' marks parameter as optional
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
    annotations?: Annotation[];
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
    annotations?: Annotation[];
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
    annotations?: Annotation[];
}

// ---------------------------------------------------------------------------
// Genus (Struct) Declarations
// ---------------------------------------------------------------------------

/**
 * Visibility level for genus members.
 *
 * WHY: Three-level visibility matching TypeScript/C++/Python:
 *   - public: accessible from anywhere (default, struct semantics)
 *   - protected: accessible from class and subclasses
 *   - private: accessible only within the class
 */
export type Visibility = 'public' | 'protected' | 'private';

/**
 * Annotation (nota) for modifying declarations.
 *
 * GRAMMAR (in EBNF):
 *   annotation := '@' IDENTIFIER (STRING | expression)?
 *
 * INVARIANT: Each annotation has exactly one name (first identifier after @).
 * INVARIANT: Annotations must appear on their own line before a declaration.
 * INVARIANT: One annotation per @ line.
 *
 * WHY: Annotations provide a clean way to attach metadata to declarations
 *      without cluttering the declaration syntax itself. Each annotation
 *      is one per line with a single name and optional argument.
 *
 * Gender agreement: All gender variants are semantically equivalent.
 *   publicum/publica/publicus → public
 *   privatum/privata/privatus → private
 *   protectum/protecta/protectus → protected
 *
 * Examples:
 *   @ publicum
 *   @ abstractum
 *   @ ad "users:list"
 *   @ ad sed "users/\w+"
 */
export interface Annotation extends BaseNode {
    type: 'Annotation';
    /** The annotation name (first identifier after @) */
    name: string;
    /** Optional argument (string literal, sed pattern, or expression) */
    argument?: Expression;
}

/**
 * Field declaration within a genus.
 *
 * GRAMMAR (in EBNF):
 *   fieldDecl := ('privatus' | 'protectus')? 'generis'? 'nexum'? typeAnnotation IDENTIFIER (':' expression)?
 *
 * INVARIANT: typeAnnotation uses Latin word order (type before name).
 * INVARIANT: visibility defaults to 'public' (struct semantics).
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
 *   protectus textus nomen          -> protected field
 *   numerus aetas: 0                -> field with default
 *   generis fixum PI: 3.14159       -> static constant
 *   nexum numerus count: 0          -> reactive field (triggers pingo on change)
 */
export interface FieldDeclaration extends BaseNode {
    type: 'FieldDeclaration';
    name: Identifier;
    fieldType: TypeAnnotation;
    init?: Expression;
    visibility: Visibility;
    isStatic: boolean;
    isReactive: boolean; // nexum fields trigger re-render on change
    annotations?: Annotation[];
}

/**
 * Genus (struct/class) declaration.
 *
 * GRAMMAR (in EBNF):
 *   genusDecl := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
 *   genusMember := fieldDecl | methodDecl
 *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
 *
 * INVARIANT: name is the type name (lowercase by convention).
 * INVARIANT: fields contains all field declarations.
 * INVARIANT: methods contains all method declarations (FunctioDeclaration with implicit ego).
 * INVARIANT: implements lists pactum names this genus fulfills.
 * INVARIANT: extends is the parent class (single inheritance only).
 * INVARIANT: isAbstract is true when 'abstractus' modifier present.
 *
 * WHY: Latin 'genus' (kind/type) for data structures with fields and methods.
 * WHY: 'sub' (under) for inheritance - child class is "under" parent.
 * WHY: 'abstractus' for classes that cannot be instantiated directly.
 *
 * Target mappings:
 *   genus     → class (TS), class (Py), struct (Zig), struct (Rust)
 *   sub       → extends (TS), inherits (Py), ERROR (Zig/Rust - no class inheritance)
 *   implet    → implements (TS), Protocol (Py), comptime duck typing (Zig), impl Trait (Rust)
 *   abstractus → abstract class (TS), ABC (Py), ERROR (Zig/Rust)
 *   ego       → this (TS), self (Py), self (Zig), self (Rust)
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
 *
 *   genus employee sub persona {
 *       textus title
 *   }
 *
 *   abstractus genus animal {
 *       abstractus functio speak() -> textus
 *   }
 */
export interface GenusDeclaration extends BaseNode {
    type: 'GenusDeclaration';
    name: Identifier;
    typeParameters?: Identifier[];
    extends?: Identifier;
    implements?: Identifier[];
    isAbstract: boolean;
    fields: FieldDeclaration[];
    constructor?: FunctioDeclaration;
    methods: FunctioDeclaration[];
    annotations?: Annotation[];
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
    annotations?: Annotation[];
}

/**
 * Pactum method signature (no body, contract only).
 *
 * GRAMMAR:
 *   pactumMethod := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause?
 *   funcModifier := 'futura' | 'cursor' | 'curata' IDENTIFIER
 */
export interface PactumMethod extends BaseNode {
    type: 'PactumMethod';
    name: Identifier;
    params: Parameter[];
    returnType?: TypeAnnotation;
    async: boolean;
    generator: boolean;
    curatorName?: string;
    annotations?: Annotation[];
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
 *   ifStmt := 'si' expression blockStmt ('cape' IDENTIFIER blockStmt)? ('secus' (ifStmt | blockStmt))?
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
 *   si x > 0 { ... } secus { ... }
 *   si x > 0 { ... } sin x < 0 { ... } secus { ... }
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
 *   eligeCase := 'casu' expression (blockStmt | 'ergo' expression)
 *   defaultCase := 'ceterum' (blockStmt | statement)
 *
 * WHY: Latin 'elige' (choose) for value-based switch.
 *      Use 'discerne' for variant pattern matching on discretio types.
 *
 * Example:
 *   elige status {
 *       casu "pending" { processPending() }
 *       casu "active" { processActive() }
 *       ceterum { processDefault() }
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
 *   variantCase := 'casu' IDENTIFIER (('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*))? blockStmt
 *
 * WHY: 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
 *      Uses 'casu' for match arms, 'ut' to bind whole variants, and 'pro' for positional bindings.
 *
 * Example:
 *   discerne event {
 *       casu Click pro x, y { scribe "clicked at " + x + ", " + y }
 *       casu Keypress pro key { scribe "pressed " + key }
 *       casu Quit { mori "goodbye" }
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
 *   variantCase := 'casu' IDENTIFIER ('ut' IDENTIFIER | 'pro' IDENTIFIER (',' IDENTIFIER)*)? blockStmt
 *
 * INVARIANT: variant is the variant name to match (e.g., Click, Keypress).
 * INVARIANT: Either alias OR bindings is used, not both.
 * INVARIANT: consequent is the block to execute on match.
 *
 * WHY: 'casu' (in the case of) for variant match arms.
 *      'ut' (as) binds the whole variant to a name.
 *      'pro' (for) destructures fields positionally.
 *
 * Examples:
 *   casu Click ut c { ... }       -> variant=Click, alias=c (bind whole variant)
 *   casu Click pro x, y { ... }   -> variant=Click, bindings=[x, y] (destructure)
 *   casu Keypress pro key { ... } -> variant=Keypress, bindings=[key]
 *   casu Quit { ... }             -> variant=Quit, no bindings
 */
export interface VariantCase extends BaseNode {
    type: 'VariantCase';
    variant: Identifier;
    alias?: Identifier;
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

/**
 * Entry point statement (sync).
 *
 * GRAMMAR (in EBNF):
 *   incipitStmt := 'incipit' (blockStmt | 'ergo' statement)
 *
 * INVARIANT: body is always a BlockStatement OR erpiStatement is set.
 *
 * WHY: Latin 'incipit' (it begins) marks the sync program entry point.
 *      This is a pure structural marker — it does not inject magic.
 *      Source is responsible for setup (allocators via cura, etc.).
 *
 *      The 'ergo' (therefore) form chains to a single statement, typically
 *      a cura block for allocator setup. This avoids extra nesting.
 *
 * Target mappings:
 *   TypeScript: top-level statements (no wrapper needed)
 *   Python:     if __name__ == "__main__": ...
 *   Zig:        pub fn main() void { ... }
 *   Rust:       fn main() { ... }
 *   C++:        int main() { ... }
 *
 * Examples:
 *   incipit {
 *       scribe "Hello"
 *   }
 *
 *   incipit ergo cura arena {
 *       // allocator-scoped work, one-liner header
 *   }
 *
 *   incipit ergo runMain()
 */
export interface IncipitStatement extends BaseNode {
    type: 'IncipitStatement';
    body?: BlockStatement;
    ergoStatement?: Statement;
}

/**
 * Entry point statement (async).
 *
 * GRAMMAR (in EBNF):
 *   incipietStmt := 'incipiet' (blockStmt | 'ergo' statement)
 *
 * INVARIANT: body is always a BlockStatement OR ergoStatement is set.
 *
 * WHY: Latin 'incipiet' (it will begin) marks the async program entry point.
 *      Mirrors the fit/fiet pattern: present tense for sync, future for async.
 *
 *      The 'ergo' (therefore) form chains to a single statement, typically
 *      a cura block for allocator setup.
 *
 * Target mappings:
 *   TypeScript: (async () => { ... })()
 *   Python:     asyncio.run(main()) with async def main()
 *   Rust:       #[tokio::main] async fn main() { ... }
 *   C++:        int main() { std::async(...).get(); }
 *   Zig:        Not supported (Zig has no async main)
 *
 * Examples:
 *   incipiet {
 *       fixum data = cede fetchData()
 *       scribe data
 *   }
 *
 *   incipiet ergo cura arena {
 *       fixum data = cede fetchData()
 *   }
 */
export interface IncipietStatement extends BaseNode {
    type: 'IncipietStatement';
    body?: BlockStatement;
    ergoStatement?: Statement;
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
 * Read input expression.
 *
 * GRAMMAR (in EBNF):
 *   legeExpr := 'lege' ('lineam')?
 *
 * Reads from stdin as a string.
 *   lege        → read all input until EOF
 *   lege lineam → read one line
 *
 * Target mappings (mode: 'all'):
 *   lege → await Bun.stdin.text() (TS)
 *   lege → sys.stdin.read() (Py)
 *   lege → stdin.readAllAlloc(alloc, max) (Zig)
 *   lege → std::cin (C++)
 *   lege → std::io::stdin().read_to_string() (Rust)
 *
 * Target mappings (mode: 'line'):
 *   lege lineam → (await readline()).trim() (TS)
 *   lege lineam → input() (Py)
 *   lege lineam → stdin.readUntilDelimiter('\n') (Zig)
 *   lege lineam → std::getline(std::cin, line) (C++)
 *   lege lineam → std::io::stdin().read_line() (Rust)
 */
export interface LegeExpression extends BaseNode {
    type: 'LegeExpression';
    mode: 'all' | 'line';
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
 * Fac block statement (explicit scope block or do-while loop).
 *
 * GRAMMAR (in EBNF):
 *   facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
 *
 * INVARIANT: body is always a BlockStatement.
 * INVARIANT: catchClause is optional - for error handling.
 * INVARIANT: test is optional - when present, creates do-while loop.
 *
 * WHY: Latin 'fac' (do!) creates an explicit scope boundary.
 *      Unlike `si verum { }`, this communicates intent clearly.
 *      When paired with 'cape', provides error boundary semantics.
 *      When paired with 'dum', creates do-while loop (body executes first).
 *
 * Examples:
 *   fac { riskyOperation() }
 *   fac { riskyOperation() } cape err { handleError(err) }
 *   fac { process() } dum hasMore()
 *   fac { process() } cape err { log(err) } dum hasMore()
 */
export interface FacBlockStatement extends BaseNode {
    type: 'FacBlockStatement';
    body: BlockStatement;
    catchClause?: CapeClause;
    test?: Expression;
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
 *       praepara { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 */
export interface ProbandumStatement extends BaseNode {
    type: 'ProbandumStatement';
    name: string;
    body: (PraeparaBlock | ProbandumStatement | ProbaStatement)[];
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
 * Timing for praepara/postpara blocks in test context.
 *
 * WHY: 'praepara' = setup (before), 'postpara' = teardown (after)
 */
export type PraeparaTiming = 'praepara' | 'postpara';

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
 * Test setup/teardown block.
 *
 * GRAMMAR (in EBNF):
 *   praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
 *
 * INVARIANT: timing distinguishes setup (praepara) vs teardown (postpara).
 * INVARIANT: async flag distinguishes sync (-a) vs async (-bit) variants.
 * INVARIANT: omnia flag distinguishes all vs each.
 *
 * WHY: Latin "praepara" (prepare!) for test setup, "postpara" (cleanup!) for teardown.
 *      Uses -bit suffix for async (future tense), matching fit/fiet pattern.
 *
 * Target mappings:
 *   TypeScript: beforeEach() / beforeAll() / afterEach() / afterAll()
 *   Python:     @pytest.fixture / setup_module / teardown
 *   Zig:        inlined into each test
 *   Rust:       inlined into each test
 *   C++:        inlined into each test
 *
 * Examples:
 *   praepara { lexer = init() }
 *   praepara omnia { db = connect() }
 *   praeparabit omnia { db = cede connect() }
 *   postpara { cleanup() }
 *   postpara omnia { db.close() }
 *   postparabit omnia { cede db.close() }
 */
export interface PraeparaBlock extends BaseNode {
    type: 'PraeparaBlock';
    timing: PraeparaTiming;
    async: boolean;
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
 *   lambdaExpr := 'pro' params? ('->' typeAnnotation)? (':' expression | blockStmt)
 *   params := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: params is always an array (empty for zero-arg lambdas).
 * INVARIANT: async inferred from presence of cede in block body.
 * INVARIANT: returnType is optional - required for Zig target.
 *
 * WHY: Latin 'pro' (for) creates lambda syntax.
 *      Expression form uses colon: "pro x: x * 2"
 *      Block form uses braces: "pro x { ... }" for multi-statement bodies
 *      Return type uses thin arrow: "pro x -> numerus: x * 2"
 *
 * Examples:
 *   pro x: x * 2               -> (x) => x * 2
 *   pro x, y: x + y            -> (x, y) => x + y
 *   pro: 42                    -> () => 42
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

/**
 * Ab expression - collection filtering DSL.
 *
 * GRAMMAR (in EBNF):
 *   abExpr := 'ab' expression filter? (',' transform)*
 *   filter := ['non'] ('ubi' condition | identifier)
 *   condition := expression
 *   transform := 'ordina' 'per' property [direction]
 *              | 'prima' number
 *              | 'ultima' number
 *              | 'collige' property
 *              | 'grupa' 'per' property
 *
 * WHY: 'ab' (away from) is the dedicated DSL entry point for filtering.
 *      The 'ex' preposition remains unchanged for iteration/import/destructuring.
 *      Include/exclude is handled via 'non' keyword: ab users activus vs ab users non banned.
 *
 * Examples:
 *   ab users activus                    -> users.filter(u => u.activus)
 *   ab users non banned                 -> users.filter(u => !u.banned)
 *   ab users ubi aetas >= 18            -> users.filter(u => u.aetas >= 18)
 *   ab users non ubi banned et suspended -> users.filter(u => !(u.banned && u.suspended))
 *   ab users activus, prima 10          -> users.filter(u => u.activus).slice(0, 10)
 *
 * Iteration form:
 *   ab users activus pro user { }       -> for (const user of users.filter(u => u.activus)) { }
 */
export interface AbExpression extends BaseNode {
    type: 'AbExpression';
    source: Expression;
    /** Whether the filter is negated (non ubi vs ubi) */
    negated: boolean;
    /** Filter condition - either a property name (boolean shorthand) or full expression */
    filter?: {
        /** true if 'ubi' was used, false for boolean property shorthand */
        hasUbi: boolean;
        /** The filter condition expression */
        condition: Expression;
    };
    /** Optional transforms after filtering */
    transforms?: CollectionDSLTransform[];
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
    | AssignmentExpression
    | ConditionalExpression
    | CedeExpression
    | NovumExpression
    | FingeExpression
    | TemplateLiteral
    | LambdaExpression
    | PraefixumExpression
    | CollectionDSLExpression
    | AbExpression
    | ScriptumExpression
    | LegeExpression
    | RegexLiteral;

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
 * Regex literal expression.
 *
 * GRAMMAR (in EBNF):
 *   regexLiteral := 'sed' STRING IDENTIFIER?
 *
 * INVARIANT: pattern is the regex pattern string (without quotes).
 * INVARIANT: flags is the optional flags identifier (i, m, s, x, u combinations).
 *
 * WHY: "sed" (the Unix stream editor) is synonymous with pattern matching.
 *      The pattern string is passed through verbatim to the target - Faber
 *      does not validate regex syntax (that's the target compiler's job).
 *
 * No "g" flag: Global matching (first vs all) is determined by the method
 * called (quaere vs para), not by the regex pattern itself.
 *
 * Target mappings:
 *   TypeScript: /pattern/flags (native regex literal)
 *   Python:     re.compile(r'(?flags)pattern')
 *   Rust:       Regex::new(r"(?flags)pattern")
 *   C++:        std::regex("pattern") (flags not supported inline)
 *   Zig:        "(?flags)pattern" (string for external library)
 *
 * Examples:
 *   sed "\\d+"           -> pattern="\\d+", flags=""
 *   sed "hello" i        -> pattern="hello", flags="i"
 *   sed "^start" im      -> pattern="^start", flags="im"
 */
export interface RegexLiteral extends BaseNode {
    type: 'RegexLiteral';
    pattern: string;
    flags: string;
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
    /** WHY: Set by semantic analyzer when callee has curator param - codegen injects allocator */
    needsCurator?: boolean;
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

/**
 * Discretio variant construction expression.
 *
 * GRAMMAR (in EBNF):
 *   fingeExpr := 'finge' IDENTIFIER ('{' fieldList '}')? ('qua' IDENTIFIER)?
 *
 * WHY: Latin 'finge' (imperative of fingere - to form, shape, mold) for
 *      constructing discretio variants. Distinct from novum (object instantiation).
 *
 * The variant name comes first, optional fields in braces, optional qua for
 * explicit discretio type when not inferrable from context.
 *
 * Examples:
 *   finge Click { x: 10, y: 20 }           - payload variant, type inferred
 *   finge Click { x: 10, y: 20 } qua Event - payload variant, explicit type
 *   finge Active                            - unit variant, type inferred
 *   finge Active qua Status                 - unit variant, explicit type
 *
 * Target mappings:
 *   TS:   { tag: 'Click', x: 10, y: 20 }
 *   Py:   Event_Click(x=10, y=20)
 *   Rust: Event::Click { x: 10, y: 20 }
 *   Zig:  Event{ .click = .{ .x = 10, .y = 20 } }
 *   C++:  Click{.x = 10, .y = 20}
 */
export interface FingeExpression extends BaseNode {
    type: 'FingeExpression';
    variant: Identifier;
    fields?: ObjectExpression;
    discretioType?: Identifier;
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
