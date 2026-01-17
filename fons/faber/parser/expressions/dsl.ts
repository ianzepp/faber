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
    return ctx.checkKeyword('prima')
        || ctx.checkKeyword('ultima')
        || ctx.checkKeyword('summa')
        || ctx.checkKeyword('ordina')
        || ctx.checkKeyword('collige')
        || ctx.checkKeyword('grupa')
        || ctx.checkKeyword('maximum')
        || ctx.checkKeyword('minimum')
        || ctx.checkKeyword('medium')
        || ctx.checkKeyword('numera');
}

// =============================================================================
// DSL TRANSFORMS
// =============================================================================

/**
 * Parse collection DSL transforms.
 *
 * GRAMMAR:
 *   dslTransforms := dslTransform (',' dslTransform)*
 *   dslTransform := 'prima' expression
 *                 | 'ultima' expression
 *                 | 'summa' expression?
 *                 | 'ordina' 'per' expression ('ascendens' | 'descendens')?
 *                 | 'collige' expression
 *                 | 'grupa' 'per' expression
 *                 | 'maximum' | 'minimum' | 'medium' | 'numera'
 *
 * WHY: DSL provides concise syntax for common collection operations.
 *      Transforms chain with commas: ab items activus, ordina per nomen, prima 5
 *
 * Examples:
 *   prima 5                     -> first 5 elements
 *   ultima 3                    -> last 3 elements
 *   summa                       -> sum all elements
 *   summa pretium               -> sum by property
 *   ordina per nomen            -> sort by property ascending
 *   ordina per nomen descendens -> sort by property descending
 *   collige nomen               -> pluck/map to property
 *   grupa per categoria         -> group by property
 *   maximum                     -> max value
 *   minimum                     -> min value
 *   medium                      -> average
 *   numera                      -> count
 */
export function parseDSLTransforms(r: Resolver): CollectionDSLTransform[] {
    const ctx = r.ctx();
    const transforms: CollectionDSLTransform[] = [];

    while (isDSLVerb(r)) {
        const transformPos = ctx.peek().position;
        const verb = ctx.advance().keyword!;

        let argument: Expression | undefined;
        let property: Expression | undefined;
        let direction: 'ascendens' | 'descendens' | undefined;

        switch (verb) {
            case 'prima':
            case 'ultima':
                // These verbs require a count argument
                argument = r.expression();
                break;

            case 'summa':
                // summa optionally takes a property: summa pretium
                // Only parse if next token looks like a property (identifier, not comma/DSL verb)
                if (ctx.check('IDENTIFIER') && !isDSLVerb(r)) {
                    property = r.expression();
                }
                break;

            case 'ordina':
                // ordina per property [ascendens|descendens]
                ctx.expectKeyword('per', ParserErrorCode.UnexpectedToken);
                property = r.expression();
                if (ctx.matchKeyword('ascendens')) {
                    direction = 'ascendens';
                } else if (ctx.matchKeyword('descendens')) {
                    direction = 'descendens';
                }
                break;

            case 'collige':
                // collige property
                property = r.expression();
                break;

            case 'grupa':
                // grupa per property
                ctx.expectKeyword('per', ParserErrorCode.UnexpectedToken);
                property = r.expression();
                break;

            case 'maximum':
            case 'minimum':
            case 'medium':
            case 'numera':
                // These take no arguments
                break;
        }

        transforms.push({
            type: 'CollectionDSLTransform',
            verb,
            argument,
            property,
            direction,
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
