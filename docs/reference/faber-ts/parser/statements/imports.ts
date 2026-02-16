/**
 * Import Statement Parsing
 *
 * Handles parsing of import declarations and specifiers.
 *
 * GRAMMAR: See `EBNF.md` "Imports" section
 *
 * @module parser/statements/imports
 */

import type { Resolver } from '../resolver';
import type { ImportaDeclaration, ImportSpecifier, Identifier } from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// IMPORT SPECIFIER PARSING
// =============================================================================

/**
 * Parse import specifier.
 *
 * GRAMMAR:
 *   specifier := 'ceteri'? IDENTIFIER ('ut' IDENTIFIER)?
 *
 * WHY: Shared between imports and destructuring.
 *      'ceteri' (rest) is only valid in destructuring contexts.
 *      'ut' provides aliasing: nomen ut n
 *
 * Examples:
 *   scribe             -> imported=scribe, local=scribe
 *   scribe ut s        -> imported=scribe, local=s
 *   ceteri rest        -> imported=rest, local=rest, rest=true
 */
export function parseSpecifier(r: Resolver): ImportSpecifier {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Check for rest pattern: ceteri restName
    let rest = false;
    if (ctx.checkKeyword('ceteri')) {
        ctx.advance(); // consume 'ceteri'
        rest = true;
    }

    // WHY: Names can be keywords (ex norma importa scribe)
    const imported = ctx.parseIdentifierOrKeyword();
    let local = imported;

    // Check for alias: name ut alias
    if (ctx.checkKeyword('ut')) {
        ctx.advance(); // consume 'ut'
        local = ctx.parseIdentifierOrKeyword();
    }

    return {
        type: 'ImportSpecifier',
        imported,
        local,
        rest: rest || undefined,
        position,
    };
}

// =============================================================================
// IMPORT DECLARATION PARSING
// =============================================================================

/**
 * Parse import declaration.
 *
 * GRAMMAR (new):
 *   importDecl := 'ex' (STRING | IDENTIFIER) (specifierList | '*')
 * GRAMMAR (legacy):
 *   importDecl := 'ex' (STRING | IDENTIFIER) 'importa' (specifierList | '*')
 *
 * The caller determines which syntax is used:
 *   - New syntax: § importa ex "path" bindings (importa already consumed)
 *   - Legacy syntax: § ex "path" importa bindings (starts with 'ex')
 *
 *   specifierList := specifier (',' specifier)*
 *   specifier := IDENTIFIER ('ut' IDENTIFIER)?
 *
 * Examples:
 *   ex norma importa scribe, lege
 *   ex norma importa scribe ut s, lege ut l
 *   ex "norma/tempus" importa nunc, dormi
 *   ex norma importa *
 */
export function parseImportaDeclaration(r: Resolver): ImportaDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ex', ParserErrorCode.ExpectedKeywordEx);

    // WHY: Accept both bare identifiers (ex norma) and strings (ex "norma/tempus")
    // String paths enable hierarchical module organization for stdlib
    let source: string;

    if (ctx.check('STRING')) {
        const sourceToken = ctx.advance();
        source = sourceToken.value;
    }
    else {
        const sourceToken = ctx.expect('IDENTIFIER', ParserErrorCode.ExpectedModuleName);
        source = sourceToken.value;
    }

    // Support both new syntax (importa already consumed) and legacy syntax (importa here)
    ctx.matchKeyword('importa');

    if (ctx.match('STAR')) {
        // Optional alias: ex "source" importa * ut alias
        let wildcardAlias: Identifier | undefined;
        if (ctx.matchKeyword('ut')) {
            wildcardAlias = ctx.parseIdentifier();
        }
        return { type: 'ImportaDeclaration', source, specifiers: [], wildcard: true, wildcardAlias, position };
    }

    const specifiers: ImportSpecifier[] = [];

    do {
        specifiers.push(parseSpecifier(r));
    } while (ctx.match('COMMA'));

    return { type: 'ImportaDeclaration', source, specifiers, wildcard: false, position };
}
