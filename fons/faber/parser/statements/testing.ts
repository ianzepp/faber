/**
 * Test Statement Parsing
 *
 * Handles parsing of test-related statements: test suites (probandum),
 * individual test cases (proba), and setup/teardown blocks (praepara/postpara).
 *
 * GRAMMAR: See `EBNF.md` "Testing" section
 *
 * LATIN VOCABULARY:
 * - probandum = to be tested (gerundive)
 * - proba = test (imperative)
 * - praepara = prepare (imperative - beforeEach)
 * - praeparabit = it will prepare (future - async beforeEach)
 * - postpara = prepare after (afterEach)
 * - postparabit = it will prepare after (async afterEach)
 * - omnia = all (beforeAll/afterAll marker)
 * - omitte = skip
 * - futurum = future/todo
 *
 * @module parser/statements/testing
 */

import type { Resolver } from '../resolver';
import type {
    ProbandumStatement,
    ProbaStatement,
    ProbaModifier,
    PraeparaBlock,
    PraeparaTiming,
} from '../ast';
import { ParserErrorCode } from '../errors';

// =============================================================================
// TEST SUITE STATEMENT
// =============================================================================

/**
 * Parse test suite declaration.
 *
 * GRAMMAR:
 *   probandumStmt := 'probandum' STRING '{' probandumBody '}'
 *   probandumBody := (praeparaBlock | probandumStmt | probaStmt)*
 *
 * WHY: Latin "probandum" (to be tested) for test suite declarations.
 *      Analogous to describe() in Jest/Vitest.
 *
 * Example:
 *   probandum "Tokenizer" {
 *       praepara { lexer = init() }
 *       proba "parses numbers" { ... }
 *   }
 */
export function parseProbandumStatement(r: Resolver): ProbandumStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('probandum', ParserErrorCode.ExpectedKeywordProbandum);

    // Parse suite name string
    const nameToken = ctx.expect('STRING', ParserErrorCode.ExpectedString);
    const name = nameToken.value;

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const body: (PraeparaBlock | ProbandumStatement | ProbaStatement)[] = [];

    // True while there are unparsed members (not at '}' or EOF)
    const hasMoreMembers = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreMembers()) {
        if (ctx.checkKeyword('probandum')) {
            body.push(parseProbandumStatement(r));
        }
        else if (ctx.checkKeyword('proba')) {
            body.push(parseProbaStatement(r));
        }
        else if (
            ctx.checkKeyword('praepara') ||
            ctx.checkKeyword('praeparabit') ||
            ctx.checkKeyword('postpara') ||
            ctx.checkKeyword('postparabit')
        ) {
            body.push(parsePraeparaBlock(r));
        }
        else {
            // Unknown token in probandum body
            ctx.reportError(ParserErrorCode.UnexpectedToken, `got '${ctx.peek().value}'`);
            ctx.advance(); // Skip to prevent infinite loop
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'ProbandumStatement', name, body, position };
}

// =============================================================================
// TEST CASE STATEMENT
// =============================================================================

/**
 * Parse individual test case.
 *
 * GRAMMAR:
 *   probaStmt := 'proba' probaModifier? STRING blockStmt
 *   probaModifier := 'omitte' STRING | 'futurum' STRING
 *
 * WHY: Latin "proba" (imperative of probare) = "test!" / "prove!".
 *      Analogous to test() or it() in Jest/Vitest.
 *
 * Examples:
 *   proba "parses integers" { adfirma parse("42") est 42 }
 *   proba omitte "blocked by #42" { ... }
 *   proba futurum "needs async support" { ... }
 */
export function parseProbaStatement(r: Resolver): ProbaStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('proba', ParserErrorCode.ExpectedKeywordProba);

    // Check for modifier: omitte or futurum
    let modifier: ProbaModifier | undefined;
    let modifierReason: string | undefined;

    if (ctx.matchKeyword('omitte')) {
        modifier = 'omitte';
        const reasonToken = ctx.expect('STRING', ParserErrorCode.ExpectedString);
        modifierReason = reasonToken.value;
    }
    else if (ctx.matchKeyword('futurum')) {
        modifier = 'futurum';
        const reasonToken = ctx.expect('STRING', ParserErrorCode.ExpectedString);
        modifierReason = reasonToken.value;
    }

    // Parse test name string
    const nameToken = ctx.expect('STRING', ParserErrorCode.ExpectedString);
    const name = nameToken.value;

    // Parse test body
    const body = r.block();

    return { type: 'ProbaStatement', name, modifier, modifierReason, body, position };
}

// =============================================================================
// SETUP/TEARDOWN BLOCK
// =============================================================================

/**
 * Parse praepara/postpara block (test setup-teardown).
 *
 * GRAMMAR:
 *   praeparaBlock := ('praepara' | 'praeparabit' | 'postpara' | 'postparabit') 'omnia'? blockStmt
 *
 * WHY: Latin "praepara" (prepare!) for test setup, "postpara" (cleanup!) for teardown.
 *      Uses -bit suffix for async (future tense), matching fit/fiet pattern.
 *
 * Examples:
 *   praepara { lexer = init() }
 *   praepara omnia { db = connect() }
 *   praeparabit omnia { db = cede connect() }
 *   postpara { cleanup() }
 *   postpara omnia { db.close() }
 *   postparabit omnia { cede db.close() }
 */
export function parsePraeparaBlock(r: Resolver): PraeparaBlock {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Determine timing and async from keyword
    let timing: PraeparaTiming;
    let async = false;

    if (ctx.matchKeyword('praepara')) {
        timing = 'praepara';
    }
    else if (ctx.matchKeyword('praeparabit')) {
        timing = 'praepara';
        async = true;
    }
    else if (ctx.matchKeyword('postpara')) {
        timing = 'postpara';
    }
    else if (ctx.matchKeyword('postparabit')) {
        timing = 'postpara';
        async = true;
    }
    else {
        // Should not reach here due to caller checks
        ctx.reportError(ParserErrorCode.UnexpectedToken, `expected praepara/postpara, got '${ctx.peek().value}'`);
        timing = 'praepara';
    }

    // Check for 'omnia' modifier
    const omnia = ctx.matchKeyword('omnia');

    const body = r.block();

    return { type: 'PraeparaBlock', timing, async, omnia, body, position };
}
