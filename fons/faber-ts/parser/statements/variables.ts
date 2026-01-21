/**
 * Variable Declaration Parsing
 *
 * Handles parsing of variable declarations and destructuring patterns.
 *
 * GRAMMAR: See `EBNF.md` "Declarations" section
 *
 * @module parser/statements/variables
 */

import type { Resolver } from '../resolver';
import type {
    VariaDeclaration,
    ObjectPattern,
    ObjectPatternProperty,
    ArrayPattern,
    ArrayPatternElement,
    TypeAnnotation,
    Identifier,
    Expression,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// VARIABLE DECLARATION PARSING
// =============================================================================

/**
 * Parse variable declaration.
 *
 * GRAMMAR:
 *   variaDecl := ('varia' | 'fixum' | 'figendum' | 'variandum') typeAnnotation? (IDENTIFIER | arrayPattern) ('=' expression)?
 *
 * WHY: Four levels of mutability mirror Rust's approach:
 *      varia = fully mutable (let mut)
 *      fixum = immutable binding (let)
 *      figendum = compile-time constant (const)
 *      variandum = thread-local mutable
 *
 * Examples:
 *   varia numerus x = 42
 *   fixum textus nomen = "Faber"
 *   fixum [a, b, c] = coords
 *
 * NOT SUPPORTED (will produce parser errors):
 *   - JS spread: { ...rest }
 *   - Python unpack: { *rest } or { **rest }
 *   - TS-style annotation: fixum nomen: textus = "x" (use: fixum textus nomen = "x")
 *   - Brace object destructuring: fixum { a, b } = obj (use: ex obj fixum a, b)
 *   - Increment/decrement: x++, ++x, x--, --x
 *   - Compound assignment: x += 1, x -= 1, x *= 2, x /= 2
 */
export function parseVariaDeclaration(r: Resolver): VariaDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;
    const kind = ctx.peek().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';

    ctx.advance(); // varia, fixum, figendum, or variandum

    let typeAnnotation: TypeAnnotation | undefined;
    let name: Identifier | ArrayPattern;

    // Array destructuring pattern: fixum [a, b] = arr
    if (ctx.check('LBRACKET')) {
        name = parseArrayPattern(r);
    }
    else if (ctx.isTypeName(ctx.peek()) && ctx.peek(1).type !== 'EQUAL') {
        // Builtin type: fixum numerus x = 42
        // WHY: Check peek(1) !== '=' to allow type names as variable names.
        //      "fixum textus = x" means textus is the variable, not a type annotation.
        typeAnnotation = r.typeAnnotation();
        name = ctx.parseIdentifierOrKeyword();
    }
    else if (ctx.check('IDENTIFIER') && ctx.peek(1).type === 'IDENTIFIER') {
        // Custom type: fixum UserId id = 42
        // WHY: Two consecutive identifiers means first is type, second is name.
        // This handles user-defined types (typus aliases) without requiring
        // two-pass parsing or explicit type markers.
        typeAnnotation = r.typeAnnotation();
        name = ctx.parseIdentifierOrKeyword();
    }
    else {
        // WHY: Allow keywords as variable names for consistency with fields/params
        name = ctx.parseIdentifierOrKeyword();
    }

    let init: Expression | undefined;

    if (ctx.match('EQUAL')) {
        init = r.expression();
    }

    return { type: 'VariaDeclaration', kind, name, typeAnnotation, init, position };
}

// =============================================================================
// DESTRUCTURING PATTERN PARSING
// =============================================================================

/**
 * Parse object destructuring pattern.
 *
 * GRAMMAR:
 *   objectPattern := '{' patternProperty (',' patternProperty)* '}'
 *   patternProperty := 'ceteri'? IDENTIFIER (':' IDENTIFIER)?
 *
 * Used by two destructuring syntaxes:
 *   1. Direct assignment: fixum { nomen, aetas } = user
 *   2. Ex-prefix (Latin): ex user fixum { nomen, aetas }
 *
 * Examples:
 *   { nomen, aetas }              // extract nomen and aetas
 *   { nomen: localName, aetas }   // rename nomen to localName
 *   { nomen, ceteri rest }        // extract nomen, collect rest
 *
 * NOT SUPPORTED (will produce parser errors):
 *   { ...rest }    // JS spread syntax
 *   { *rest }      // Python unpack syntax
 *   { **rest }     // Python kwargs syntax
 */
export function parseObjectPattern(r: Resolver): ObjectPattern {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const properties: ObjectPatternProperty[] = [];

    // True while there are unparsed properties (not at '}' or EOF)
    const hasMoreProperties = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreProperties()) {
        const propPosition = ctx.peek().position;

        // Check for rest pattern: ceteri restName
        let rest = false;
        if (ctx.checkKeyword('ceteri')) {
            ctx.advance(); // consume 'ceteri'
            rest = true;
        }

        const key = ctx.parseIdentifierOrKeyword();

        let value = key;

        // Check for rename: { nomen: localName } (not valid with ceteri)
        if (ctx.match('COLON') && !rest) {
            value = ctx.parseIdentifierOrKeyword();
        }

        properties.push({
            type: 'ObjectPatternProperty',
            key,
            value,
            rest,
            position: propPosition,
        });

        if (!ctx.check('RBRACE')) {
            ctx.expect('COMMA', ParserErrorCode.ExpectedComma);
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'ObjectPattern', properties, position };
}

/**
 * Parse array destructuring pattern.
 *
 * GRAMMAR:
 *   arrayPattern := '[' arrayPatternElement (',' arrayPatternElement)* ']'
 *   arrayPatternElement := '_' | 'ceteri'? IDENTIFIER
 *
 * Used by two destructuring syntaxes:
 *   1. Direct assignment: fixum [a, b, c] = coords
 *   2. Ex-prefix (Latin): ex coords fixum [a, b, c]
 *
 * Examples:
 *   [a, b, c]                 // extract first three elements
 *   [first, ceteri rest]     // extract first, collect rest
 *   [_, second, _]           // skip first and third, extract second
 *
 * NOT SUPPORTED:
 *   [...rest]                // JS spread syntax
 *   [*rest]                  // Python unpack syntax
 */
export function parseArrayPattern(r: Resolver): ArrayPattern {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expect('LBRACKET', ParserErrorCode.ExpectedOpeningBracket);

    const elements: ArrayPatternElement[] = [];

    // True while there are unparsed elements (not at ']' or EOF)
    const hasMoreElements = () => !ctx.check('RBRACKET') && !ctx.isAtEnd();

    while (hasMoreElements()) {
        const elemPosition = ctx.peek().position;

        // Check for rest pattern: ceteri restName
        let rest = false;
        if (ctx.checkKeyword('ceteri')) {
            ctx.advance(); // consume 'ceteri'
            rest = true;
        }

        // Check for skip pattern: _
        if (!rest && ctx.check('IDENTIFIER') && ctx.peek().value === '_') {
            ctx.advance(); // consume '_'
            elements.push({
                type: 'ArrayPatternElement',
                name: { type: 'Identifier', name: '_', position: elemPosition },
                skip: true,
                position: elemPosition,
            });
        }
        else {
            // Regular binding or rest binding
            const name = ctx.parseIdentifier();
            elements.push({
                type: 'ArrayPatternElement',
                name,
                rest,
                position: elemPosition,
            });
        }

        if (!ctx.check('RBRACKET')) {
            ctx.expect('COMMA', ParserErrorCode.ExpectedComma);
        }
    }

    ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

    return { type: 'ArrayPattern', elements, position };
}
