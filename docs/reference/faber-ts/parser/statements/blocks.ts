/**
 * Block Statement Parsing
 *
 * Handles parsing of block statements, fac blocks, and expression statements.
 *
 * GRAMMAR: See `EBNF.md` "Statements" section
 *
 * @module parser/statements/blocks
 */

import type { Resolver } from '../resolver';
import type {
    BlockStatement,
    FacBlockStatement,
    ExpressionStatement,
    CapeClause,
    Statement,
    Expression,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Parse cape (catch) clause for fac blocks.
 *
 * GRAMMAR:
 *   capeClause := 'cape' IDENTIFIER blockStmt
 *
 * WHY: 'cape' (catch) provides error handling within fac blocks.
 */
function parseCapeClause(r: Resolver): CapeClause {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('cape', ParserErrorCode.ExpectedKeywordCape);

    const param = ctx.parseIdentifierOrKeyword();
    const body = r.block();

    return { type: 'CapeClause', param, body, position };
}

// =============================================================================
// BLOCK STATEMENT PARSING
// =============================================================================

/**
 * Parse block statement.
 *
 * GRAMMAR:
 *   blockStmt := '{' statement* '}'
 */
export function parseBlockStatement(r: Resolver): BlockStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const body: Statement[] = [];

    // True while there are unparsed statements (not at '}' or EOF)
    const hasMoreStatements = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreStatements()) {
        // Consume optional semicolons between statements
        while (ctx.match('SEMICOLON')) {
            // do nothing - avoids linter no-empty
        }

        if (!hasMoreStatements()) {
            break;
        }

        body.push(r.statement());
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'BlockStatement', body, position };
}

/**
 * Parse fac block statement (explicit scope block or do-while loop).
 *
 * GRAMMAR:
 *   facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
 *
 * WHY: 'fac' (do/make) creates an explicit scope boundary for grouping
 *      statements with optional error handling via 'cape' (catch).
 *      When followed by 'dum', creates a do-while loop where the body
 *      executes at least once before the condition is checked.
 *      Useful for:
 *      - Scoped variable declarations
 *      - Grouping related operations with shared error handling
 *      - Creating IIFE-like constructs
 *      - Do-while loops (body executes first, then condition checked)
 *
 * Examples:
 *   fac { fixum x = computeValue() }
 *   fac { riskyOperation() } cape e { scribe e }
 *   fac { process() } dum hasMore()
 *   fac { process() } cape e { log(e) } dum hasMore()
 */
export function parseFacBlockStatement(r: Resolver): FacBlockStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('fac', ParserErrorCode.ExpectedKeywordFac);

    const body = r.block();

    let catchClause: CapeClause | undefined;
    let test: Expression | undefined;

    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    if (ctx.matchKeyword('dum')) {
        test = r.expression();
    }

    return { type: 'FacBlockStatement', body, catchClause, test, position };
}

// =============================================================================
// EXPRESSION STATEMENT PARSING
// =============================================================================

/**
 * Parse expression statement.
 *
 * GRAMMAR:
 *   exprStmt := expression
 */
export function parseExpressionStatement(r: Resolver): ExpressionStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;
    const expression = r.expression();

    return { type: 'ExpressionStatement', expression, position };
}
