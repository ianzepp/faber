/**
 * AST Expression Types - All expression node definitions
 *
 * @module parser/ast/expressions
 */

import type { Case, Number as GramNumber } from '../../lexicon/types';
import type { BaseNode } from './base';
import type { TypeAnnotation } from './types';
import type { BlockStatement, CollectionDSLTransform } from './control';
import type { Parameter } from './declarations';

// =============================================================================
// EXPRESSION UNION
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
    | InnatumExpression
    | ConversionExpression
    | ShiftExpression
    | CallExpression
    | MemberExpression
    | AssignmentExpression
    | ConditionalExpression
    | CedeExpression
    | NovumExpression
    | PostfixNovumExpression
    | FingeExpression
    | TemplateLiteral
    | ClausuraExpression
    | PraefixumExpression
    | AbExpression
    | ScriptumExpression
    | LegeExpression
    | RegexLiteral;

// =============================================================================
// PRIMARY EXPRESSIONS
// =============================================================================

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
    /** WHY: Used by semantic/codegen for DSL implicit property access (e.g. `ab ... ubi`). */
    isImplicitProperty?: boolean;
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

// =============================================================================
// COLLECTION EXPRESSIONS
// =============================================================================

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

// =============================================================================
// BINARY AND UNARY EXPRESSIONS
// =============================================================================

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

// =============================================================================
// TYPE EXPRESSIONS
// =============================================================================

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

/**
 * Native type construction expression (innatum operator).
 *
 * GRAMMAR (in EBNF):
 *   innatumExpr := literal 'innatum' typeAnnotation
 *
 * INVARIANT: expression is the literal being constructed (empty object or array).
 * INVARIANT: targetType is the builtin type to construct.
 *
 * WHY: Latin 'innatum' (inborn, innate) for constructing native builtin types.
 *      Unlike 'qua' (cast), this actually constructs the native type.
 *      Used for tabula<K,V> and lista<T> which need proper initialization.
 *
 * Target mappings:
 *   {} innatum tabula<K,V>:
 *     TypeScript: new Map<K,V>()
 *     Python:     {}
 *     Zig:        std.AutoHashMap(K,V).init(allocator)
 *     Rust:       HashMap::new()
 *     C++:        std::map<K,V>{}
 *
 *   [] innatum lista<T>:
 *     TypeScript: []
 *     Python:     []
 *     Zig:        std.ArrayList(T).init(allocator)
 *     Rust:       Vec::new()
 *     C++:        std::vector<T>{}
 *
 * Examples:
 *   {} innatum tabula<textus, numerus>   -> new Map<string, number>()
 *   [] innatum lista<textus>             -> [] (array already native in TS)
 */
export interface InnatumExpression extends BaseNode {
    type: 'InnatumExpression';
    expression: Expression;
    targetType: TypeAnnotation;
}

/**
 * Type conversion expression.
 *
 * GRAMMAR (in EBNF):
 *   conversionExpr := expression ('numeratum' | 'fractatum' | 'textatum' | 'bivalentum')
 *                     typeParams? ('vel' expression)?
 *   typeParams := '<' typeAnnotation (',' radixType)? '>'
 *   radixType := 'Dec' | 'Hex' | 'Oct' | 'Bin'
 *
 * SEMANTICS:
 *   - numeratum: string/value to integer (can fail, use vel for fallback)
 *   - fractatum: string/value to float (can fail, use vel for fallback)
 *   - textatum: any to string (infallible)
 *   - bivalentum: any to boolean (infallible, follows nonnulla semantics)
 *
 * EXAMPLES:
 *   "42" numeratum              // parse to number, panic on failure
 *   "42" numeratum vel 0        // parse with fallback
 *   "ff" numeratum<i32, Hex>    // parse hex string to i32
 *   42 textatum                 // convert to string
 *   x bivalentum                // convert to boolean (truthiness)
 */
export interface ConversionExpression extends BaseNode {
    type: 'ConversionExpression';
    expression: Expression;
    conversion: 'numeratum' | 'fractatum' | 'textatum' | 'bivalentum';
    targetType?: TypeAnnotation;
    radix?: 'Dec' | 'Hex' | 'Oct' | 'Bin';
    fallback?: Expression;
}

/**
 * Bit shift expression.
 *
 * GRAMMAR (in EBNF):
 *   shiftExpr := expression ('dextratum' | 'sinistratum') expression
 *
 * SEMANTICS:
 *   - dextratum: shift right (>>)
 *   - sinistratum: shift left (<<)
 *
 * WHY: Using Latin keywords instead of << and >> tokens avoids ambiguity
 *      with nested generics like lista<lista<T>>. The -atum suffix follows
 *      the conversion operator pattern (numeratum, textatum, etc.).
 *
 * EXAMPLES:
 *   x dextratum 3       // x >> 3
 *   1 sinistratum n     // 1 << n
 *   (a dextratum 8) et 0xff  // (a >> 8) & 0xff
 */
export interface ShiftExpression extends BaseNode {
    type: 'ShiftExpression';
    expression: Expression;
    direction: 'dextratum' | 'sinistratum';
    amount: Expression;
}

// =============================================================================
// CALL AND MEMBER ACCESS
// =============================================================================

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

// =============================================================================
// ASSIGNMENT AND CONDITIONAL
// =============================================================================

/**
 * Assignment expression.
 *
 * GRAMMAR (in EBNF):
 *   assignExpr := (IDENTIFIER | memberExpr) '=' expression
 *
 * INVARIANT: left must be Identifier or MemberExpression (lvalue).
 * INVARIANT: operator may be '=', '+=', '-=', '*=', '/=', '%=', '&=', '|='.
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

// =============================================================================
// ASYNC AND OBJECT CREATION
// =============================================================================

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
 * New expression (object construction) - PREFIX form.
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
 * Postfix construction expression - POSTFIX form.
 *
 * GRAMMAR (in EBNF):
 *   postfixNovum := objectLiteral 'novum' typeAnnotation
 *
 * INVARIANT: expression is the object literal to pass to constructor.
 * INVARIANT: targetType is the class to instantiate.
 *
 * WHY: Provides postfix construction syntax parallel to 'qua' casting syntax.
 *      Makes the distinction between casting and construction explicit:
 *        { ... } qua Type   → type assertion (compile-time only)
 *        { ... } novum Type → constructor call (runtime instantiation)
 *
 * Target mappings:
 *   { x: 1 } novum Type → new Type({ x: 1 }) (TS)
 */
export interface PostfixNovumExpression extends BaseNode {
    type: 'PostfixNovumExpression';
    expression: Expression;
    targetType: TypeAnnotation;
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
// CLAUSURA AND COMPILE-TIME
// =============================================================================

/**
 * Clausura expression (closure/anonymous function).
 *
 * GRAMMAR (in EBNF):
 *   clausuraExpr := 'clausura' params? ('->' typeAnnotation)? (':' expression | blockStmt)
 *   params := IDENTIFIER (',' IDENTIFIER)*
 *
 * INVARIANT: params is always an array (empty for zero-arg clausuras).
 * INVARIANT: returnType is optional - required for Zig target.
 *
 * WHY: Latin 'clausura' (closure) for anonymous functions.
 *      Expression form uses colon: "clausura x: x * 2"
 *      Block form uses braces: "clausura x { ... }" for multi-statement bodies
 *      Return type uses thin arrow: "clausura x -> numerus: x * 2"
 *
 * Examples:
 *   clausura x: x * 2               -> (x) => x * 2
 *   clausura x, y: x + y            -> (x, y) => x + y
 *   clausura: 42                    -> () => 42
 *   clausura x { redde x * 2 }      -> (x) => { return x * 2; }
 *   clausura { scribe "hi" }        -> () => { console.log("hi"); }
 *   clausura x -> numerus: x * 2    -> (x): number => x * 2 (typed return)
 *   clausura -> textus: "hello"     -> (): string => "hello" (typed, zero-param)
 */
export interface ClausuraExpression extends BaseNode {
    type: 'ClausuraExpression';
    params: Parameter[];
    returnType?: TypeAnnotation;
    body: Expression | BlockStatement;
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

// =============================================================================
// AB EXPRESSION
// =============================================================================

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
// I/O EXPRESSIONS
// =============================================================================

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
 * WHY: The § placeholder is converted to target-appropriate format specifiers.
 *
 * Target mappings:
 *   scriptum("Hello, §", name) →
 *     TS:   `Hello, ${name}`
 *     Py:   "Hello, {}".format(name)
 *     Rust: format!("Hello, {}", name)
 *     C++:  std::format("Hello, {}", name)
 *     Zig:  std.fmt.allocPrint(alloc, "Hello, {any}", .{name})
 *
 * Examples:
 *   scriptum("Hello, §", name)
 *   scriptum("§ + § = §", a, b, a + b)
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
