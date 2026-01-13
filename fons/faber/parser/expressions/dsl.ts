/**
 * DSL Expression Parsing
 *
 * Handles parsing of collection DSL expressions, ab expressions (filtering),
 * and regex literals.
 *
 * GRAMMAR: See `EBNF.md` "Expressions" section
 *
 * @module parser/expressions/dsl
 */

import type { Resolver } from '../resolver';
import type {
    Expression,
    CollectionDSLExpression,
    CollectionDSLTransform,
    AbExpression,
    RegexLiteral,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Check if current token is a DSL verb.
 *
 * WHY: DSL verbs are collection transform operations.
 */
export function isDSLVerb(r: Resolver): boolean {
    const ctx = r.ctx();
    return ctx.checkKeyword('prima') || ctx.checkKeyword('ultima') || ctx.checkKeyword('summa');
}

// =============================================================================
// DSL TRANSFORMS
// =============================================================================

/**
 * Parse collection DSL transforms.
 *
 * GRAMMAR:
 *   dslTransforms := dslTransform (',' dslTransform)*
 *   dslTransform := dslVerb expression?
 *   dslVerb := 'prima' | 'ultima' | 'summa'
 *
 * WHY: DSL provides concise syntax for common collection operations.
 *      Transforms chain with commas: prima 5, ultima 3
 *
 * Examples:
 *   prima 5           -> first 5 elements
 *   ultima 3          -> last 3 elements
 *   summa             -> sum (no argument)
 *   prima 5, ultima 2 -> first 5, then last 2 of those
 */
export function parseDSLTransforms(r: Resolver): CollectionDSLTransform[] {
    const ctx = r.ctx();
    const transforms: CollectionDSLTransform[] = [];

    while (isDSLVerb(r)) {
        const transformPos = ctx.peek().position;
        const verb = ctx.advance().keyword!;

        // Check if this verb takes an argument
        // prima and ultima require numeric argument
        // summa takes no argument
        let argument: Expression | undefined;
        if (verb === 'prima' || verb === 'ultima') {
            // These verbs require a numeric argument
            argument = r.expression();
        }
        // summa takes no argument

        transforms.push({
            type: 'CollectionDSLTransform',
            verb,
            argument,
            position: transformPos,
        });

        // Check for comma to continue chain
        if (!ctx.match('COMMA')) {
            break;
        }
    }

    return transforms;
}

// =============================================================================
// COLLECTION DSL EXPRESSION
// =============================================================================

/**
 * Parse collection DSL expression (expression context).
 *
 * GRAMMAR:
 *   dslExpr := 'ex' expression dslTransform (',' dslTransform)*
 *
 * WHY: When 'ex' appears in expression context with DSL verbs (not pro/fit/fiet),
 *      it creates a collection pipeline expression that can be assigned.
 *
 * Examples:
 *   fixum top5 = ex items prima 5
 *   fixum total = ex prices summa
 *   fixum result = ex items prima 10, ultima 3
 */
export function parseCollectionDSLExpression(r: Resolver): CollectionDSLExpression {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ex', ParserErrorCode.InvalidExDeStart);

    const source = r.expression();

    // Parse DSL transforms (at least one required for expression form)
    const transforms = parseDSLTransforms(r);

    if (transforms.length === 0) {
        // No transforms means this shouldn't be parsed as DSL expression
        // This case shouldn't happen if called correctly from parsePrimary
        ctx.error(ParserErrorCode.UnexpectedToken, 'expected DSL verb after ex');
    }

    return {
        type: 'CollectionDSLExpression',
        source,
        transforms,
        position,
    };
}

// =============================================================================
// AB EXPRESSION
// =============================================================================

/**
 * Parse 'ab' expression (collection filtering DSL).
 *
 * GRAMMAR:
 *   abExpr := 'ab' expression filter? (',' transform)*
 *   filter := ['non'] ('ubi' condition | identifier)
 *   condition := expression
 *
 * WHY: 'ab' (away from) is the dedicated DSL entry point for filtering.
 *      The 'ex' preposition remains unchanged for iteration/import/destructuring.
 *      Include/exclude is handled via 'non' keyword.
 *
 * Examples:
 *   ab users activus                     // boolean property shorthand
 *   ab users non banned                  // negated boolean property
 *   ab users ubi aetas >= 18             // condition with ubi
 *   ab users non ubi banned et suspended // negated compound condition
 *   ab users activus, prima 10           // filter + transforms
 *   ab users activus pro user { }        // iteration form
 */
export function parseAbExpression(r: Resolver): AbExpression {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ab', ParserErrorCode.UnexpectedToken);

    const source = r.expression();

    // Check for negation
    const negated = ctx.matchKeyword('non');

    // Check for filter (ubi or boolean property shorthand)
    let filter: AbExpression['filter'];

    if (ctx.matchKeyword('ubi')) {
        // Full condition: ab users ubi aetas >= 18
        const condition = r.expression();
        filter = { hasUbi: true, condition };
    }
    else if (ctx.check('IDENTIFIER') && !ctx.checkKeyword('pro') && !ctx.checkKeyword('fit') && !ctx.checkKeyword('fiet') && !isDSLVerb(r)) {
        // Boolean property shorthand: ab users activus
        // But only if it's not a binding keyword or DSL verb
        const propName = ctx.parseIdentifier();
        filter = { hasUbi: false, condition: propName };
    }

    // Parse optional transforms
    let transforms: CollectionDSLTransform[] | undefined;
    if (ctx.match('COMMA') || isDSLVerb(r)) {
        transforms = parseDSLTransforms(r);
    }

    return {
        type: 'AbExpression',
        source,
        negated,
        filter,
        transforms: transforms && transforms.length > 0 ? transforms : undefined,
        position,
    };
}

// =============================================================================
// REGEX LITERAL
// =============================================================================

/**
 * Parse regex literal expression.
 *
 * GRAMMAR:
 *   regexLiteral := 'sed' STRING IDENTIFIER?
 *
 * WHY: 'sed' (the Unix stream editor) is synonymous with pattern matching.
 *      The pattern string is passed through verbatim to the target.
 *      Flags are a bare identifier after the pattern (no comma).
 *
 * Examples:
 *   sed "\\d+"           // pattern only
 *   sed "hello" i        // case insensitive
 *   sed "^start" im      // multiple flags
 */
export function parseRegexLiteral(r: Resolver): RegexLiteral {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('sed', ParserErrorCode.UnexpectedToken);

    if (!ctx.check('STRING')) {
        ctx.error(ParserErrorCode.ExpectedStringAfterSed);
        // Return minimal node to continue parsing
        return { type: 'RegexLiteral', pattern: '', flags: '', position };
    }

    const patternToken = ctx.advance();
    const pattern = patternToken.value;

    // Parse optional flags identifier
    // WHY: Flags are bare identifier (no comma) to distinguish from next argument
    // Only match if it looks like flags (letters only, no comma before it)
    let flags = '';
    if (ctx.check('IDENTIFIER') && !ctx.check('COMMA')) {
        const flagsToken = ctx.peek();
        // Only consume if it looks like valid flags (letters only)
        if (/^[imsxu]+$/.test(flagsToken.value)) {
            ctx.advance();
            flags = flagsToken.value;
        }
    }

    return { type: 'RegexLiteral', pattern, flags, position };
}
