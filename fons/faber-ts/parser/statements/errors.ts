/**
 * Error Handling Statement Parsing
 *
 * Handles parsing of error handling constructs: try-catch (tempta-cape),
 * catch clauses, and guard statements (custodi).
 *
 * GRAMMAR: See `EBNF.md` "Statements" section
 *
 * @module parser/statements/errors
 */

import type { Resolver } from '../resolver';
import type {
    CapeClause,
    CustodiStatement,
    CustodiClause,
    TemptaStatement,
    BlockStatement,
    ReddeStatement,
    IaceStatement,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// TRY-CATCH STATEMENT
// =============================================================================

/**
 * Parse try-catch statement.
 *
 * GRAMMAR:
 *   tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
 *
 * WHY: 'tempta' (attempt/try), 'cape' (catch/seize), 'demum' (finally/at last).
 */
export function parseTemptaStatement(r: Resolver): TemptaStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('tempta', ParserErrorCode.ExpectedKeywordTempta);

    const block = r.block();

    let handler: CapeClause | undefined;

    if (ctx.checkKeyword('cape')) {
        handler = parseCapeClause(r);
    }

    let finalizer: BlockStatement | undefined;

    if (ctx.matchKeyword('demum')) {
        finalizer = r.block();
    }

    return { type: 'TemptaStatement', block, handler, finalizer, position };
}

// =============================================================================
// CATCH CLAUSE
// =============================================================================

/**
 * Parse catch clause.
 *
 * GRAMMAR:
 *   catchClause := 'cape' IDENTIFIER blockStmt
 */
export function parseCapeClause(r: Resolver): CapeClause {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('cape', ParserErrorCode.ExpectedKeywordCape);

    const param = ctx.parseIdentifierOrKeyword();
    const body = r.block();

    return { type: 'CapeClause', param, body, position };
}

// =============================================================================
// GUARD STATEMENT
// =============================================================================

/**
 * Parse guard statement.
 *
 * GRAMMAR:
 *   custodiStmt := 'custodi' '{' (custodiClause)* '}'
 *   custodiClause := 'si' expression ('reddit' expression | 'iacit' expression | 'moritor' expression | 'ergo' statement | blockStmt)
 *
 * WHY: 'custodi' (guard) provides early-exit checks with multiple clauses.
 *      Each clause tests a condition and executes consequent on match.
 *
 * Example:
 *   custodi {
 *       si user == nihil reddit nihil
 *       si user.age < 0 iacit "Invalid age"
 *       si user.name == "" { redde defaultUser() }
 *   }
 */
export function parseCustodiStatement(r: Resolver): CustodiStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('custodi', ParserErrorCode.ExpectedKeywordCustodi);

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const clauses: CustodiClause[] = [];

    // True while there are unparsed clauses (not at '}' or EOF)
    const hasMoreClauses = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreClauses()) {
        if (ctx.checkKeyword('si')) {
            const clausePosition = ctx.peek().position;

            ctx.expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);

            const test = r.expression();

            // Parse consequent: reddit, iacit, moritor, ergo, or block
            let consequent: BlockStatement;
            if (ctx.matchKeyword('reddit')) {
                const stmtPos = ctx.peek().position;
                const expr = r.expression();
                const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
                consequent = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
            }
            else if (ctx.matchKeyword('iacit')) {
                const stmtPos = ctx.peek().position;
                const expr = r.expression();
                const throwStmt: IaceStatement = { type: 'IaceStatement', fatal: false, argument: expr, position: stmtPos };
                consequent = { type: 'BlockStatement', body: [throwStmt], position: stmtPos };
            }
            else if (ctx.matchKeyword('moritor')) {
                const stmtPos = ctx.peek().position;
                const expr = r.expression();
                const panicStmt: IaceStatement = { type: 'IaceStatement', fatal: true, argument: expr, position: stmtPos };
                consequent = { type: 'BlockStatement', body: [panicStmt], position: stmtPos };
            }
            else if (ctx.matchKeyword('ergo')) {
                const stmtPos = ctx.peek().position;
                const stmt = r.statement();
                consequent = { type: 'BlockStatement', body: [stmt], position: stmtPos };
            }
            else {
                consequent = r.block();
            }

            clauses.push({ type: 'CustodiClause', test, consequent, position: clausePosition });
        }
        else {
            ctx.error(ParserErrorCode.InvalidCustodiClauseStart);
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'CustodiStatement', clauses, position };
}
