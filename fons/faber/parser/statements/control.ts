/**
 * Control Flow Statement Parsing
 *
 * Handles parsing of control flow statements: conditionals, loops, and pattern matching.
 *
 * GRAMMAR: See `EBNF.md` "Control Flow" section
 *
 * @module parser/statements/control
 */

import type { Resolver } from '../resolver';
import type {
    SiStatement,
    DumStatement,
    EligeStatement,
    EligeCasus,
    DiscerneStatement,
    VariantPattern,
    VariantCase,
    BlockStatement,
    ReddeStatement,
    IaceStatement,
    CapeClause,
    Expression,
    Identifier,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// HELPER: CATCH CLAUSE PARSING
// =============================================================================

/**
 * Parse catch clause.
 *
 * GRAMMAR:
 *   catchClause := 'cape' IDENTIFIER blockStmt
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
// HELPER: INLINE THROW/PANIC PARSING
// =============================================================================

/**
 * Try to parse an inline throw/panic statement (iacit/moritor).
 *
 * GRAMMAR:
 *   inlineThrow := ('iacit' | 'moritor') expression
 *
 * WHY: iacit/moritor are syntactic sugar for "ergo iace/mori" in case bodies.
 *      iacit = recoverable throw, moritor = fatal panic.
 */
function tryParseInlineThrow(r: Resolver): BlockStatement | null {
    const ctx = r.ctx();

    if (ctx.matchKeyword('iacit')) {
        const stmtPos = ctx.peek().position;
        const expr = r.expression();
        const throwStmt: IaceStatement = { type: 'IaceStatement', fatal: false, argument: expr, position: stmtPos };
        return { type: 'BlockStatement', body: [throwStmt], position: stmtPos };
    }

    if (ctx.matchKeyword('moritor')) {
        const stmtPos = ctx.peek().position;
        const expr = r.expression();
        const panicStmt: IaceStatement = { type: 'IaceStatement', fatal: true, argument: expr, position: stmtPos };
        return { type: 'BlockStatement', body: [panicStmt], position: stmtPos };
    }

    return null;
}

// =============================================================================
// IF STATEMENT (SI)
// =============================================================================

/**
 * Parse if statement.
 *
 * GRAMMAR:
 *   siStmt := 'si' expression consequent catchClause? alternate?
 *   consequent := blockStmt | 'ergo' statement | 'reddit' expression
 *   alternate := 'secus' (blockStmt | 'si' siStmt | statement | 'reddit' expression) | 'sin' siStmt
 *
 * INVARIANT: consequent is always wrapped in BlockStatement for consistency.
 * INVARIANT: "reddit" is shorthand for "ergo redde" (early return).
 *
 * WHY: Latin 'si' (if) for conditionals. 'sin' (but if) is classical else-if.
 *      Keywords are interchangeable at each branch point:
 *      - 'sin' ≡ 'sin' (else-if)
 *      - 'secus' ≡ 'secus' (else)
 *      - Mixed: si ... sin ... secus { } is valid
 *
 * Examples:
 *   si x > 5 ergo scribe("big")
 *   si x > 5 reddit verum            // early return
 *   si x > 5 { scribe("big") } secus scribe("small")
 *   si x < 0 { ... } sin x == 0 { ... } secus { ... }
 */
export function parseSiStatement(r: Resolver, skipSiKeyword = false): SiStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    if (!skipSiKeyword) {
        ctx.expectKeyword('si', ParserErrorCode.ExpectedKeywordSi);
    }

    const test = r.expression();

    // Parse consequent: block, ergo one-liner, reddit return, or iacit/moritor throw
    let consequent: BlockStatement;
    if (ctx.matchKeyword('reddit')) {
        // reddit = ergo redde (syntactic sugar for early return)
        const stmtPos = ctx.peek().position;
        const expr = r.expression();
        const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
        consequent = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
    }
    else {
        const inlineThrow = tryParseInlineThrow(r);
        if (inlineThrow) {
            consequent = inlineThrow;
        }
        else if (ctx.matchKeyword('ergo')) {
            const stmtPos = ctx.peek().position;
            const stmt = r.statement();
            consequent = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        }
        else {
            consequent = r.block();
        }
    }

    // Check for cape (catch) clause
    let catchClause: CapeClause | undefined;

    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    // Check for alternate: secus (else) or sin (else-if)
    let alternate: BlockStatement | SiStatement | undefined;

    if (ctx.matchKeyword('secus')) {
        if (ctx.checkKeyword('si')) {
            alternate = parseSiStatement(r);
        }
        else if (ctx.check('LBRACE')) {
            alternate = r.block();
        }
        else if (ctx.matchKeyword('reddit')) {
            // secus reddit expression (early return one-liner)
            const stmtPos = ctx.peek().position;
            const expr = r.expression();
            const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
            alternate = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
        }
        else {
            const inlineThrow = tryParseInlineThrow(r);
            if (inlineThrow) {
                alternate = inlineThrow;
            }
            else {
                // One-liner: secus statement (no ergo needed)
                const stmtPos = ctx.peek().position;
                const stmt = r.statement();
                alternate = { type: 'BlockStatement', body: [stmt], position: stmtPos };
            }
        }
    }
    else if (ctx.matchKeyword('sin')) {
        // "sin" (but if) is classical Latin for else-if
        alternate = parseSiStatement(r, true);
    }

    return { type: 'SiStatement', test, consequent, alternate, catchClause, position };
}

// =============================================================================
// WHILE STATEMENT (DUM)
// =============================================================================

/**
 * Parse while loop statement.
 *
 * GRAMMAR:
 *   whileStmt := 'dum' expression (blockStmt | 'ergo' statement | 'reddit' expression) ('cape' IDENTIFIER blockStmt)?
 *
 * WHY: 'dum' (while/until) for while loops.
 *
 * Examples:
 *   dum x > 0 { x = x - 1 }
 *   dum x > 0 ergo x = x - 1
 *   dum x > 0 reddit x
 */
export function parseDumStatement(r: Resolver): DumStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('dum', ParserErrorCode.ExpectedKeywordDum);

    const test = r.expression();

    // Parse body: block, ergo one-liner, reddit return, or iacit/moritor throw
    let body: BlockStatement;
    if (ctx.matchKeyword('reddit')) {
        const stmtPos = ctx.peek().position;
        const expr = r.expression();
        const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
        body = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
    }
    else {
        const inlineThrow = tryParseInlineThrow(r);
        if (inlineThrow) {
            body = inlineThrow;
        }
        else if (ctx.matchKeyword('ergo')) {
            const stmtPos = ctx.peek().position;
            const stmt = r.statement();
            body = { type: 'BlockStatement', body: [stmt], position: stmtPos };
        }
        else {
            body = r.block();
        }
    }

    let catchClause: CapeClause | undefined;

    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return { type: 'DumStatement', test, body, catchClause, position };
}

// =============================================================================
// SWITCH STATEMENT (ELIGE)
// =============================================================================

/**
 * Parse switch statement (value matching).
 *
 * GRAMMAR:
 *   eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
 *   eligeCase := 'casu' expression (blockStmt | 'ergo' statement | 'reddit' expression)
 *   defaultCase := 'ceterum' (blockStmt | statement)
 *
 * WHY: 'elige' (choose) for value-based switch.
 *      'ergo' (therefore) for one-liners, 'ceterum' (otherwise) for default.
 *      'reddit' (it returns) for early return one-liners.
 *      For variant matching on discretio types, use 'discerne' instead.
 *
 * Example:
 *   elige status {
 *       casu "pending" ergo scribe("waiting")
 *       casu "active" reddit verum
 *       ceterum iace "Unknown status"
 *   }
 */
export function parseEligeStatement(r: Resolver): EligeStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('elige', ParserErrorCode.ExpectedKeywordElige);

    const discriminant = r.expression();

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const cases: EligeCasus[] = [];
    let defaultCase: BlockStatement | undefined;

    // Helper: parse 'casu' case body (requires reddit, iacit, moritor, ergo, or block)
    function parseCasuBody(): BlockStatement {
        if (ctx.matchKeyword('reddit')) {
            const stmtPos = ctx.peek().position;
            const expr = r.expression();
            const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
            return { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
        }
        const inlineThrow = tryParseInlineThrow(r);
        if (inlineThrow) {
            return inlineThrow;
        }
        if (ctx.matchKeyword('ergo')) {
            const stmtPos = ctx.peek().position;
            const stmt = r.statement();
            return {
                type: 'BlockStatement',
                body: [stmt],
                position: stmtPos,
            };
        }
        return r.block();
    }

    // Helper: parse 'ceterum' body (block, reddit, iacit, moritor, or direct statement)
    function parseCeterumBody(): BlockStatement {
        if (ctx.check('LBRACE')) {
            return r.block();
        }
        if (ctx.matchKeyword('reddit')) {
            const stmtPos = ctx.peek().position;
            const expr = r.expression();
            const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
            return { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
        }
        const inlineThrow = tryParseInlineThrow(r);
        if (inlineThrow) {
            return inlineThrow;
        }
        const stmtPos = ctx.peek().position;
        const stmt = r.statement();
        return {
            type: 'BlockStatement',
            body: [stmt],
            position: stmtPos,
        };
    }

    // True while there are unparsed cases (not at '}' or EOF)
    const hasMoreCases = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreCases()) {
        if (ctx.checkKeyword('casu')) {
            // Value case: casu expression { ... }
            const casePosition = ctx.peek().position;

            ctx.expectKeyword('casu', ParserErrorCode.ExpectedKeywordCasu);

            const test = r.expression();
            const consequent = parseCasuBody();

            cases.push({ type: 'EligeCasus', test, consequent, position: casePosition });
        }
        else if (ctx.checkKeyword('ceterum')) {
            ctx.advance(); // consume ceterum

            defaultCase = parseCeterumBody();
            break; // Default must be last
        }
        else {
            ctx.reportError(ParserErrorCode.InvalidEligeCaseStart);
            break;
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    let catchClause: CapeClause | undefined;

    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return { type: 'EligeStatement', discriminant, cases, defaultCase, catchClause, position };
}

// =============================================================================
// PATTERN MATCHING STATEMENT (DISCERNE)
// =============================================================================

/**
 * Parse variant matching statement (for discretio types).
 *
 * GRAMMAR:
 *   discerneStmt := 'discerne' discriminants '{' variantCase* '}'
 *   discriminants := expression (',' expression)*
 *   variantCase := 'casu' patterns (blockStmt | 'ergo' statement | 'reddit' expression)
 *   patterns := pattern (',' pattern)*
 *   pattern := '_' | (IDENTIFIER patternBind?)
 *   patternBind := ('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*)
 *
 * WHY: 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
 *      Uses 'casu' for match arms, 'ut' to bind whole variants, and 'pro' for positional bindings.
 *      Multi-discriminant matching reduces nesting when comparing multiple values.
 *
 * Examples:
 *   # Single discriminant
 *   discerne event {
 *       casu Click pro x, y { scribe "clicked at " + x + ", " + y }
 *       casu Keypress pro key reddit key
 *       casu Quit ergo mori "goodbye"
 *   }
 *
 *   # Multi-discriminant
 *   discerne left, right {
 *       casu Primitivum ut l, Primitivum ut r { redde l.nomen == r.nomen }
 *       casu _, _ { redde falsum }
 *   }
 */
export function parseDiscerneStatement(r: Resolver): DiscerneStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('discerne', ParserErrorCode.ExpectedKeywordDiscerne);

    // Parse comma-separated discriminants
    const discriminants: Expression[] = [];
    discriminants.push(r.expression());
    while (ctx.match('COMMA')) {
        discriminants.push(r.expression());
    }

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const cases: VariantCase[] = [];
    let defaultCase: BlockStatement | undefined;

    // Helper: parse 'ceterum' body (block, reddit, iacit, moritor, or direct statement)
    function parseCeterumBody(): BlockStatement {
        if (ctx.check('LBRACE')) {
            return r.block();
        }
        if (ctx.matchKeyword('reddit')) {
            const stmtPos = ctx.peek().position;
            const expr = r.expression();
            const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
            return { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
        }
        const inlineThrow = tryParseInlineThrow(r);
        if (inlineThrow) {
            return inlineThrow;
        }
        const stmtPos = ctx.peek().position;
        const stmt = r.statement();
        return {
            type: 'BlockStatement',
            body: [stmt],
            position: stmtPos,
        };
    }

    // True while there are unparsed cases (not at '}' or EOF)
    const hasMoreCases = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreCases()) {
        if (ctx.checkKeyword('casu')) {
            const casePosition = ctx.peek().position;
            ctx.expectKeyword('casu', ParserErrorCode.ExpectedKeywordCasu);

            // Parse comma-separated patterns (one per discriminant)
            const patterns: VariantPattern[] = [];
            patterns.push(parseVariantPattern(r));
            while (ctx.match('COMMA')) {
                patterns.push(parseVariantPattern(r));
            }

            // Parse consequent: reddit, iacit, moritor, ergo, or block
            let consequent: BlockStatement;
            if (ctx.matchKeyword('reddit')) {
                const stmtPos = ctx.peek().position;
                const expr = r.expression();
                const returnStmt: ReddeStatement = { type: 'ReddeStatement', argument: expr, position: stmtPos };
                consequent = { type: 'BlockStatement', body: [returnStmt], position: stmtPos };
            }
            else {
                const inlineThrow = tryParseInlineThrow(r);
                if (inlineThrow) {
                    consequent = inlineThrow;
                }
                else if (ctx.matchKeyword('ergo')) {
                    const stmtPos = ctx.peek().position;
                    const stmt = r.statement();
                    consequent = { type: 'BlockStatement', body: [stmt], position: stmtPos };
                }
                else {
                    consequent = r.block();
                }
            }

            cases.push({ type: 'VariantCase', patterns, consequent, position: casePosition });
        }
        else if (ctx.checkKeyword('ceterum')) {
            ctx.advance(); // consume ceterum
            defaultCase = parseCeterumBody();
            break; // Default must be last
        }
        else {
            ctx.reportError(ParserErrorCode.InvalidDiscerneCaseStart);
            break;
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'DiscerneStatement', discriminants, cases, defaultCase, position };
}

// =============================================================================
// VARIANT PATTERN PARSING
// =============================================================================

/**
 * Parse a single variant pattern.
 *
 * GRAMMAR:
 *   pattern := '_' | (IDENTIFIER patternBind?)
 *   patternBind := ('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*)
 *
 * WHY: Patterns match against discriminants in discerne statements.
 *      Wildcard '_' matches any variant without binding.
 *      'ut' binds the whole variant, 'pro' destructures fields.
 *
 * DISAMBIGUATION: After 'pro', commas separate bindings until we see:
 *   - '_' (wildcard pattern)
 *   - An identifier followed by 'ut' or 'pro' (new pattern with binding)
 *   - '{', 'ergo', 'reddit' (end of patterns)
 */
export function parseVariantPattern(r: Resolver): VariantPattern {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Wildcard pattern: _
    if (ctx.check('IDENTIFIER') && ctx.peek().value === '_') {
        const variant: Identifier = { type: 'Identifier', name: '_', position };
        ctx.advance();
        return { type: 'VariantPattern', variant, isWildcard: true, bindings: [], position };
    }

    // Variant name
    const variant = ctx.parseIdentifierOrKeyword();
    let alias: Identifier | undefined;
    const bindings: Identifier[] = [];

    if (ctx.matchKeyword('ut')) {
        // Alias binding: Click ut c
        alias = ctx.parseIdentifierOrKeyword();
    }
    else if (ctx.matchKeyword('pro')) {
        // Positional bindings: Click pro x, y
        bindings.push(ctx.parseIdentifierOrKeyword());

        // Continue parsing bindings while comma followed by binding (not pattern)
        while (ctx.check('COMMA') && isNextTokenBinding(ctx)) {
            ctx.advance(); // consume comma
            bindings.push(ctx.parseIdentifierOrKeyword());
        }
    }

    return { type: 'VariantPattern', variant, isWildcard: false, alias, bindings, position };
}

/**
 * Check if the token after a comma is a binding (not a new pattern).
 *
 * WHY: In multi-discriminant patterns, commas separate both:
 *   - Bindings within 'pro': `casu Click pro x, y { ... }`
 *   - Patterns: `casu Click ut c, Quit { ... }`
 *
 * RULE: After comma, it's a new pattern ONLY if:
 *   - Next token is '_' (wildcard)
 *   - Next token is identifier followed by 'ut' or 'pro' (pattern with binding)
 * Otherwise it's another binding.
 *
 * NOTE: This means `casu X pro a, b {` parses as X with bindings [a, b].
 *       To have multiple patterns, use explicit binding syntax: `casu X pro a, Y ut y {`.
 */
function isNextTokenBinding(ctx: ReturnType<Resolver['ctx']>): boolean {
    // Must be at comma
    if (!ctx.check('COMMA')) return false;

    const next = ctx.peek(1);

    // Comma followed by '_' is next pattern (wildcard)
    if (next.type === 'IDENTIFIER' && next.value === '_') return false;

    // Comma followed by non-identifier is not a binding (syntax error will follow)
    if (next.type !== 'IDENTIFIER' && next.type !== 'KEYWORD') return false;

    // Look at what follows the identifier
    const afterIdent = ctx.peek(2);

    // If followed by 'ut' or 'pro', it's a new pattern with binding
    if (afterIdent.type === 'KEYWORD' && (afterIdent.keyword === 'ut' || afterIdent.keyword === 'pro')) {
        return false;
    }

    // Otherwise assume it's a binding (including when followed by ',', '{', 'ergo', 'reddit')
    // The semantic analyzer will validate pattern count vs discriminant count
    return true;
}
