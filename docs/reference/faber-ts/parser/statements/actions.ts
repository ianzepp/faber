/**
 * Action Statement Parsing
 *
 * Handles parsing of simple action statements: assertions, returns, breaks,
 * continues, throws, and output statements.
 *
 * GRAMMAR: See `EBNF.md` "Statements" section
 *
 * @module parser/statements/actions
 */

import type { Resolver } from '../resolver';
import type {
    AdfirmaStatement,
    ReddeStatement,
    RumpeStatement,
    PergeStatement,
    TacetStatement,
    IaceStatement,
    ScribeStatement,
    Expression,
    OutputLevel,
    ScriptumExpression,
    Literal,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// ASSERTION STATEMENT
// =============================================================================

/**
 * Parse assertion statement.
 *
 * GRAMMAR:
 *   assertStmt := 'adfirma' expression (',' expression)?
 *
 * WHY: 'adfirma' (affirm/assert) for runtime invariant checks.
 *
 * Example:
 *   adfirma x > 0
 *   adfirma x > 0, "x must be positive"
 */
export function parseAdfirmaStatement(r: Resolver): AdfirmaStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('adfirma', ParserErrorCode.ExpectedKeywordAdfirma);

    const test = r.expression();

    let message: Expression | undefined;

    if (ctx.match('COMMA')) {
        message = r.expression();
    }

    return { type: 'AdfirmaStatement', test, message, position };
}

// =============================================================================
// RETURN STATEMENT
// =============================================================================

/**
 * Parse return statement.
 *
 * GRAMMAR:
 *   returnStmt := 'redde' expression?
 *
 * WHY: 'redde' (give back/return) for return statements.
 */
export function parseReddeStatement(r: Resolver): ReddeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('redde', ParserErrorCode.ExpectedKeywordRedde);

    let argument: Expression | undefined;

    if (!ctx.check('RBRACE') && !ctx.isAtEnd()) {
        argument = r.expression();
    }

    return { type: 'ReddeStatement', argument, position };
}

// =============================================================================
// BREAK STATEMENT
// =============================================================================

/**
 * Parse break statement.
 *
 * GRAMMAR:
 *   breakStmt := 'rumpe'
 *
 * WHY: 'rumpe' (break!) exits the innermost loop.
 */
export function parseRumpeStatement(r: Resolver): RumpeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the 'rumpe' keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    return { type: 'RumpeStatement', position };
}

// =============================================================================
// CONTINUE STATEMENT
// =============================================================================

/**
 * Parse continue statement.
 *
 * GRAMMAR:
 *   continueStmt := 'perge'
 *
 * WHY: 'perge' (continue/proceed!) skips to the next loop iteration.
 */
export function parsePergeStatement(r: Resolver): PergeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the 'perge' keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    return { type: 'PergeStatement', position };
}

// =============================================================================
// NO-OP STATEMENT
// =============================================================================

/**
 * Parse no-op statement.
 *
 * GRAMMAR:
 *   tacetStmt := 'tacet'
 *
 * WHY: 'tacet' (it is silent) for explicit empty blocks.
 *      From musical notation - makes intentional emptiness explicit.
 */
export function parseTacetStatement(r: Resolver): TacetStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the 'tacet' keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    return { type: 'TacetStatement', position };
}

// =============================================================================
// THROW STATEMENT
// =============================================================================

/**
 * Parse throw/panic statement.
 *
 * GRAMMAR:
 *   throwStmt := ('iace' | 'mori') expression
 *
 * WHY: Two error severity levels:
 *   iace (throw!) -> recoverable, can be caught
 *   mori (die!)   -> fatal/panic, unrecoverable
 */
export function parseIaceStatement(r: Resolver, fatal: boolean): IaceStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    const argument = r.expression();

    return { type: 'IaceStatement', fatal, argument, position };
}

// =============================================================================
// OUTPUT STATEMENT
// =============================================================================

/**
 * Parse output statement (scribe/vide/mone).
 *
 * GRAMMAR:
 *   outputStmt := ('scribe' | 'vide' | 'mone') scribeItem (',' scribeItem)*
 *   scribeItem := STRING expression* | expression
 *
 * WHY: Latin output keywords as statement forms:
 *   scribe (write!) -> console.log
 *   vide (see!)     -> console.debug
 *   mone (warn!)    -> console.warn
 *
 * STRING DESUGARING: String literals are automatically wrapped in scriptum expressions.
 * The § placeholder count determines how many following arguments are consumed:
 *   scribe "Found § errors", count  ->  scribe scriptum("Found § errors", count)
 *   scribe "hello"                  ->  scribe scriptum("hello")
 *
 * Examples:
 *   scribe "hello"
 *   scribe "Found § errors in §", errorCount, fileName
 *   vide "debugging:", value
 *   mone "warning:", message
 */
export function parseScribeStatement(r: Resolver, level: OutputLevel): ScribeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    const args: Expression[] = [];

    // Parse first item (required)
    args.push(parseScribeItem(r));

    // Parse additional comma-separated items
    while (ctx.match('COMMA')) {
        args.push(parseScribeItem(r));
    }

    return { type: 'ScribeStatement', level, arguments: args, position };
}

/**
 * Parse a single scribe item, converting string literals to scriptum expressions.
 *
 * WHY: Syntactic sugar - string literals in scribe statements are implicitly
 * wrapped in scriptum() for consistent formatting semantics. The § placeholder
 * count determines how many following comma-separated arguments are consumed.
 */
function parseScribeItem(r: Resolver): Expression {
    const ctx = r.ctx();

    // String literals become scriptum expressions
    if (ctx.check('STRING')) {
        const token = ctx.advance();
        const format: Literal = {
            type: 'Literal',
            value: token.value,
            raw: `"${token.value}"`,
            position: token.position,
        };

        // Count § placeholders to determine how many args to consume
        const placeholderCount = (token.value.match(/§/g) || []).length;
        const scriptumArgs: Expression[] = [];

        for (let i = 0; i < placeholderCount; i++) {
            if (!ctx.match('COMMA')) {
                ctx.error(
                    ParserErrorCode.GenericError,
                    `Expected argument for § placeholder (need ${placeholderCount}, got ${i})`
                );
            }
            scriptumArgs.push(r.expression());
        }

        return {
            type: 'ScriptumExpression',
            format,
            arguments: scriptumArgs,
            position: token.position,
        } as ScriptumExpression;
    }

    // Non-string expressions are parsed normally
    return r.expression();
}
