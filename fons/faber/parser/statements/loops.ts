/**
 * Loop Statement Parsing
 *
 * Handles parsing of iteration statements (ex, de, in) and related constructs
 * (cura resource management, ad dispatch, incipit/incipiet entry points).
 *
 * GRAMMAR: See `EBNF.md` "Iteration" section
 *
 * @module parser/statements/loops
 */

import type { Resolver } from '../resolver';
import type {
    IteratioStatement,
    InStatement,
    CuraStatement,
    CuratorKind,
    AdStatement,
    AdBinding,
    AdBindingVerb,
    IncipitStatement,
    IncipietStatement,
    CollectionDSLTransform,
    BlockStatement,
    CapeClause,
    TypeAnnotation,
    Expression,
    SpreadElement,
    Identifier,
    VariaDeclaration,
    DestructureDeclaration,
    ImportSpecifier,
    ExitusModifier,
    Literal,
} from '../ast';
import { ParserErrorCode } from '../errors';
import { parseSpecifier } from './imports';
import { parseBody } from './control';

// =============================================================================
// HELPERS
// =============================================================================

/**
 * Check if current token is a DSL verb.
 *
 * WHY: DSL verbs are collection transform operations.
 */
function isDSLVerb(r: Resolver): boolean {
    const ctx = r.ctx();
    return ctx.checkKeyword('prima') || ctx.checkKeyword('ultima') || ctx.checkKeyword('summa');
}

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
function parseDSLTransforms(r: Resolver): CollectionDSLTransform[] {
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

/**
 * Parse array pattern for destructuring.
 *
 * WHY: Reused from main parser for ex statement array destructuring.
 */
function parseArrayPattern(r: Resolver): import('../ast').ArrayPattern {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expect('LBRACKET', ParserErrorCode.ExpectedOpeningBracket);

    const elements: import('../ast').ArrayPatternElement[] = [];

    while (!ctx.check('RBRACKET') && !ctx.isAtEnd()) {
        const elemPos = ctx.peek().position;

        // Check for rest element: ceteri rest
        let rest = false;
        if (ctx.checkKeyword('ceteri')) {
            ctx.advance();
            rest = true;
        }

        const name = ctx.parseIdentifierOrKeyword();
        elements.push({ type: 'ArrayPatternElement', name, rest: rest || undefined, position: elemPos });

        if (!ctx.match('COMMA')) {
            break;
        }
    }

    ctx.expect('RBRACKET', ParserErrorCode.ExpectedClosingBracket);

    return { type: 'ArrayPattern', elements, position };
}

/**
 * Parse catch clause (cape).
 *
 * WHY: Reused for all statements that support catch clauses.
 */
function parseCapeClause(r: Resolver): CapeClause {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('cape', ParserErrorCode.ExpectedKeywordCape);

    const param = ctx.parseIdentifier();
    const body = r.block();

    return { type: 'CapeClause', param, body, position };
}

// =============================================================================
// EX STATEMENT (FOR-OF ITERATION)
// =============================================================================

/**
 * Parse 'ex' statement (for-of loop or destructuring).
 *
 * GRAMMAR:
 *   exStmt := 'ex' expression
 *             (destructure | iteration)
 *   destructure := ('fixum' | 'varia' | 'figendum' | 'variandum')
 *                  ('[' pattern ']' | specifiers)
 *   iteration := dslTransforms? ('fixum' | 'varia') IDENTIFIER
 *                (blockStmt | 'ergo' statement | 'reddit' expression) catchClause?
 *
 * WHY: 'ex' (from/out of) for extracting values from collections.
 *      Semantically read-only - contrasts with 'in' for mutation.
 *      Supports both iteration and destructuring patterns.
 *      Async iteration (figendum/variandum) deferred pending backend support.
 *
 * Collection DSL forms:
 *   ex items prima 5 fixum item { }        // iteration with transforms
 *   ex items prima 5, ultima 2 fixum x {}  // multiple transforms
 */
export function parseExStatement(r: Resolver): IteratioStatement | VariaDeclaration | DestructureDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ex', ParserErrorCode.InvalidExDeStart);

    const source = r.expression();

    // Dispatch based on what follows the expression
    if (ctx.checkKeyword('fixum') || ctx.checkKeyword('varia') || ctx.checkKeyword('figendum') || ctx.checkKeyword('variandum')) {
        const kind = ctx.advance().keyword as 'varia' | 'fixum' | 'figendum' | 'variandum';

        // Array destructuring: ex coords fixum [x, y, z] or for-of with destructure: ex map fixum [k, v] { }
        if (ctx.check('LBRACKET')) {
            const pattern = parseArrayPattern(r);

            // Check for block body -> for-of loop with destructuring
            if (ctx.check('LBRACE')) {
                const body = r.block();
                let catchClause: CapeClause | undefined;
                if (ctx.checkKeyword('cape')) {
                    catchClause = parseCapeClause(r);
                }
                const async = kind === 'figendum' || kind === 'variandum';
                const mutable = kind === 'varia' || kind === 'variandum';
                return {
                    type: 'IteratioStatement',
                    kind: 'ex',
                    variable: pattern,
                    iterable: source,
                    body,
                    async,
                    mutable,
                    catchClause,
                    position,
                };
            }

            // No block -> one-shot destructuring
            return { type: 'VariaDeclaration', kind, name: pattern, init: source, position };
        }

        // Object destructuring: ex persona fixum nomen, aetas
        // WHY: Brace-less syntax matches import pattern: ex norma importa scribe, lege
        const specifiers: ImportSpecifier[] = [];
        do {
            specifiers.push(parseSpecifier(r));
        } while (ctx.match('COMMA'));

        return { type: 'DestructureDeclaration', source, kind, specifiers, position };
    }

    // Check for DSL transforms before binding keyword
    const transforms = parseDSLTransforms(r);

    // Now expect for-loop binding: ex source [transforms] fixum/varia variable { }
    let mutable = false;

    if (ctx.matchKeyword('varia')) {
        mutable = true;
    }
    else {
        ctx.expectKeyword('fixum', ParserErrorCode.ExpectedKeywordFixum);
    }

    const variable = ctx.parseIdentifierOrKeyword();
    const body = parseBody(r);

    let catchClause: CapeClause | undefined;
    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return {
        type: 'IteratioStatement',
        kind: 'ex',
        variable,
        iterable: source,
        body,
        async: false,
        mutable,
        catchClause,
        transforms: transforms.length > 0 ? transforms : undefined,
        position,
    };
}

// =============================================================================
// DE STATEMENT (FOR-IN ITERATION)
// =============================================================================

/**
 * Parse 'de' statement (for-in loop).
 *
 * GRAMMAR:
 *   deStmt := 'de' expression ('fixum' | 'varia') IDENTIFIER
 *             (blockStmt | 'ergo' statement | 'reddit' expression) catchClause?
 *
 * WHY: 'de' (from/concerning) for extracting keys from an object.
 *      Semantically read-only - contrasts with 'in' for mutation.
 *
 * Examples:
 *   de tabula fixum clavis { ... }  // from table, for each key
 *   de object fixum k ergo scribe k // one-liner form
 *   de object fixum k reddit k      // return first key
 */
export function parseDeStatement(r: Resolver): IteratioStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('de', ParserErrorCode.ExpectedKeywordDe);

    const expr = r.expression();

    // Require binding keyword: fixum (const) or varia (let)
    let mutable = false;

    if (ctx.matchKeyword('varia')) {
        mutable = true;
    }
    else {
        ctx.expectKeyword('fixum', ParserErrorCode.ExpectedKeywordFixum);
    }

    const variable = ctx.parseIdentifierOrKeyword();
    const body = parseBody(r);

    let catchClause: CapeClause | undefined;
    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return {
        type: 'IteratioStatement',
        kind: 'in', // Still 'in' kind for codegen (for-in loop)
        variable,
        iterable: expr,
        body,
        async: false,
        mutable,
        catchClause,
        position,
    };
}

// =============================================================================
// IN STATEMENT (MUTATION BLOCK)
// =============================================================================

/**
 * Parse 'in' statement (mutation block).
 *
 * GRAMMAR:
 *   inStmt := 'in' expression blockStmt
 *
 * WHY: 'in' (into) for reaching into an object to modify it.
 *      Semantically mutable - contrasts with 'de' for read-only iteration.
 *
 * Example:
 *   in user { nomen = "Marcus" }  // mutation block
 */
export function parseInStatement(r: Resolver): InStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('in', ParserErrorCode.ExpectedKeywordIn);

    const expr = r.expression();
    const body = r.block();

    return { type: 'InStatement', object: expr, body, position };
}

// =============================================================================
// CURA STATEMENT (RESOURCE MANAGEMENT)
// =============================================================================

/**
 * Parse cura statement (resource management).
 *
 * GRAMMAR:
 *   curaStmt := 'cura' curatorKind? expression? ('fixum' | 'varia') typeAnnotation? IDENTIFIER blockStmt catchClause?
 *   curatorKind := 'arena' | 'page'
 *
 * WHY: Latin "cura" (care) + binding keyword for scoped resources.
 *      - fixum: immutable binding (const)
 *      - varia: mutable binding (let)
 *      Curator kinds declare explicit allocator types (arena, page).
 *      Guarantees cleanup via solve() on scope exit.
 *
 * Examples:
 *   cura arena fixum mem { ... }                    // arena allocator
 *   cura page fixum mem { ... }                     // page allocator
 *   cura aperi("data.bin") fixum fd { lege(fd) }   // generic resource
 *   cura connect(url) fixum conn { ... }           // resource binding
 */
export function parseCuraStatement(r: Resolver): CuraStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('cura', ParserErrorCode.ExpectedKeywordCura);

    // Check for curator kind: arena or page
    // WHY: Explicit curator kinds declare allocator type for non-GC targets
    let curatorKind: CuratorKind | undefined;
    if (ctx.matchKeyword('arena')) {
        curatorKind = 'arena';
    }
    else if (ctx.matchKeyword('page')) {
        curatorKind = 'page';
    }

    // Parse optional resource acquisition expression
    // WHY: For arena/page, expression is optional (they create their own allocator)
    //      For generic resources, expression is required
    let resource: Expression | undefined;
    if (!ctx.checkKeyword('fixum') && !ctx.checkKeyword('varia') && !ctx.check('LBRACE')) {
        resource = r.expression();
    }

    // Optional binding keyword: fixum (const) or varia (let)
    // WHY: If no keyword, auto-generate binding name for convenience (cura arena { })
    let mutable = false;
    let hasBinding = false;

    if (ctx.matchKeyword('varia')) {
        mutable = true;
        hasBinding = true;
    }
    else if (ctx.matchKeyword('fixum')) {
        mutable = false;
        hasBinding = true;
    }

    let typeAnnotation: TypeAnnotation | undefined;
    let binding: Identifier;

    if (hasBinding) {
        // Optional type annotation before binding identifier
        // WHY: Allows explicit typing: cura aperi("file") fit File fd { ... }
        // Detection: if two identifiers before '{', first is type, second is binding
        if (ctx.check('IDENTIFIER') && ctx.peek(1).type === 'IDENTIFIER') {
            typeAnnotation = r.typeAnnotation();
        }

        // Parse binding identifier
        binding = ctx.parseIdentifier();
    }
    else {
        // Auto-generate binding name
        // WHY: Allows concise syntax: cura arena { } without explicit name
        const prefix = curatorKind ?? 'cura';
        binding = {
            type: 'Identifier',
            name: ctx.genUniqueId(prefix),
            position: ctx.peek().position,
        };
    }

    // Parse body block
    const body = r.block();

    // Optional catch clause
    let catchClause: CapeClause | undefined;
    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return { type: 'CuraStatement', curatorKind, resource, binding, typeAnnotation, async: false, mutable, body, catchClause, position };
}

// =============================================================================
// AD STATEMENT (DISPATCH)
// =============================================================================

/**
 * Parse ad statement (dispatch).
 *
 * GRAMMAR:
 *   adStmt := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
 *   adBinding := ('fit' | 'fiet' | 'fiunt' | 'fient')? typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
 *   argumentList := (expression (',' expression)*)?
 *
 * WHY: Latin 'ad' (to/toward) dispatches to named endpoints:
 *      - Stdlib syscalls: "fasciculus:lege", "console:log"
 *      - External packages: "hono/Hono", "hono/app:serve"
 *      - Remote services: "https://api.example.com/users"
 *
 * Binding verbs encode sync/async and single/plural:
 *      - fit: sync, single ("it becomes")
 *      - fiet: async, single ("it will become")
 *      - fiunt: sync, plural ("they become")
 *      - fient: async, plural ("they will become")
 *
 * Examples:
 *   ad "console:log" ("hello")                           // fire-and-forget
 *   ad "fasciculus:lege" ("file.txt") fit textus pro c { }  // sync binding
 *   ad "http:get" (url) fiet Response pro r { }          // async binding
 *   ad "http:batch" (urls) fient Response[] pro rs { }   // async plural
 */
export function parseAdStatement(r: Resolver): AdStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ad', ParserErrorCode.ExpectedKeywordAd);

    // Parse target string
    const targetToken = ctx.expect('STRING', ParserErrorCode.ExpectedString);
    const target = targetToken.value;

    // Parse argument list: (args...)
    ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);
    const args: (Expression | SpreadElement)[] = [];
    if (!ctx.check('RPAREN')) {
        do {
            if (ctx.matchKeyword('sparge')) {
                const argument = r.expression();
                args.push({ type: 'SpreadElement', argument, position: argument.position });
            }
            else {
                args.push(r.expression());
            }
        } while (ctx.match('COMMA'));
    }
    ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

    // Parse optional binding clause
    // Binding starts with: fit | fiet | fiunt | fient | pro (type inference)
    let binding: AdBinding | undefined;
    if (ctx.checkKeyword('fit') || ctx.checkKeyword('fiet') || ctx.checkKeyword('fiunt') || ctx.checkKeyword('fient') || ctx.checkKeyword('pro')) {
        const bindingPosition = ctx.peek().position;

        // Parse binding verb (default to 'fit' if only 'pro' is used)
        let verb: AdBindingVerb = 'fit';
        if (ctx.matchKeyword('fit')) {
            verb = 'fit';
        }
        else if (ctx.matchKeyword('fiet')) {
            verb = 'fiet';
        }
        else if (ctx.matchKeyword('fiunt')) {
            verb = 'fiunt';
        }
        else if (ctx.matchKeyword('fient')) {
            verb = 'fient';
        }
        // If 'pro' is next without verb, verb defaults to 'fit'

        // Parse optional type annotation before 'pro'
        // Detection: if identifier before 'pro', it's a type
        let typeAnnotation: TypeAnnotation | undefined;
        if (ctx.check('IDENTIFIER') && !ctx.checkKeyword('pro')) {
            typeAnnotation = r.typeAnnotation();
        }

        // Expect 'pro' keyword
        ctx.expectKeyword('pro', ParserErrorCode.ExpectedKeywordPro);

        // Parse binding name
        const name = ctx.parseIdentifier();

        // Parse optional alias: ut alias
        let alias: Identifier | undefined;
        if (ctx.matchKeyword('ut')) {
            alias = ctx.parseIdentifier();
        }

        binding = { type: 'AdBinding', verb, typeAnnotation, name, alias, position: bindingPosition };
    }

    // Parse optional body block
    let body: BlockStatement | undefined;
    if (ctx.check('LBRACE')) {
        body = r.block();
    }

    // Parse optional catch clause
    let catchClause: CapeClause | undefined;
    if (ctx.checkKeyword('cape')) {
        catchClause = parseCapeClause(r);
    }

    return { type: 'AdStatement', target, arguments: args, binding, body, catchClause, position };
}

// =============================================================================
// INCIPIT STATEMENT (ENTRY POINT)
// =============================================================================

/**
 * Parse incipit (entry point) statement.
 *
 * GRAMMAR:
 *   incipitStmt := 'incipit' (blockStmt | 'ergo' statement | 'reddit' expression)
 *
 * WHY: 'incipit' (it begins) marks the program entry point.
 *      This is a pure structural marker with no magic injection.
 *      The source is responsible for any setup (allocators via cura, etc.).
 *
 *      The 'ergo' (therefore) form chains to a single statement, typically
 *      a cura block for allocator setup. This avoids extra nesting.
 *      The 'reddit' form returns an exit code directly.
 *
 * Examples:
 *   incipit {
 *       scribe "Hello"
 *   }
 *
 *   incipit ergo cura arena {
 *       // allocator-scoped work, one-liner header
 *   }
 *
 *   incipit ergo runMain()
 *   incipit reddit 0              // return exit code
 */
export function parseIncipitStatement(r: Resolver): IncipitStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('incipit', ParserErrorCode.ExpectedKeywordIncipit);

    // Parse optional modifiers: argumenta <ident>, exitus <literal|ident>
    let argumentaBinding: Identifier | undefined;
    let exitusModifier: ExitusModifier | undefined;

    // Check for argumenta modifier: incipit argumenta args ...
    if (ctx.checkKeyword('argumenta')) {
        ctx.advance(); // consume 'argumenta'
        if (ctx.check('IDENTIFIER') || ctx.check('KEYWORD')) {
            const ident = ctx.advance();
            argumentaBinding = {
                type: 'Identifier',
                name: ident.value,
                position: ident.position,
            };
        }
        else {
            ctx.errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected identifier after 'argumenta' in incipit, got '${ctx.peek().value}'`,
                position: ctx.peek().position,
            });
        }
    }

    // Check for exitus modifier: incipit exitus 0 ... or incipit exitus code ...
    if (ctx.checkKeyword('exitus')) {
        const exitusPos = ctx.peek().position;
        ctx.advance(); // consume 'exitus'
        if (ctx.check('NUMBER')) {
            const numToken = ctx.advance();
            const code: Literal = {
                type: 'Literal',
                value: Number(numToken.value),
                raw: numToken.value,
                position: numToken.position,
            };
            exitusModifier = { type: 'ExitusModifier', code, position: exitusPos };
        }
        else if (ctx.check('IDENTIFIER') || ctx.check('KEYWORD')) {
            const ident = ctx.advance();
            const code: Identifier = {
                type: 'Identifier',
                name: ident.value,
                position: ident.position,
            };
            exitusModifier = { type: 'ExitusModifier', code, position: exitusPos };
        }
        else {
            ctx.errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected number or identifier after 'exitus' in incipit, got '${ctx.peek().value}'`,
                position: ctx.peek().position,
            });
        }
    }

    const body = parseBody(r);

    return { type: 'IncipitStatement', body, argumentaBinding, exitusModifier, position };
}

// =============================================================================
// INCIPIET STATEMENT (ASYNC ENTRY POINT)
// =============================================================================

/**
 * Parse incipiet (async entry point) statement.
 *
 * GRAMMAR:
 *   incipietStmt := 'incipiet' (blockStmt | 'ergo' statement | 'reddit' expression)
 *
 * WHY: 'incipiet' (it will begin) marks the async program entry point.
 *      Mirrors the fit/fiet pattern: present for sync, future for async.
 *
 *      The 'ergo' form chains to a single statement for concise setup.
 *      The 'reddit' form returns an exit code directly.
 *
 * Examples:
 *   incipiet {
 *       fixum data = cede fetchData()
 *       scribe data
 *   }
 *
 *   incipiet ergo cura arena {
 *       fixum data = cede fetchData()
 *   }
 *
 *   incipiet reddit 0
 */
export function parseIncipietStatement(r: Resolver): IncipietStatement {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('incipiet', ParserErrorCode.ExpectedKeywordIncipiet);

    // Parse optional modifiers: argumenta <ident>, exitus <literal|ident>
    let argumentaBinding: Identifier | undefined;
    let exitusModifier: ExitusModifier | undefined;

    // Check for argumenta modifier: incipiet argumenta args ...
    if (ctx.checkKeyword('argumenta')) {
        ctx.advance(); // consume 'argumenta'
        if (ctx.check('IDENTIFIER') || ctx.check('KEYWORD')) {
            const ident = ctx.advance();
            argumentaBinding = {
                type: 'Identifier',
                name: ident.value,
                position: ident.position,
            };
        }
        else {
            ctx.errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected identifier after 'argumenta' in incipiet, got '${ctx.peek().value}'`,
                position: ctx.peek().position,
            });
        }
    }

    // Check for exitus modifier: incipiet exitus 0 ... or incipiet exitus code ...
    if (ctx.checkKeyword('exitus')) {
        const exitusPos = ctx.peek().position;
        ctx.advance(); // consume 'exitus'
        if (ctx.check('NUMBER')) {
            const numToken = ctx.advance();
            const code: Literal = {
                type: 'Literal',
                value: Number(numToken.value),
                raw: numToken.value,
                position: numToken.position,
            };
            exitusModifier = { type: 'ExitusModifier', code, position: exitusPos };
        }
        else if (ctx.check('IDENTIFIER') || ctx.check('KEYWORD')) {
            const ident = ctx.advance();
            const code: Identifier = {
                type: 'Identifier',
                name: ident.value,
                position: ident.position,
            };
            exitusModifier = { type: 'ExitusModifier', code, position: exitusPos };
        }
        else {
            ctx.errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected number or identifier after 'exitus' in incipiet, got '${ctx.peek().value}'`,
                position: ctx.peek().position,
            });
        }
    }

    const body = parseBody(r);

    return { type: 'IncipietStatement', body, argumentaBinding, exitusModifier, position };
}
