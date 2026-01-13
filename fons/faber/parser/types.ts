/**
 * Type Annotation Parsing
 *
 * Handles parsing of type annotations, parameters, and type parameter lists.
 *
 * GRAMMAR: See `EBNF.md` "Types" section
 *
 * @module parser/types
 */

import type { Token } from '../tokenizer/types';
import type { Resolver } from './resolver';
import type {
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    Parameter,
    Expression,
    Identifier,
} from './ast';
import { ParserErrorCode } from './errors';

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Check if token is a borrow preposition (de/in for ownership semantics).
 *
 * WHY: These prepositions encode ownership for systems targets (Rust/Zig):
 *      de = borrowed/read-only (&T, []const u8)
 *      in = mutable borrow (&mut T, *T)
 */
function isBorrowPreposition(token: Token): boolean {
    return token.type === 'KEYWORD' && ['de', 'in'].includes(token.keyword ?? '');
}

// =============================================================================
// TYPE ANNOTATION PARSING
// =============================================================================

/**
 * Parse type annotation.
 *
 * GRAMMAR: typeAnnotation (see `EBNF.md` "Types")
 *
 * WHY: Supports generics (lista<textus>), nullable (?), union types (unio<A, B>),
 *      and array shorthand (numerus[] desugars to lista<numerus>).
 *
 * EDGE: Numeric parameters for sized types (numerus<32>).
 *       Array shorthand preserves source form via arrayShorthand flag.
 *       Borrow prepositions (de/in) for systems targets (Rust/Zig).
 *       Union types use unio<A, B> syntax (pipe reserved for bitwise OR).
 */
export function parseTypeAnnotation(r: Resolver): TypeAnnotation {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Check for function type: (T, U) -> V
    // WHY: Function types in parameter positions enable higher-order functions
    //      e.g., functio filtrata((T) -> bivalens pred) -> lista<T>
    if (ctx.match('LPAREN')) {
        const parameterTypes: TypeAnnotation[] = [];

        if (!ctx.check('RPAREN')) {
            do {
                parameterTypes.push(parseTypeAnnotation(r));
            } while (ctx.match('COMMA'));
        }

        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);
        ctx.expect('THIN_ARROW', ParserErrorCode.ExpectedThinArrow);

        const returnType = parseTypeAnnotation(r);

        return {
            type: 'TypeAnnotation',
            name: '',
            parameterTypes,
            returnType,
            position,
        };
    }

    // Check for borrow preposition (de/in for ownership semantics)
    let preposition: string | undefined;
    if (isBorrowPreposition(ctx.peek())) {
        preposition = ctx.advance().keyword;
    }

    // WHY: Type names are usually identifiers, but some spellings (notably `nihil`)
    //      are keywords and still valid type names.
    let name: string;
    if (ctx.check('IDENTIFIER')) {
        name = ctx.advance().value;
    }
    else if (ctx.check('KEYWORD')) {
        name = ctx.advance().value;
    }
    else {
        ctx.reportError(ParserErrorCode.ExpectedTypeName, `got '${ctx.peek().value}'`);
        name = ctx.peek().value;
        ctx.advance(); // Skip to avoid infinite loop
    }

    let typeParameters: TypeParameter[] | undefined;

    if (ctx.match('LESS')) {
        typeParameters = [];

        do {
            if (ctx.check('NUMBER')) {
                // Numeric parameter (e.g., numerus<32>)
                const numToken = ctx.advance();
                const value = numToken.value.includes('.')
                    ? parseFloat(numToken.value)
                    : parseInt(numToken.value, 10);

                typeParameters.push({
                    type: 'Literal',
                    value,
                    raw: numToken.value,
                    position: numToken.position,
                });
            }
            else {
                // Type parameter (e.g., lista<textus>, numerus<i32>)
                typeParameters.push(parseTypeAnnotation(r));
            }
        } while (ctx.match('COMMA'));

        ctx.expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
    }

    let nullable = false;

    if (ctx.match('QUESTION')) {
        nullable = true;
    }

    // Handle unio<A, B> -> union type with type parameters as union members
    // WHY: unio<A, B> syntax frees pipe for bitwise OR
    if (name === 'unio' && typeParameters && typeParameters.length > 0) {
        // Convert type parameters to union members (must all be TypeAnnotations)
        const union: TypeAnnotation[] = typeParameters.filter(
            (p): p is TypeAnnotation => p.type === 'TypeAnnotation'
        );

        return {
            type: 'TypeAnnotation',
            name: 'union',
            union,
            nullable,
            preposition,
            position,
        };
    }

    // Build the base type
    let result: TypeAnnotation = {
        type: 'TypeAnnotation',
        name,
        typeParameters,
        nullable,
        preposition,
        position,
    };

    // Handle array shorthand: numerus[] -> lista<numerus>
    // Each [] wraps in lista with arrayShorthand flag for round-trip fidelity
    while (ctx.check('LBRACKET') && ctx.peek(1).type === 'RBRACKET') {
        ctx.advance(); // [
        ctx.advance(); // ]

        let arrayNullable = false;
        if (ctx.match('QUESTION')) {
            arrayNullable = true;
        }

        result = {
            type: 'TypeAnnotation',
            name: 'lista',
            typeParameters: [result],
            nullable: arrayNullable,
            arrayShorthand: true,
            position,
        };
    }

    return result;
}

// =============================================================================
// PARAMETER PARSING
// =============================================================================

/**
 * Parse type parameters and regular parameters together.
 *
 * GRAMMAR:
 *   typeAndParamList := ('prae typus' IDENTIFIER ',')* (parameter (',' parameter)*)?
 *
 * WHY: Type parameters and regular parameters share a single parenthesized list.
 *      Type params are declared with 'prae typus T'.
 *
 * Examples:
 *   (prae typus T, T a, T b)     -> typeParams=[T], params=[a, b]
 *   (prae typus T, prae typus U) -> typeParams=[T, U], params=[]
 *   (numerus a, numerus b)       -> typeParams=[], params=[a, b]
 */
export function parseTypeAndParameterList(
    r: Resolver
): { typeParams: TypeParameterDeclaration[]; params: Parameter[] } {
    const ctx = r.ctx();
    const typeParams: TypeParameterDeclaration[] = [];
    const params: Parameter[] = [];

    if (ctx.check('RPAREN')) {
        return { typeParams, params };
    }

    // Parse leading type parameters: prae typus T
    while (ctx.checkKeyword('prae')) {
        const typeParamPos = ctx.peek().position;
        ctx.advance(); // consume 'prae'
        ctx.expectKeyword('typus', ParserErrorCode.ExpectedKeywordTypus);
        const typeParamName = ctx.parseIdentifier();
        typeParams.push({
            type: 'TypeParameterDeclaration',
            name: typeParamName,
            position: typeParamPos,
        });

        // If no comma after type param, we're done with type params
        if (!ctx.match('COMMA')) {
            return { typeParams, params };
        }

        // Check if next is another type param or a regular param
        if (!ctx.checkKeyword('prae')) {
            break; // Switch to parsing regular params
        }
    }

    // Parse remaining regular parameters
    if (!ctx.check('RPAREN')) {
        do {
            params.push(parseParameter(r));
        } while (ctx.match('COMMA'));
    }

    return { typeParams, params };
}

/**
 * Parse function parameter list (simple form without type params).
 *
 * GRAMMAR:
 *   paramList := (parameter (',' parameter)*)?
 */
export function parseParameterList(r: Resolver): Parameter[] {
    const ctx = r.ctx();
    const params: Parameter[] = [];

    if (ctx.check('RPAREN')) {
        return params;
    }

    do {
        params.push(parseParameter(r));
    } while (ctx.match('COMMA'));

    return params;
}

/**
 * Parse single function parameter.
 *
 * GRAMMAR:
 *   parameter := ('de' | 'in' | 'ex')? 'si'? 'ceteri'? (typeAnnotation IDENTIFIER | IDENTIFIER) ('ut' IDENTIFIER)? ('vel' expression)?
 *
 * WHY: Type-first syntax: "textus name" or "de textus source"
 *      Prepositional prefixes indicate semantic roles:
 *      de = from/concerning (borrowed, read-only),
 *      in = in/into (mutable borrow),
 *      ex = from/out of (source)
 *
 * OPTIONAL PARAMETERS:
 *      'si' marks a parameter as optional. Without 'vel', type becomes ignotum<T>.
 *      With 'vel', parameter has a default value and type stays T.
 *      Order: preposition, then si, then ceteri, then type, then name.
 *
 * EDGE: Preposition comes first (if present), then si, then type (if present), then identifier.
 *
 * TYPE DETECTION: Uses lookahead to detect type annotations for user-defined types.
 *   - Builtin type names (textus, numerus, etc.) are recognized directly
 *   - IDENT IDENT pattern: first is type, second is name (e.g., "coordinate point")
 *   - IDENT< pattern: generic type (e.g., "lista<textus>")
 */
export function parseParameter(r: Resolver): Parameter {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    let preposition: string | undefined;

    if (ctx.isPreposition(ctx.peek())) {
        preposition = ctx.advance().keyword;
    }

    // Check for optional parameter: si [type] name [vel default]
    let optional = false;
    if (ctx.checkKeyword('si')) {
        ctx.advance(); // consume 'si'
        optional = true;
    }

    // Check for rest parameter: ceteri [type] name
    let rest = false;
    if (ctx.checkKeyword('ceteri')) {
        ctx.advance(); // consume 'ceteri'
        rest = true;
    }

    let typeAnnotation: TypeAnnotation | undefined;

    // WHY: Use lookahead to detect user-defined types, not just builtins.
    // If we see IDENT followed by IDENT, first is type, second is name.
    // If we see IDENT followed by <, it's a generic type.
    // If we see IDENT followed by [, it's an array type (e.g., Point[]).
    // If we see (, it's a function type: (T) -> U
    const hasTypeAnnotation =
        ctx.isTypeName(ctx.peek()) ||
        ctx.check('LPAREN') || // function type: (T) -> U
        (ctx.check('IDENTIFIER') && ctx.peek(1).type === 'IDENTIFIER') ||
        (ctx.check('IDENTIFIER') && ctx.peek(1).type === 'LESS') ||
        (ctx.check('IDENTIFIER') && ctx.peek(1).type === 'LBRACKET');

    if (hasTypeAnnotation) {
        typeAnnotation = parseTypeAnnotation(r);
    }

    const name = ctx.parseIdentifierOrKeyword();

    // Check for dual naming: 'ut' introduces internal alias
    // textus location ut loc -> external: location, internal: loc
    let alias: Identifier | undefined;
    if (ctx.checkKeyword('ut')) {
        ctx.advance(); // consume 'ut'
        alias = ctx.parseIdentifierOrKeyword();
    }

    // Check for default value: 'vel' introduces default expression
    // si numerus aetas vel 18 -> optional with default
    let defaultValue: Expression | undefined;
    if (ctx.checkKeyword('vel')) {
        ctx.advance(); // consume 'vel'
        defaultValue = r.expression();
    }

    return {
        type: 'Parameter',
        name,
        alias,
        defaultValue,
        typeAnnotation,
        preposition,
        rest,
        optional: optional || undefined,
        position,
    };
}
