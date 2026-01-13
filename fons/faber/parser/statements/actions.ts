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
    IaceStatement,
    ScribeStatement,
    Expression,
    OutputLevel,
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
 *   outputStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
 *
 * WHY: Latin output keywords as statement forms:
 *   scribe (write!) -> console.log
 *   vide (see!)     -> console.debug
 *   mone (warn!)    -> console.warn
 *
 * Examples:
 *   scribe "hello"
 *   vide "debugging:", value
 *   mone "warning:", message
 */
export function parseScribeStatement(r: Resolver, level: OutputLevel): ScribeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Consume the keyword (already validated by checkKeyword in parseStatement)
    ctx.advance();

    const args: Expression[] = [];

    // Parse first argument (required)
    args.push(r.expression());

    // Parse additional comma-separated arguments
    while (ctx.match('COMMA')) {
        args.push(r.expression());
    }

    return { type: 'ScribeStatement', level, arguments: args, position };
}
