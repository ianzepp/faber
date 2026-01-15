/**
 * Primary Expression Parsing
 *
 * Handles parsing of primary expressions (literals, identifiers, grouped expressions),
 * call expressions (function calls, member access, subscript), and lambda expressions.
 *
 * GRAMMAR: See `EBNF.md` "Expressions" section
 *
 * @module parser/expressions/primary
 */

import type { Resolver } from '../resolver';
import type {
    Expression,
    Identifier,
    Literal,
    EgoExpression,
    TemplateLiteral,
    ArrayExpression,
    ObjectExpression,
    ObjectProperty,
    SpreadElement,
    CallExpression,
    MemberExpression,
    LambdaExpression,
    ScriptumExpression,
    LegeExpression,
    QuaExpression,
    InnatumExpression,
    ConversionExpression,
    ShiftExpression,
    NovumExpression,
    FingeExpression,
    TypeAnnotation,
    BlockStatement,
} from '../ast';
import { ParserErrorCode } from '../errors';
import {
    isDSLVerb,
    parseAbExpression,
    parseCollectionDSLExpression,
    parseRegexLiteral,
} from './dsl';

// =============================================================================
// FORWARD REFERENCE
// =============================================================================

/**
 * Forward reference to unary parsing.
 *
 * WHY: parseQuaExpression needs to call parseUnary for shift amounts and
 *      conversion fallbacks, but parseUnary is in a separate module that
 *      imports from this one. Forward reference breaks the cycle.
 */
let parseUnaryImpl: (r: Resolver) => Expression;

/**
 * Set the unary parser implementation.
 *
 * WHY: Allows the main parser to inject the unary parser after module loading.
 */
export function setUnaryParser(fn: (r: Resolver) => Expression): void {
    parseUnaryImpl = fn;
}

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Check if token type is a chain accessor (for optional chaining / non-null assertion).
 */
function isChainAccessor(type: string): boolean {
    return type === 'DOT' || type === 'LBRACKET' || type === 'LPAREN';
}

// =============================================================================
// SCRIPTUM EXPRESSION
// =============================================================================

/**
 * Parse format string expression.
 *
 * GRAMMAR:
 *   scriptumExpr := 'scriptum' '(' STRING (',' expression)* ')'
 *
 * WHY: "scriptum" (that which has been written) is the perfect passive participle
 *      of scribere. While scribe outputs to console, scriptum returns a formatted string.
 *
 * WHY: The section placeholder is converted to target-appropriate format specifiers.
 *
 * Examples:
 *   scriptum("Hello, section", name)
 *   scriptum("section + section = section", a, b, a + b)
 */
export function parseScriptumExpression(r: Resolver): ScriptumExpression {
    const ctx = r.ctx();
    // Position of 'scriptum' keyword (already consumed by caller)
    const position = ctx.tokens[ctx.current - 1]!.position;

    ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

    // First argument must be the format string literal
    const formatToken = ctx.peek();
    if (formatToken.type !== 'STRING') {
        ctx.error(ParserErrorCode.ExpectedString, 'scriptum requires a format string literal as first argument');
    }
    ctx.advance();
    const format: Literal = {
        type: 'Literal',
        value: formatToken.value,
        raw: `"${formatToken.value}"`,
        position: formatToken.position,
    };

    // Parse remaining arguments
    const args: Expression[] = [];
    while (ctx.match('COMMA')) {
        args.push(r.expression());
    }

    ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

    return { type: 'ScriptumExpression', format, arguments: args, position };
}

// =============================================================================
// LEGE EXPRESSION
// =============================================================================

/**
 * Parse stdin read expression.
 *
 * GRAMMAR:
 *   legeExpr := 'lege' ('lineam')?
 *
 * Reads from stdin:
 *   lege        -> read all input until EOF
 *   lege lineam -> read one line
 */
export function parseLegeExpression(r: Resolver): LegeExpression {
    const ctx = r.ctx();
    // Position of 'lege' keyword (already consumed by caller)
    const position = ctx.tokens[ctx.current - 1]!.position;

    // Check for 'lineam' modifier
    const mode = ctx.matchKeyword('lineam') ? 'line' : 'all';

    return { type: 'LegeExpression', mode, position };
}

// =============================================================================
// QUA EXPRESSION
// =============================================================================

/**
 * Parse type cast and native construction expressions.
 *
 * GRAMMAR:
 *   castExpr := call ('qua' typeAnnotation | 'innatum' typeAnnotation)*
 *
 * PRECEDENCE: Between unary and call. This means:
 *   -x qua T     parses as -(x qua T)    - unary binds looser
 *   x.y qua T    parses as (x.y) qua T   - member access binds tighter
 *   x qua A qua B parses as (x qua A) qua B - left-associative
 *
 * WHY: Latin 'qua' (as, in the capacity of) for type assertions.
 *      Compile-time only - no runtime construction or conversion. Maps to:
 *      - TypeScript: (x as T)
 *      - Python: x (ignored, dynamic typing)
 *      - Zig: @as(T, x)
 *      - Rust: x as T
 *      - C++: static_cast<T>(x)
 *
 * WHY: Latin 'innatum' (inborn, innate) for native type construction.
 *      Unlike qua, this constructs the actual native representation.
 *      Use for built-in collection types that need proper initialization:
 *        - [] innatum lista<T>    -> typed array
 *        - {} innatum tabula<K,V> -> new Map<K,V>()
 *        - [] innatum copia<T>    -> new Set<T>()
 *
 * IMPORTANT: Do NOT use `qua` for collection construction. For example:
 *      `{} qua copia<T>` produces a plain object cast, not a Set.
 *      Use `[] innatum copia<T>` to get an actual Set with .add(), .has(), etc.
 */
export function parseQuaExpression(r: Resolver): Expression {
    const ctx = r.ctx();
    let expr = parseCall(r);

    while (true) {
        let keyword: string | undefined;
        let position = ctx.peek().position;

        if (ctx.checkKeyword('qua')) {
            keyword = 'qua';
            ctx.advance();
        }
        else if (ctx.checkKeyword('innatum')) {
            keyword = 'innatum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('numeratum')) {
            keyword = 'numeratum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('fractatum')) {
            keyword = 'fractatum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('textatum')) {
            keyword = 'textatum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('bivalentum')) {
            keyword = 'bivalentum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('dextratum')) {
            keyword = 'dextratum';
            ctx.advance();
        }
        else if (ctx.checkKeyword('sinistratum')) {
            keyword = 'sinistratum';
            ctx.advance();
        }
        else {
            break;
        }

        if (keyword === 'qua') {
            const targetType = r.typeAnnotation();
            expr = {
                type: 'QuaExpression',
                expression: expr,
                targetType,
                position,
            } as QuaExpression;
        }
        else if (keyword === 'innatum') {
            const targetType = r.typeAnnotation();
            expr = {
                type: 'InnatumExpression',
                expression: expr,
                targetType,
                position,
            } as InnatumExpression;
        }
        else if (keyword === 'dextratum' || keyword === 'sinistratum') {
            // Bit shift operators: x dextratum 3 -> x >> 3, x sinistratum 3 -> x << 3
            const direction = keyword as ShiftExpression['direction'];
            const amount = parseUnaryImpl(r);

            expr = {
                type: 'ShiftExpression',
                expression: expr,
                direction,
                amount,
                position,
            } as ShiftExpression;
        }
        else {
            // Conversion operators: numeratum, fractatum, textatum, bivalentum
            const conversion = keyword as ConversionExpression['conversion'];
            let targetType: TypeAnnotation | undefined;
            let radix: ConversionExpression['radix'];
            let fallback: Expression | undefined;

            // Parse optional type parameters for numeratum/fractatum
            if ((conversion === 'numeratum' || conversion === 'fractatum') && ctx.match('LESS')) {
                targetType = r.typeAnnotation();
                if (ctx.match('COMMA')) {
                    // Radix type: Dec, Hex, Oct, Bin
                    const radixToken = ctx.peek();
                    if (['Dec', 'Hex', 'Oct', 'Bin'].includes(radixToken.value)) {
                        radix = ctx.advance().value as ConversionExpression['radix'];
                    }
                    else {
                        ctx.reportError(
                            ParserErrorCode.UnexpectedToken,
                            `Expected radix type (Dec, Hex, Oct, Bin), got '${radixToken.value}'`
                        );
                    }
                }
                ctx.expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
            }

            // Parse optional fallback with 'vel'
            if (ctx.matchKeyword('vel')) {
                fallback = parseUnaryImpl(r);
            }

            expr = {
                type: 'ConversionExpression',
                expression: expr,
                conversion,
                targetType,
                radix,
                fallback,
                position,
            } as ConversionExpression;
        }
    }

    return expr;
}

// =============================================================================
// NOVUM EXPRESSION
// =============================================================================

/**
 * Parse new expression (object construction).
 *
 * GRAMMAR:
 *   newExpr := 'novum' IDENTIFIER ('(' argumentList ')')? (objectLiteral | 'de' expression)?
 *
 * WHY: Two forms for property overrides:
 *      - Inline literal: `novum Persona { nomen: "Marcus" }`
 *      - From expression: `novum Persona de props` (props is variable/call/etc.)
 *
 *      The `de` (from) form allows dynamic overrides from variables or function results.
 */
export function parseNovumExpression(r: Resolver): NovumExpression {
    const ctx = r.ctx();
    // Position of 'novum' keyword (already consumed by caller)
    const position = ctx.tokens[ctx.current - 1]!.position;

    const callee = ctx.parseIdentifier();

    let args: (Expression | SpreadElement)[] = [];

    if (ctx.match('LPAREN')) {
        args = parseArgumentList(r);
        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
    }

    let withExpression: Expression | undefined;

    // Check for property overrides: novum X { ... } or novum X de expr
    if (ctx.check('LBRACE')) {
        withExpression = parsePrimary(r);
    }
    else if (ctx.matchKeyword('de')) {
        withExpression = r.expression();
    }

    return { type: 'NovumExpression', callee, arguments: args, withExpression, position };
}

// =============================================================================
// FINGE EXPRESSION
// =============================================================================

/**
 * Parse discretio variant construction expression.
 *
 * GRAMMAR:
 *   fingeExpr := 'finge' IDENTIFIER ('{' fieldList '}')? ('qua' IDENTIFIER)?
 *
 * WHY: Latin 'finge' (form/shape) for constructing discretio variants.
 *      Variant name comes first, optional fields in braces, optional qua for
 *      explicit discretio type when not inferrable from context.
 *
 * Examples:
 *   finge Click { x: 10, y: 20 }           - payload variant
 *   finge Click { x: 10, y: 20 } qua Event - with explicit type
 *   finge Active                            - unit variant
 *   finge Active qua Status                 - unit variant with explicit type
 */
export function parseFingeExpression(r: Resolver): FingeExpression {
    const ctx = r.ctx();
    // Position of 'finge' keyword (already consumed by caller)
    const position = ctx.tokens[ctx.current - 1]!.position;

    const variant = ctx.parseIdentifier();

    let fields: ObjectExpression | undefined;

    // Check for payload fields: finge Click { x: 10, y: 20 }
    if (ctx.check('LBRACE')) {
        const fieldsExpr = parsePrimary(r);

        if (fieldsExpr.type === 'ObjectExpression') {
            fields = fieldsExpr;
        }
    }

    let discretioType: Identifier | undefined;

    // Check for explicit type: finge Click { } qua Event
    if (ctx.matchKeyword('qua')) {
        discretioType = ctx.parseIdentifier();
    }

    return { type: 'FingeExpression', variant, fields, discretioType, position };
}

// =============================================================================
// CALL EXPRESSION
// =============================================================================

/**
 * Parse call expression with postfix operators.
 *
 * GRAMMAR:
 *   call := primary (callSuffix | memberSuffix | optionalSuffix | nonNullSuffix)*
 *   callSuffix := '(' argumentList ')'
 *   memberSuffix := '.' IDENTIFIER | '[' expression ']'
 *   optionalSuffix := '?.' IDENTIFIER | '?[' expression ']' | '?(' argumentList ')'
 *   nonNullSuffix := '!.' IDENTIFIER | '![' expression ']' | '!(' argumentList ')'
 *
 * PRECEDENCE: Highest (binds tightest after primary).
 *
 * WHY: Handles function calls, member access, and computed member access.
 *      Left-associative via loop (obj.a.b parsed as (obj.a).b).
 *
 * OPTIONAL CHAINING: ?. ?[ ?( return nihil if object is nihil
 * NON-NULL ASSERTION: !. ![ !( assert object is not nihil
 */
export function parseCall(r: Resolver): Expression {
    const ctx = r.ctx();
    let expr = parsePrimary(r);

    while (true) {
        // ---------------------------------------------------------------
        // Optional chaining: ?. ?[ ?(
        // WHY: Check for QUESTION followed by accessor token to disambiguate from ternary
        // ---------------------------------------------------------------
        if (ctx.check('QUESTION') && isChainAccessor(ctx.peek(1).type)) {
            const position = ctx.peek().position;
            ctx.advance(); // consume QUESTION

            if (ctx.match('DOT')) {
                // WHY: Allow keywords as property names (e.g., items.omitte)
                const property = ctx.parseIdentifierOrKeyword();
                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: false,
                    optional: true,
                    position,
                };
            }
            else if (ctx.match('LBRACKET')) {
                const property = r.expression();
                ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);
                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: true,
                    optional: true,
                    position,
                };
            }
            else if (ctx.match('LPAREN')) {
                const args = parseArgumentList(r);
                ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
                expr = { type: 'CallExpression', callee: expr, arguments: args, optional: true, position };
            }
        }
        // ---------------------------------------------------------------
        // Non-null assertion: !. ![ !(
        // WHY: BANG is no longer used for prefix logical not, so it's unambiguous
        // ---------------------------------------------------------------
        else if (ctx.check('BANG') && isChainAccessor(ctx.peek(1).type)) {
            const position = ctx.peek().position;
            ctx.advance(); // consume BANG

            if (ctx.match('DOT')) {
                // WHY: Allow keywords as property names (e.g., items.omitte)
                const property = ctx.parseIdentifierOrKeyword();
                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: false,
                    nonNull: true,
                    position,
                };
            }
            else if (ctx.match('LBRACKET')) {
                const property = r.expression();
                ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);
                expr = {
                    type: 'MemberExpression',
                    object: expr,
                    property,
                    computed: true,
                    nonNull: true,
                    position,
                };
            }
            else if (ctx.match('LPAREN')) {
                const args = parseArgumentList(r);
                ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
                expr = { type: 'CallExpression', callee: expr, arguments: args, nonNull: true, position };
            }
        }
        // ---------------------------------------------------------------
        // Regular accessors
        // ---------------------------------------------------------------
        else if (ctx.match('LPAREN')) {
            const position = ctx.peek().position;
            const args = parseArgumentList(r);

            ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

            expr = { type: 'CallExpression', callee: expr, arguments: args, position };
        }
        else if (ctx.match('DOT')) {
            const position = ctx.peek().position;
            // WHY: Allow keywords as property names (e.g., items.omitte)
            const property = ctx.parseIdentifierOrKeyword();

            expr = {
                type: 'MemberExpression',
                object: expr,
                property,
                computed: false,
                position,
            };
        }
        else if (ctx.match('LBRACKET')) {
            const position = ctx.peek().position;
            const property = r.expression();

            ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

            expr = {
                type: 'MemberExpression',
                object: expr,
                property,
                computed: true,
                position,
            };
        }
        else {
            break;
        }
    }

    return expr;
}

// =============================================================================
// ARGUMENT LIST
// =============================================================================

/**
 * Parse function call argument list.
 *
 * GRAMMAR:
 *   argumentList := (argument (',' argument)*)?
 *   argument := 'sparge' expression | expression
 */
export function parseArgumentList(r: Resolver): (Expression | SpreadElement)[] {
    const ctx = r.ctx();
    const args: (Expression | SpreadElement)[] = [];

    if (ctx.check('RPAREN')) {
        return args;
    }

    do {
        // Check for spread: sparge expr
        if (ctx.checkKeyword('sparge')) {
            const spreadPos = ctx.peek().position;
            ctx.advance(); // consume 'sparge'
            const argument = r.expression();
            args.push({ type: 'SpreadElement', argument, position: spreadPos });
        }
        else {
            args.push(r.expression());
        }
    } while (ctx.match('COMMA'));

    return args;
}

// =============================================================================
// PRIMARY EXPRESSION
// =============================================================================

/**
 * Parse primary expression (literals, identifiers, grouped expressions).
 *
 * GRAMMAR:
 *   primary := IDENTIFIER | NUMBER | STRING | TEMPLATE_STRING
 *            | 'ego' | 'verum' | 'falsum' | 'nihil'
 *            | '(' (expression | arrowFunction) ')'
 *
 * PRECEDENCE: Highest (atoms of the language).
 *
 * WHY: Latin literals: verum (true), falsum (false), nihil (null).
 *      'ego' (I/self) is the self-reference keyword (like 'this' in JS).
 *      Parenthesized expressions require lookahead to distinguish from arrow functions.
 */
export function parsePrimary(r: Resolver): Expression {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    if (ctx.matchKeyword('ego')) {
        const thisExpr: EgoExpression = { type: 'EgoExpression', position };
        return thisExpr;
    }

    // Boolean literals
    if (ctx.matchKeyword('verum')) {
        return { type: 'Literal', value: true, raw: 'verum', position };
    }

    if (ctx.matchKeyword('falsum')) {
        return { type: 'Literal', value: false, raw: 'falsum', position };
    }

    if (ctx.matchKeyword('nihil')) {
        return { type: 'Literal', value: null, raw: 'nihil', position };
    }

    // Lambda expression: pro x redde expr, pro x, y redde expr, pro redde expr
    // Also: fit x: expr (sync, explicit), fiet x: expr (async)
    if (ctx.checkKeyword('pro') || ctx.checkKeyword('fit')) {
        return parseLambdaExpression(r, false);
    }

    if (ctx.checkKeyword('fiet')) {
        return parseLambdaExpression(r, true);
    }

    // DSL expressions
    // ab expression: filtering DSL (ab users activus)
    if (ctx.checkKeyword('ab')) {
        return parseAbExpression(r);
    }

    // sed expression: regex literal (sed "\\d+" i)
    if (ctx.checkKeyword('sed')) {
        return parseRegexLiteral(r);
    }

    // ex expression in expression context: collection DSL (ex items prima 5)
    // WHY: 'ex' in expression context (not statement start) with DSL verb is collection pipeline
    if (ctx.checkKeyword('ex')) {
        // Look ahead to see if this is DSL (prima/ultima/summa after expression)
        // vs iteration (pro/fit/fiet) or import (importa) or destructuring (fixum/varia)
        // Save position for lookahead
        const savedCurrent = ctx.current;
        ctx.advance(); // consume 'ex'

        // Parse source expression (identifier or more complex expression)
        // For DSL detection, we just check if a DSL verb follows
        // We need to skip one expression-like token and check for DSL verb
        let depth = 0;
        let foundDSL = false;

        // Simple lookahead: skip tokens until we find DSL verb or statement boundary
        while (!ctx.isAtEnd() && depth < 20) {
            if (isDSLVerb(r)) {
                foundDSL = true;
                break;
            }

            // Stop on keywords that indicate non-DSL usage
            const kw = ctx.peek().keyword;
            if (kw === 'pro' || kw === 'fit' || kw === 'fiet' || kw === 'importa' ||
                kw === 'fixum' || kw === 'varia' || kw === 'figendum' || kw === 'variandum') {
                break;
            }

            // Stop on statement boundaries
            if (ctx.check('RBRACE') || ctx.check('SEMICOLON') || ctx.check('EOF')) {
                break;
            }

            ctx.advance();
            depth++;
        }

        // Restore position
        ctx.current = savedCurrent;

        if (foundDSL) {
            return parseCollectionDSLExpression(r);
        }

        // Fall through - 'ex' is in statement context, not expression DSL
    }

    // Number literal (decimal or hex)
    if (ctx.check('NUMBER')) {
        const token = ctx.advance();
        // WHY: Number() handles both decimal and hex (0x) prefixes correctly
        const value = Number(token.value);

        return { type: 'Literal', value, raw: token.value, position };
    }

    // Bigint literal
    if (ctx.check('BIGINT')) {
        const token = ctx.advance();

        return { type: 'Literal', value: BigInt(token.value), raw: `${token.value}n`, position };
    }

    // String literal
    if (ctx.check('STRING')) {
        const token = ctx.advance();

        return { type: 'Literal', value: token.value, raw: `"${token.value}"`, position };
    }

    // Template string
    if (ctx.check('TEMPLATE_STRING')) {
        const token = ctx.advance();

        return { type: 'TemplateLiteral', raw: token.value, position };
    }

    // Array literal
    if (ctx.match('LBRACKET')) {
        const elements: (Expression | SpreadElement)[] = [];

        if (!ctx.check('RBRACKET')) {
            do {
                // Check for spread element: sparge expr
                if (ctx.checkKeyword('sparge')) {
                    const spreadPos = ctx.peek().position;
                    ctx.advance(); // consume 'sparge'
                    const argument = r.expression();
                    elements.push({ type: 'SpreadElement', argument, position: spreadPos });
                }
                else {
                    elements.push(r.expression());
                }
            } while (ctx.match('COMMA') && !ctx.check('RBRACKET'));
        }

        ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

        return { type: 'ArrayExpression', elements, position };
    }

    // Object literal
    if (ctx.match('LBRACE')) {
        const properties: (ObjectProperty | SpreadElement)[] = [];

        if (!ctx.check('RBRACE')) {
            do {
                const propPosition = ctx.peek().position;

                // Check for spread: sparge expr
                if (ctx.checkKeyword('sparge')) {
                    ctx.advance(); // consume 'sparge'
                    const argument = r.expression();
                    properties.push({ type: 'SpreadElement', argument, position: propPosition });
                }
                else {
                    // Key can be identifier or string
                    let key: Identifier | Literal;

                    if (ctx.check('STRING')) {
                        const token = ctx.advance();

                        key = {
                            type: 'Literal',
                            value: token.value,
                            raw: `"${token.value}"`,
                            position: propPosition,
                        };
                    }
                    else {
                        key = ctx.parseIdentifierOrKeyword();
                    }

                    ctx.expect('COLON', ParserErrorCode.ExpectedColon);

                    const value = r.expression();

                    properties.push({ type: 'ObjectProperty', key, value, position: propPosition });
                }
            } while (ctx.match('COMMA'));
        }

        ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

        return { type: 'ObjectExpression', properties, position };
    }

    // Parenthesized (grouped) expression
    if (ctx.match('LPAREN')) {
        const expr = r.expression();

        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        return expr;
    }

    // Identifier
    if (ctx.check('IDENTIFIER')) {
        return ctx.parseIdentifier();
    }

    // Keywords used as identifiers in expression context
    // WHY: Keywords like 'typus', 'genus', 'proba' can be variable names
    //      when they appear in expression position (not starting a statement)
    if (ctx.check('KEYWORD')) {
        const kw = ctx.peek().keyword ?? '';
        // Reject statement-starting keywords that would never be variable names
        // WHY: Only true statement-starting keywords belong here.
        // Contextual keywords like 'cape'/'demum' (only meaningful within
        // tempta/fac) should NOT be blocked - they're valid identifiers
        // in expression context.
        //
        // NOTE: Some keywords (iace, mori, scribe, vide, mone, cura, incipit, incipiet)
        // are intentionally OMITTED from this list. They can be used as function names
        // when called with parentheses: `scribe("hello")` is a function call,
        // while `scribe "hello"` is the keyword statement.
        const statementKeywords = [
            'si',
            'dum',
            'ex',
            'de',
            'in',
            'redde',
            'rumpe',
            'perge',
            'tempta',
            'fac',
            'adfirma',
            'custodi',
            'elige',
            'discerne',
            'ad',
            'probandum',
            'praepara',
            'praeparabit',
            'postpara',
            'postparabit',
        ];
        if (!statementKeywords.includes(kw)) {
            return ctx.parseIdentifierOrKeyword();
        }
    }

    // ctx.error returns never (throws), satisfying TypeScript's return analysis
    return ctx.error(ParserErrorCode.UnexpectedToken, `token '${ctx.peek().value}'`);
}

// =============================================================================
// LAMBDA EXPRESSION
// =============================================================================

/**
 * Parse lambda expression (anonymous function).
 *
 * GRAMMAR:
 *   lambdaExpr := ('pro' | 'fit' | 'fiet') params? ('->' type)? (':' expression | blockStmt)
 *   params := IDENTIFIER (',' IDENTIFIER)*
 *
 * Three keyword forms with different semantics:
 *   - 'pro' / 'fit': sync lambda (pro is casual, fit is explicit verb form)
 *   - 'fiet': async lambda (future tense verb form)
 *
 * Expression form (colon required):
 *      pro x: x * 2              -> (x) => x * 2
 *      pro: 42                   -> () => 42
 *      pro x, y: x + y           -> (x, y) => x + y
 *      pro x -> numerus: x * 2   -> (x): number => x * 2
 *      fiet x: expr              -> async (x) => expr
 *
 * Block form (for multi-statement bodies):
 *      pro x { redde x * 2 }     -> (x) => { return x * 2; }
 *      fiet c { cede fetch() }   -> async (c) => { await fetch(); }
 */
export function parseLambdaExpression(r: Resolver, async: boolean): LambdaExpression {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the keyword (pro, fit, or fiet)
    ctx.advance();

    const params: Identifier[] = [];

    // Check for immediate :, ->, or { (zero-param lambda)
    if (!ctx.check('COLON') && !ctx.check('LBRACE') && !ctx.check('THIN_ARROW')) {
        // Parse parameters until we hit :, ->, or {
        do {
            params.push(ctx.parseIdentifierOrKeyword());
        } while (ctx.match('COMMA'));
    }

    // Parse optional return type annotation: -> Type
    let returnType: TypeAnnotation | undefined;

    if (ctx.match('THIN_ARROW')) {
        returnType = r.typeAnnotation();
    }

    let body: Expression | BlockStatement;

    if (ctx.check('LBRACE')) {
        // Block form: pro x { ... } or pro x -> T { ... }
        body = r.block();
    }
    else {
        // Expression form: pro x: expr or pro x -> T: expr
        ctx.expect('COLON', ParserErrorCode.ExpectedColon);
        body = r.expression();
    }

    return { type: 'LambdaExpression', params, returnType, body, async, position };
}
