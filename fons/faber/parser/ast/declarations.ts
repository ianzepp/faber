/**
 * AST Declaration Types - Import/export, variable, function, and type declarations
 *
 * @module parser/ast/declarations
 */

import type { BaseNode, Visibility } from './base';
import type { Expression, Identifier, Literal } from './expressions';
import type { TypeAnnotation } from './types';
import type { BlockStatement } from './control';

// =============================================================================
// IMPORT/EXPORT DECLARATIONS
// =============================================================================

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

// =============================================================================
// PATTERN TYPES (for destructuring)
// =============================================================================

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

// =============================================================================
// VARIABLE DECLARATIONS
// =============================================================================

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

// =============================================================================
// FUNCTION DECLARATIONS
// =============================================================================

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
 * Function modifiers that appear after the parameter list.
 *
 * GRAMMAR (in EBNF):
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'exitus' (IDENTIFIER | NUMBER)
 *                | 'immutata'
 *                | 'iacit'
 */
export type FunctioModifier = CurataModifier | ErrataModifier | ExitusModifier | ImmutataModifier | IacitModifier;

export interface CurataModifier extends BaseNode {
    type: 'CurataModifier';
    name: Identifier;
}

export interface ErrataModifier extends BaseNode {
    type: 'ErrataModifier';
    name: Identifier;
}

export interface ExitusModifier extends BaseNode {
    type: 'ExitusModifier';
    code: Identifier | Literal;
}

export interface ImmutataModifier extends BaseNode {
    type: 'ImmutataModifier';
}

export interface IacitModifier extends BaseNode {
    type: 'IacitModifier';
}

/**
 * Function declaration statement.
 *
 * GRAMMAR (in EBNF):
 *   funcDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
 *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'exitus' (IDENTIFIER | NUMBER)
 *                | 'immutata'
 *                | 'iacit'
 *   returnClause := ('->' | 'fit' | 'fiet' | 'fiunt' | 'fient') typeAnnotation
 *
 * INVARIANT: async flag set by presence of 'futura' modifier or fiet/fient verb.
 * INVARIANT: generator flag set by presence of 'cursor' modifier or fiunt/fient verb.
 * INVARIANT: params is always an array (empty if no parameters).
 * INVARIANT: typeParams contains compile-time type parameters (prae typus T).
 * INVARIANT: isAbstract is true for abstract methods (no body).
 * INVARIANT: body is optional only when isAbstract is true.
 * INVARIANT: modifiers includes CurataModifier when 'curata NAME' modifier present.
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
    modifiers?: FunctioModifier[];
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
    case?: import('../../lexicon/types').Case;
    preposition?: string;
    rest?: boolean;
    optional?: boolean; // WHY: 'si' marks parameter as optional
}

// =============================================================================
// TYPE ALIAS DECLARATION
// =============================================================================

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

// =============================================================================
// ENUM DECLARATIONS
// =============================================================================

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

// =============================================================================
// DISCRETIO (TAGGED UNION) DECLARATIONS
// =============================================================================

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

// =============================================================================
// GENUS (STRUCT) DECLARATIONS
// =============================================================================

/**
 * Annotation (nota) for modifying declarations.
 *
 * GRAMMAR (in EBNF):
 *   annotation := '@' IDENTIFIER (STRING | expression)?
 *              | '@' 'innatum' targetMapping (',' targetMapping)*
 *              | '@' 'subsidia' targetMapping (',' targetMapping)*
 *              | '@' 'radix' IDENTIFIER (',' IDENTIFIER)*
 *              | '@' 'verte' IDENTIFIER (STRING | '(' paramList ')' '->' STRING)
 *
 *   targetMapping := IDENTIFIER STRING
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
 *   @ innatum ts "Array", py "list", zig "Lista"
 *   @ subsidia zig "subsidia/zig/lista.zig"
 *   @ radix filtr, imperativus, perfectum
 *   @ verte ts "push"
 *   @ verte ts (ego, elem) -> "[...§, §]"
 */
export interface Annotation extends BaseNode {
    type: 'Annotation';
    /** The annotation name (first identifier after @) */
    name: string;
    /** Optional argument (string literal, sed pattern, or expression) */
    argument?: Expression;

    /**
     * For @ innatum and @ subsidia: target-to-value mappings.
     * Example: { ts: "Array", py: "list", zig: "Lista" }
     */
    targetMappings?: Map<string, string>;

    /**
     * For @ radix: stem and valid morphological forms.
     * First element is the stem, rest are form names.
     * Example: ["filtr", "imperativus", "perfectum"]
     */
    radixForms?: string[];

    /**
     * For @ verte: the target language (ts, py, rs, cpp, zig).
     */
    verteTarget?: string;

    /**
     * For @ verte with simple method: the method name.
     * Example: "push" for `@ verte ts "push"`
     */
    verteMethod?: string;

    /**
     * For @ verte with template: parameter names.
     * Example: ["ego", "elem"] for `@ verte ts (ego, elem) -> "..."`
     */
    verteParams?: string[];

    /**
     * For @ verte with template: the template string with § placeholders.
     * Example: "[...§, §]" for `@ verte ts (ego, elem) -> "[...§, §]"`
     */
    verteTemplate?: string;

    /**
     * For @ imperia: the module reference identifier.
     * Example: `remote` in `@ imperia "remote" ex remote`
     */
    exClause?: Identifier;

    // -------------------------------------------------------------------------
    // CLI Option Annotations (@ optio, @ operandus)
    // -------------------------------------------------------------------------

    /**
     * For @ optio: the type of the option (bivalens, textus, numerus, etc.)
     */
    optioType?: TypeAnnotation;

    /**
     * For @ optio: the binding name (valid identifier, always position 2).
     * Example: `v` in `@ optio bivalens v brevis "v" longum "verbose"`
     */
    optioInternal?: Identifier;

    /**
     * For @ optio: the short flag (single character, from 'brevis').
     * Example: "v" in `@ optio bivalens v brevis "v"`
     */
    optioShort?: string;

    /**
     * For @ optio: the long flag (from 'longum').
     * Example: "verbose" in `@ optio bivalens v longum "verbose"`
     */
    optioLong?: string;

    /**
     * For @ optio / @ operandus: the description for help text.
     * Example: "Enable verbose output" in `@ optio bivalens v brevis "v" descriptio "Enable verbose output"`
     */
    optioDescription?: string;

    /**
     * For @ operandus: the type of the positional operand.
     */
    operandusType?: TypeAnnotation;

    /**
     * For @ operandus: the binding name.
     */
    operandusName?: Identifier;

    /**
     * For @ operandus: whether this is a rest/variadic operand (ceteri prefix).
     */
    operandusRest?: boolean;

    /**
     * For @ operandus: the default value expression.
     */
    operandusDefault?: Expression;

    /**
     * For @ operandus: the description for help text.
     */
    operandusDescription?: string;
}

/**
 * Field declaration within a genus.
 *
 * GRAMMAR (in EBNF):
 *   fieldDecl := ('privatus' | 'protectus')? 'generis'? typeAnnotation IDENTIFIER (':' expression)?
 *
 * INVARIANT: typeAnnotation uses Latin word order (type before name).
 * INVARIANT: visibility defaults to 'public' (struct semantics).
 * INVARIANT: isStatic is true when 'generis' modifier present.
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
 */
export interface FieldDeclaration extends BaseNode {
    type: 'FieldDeclaration';
    name: Identifier;
    fieldType: TypeAnnotation;
    init?: Expression;
    visibility: Visibility;
    isStatic: boolean;
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

// =============================================================================
// PACTUM (INTERFACE) DECLARATIONS
// =============================================================================

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
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'immutata'
 *                | 'iacit'
 */
export interface PactumMethod extends BaseNode {
    type: 'PactumMethod';
    name: Identifier;
    params: Parameter[];
    returnType?: TypeAnnotation;
    async: boolean;
    generator: boolean;
    modifiers?: FunctioModifier[];
    annotations?: Annotation[];
}
