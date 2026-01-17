/**
 * Parser - Recursive Descent Parser for Latin Source Code
 *
 * COMPILER PHASE
 * ==============
 * syntactic
 *
 * ARCHITECTURE
 * ============
 * This module implements a recursive descent parser that transforms a stream of tokens
 * from the lexical analyzer into an Abstract Syntax Tree (AST). The parser uses
 * predictive parsing with one token of lookahead to determine which production to use.
 *
 * The parser is organized around the grammar's structure:
 * - Statement parsing functions handle declarations and control flow
 * - Expression parsing uses precedence climbing to handle operator precedence
 * - Error recovery via synchronization prevents cascading errors
 *
 * Key design decisions:
 * 1. Collects errors and continues parsing
 * 2. Uses local throws for recovery, caught at statement boundaries
 * 3. Synchronizes at statement boundaries after errors
 * 4. Uses helper functions (peek, match, expect) for token manipulation
 * 5. Preserves Latin keywords in AST for semantic analysis phase
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Token[] from tokenizer (includes position info for error reporting)
 * OUTPUT: ParserResult containing Program AST and array of ParserErrors
 * ERRORS: Syntax errors (unexpected tokens, missing punctuation, malformed constructs)
 *
 * INVARIANTS
 * ==========
 * INV-1: Parser never crashes on malformed input (errors collected, not thrown)
 * INV-2: After error, synchronize() finds next statement boundary
 * INV-3: All AST nodes include position for error reporting
 * INV-4: Empty input produces valid Program with empty body
 * INV-5: Parser maintains single token lookahead (peek() without consuming)
 *
 * GRAMMAR
 * =======
 * The authoritative grammar is documented in `EBNF.md`.
 *
 * NOTE: Many functions in this file include `GRAMMAR:` tags for orientation, but
 * they are intentionally non-exhaustive. When updating syntax, update `EBNF.md`.
 *
 * ERROR RECOVERY STRATEGY
 * =======================
 * When a parse error occurs:
 * 1. Record error with message and position
 * 2. Call synchronize() to skip tokens until statement boundary
 * 3. Resume parsing at next statement
 * 4. Return partial AST with collected errors
 *
 * Synchronization points (keywords that start statements):
 * - functio, varia, fixum (declarations)
 * - si, dum, pro (control flow)
 * - redde, tempta (other statements)
 *
 * WHY: Allows parser to report multiple errors in one pass and produce
 *      partial AST for IDE/tooling use even with syntax errors.
 *
 * @module parser
 */

import type { Token, TokenType, Position } from '../tokenizer/types';
import type {
    Program,
    Statement,
    Expression,
    Comment,
    Annotation,
    ImportaDeclaration,
    VariaDeclaration,
    FunctioDeclaration,
    ReturnVerb,
    Visibility,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    DiscretioDeclaration,
    VariantDeclaration,
    VariantField,
    VariantPattern,
    VariantCase,
    SiStatement,
    DumStatement,
    IteratioStatement,
    InStatement,
    EligeStatement,
    EligeCasus,
    DiscerneStatement,
    CustodiStatement,
    CustodiClause,
    AdfirmaStatement,
    ReddeStatement,
    RumpeStatement,
    PergeStatement,
    BlockStatement,
    IaceStatement,
    ScribeStatement,
    OutputLevel,
    ExpressionStatement,
    Identifier,
    EgoExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    TypeParameterDeclaration,
    CapeClause,
    NovumExpression,
    FingeExpression,
    TypeAliasDeclaration,
    OrdoDeclaration,
    OrdoMember,
    Literal,
    RangeExpression,
    ConditionalExpression,
    ImportSpecifier,
    DestructureDeclaration,
    ObjectPattern,
    ObjectPatternProperty,
    ArrayPattern,
    ArrayPatternElement,
    ObjectExpression,
    ObjectProperty,
    SpreadElement,
    FacBlockStatement,
    ClausuraExpression,
    QuaExpression,
    InnatumExpression,
    ConversionExpression,
    ShiftExpression,
    EstExpression,
    ProbandumStatement,
    ProbaStatement,
    ProbaModifier,
    PraeparaBlock,
    PraeparaTiming,
    CuratorKind,
    CuraStatement,
    AdStatement,
    AdBinding,
    AdBindingVerb,
    PraefixumExpression,
    CollectionDSLTransform,
    CollectionDSLExpression,
    AbExpression,
    ScriptumExpression,
    LegeExpression,
    RegexLiteral,
} from './ast';
import { builtinTypes } from '../lexicon/types-builtin';
import { ParserErrorCode, PARSER_ERRORS } from './errors';
import { ParserContext, type ParserError } from './context';
import type { Resolver } from './resolver';
import {
    parseTypeAnnotation as parseTypeAnnotationImpl,
    parseTypeAndParameterList as parseTypeAndParameterListImpl,
    parseParameterList as parseParameterListImpl,
    parseParameter as parseParameterImpl,
} from './types';

// =============================================================================
// MODULE IMPORTS
// =============================================================================

// Expression parsing modules
import {
    parseExpression as parseExpressionModule,
    setUnaryParser as setUnaryParserBinary,
} from './expressions/binary';
import { parseUnary } from './expressions/unary';
import {
    setUnaryParser as setUnaryParserPrimary,
    parsePrimary,
    parseCall,
    parseClausuraExpression,
    parseArgumentList,
    parseNovumExpression,
    parseFingeExpression,
    parseScriptumExpression,
    parseLegeExpression,
    parseQuaExpression,
} from './expressions/primary';
import {
    isDSLVerb,
    parseDSLTransforms,
    parseCollectionDSLExpression,
    parseAbExpression,
    parseRegexLiteral,
} from './expressions/dsl';

// Statement parsing modules
import { parseImportaDeclaration, parseSpecifier } from './statements/imports';
import { parseVariaDeclaration, parseObjectPattern, parseArrayPattern } from './statements/variables';
import {
    parseFunctioDeclaration,
    parseTypeAliasDeclaration,
    parseOrdoDeclaration,
    parseDiscretioDeclaration,
    parseVariantDeclaration,
    parseGenusDeclaration,
    parseGenusMember,
    parsePactumDeclaration,
    parsePactumMethod,
} from './statements/declarations';
import {
    parseSiStatement,
    parseDumStatement,
    parseEligeStatement,
    parseDiscerneStatement,
    parseVariantPattern,
} from './statements/control';
import {
    parseExStatement,
    parseDeStatement,
    parseInStatement,
    parseCuraStatement,
    parseAdStatement,
    parseIncipitStatement,
    parseIncipietStatement,
} from './statements/loops';
import {
    parseTemptaStatement,
    parseCapeClause,
    parseCustodiStatement,
} from './statements/errors';
import {
    parseAdfirmaStatement,
    parseReddeStatement,
    parseRumpeStatement,
    parsePergeStatement,
    parseTacetStatement,
    parseIaceStatement,
    parseScribeStatement,
} from './statements/actions';
import {
    parseProbandumStatement,
    parseProbaStatement,
    parsePraeparaBlock,
} from './statements/testing';
import {
    parseBlockStatement as parseBlockStatementModule,
    parseFacBlockStatement,
    parseExpressionStatement,
} from './statements/blocks';

// Re-export types for external use
export type { ParserError } from './context';
export type { Resolver } from './resolver';
export { ParserContext } from './context';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Parser output containing AST and errors.
 *
 * INVARIANT: If parse succeeds, program is non-null and errors is empty.
 * INVARIANT: If parse fails catastrophically, program is null.
 * INVARIANT: Partial errors (recovered) have non-null program with non-empty errors.
 */
export interface ParserResult {
    program: Program | null;
    errors: ParserError[];
}

// =============================================================================
// TYPE NAME LOOKUP
// =============================================================================

/**
 * Compute nominative singular form from stem, declension, and gender.
 *
 * WHY: Users write nominative forms in source (textus, numerus), but lexicon
 *      stores stems (text, numer). This computes the nominative for lookup.
 */
function computeNominative(stem: string, declension: number, gender: string): string {
    switch (declension) {
        case 1:
            return stem + 'a'; // lista, tabula, copia
        case 2:
            return gender === 'neuter' ? stem + 'um' : stem + 'us'; // numerus, datum
        case 3:
            return stem; // 3rd decl nominative varies - handled via nominative field
        case 4:
            return stem + 'us'; // textus, fluxus
        case 5:
            return stem + 'es';
        default:
            return stem;
    }
}

/**
 * Set of all builtin type names for quick lookup.
 *
 * PERF: Pre-computed Set enables O(1) type name checking.
 *
 * WHY: Used by isTypeName() to distinguish type names from regular identifiers
 *      in type-first syntax parsing (e.g., "fixum textus nomen" vs "fixum nomen").
 *
 * DESIGN: Computed from builtinTypes to avoid duplication. Uses nominative forms
 *         since that's what users write in source code.
 */
const BUILTIN_TYPE_NAMES = new Set(builtinTypes.map(t => t.nominative ?? computeNominative(t.stem, t.declension, t.gender)));

// =============================================================================
// MAIN PARSER FUNCTION
// =============================================================================

/**
 * Parse a token stream into an Abstract Syntax Tree.
 *
 * GRAMMAR:
 *   program := statement*
 *
 * ERROR RECOVERY: Catches errors at statement level, synchronizes, and continues.
 *
 * @param tokens - Token array from tokenizer (must end with EOF token)
 * @returns ParserResult with program AST and error list
 */
export function parse(tokens: Token[]): ParserResult {
    // Create ParserContext for state management
    // All state (parserCtx.current position, errors, comments) goes through parserCtx
    const parserCtx = new ParserContext(tokens);

    // Legacy alias for error collection (points to parserCtx.errors)
    const errors = parserCtx.errors;

    // Resolver implementation - bridges closure functions and ParserContext
    // Forward declarations for functions defined later
    let parseExpressionFn: () => Expression;
    let parseStatementFn: () => Statement;
    let parseBlockStatementFn: () => BlockStatement;
    let parseAnnotationsFn: () => Annotation[];

    const resolver: Resolver = {
        ctx: () => parserCtx,
        expression: () => parseExpressionFn(),
        statement: () => parseStatementFn(),
        block: () => parseBlockStatementFn(),
        typeAnnotation: () => parseTypeAnnotationImpl(resolver),
        annotations: () => parseAnnotationsFn(),
    };

    // Wire circular dependency: binary.ts and primary.ts need parseUnary
    setUnaryParserBinary((r: Resolver) => parseUnary(r));
    setUnaryParserPrimary((r: Resolver) => parseUnary(r));

    // ---------------------------------------------------------------------------
    // Convenience aliases to ParserContext methods
    // These make the migration incremental - existing code can keep using these
    // ---------------------------------------------------------------------------

    const peek = (offset = 0) => parserCtx.peek(offset);
    const isAtEnd = () => parserCtx.isAtEnd();
    const advance = () => parserCtx.advance();
    const check = (type: TokenType) => parserCtx.check(type);
    const checkKeyword = (keyword: string) => parserCtx.checkKeyword(keyword);
    const match = (...types: TokenType[]) => parserCtx.match(...types);
    const matchKeyword = (keyword: string) => parserCtx.matchKeyword(keyword);
    const reportError = (code: ParserErrorCode, context?: string) => parserCtx.reportError(code, context);
    const expect = (type: TokenType, code: ParserErrorCode) => parserCtx.expect(type, code);
    const expectKeyword = (keyword: string, code: ParserErrorCode) => parserCtx.expectKeyword(keyword, code);
    const collectComments = () => parserCtx.collectComments();
    const consumePendingComments = () => parserCtx.consumePendingComments();
    const collectTrailingComment = (nodeLine: number) => parserCtx.collectTrailingComment(nodeLine);
    const isTypeName = (token: Token) => parserCtx.isTypeName(token);
    const isPreposition = (token: Token) => parserCtx.isPreposition(token);
    const genUniqueId = (prefix: string) => parserCtx.genUniqueId(prefix);
    const parseIdentifier = () => parserCtx.parseIdentifier();
    const parseIdentifierOrKeyword = () => parserCtx.parseIdentifierOrKeyword();


    /**
     * Record error and throw for error recovery.
     *
     * WHY: Used in expression parsing where we can't easily recover locally.
     *      Caught by statement parser which calls synchronize().
     */
    function error(code: ParserErrorCode, context?: string): never {
        throw parserCtx.error(code, context);
    }

    /**
     * Parse a single annotation.
     *
     * GRAMMAR:
     *   annotation := '@' IDENTIFIER (expression)?
     *              | '@' 'innatum' targetMapping (',' targetMapping)*
     *              | '@' 'subsidia' targetMapping (',' targetMapping)*
     *              | '@' 'radix' IDENTIFIER (',' IDENTIFIER)*
     *              | '@' 'verte' IDENTIFIER (STRING | '(' paramList ')' '->' STRING)
     *
     *   targetMapping := IDENTIFIER STRING
     *
     * Each @ starts exactly one annotation. The first identifier is the
     * annotation name, followed by an optional argument expression.
     *
     * Examples:
     *   @ publicum
     *   @ ad "users:list"
     *   @ ad sed "users/\w+"
     *   @ innatum ts "Array", py "list", zig "Lista"
     *   @ subsidia zig "subsidia/zig/lista.zig"
     *   @ radix filtr, imperativus, perfectum
     *   @ verte ts "push"
     *   @ verte ts (ego, elem) -> "[...§, §]"
     */
    function parseAnnotation(): Annotation {
        const position = peek().position;
        const startLine = position.line;
        advance(); // consume '@'

        // First token must be identifier or keyword on the same line (the annotation name)
        if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected annotation name after '@', got '${peek().value}'`,
                position,
            });
            return {
                type: 'Annotation',
                name: '',
                position,
            };
        }

        const name = advance().value;

        // Handle specialized annotation types
        if (name === 'innatum' || name === 'subsidia') {
            return parseTargetMappingAnnotation(name, position, startLine);
        } else if (name === 'radix') {
            return parseRadixAnnotation(position, startLine);
        } else if (name === 'verte') {
            return parseVerteAnnotation(position, startLine);
        } else if (name === 'optio') {
            return parseOptioAnnotation(position, startLine);
        } else if (name === 'operandus') {
            return parseOperandusAnnotation(position, startLine);
        }

        // Check for optional argument on the same line (standard annotation)
        let argument: Expression | undefined;
        if (!isAtEnd() && peek().position.line === startLine) {
            // Only parse argument if there's something on the same line
            // that isn't the start of a new statement
            const next = peek();
            if (next.type === 'STRING' || next.type === 'IDENTIFIER' || next.type === 'KEYWORD') {
                // WHY: For @ imperia, don't consume 'ex' as argument - it's a clause keyword
                // Scoped to imperia only to avoid breaking other annotations
                if (name === 'imperia' && next.value === 'ex') {
                    // Skip - will be parsed as exClause below
                }
                else {
                    argument = parseExpressionModule(resolver);
                }
            }
        }

        // Check for 'ex <identifier>' clause (e.g., @ imperia "remote" ex remoteModule)
        // WHY: Scoped strictly to @ imperia - 'ex' is common Latin and shouldn't be
        // reserved for all annotations
        let exClause: Identifier | undefined;
        if (name === 'imperia' && !isAtEnd() && peek().position.line === startLine && checkKeyword('ex')) {
            advance(); // consume 'ex'
            if ((check('IDENTIFIER') || check('KEYWORD')) && peek().position.line === startLine) {
                const exIdent = advance();
                exClause = {
                    type: 'Identifier',
                    name: exIdent.value,
                    position: exIdent.position,
                };
            }
            else {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected identifier after 'ex' in @ imperia annotation, got '${peek().value}'`,
                    position: peek().position,
                });
            }
        }

        return {
            type: 'Annotation',
            name,
            argument,
            exClause,
            position,
        };
    }

    /**
     * Parse @ innatum or @ subsidia annotation with target-to-value mappings.
     *
     * GRAMMAR:
     *   targetMappingAnnotation := targetMapping (',' targetMapping)*
     *   targetMapping := IDENTIFIER STRING
     *
     * Example:
     *   @ innatum ts "Array", py "list", zig "Lista"
     *   @ subsidia zig "subsidia/zig/lista.zig"
     */
    function parseTargetMappingAnnotation(name: string, position: Position, startLine: number): Annotation {
        const targetMappings = new Map<string, string>();

        // Parse first mapping (required)
        if (!isAtEnd() && peek().position.line === startLine && (check('IDENTIFIER') || check('KEYWORD'))) {
            do {
                // Parse target identifier (ts, py, rs, cpp, zig)
                if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
                    errors.push({
                        code: ParserErrorCode.UnexpectedToken,
                        message: `Expected target identifier in @${name} annotation, got '${peek().value}'`,
                        position: peek().position,
                    });
                    break;
                }
                const target = advance().value;

                // Parse value string
                if (!check('STRING') || peek().position.line !== startLine) {
                    errors.push({
                        code: ParserErrorCode.UnexpectedToken,
                        message: `Expected string value after '${target}' in @${name} annotation, got '${peek().value}'`,
                        position: peek().position,
                    });
                    break;
                }
                const value = advance().value;

                targetMappings.set(target, value);
            } while (!isAtEnd() && peek().position.line === startLine && match('COMMA'));
        }

        return {
            type: 'Annotation',
            name,
            targetMappings,
            position,
        };
    }

    /**
     * Parse @ radix annotation with stem and form identifiers.
     *
     * GRAMMAR:
     *   radixAnnotation := IDENTIFIER (',' IDENTIFIER)*
     *
     * Example:
     *   @ radix filtr, imperativus, perfectum
     *
     * First identifier is the stem, rest are valid morphological forms.
     */
    function parseRadixAnnotation(position: Position, startLine: number): Annotation {
        const radixForms: string[] = [];

        // Parse comma-separated identifiers
        if (!isAtEnd() && peek().position.line === startLine && (check('IDENTIFIER') || check('KEYWORD'))) {
            do {
                if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
                    errors.push({
                        code: ParserErrorCode.UnexpectedToken,
                        message: `Expected identifier in @radix annotation, got '${peek().value}'`,
                        position: peek().position,
                    });
                    break;
                }
                radixForms.push(advance().value);
            } while (!isAtEnd() && peek().position.line === startLine && match('COMMA'));
        }

        if (radixForms.length === 0) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `@radix annotation requires at least a stem identifier`,
                position,
            });
        }

        return {
            type: 'Annotation',
            name: 'radix',
            radixForms,
            position,
        };
    }

    /**
     * Parse @ verte annotation with target and either method name or template.
     *
     * GRAMMAR:
     *   verteAnnotation := IDENTIFIER (STRING | '(' paramList ')' '->' STRING)
     *   paramList := IDENTIFIER (',' IDENTIFIER)*
     *
     * Examples:
     *   @ verte ts "push"
     *   @ verte ts (ego, elem) -> "[...§, §]"
     */
    function parseVerteAnnotation(position: Position, startLine: number): Annotation {
        // Parse target identifier (ts, py, rs, cpp, zig)
        if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected target identifier in @verte annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return {
                type: 'Annotation',
                name: 'verte',
                position,
            };
        }
        const verteTarget = advance().value;

        // Check for template form: (params) -> "template"
        if (check('LPAREN') && peek().position.line === startLine) {
            advance(); // consume '('

            const verteParams: string[] = [];

            // Parse parameter list
            if (!check('RPAREN')) {
                do {
                    if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
                        errors.push({
                            code: ParserErrorCode.UnexpectedToken,
                            message: `Expected parameter name in @verte template, got '${peek().value}'`,
                            position: peek().position,
                        });
                        break;
                    }
                    verteParams.push(advance().value);
                } while (match('COMMA'));
            }

            expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

            // Expect -> arrow
            if (!match('THIN_ARROW')) {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected '->' after parameters in @verte template, got '${peek().value}'`,
                    position: peek().position,
                });
            }

            // Parse template string
            if (!check('STRING') || peek().position.line !== startLine) {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected template string in @verte annotation, got '${peek().value}'`,
                    position: peek().position,
                });
                return {
                    type: 'Annotation',
                    name: 'verte',
                    verteTarget,
                    verteParams,
                    position,
                };
            }
            const verteTemplate = advance().value;

            // Reject multiple targets on one line (use separate @ verte for each)
            if (check('COMMA') && peek().position.line === startLine) {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `@verte allows only one target per line; use separate @verte annotations`,
                    position: peek().position,
                });
            }

            return {
                type: 'Annotation',
                name: 'verte',
                verteTarget,
                verteParams,
                verteTemplate,
                position,
            };
        }

        // Simple method form: "methodName"
        if (!check('STRING') || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected method name string or template in @verte annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return {
                type: 'Annotation',
                name: 'verte',
                verteTarget,
                position,
            };
        }
        const verteMethod = advance().value;

        // Reject multiple targets on one line (use separate @ verte for each)
        if (check('COMMA') && peek().position.line === startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `@verte allows only one target per line; use separate @verte annotations`,
                position: peek().position,
            });
        }

        return {
            type: 'Annotation',
            name: 'verte',
            verteTarget,
            verteMethod,
            position,
        };
    }

    /**
     * Parse @ optio annotation for CLI option flags.
     *
     * GRAMMAR:
     *   optioAnnotation := type IDENTIFIER [brevis STRING] [longum STRING] [descriptio STRING]
     *
     * CONSTRAINTS:
     *   - At least one of brevis/longum is required
     *   - brevis must be a single character
     *
     * Examples:
     *   @ optio bivalens v brevis "v" longum "verbose" descriptio "Enable verbose output"
     *   @ optio bivalens n brevis "n" descriptio "Dry run mode"
     *   @ optio textus color longum "color" descriptio "Colorize output"
     *   @ optio bivalens singleColumn brevis "1" descriptio "One file per line"
     */
    function parseOptioAnnotation(position: Position, startLine: number): Annotation {
        // Parse type (required)
        if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected type in @optio annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return { type: 'Annotation', name: 'optio', position };
        }
        const optioType = parseTypeAnnotationImpl(resolver);

        // Parse binding name (IDENTIFIER required)
        let optioInternal: Identifier | undefined;
        if (!isAtEnd() && peek().position.line === startLine && (check('IDENTIFIER') || check('KEYWORD'))) {
            const ident = advance();
            optioInternal = {
                type: 'Identifier',
                name: ident.value,
                position: ident.position,
            };
        }
        else {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected identifier for binding name in @optio annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return { type: 'Annotation', name: 'optio', optioType, position };
        }

        // Parse optional 'brevis' short flag
        let optioShort: string | undefined;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('brevis')) {
            advance(); // consume 'brevis'
            if (check('STRING') && peek().position.line === startLine) {
                const shortValue = advance().value;
                // Validate single character
                if (shortValue.length !== 1) {
                    errors.push({
                        code: ParserErrorCode.UnexpectedToken,
                        message: `Short flag in 'brevis' must be a single character, got '${shortValue}'`,
                        position: peek().position,
                    });
                }
                else {
                    optioShort = shortValue;
                }
            }
            else {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected string after 'brevis' in @optio annotation, got '${peek().value}'`,
                    position: peek().position,
                });
            }
        }

        // Parse optional 'longum' long flag
        let optioLong: string | undefined;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('longum')) {
            advance(); // consume 'longum'
            if (check('STRING') && peek().position.line === startLine) {
                optioLong = advance().value;
            }
            else {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected string after 'longum' in @optio annotation, got '${peek().value}'`,
                    position: peek().position,
                });
            }
        }

        // Validate: at least one of brevis/longum required
        if (!optioShort && !optioLong) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `@optio requires at least one of 'brevis' or 'longum'`,
                position: position,
            });
        }

        // Parse optional 'descriptio' help text
        let optioDescription: string | undefined;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('descriptio')) {
            advance(); // consume 'descriptio'
            if (check('STRING') && peek().position.line === startLine) {
                optioDescription = advance().value;
            }
            else {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected string after 'descriptio' in @optio annotation, got '${peek().value}'`,
                    position: peek().position,
                });
            }
        }

        return {
            type: 'Annotation',
            name: 'optio',
            optioType,
            optioInternal,
            optioShort,
            optioLong,
            optioDescription,
            position,
        };
    }

    /**
     * Parse @ operandus annotation for CLI positional arguments.
     *
     * GRAMMAR:
     *   operandusAnnotation := ['ceteri'] type IDENTIFIER ['descriptio' STRING]
     *
     * Examples:
     *   @ operandus textus file descriptio "Input file"
     *   @ operandus ceteri textus files descriptio "Additional input files"
     */
    function parseOperandusAnnotation(position: Position, startLine: number): Annotation {
        // Check for optional 'ceteri' prefix (rest/variadic)
        let operandusRest = false;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('ceteri')) {
            advance(); // consume 'ceteri'
            operandusRest = true;
        }

        // Parse type (required)
        if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected type in @operandus annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return { type: 'Annotation', name: 'operandus', operandusRest, position };
        }
        const operandusType = parseTypeAnnotationImpl(resolver);

        // Parse name (IDENTIFIER required)
        let operandusName: Identifier | undefined;
        if (!isAtEnd() && peek().position.line === startLine && (check('IDENTIFIER') || check('KEYWORD'))) {
            const tok = advance();
            operandusName = {
                type: 'Identifier',
                name: tok.value,
                position: tok.position,
            };
        }
        else {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected identifier for operand name in @operandus annotation, got '${peek().value}'`,
                position: peek().position,
            });
            return { type: 'Annotation', name: 'operandus', operandusType, operandusRest, position };
        }

        // Parse optional 'vel' default value
        let operandusDefault: Expression | undefined;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('vel')) {
            advance(); // consume 'vel'
            operandusDefault = parseExpressionModule(resolver);
        }

        // Parse optional 'descriptio' help text
        let operandusDescription: string | undefined;
        if (!isAtEnd() && peek().position.line === startLine && checkKeyword('descriptio')) {
            advance(); // consume 'descriptio'
            if (check('STRING') && peek().position.line === startLine) {
                operandusDescription = advance().value;
            }
            else {
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Expected string after 'descriptio' in @operandus annotation, got '${peek().value}'`,
                    position: peek().position,
                });
            }
        }

        return {
            type: 'Annotation',
            name: 'operandus',
            operandusType,
            operandusName,
            operandusRest,
            operandusDefault,
            operandusDescription,
            position,
        };
    }

    // =============================================================================
    // STATEMENT PARSING
    // =============================================================================

    /**
     * Parse top-level program.
     *
     * GRAMMAR:
     *   program := statement*
     *
     * ERROR RECOVERY: Catches statement errors, synchronizes, continues.
     *
     * EDGE: Empty source file produces valid Program with empty body.
     */
    function parseProgram(): Program {
        const body: Statement[] = [];
        const position = peek().position;

        while (!isAtEnd()) {
            // Consume optional semicolons between statements
            while (match('SEMICOLON')) {
                // do nothing - avoids linter no-empty
            }

            if (isAtEnd()) {
                break;
            }

            try {
                body.push(parseStatement());
            } catch {
                synchronize();
            }
        }

        return { type: 'Program', body, position };
    }

    /**
     * Synchronize parser after error by skipping to next statement boundary.
     *
     * ERROR RECOVERY: Advances until finding a keyword that starts a statement.
     *
     * WHY: Prevents cascading errors by resuming at known-good parse state.
     */
    function synchronize(): void {
        advance();
        while (!isAtEnd()) {
            if (
                // Annotations may precede declarations.
                check('AT') ||
                check('SECTION') ||
                // Blocks can always start a statement.
                check('LBRACE') ||
                // Statement starters (mirror parseStatementCore).
                checkKeyword('ex') ||
                checkKeyword('de') ||
                checkKeyword('in') ||
                checkKeyword('varia') ||
                checkKeyword('fixum') ||
                checkKeyword('figendum') ||
                checkKeyword('variandum') ||
                checkKeyword('functio') ||
                checkKeyword('typus') ||
                checkKeyword('ordo') ||
                checkKeyword('genus') ||
                checkKeyword('pactum') ||
                checkKeyword('discretio') ||
                checkKeyword('si') ||
                checkKeyword('dum') ||
                checkKeyword('elige') ||
                checkKeyword('discerne') ||
                checkKeyword('custodi') ||
                checkKeyword('adfirma') ||
                checkKeyword('redde') ||
                checkKeyword('rumpe') ||
                checkKeyword('perge') ||
                checkKeyword('tacet') ||
                checkKeyword('iace') ||
                checkKeyword('mori') ||
                checkKeyword('scribe') ||
                checkKeyword('vide') ||
                checkKeyword('mone') ||
                checkKeyword('tempta') ||
                checkKeyword('fac') ||
                checkKeyword('probandum') ||
                checkKeyword('proba') ||
                checkKeyword('ad') ||
                checkKeyword('praepara') ||
                checkKeyword('praeparabit') ||
                checkKeyword('postpara') ||
                checkKeyword('postparabit') ||
                checkKeyword('cura') ||
                checkKeyword('incipit') ||
                checkKeyword('incipiet')
            ) {
                return;
            }

            advance();
        }
    }

    /**
     * Synchronize parser after error in genus member by skipping to next member boundary.
     *
     * ERROR RECOVERY: Advances until finding a token that could start a genus member.
     *
     * WHY: Prevents infinite loops when malformed syntax (e.g., TS-style `fixum name: textus`)
     *      causes parseGenusMember to return without advancing.
     */
    function synchronizeGenusMember(): void {
        advance();
        let braceDepth = 0;

        while (!isAtEnd()) {
            if (check('LBRACE')) {
                braceDepth++;
                advance();
                continue;
            }

            if (check('RBRACE')) {
                if (braceDepth === 0) {
                    return;
                }

                braceDepth--;
                advance();
                continue;
            }

            // Stop at tokens that could start a new member (only at genus-body depth).
            if (
                braceDepth === 0 &&
                (check('AT') ||
                    check('SECTION') ||
                    checkKeyword('functio') ||
                    checkKeyword('publicus') ||
                    checkKeyword('privatus') ||
                    checkKeyword('protectus') ||
                    checkKeyword('abstractus') ||
                    checkKeyword('generis') ||
                    // Type annotations may begin with borrow prepositions or a type.
                    checkKeyword('de') ||
                    checkKeyword('in') ||
                    check('LPAREN') ||
                    check('IDENTIFIER') ||
                    check('KEYWORD'))
            ) {
                return;
            }

            advance();
        }
    }

    /**
     * Attach comments to a statement node.
     *
     * WHY: Centralizes comment attachment logic for all statement types.
     *
     * @param stmt - The statement to attach comments to
     * @returns The statement with comments attached
     */
    function attachComments<T extends Statement>(stmt: T): T {
        const leading = consumePendingComments();
        if (leading) {
            stmt.leadingComments = leading;
        }
        // Check for trailing comment on the same line
        const trailing = collectTrailingComment(stmt.position.line);
        if (trailing) {
            stmt.trailingComments = trailing;
        }
        return stmt;
    }

    /**
     * Parse annotation (@ modifier+).
     *
     * GRAMMAR:
     *   annotation := '@' IDENTIFIER+
     *
     * WHY: Annotations modify the following declaration with metadata like
     *      visibility (publicum, privatum), async (futura), abstract (abstractum).
     *
     * INVARIANT: Called when parserCtx.current token is AT.
     * INVARIANT: Consumes AT and all following identifiers on the same logical line.
     *
     * Examples:
     *   @ publicum
     *   @ publica futura
     *   @ privatum abstractum
     */

    /**
     * Parse zero or more annotations before a declaration.
     *
     * GRAMMAR:
     *   annotations := annotation*
     *
     * WHY: Multiple annotations can be stacked before a declaration.
     *
     * Examples:
     *   @ publicum
     *   @ futura
     *   functio fetch() -> textus { }
     */
    function parseAnnotations(): Annotation[] {
        const annotations: Annotation[] = [];

        while (check('AT')) {
            annotations.push(parseAnnotation());
        }

        return annotations;
    }

    /**
     * Parse a section annotation (§).
     *
     * GRAMMAR:
     *   sectionAnnotation := '§' IDENTIFIER (IDENTIFIER | STRING)*
     *
     * Section annotations are file-level build/project configuration.
     * They're parsed but stored separately from statement annotations.
     *
     * Examples:
     *   § opus nomen "echo"
     *   § scopos "ts"
     *   § dependentia "norma" via "../lib/norma"
     */
    function parseSectionAnnotation(): void {
        const position = peek().position;
        const startLine = position.line;
        advance(); // consume '§'

        // First token must be identifier on the same line (the annotation name)
        if ((!check('IDENTIFIER') && !check('KEYWORD')) || peek().position.line !== startLine) {
            errors.push({
                code: ParserErrorCode.UnexpectedToken,
                message: `Expected annotation name after '§', got '${peek().value}'`,
                position,
            });
            return;
        }

        advance(); // consume annotation name

        // Consume all arguments on the same line (identifiers and strings)
        while (!isAtEnd() && peek().position.line === startLine) {
            if (check('IDENTIFIER') || check('KEYWORD') || check('STRING')) {
                advance();
            } else {
                break;
            }
        }

        // Section annotations are currently parsed and discarded
        // TODO: Store for build system integration
    }

    /**
     * Parse any statement by dispatching to specific parser.
     *
     * GRAMMAR: statement (see `EBNF.md` "Program Structure")
     *
     * WHY: Uses lookahead to determine statement type via keyword inspection.
     */

    function parseStatement(): Statement {
        // Collect any leading comments before parsing the statement
        collectComments();

        // Parse the statement and attach comments
        const stmt = parseStatementWithoutComments();
        return attachComments(stmt);
    }

    /**
     * Parse statement without comment attachment (internal helper).
     *
     * WHY: Separates statement dispatch from comment handling for cleaner code.
     */
    function parseStatementWithoutComments(): Statement {
        // Skip any section annotations (§) - these are file-level build config
        // They're parsed but not attached to statements
        while (check('SECTION')) {
            parseSectionAnnotation();
        }

        // Parse any leading annotations (@ modifier+)
        // Annotations attach to the following declaration
        const annotations = parseAnnotations();

        // Parse the actual statement
        const stmt = parseStatementCore();

        // Attach annotations to declarations that support them
        if (annotations.length > 0) {
            if (
                stmt.type === 'VariaDeclaration' ||
                stmt.type === 'FunctioDeclaration' ||
                stmt.type === 'GenusDeclaration' ||
                stmt.type === 'PactumDeclaration' ||
                stmt.type === 'OrdoDeclaration' ||
                stmt.type === 'DiscretioDeclaration' ||
                stmt.type === 'TypeAliasDeclaration' ||
                stmt.type === 'IncipitStatement' ||
                stmt.type === 'IncipietStatement'
            ) {
                stmt.annotations = annotations;
            } else if (stmt.type === 'ProbaStatement') {
                // Process test annotations
                for (const a of annotations) {
                    if (a.name === 'omitte') {
                        stmt.modifier = 'omitte';
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.modifierReason = (a.argument as { value: string }).value;
                        }
                    } else if (a.name === 'futurum') {
                        stmt.modifier = 'futurum';
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.modifierReason = (a.argument as { value: string }).value;
                        }
                    } else if (a.name === 'solum') {
                        stmt.solum = true;
                    } else if (a.name === 'tag') {
                        if (!stmt.tags) stmt.tags = [];
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.tags.push((a.argument as { value: string }).value);
                        }
                    } else if (a.name === 'temporis') {
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'NumericLiteral') {
                            stmt.temporis = (a.argument as { value: number }).value;
                        }
                    } else if (a.name === 'metior') {
                        stmt.metior = true;
                    } else if (a.name === 'repete') {
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'NumericLiteral') {
                            stmt.repete = (a.argument as { value: number }).value;
                        }
                    } else if (a.name === 'fragilis') {
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'NumericLiteral') {
                            stmt.fragilis = (a.argument as { value: number }).value;
                        }
                    } else if (a.name === 'requirit') {
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.requirit = (a.argument as { value: string }).value;
                        }
                    } else if (a.name === 'solum_in') {
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.solumIn = (a.argument as { value: string }).value;
                        }
                    }
                }
            } else if (stmt.type === 'ProbandumStatement') {
                // Process suite annotations
                for (const a of annotations) {
                    if (a.name === 'omitte') {
                        stmt.skip = true;
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.skipReason = (a.argument as { value: string }).value;
                        }
                    } else if (a.name === 'solum') {
                        stmt.solum = true;
                    } else if (a.name === 'tag') {
                        if (!stmt.tags) stmt.tags = [];
                        if (a.argument && typeof a.argument === 'object' && 'type' in a.argument && a.argument.type === 'StringLiteral') {
                            stmt.tags.push((a.argument as { value: string }).value);
                        }
                    }
                }
            } else {
                // Warn about annotations on unsupported statements
                errors.push({
                    code: ParserErrorCode.UnexpectedToken,
                    message: `Annotations are not allowed on ${stmt.type}`,
                    position: annotations[0]!.position,
                });
            }
        }

        return stmt;
    }

    /**
     * Core statement parsing logic (dispatches to specific parsers).
     *
     * WHY: Separated from parseStatementWithoutComments to allow annotation handling.
     */
    function parseStatementCore(): Statement {
        // Distinguish 'ex norma importa' (import), 'ex items pro n' (for-loop),
        // and 'ex response fixum { }' (destructuring)
        if (checkKeyword('ex')) {
            // Look ahead: ex (IDENTIFIER|STRING) importa -> import
            const nextType = peek(1).type;

            if ((nextType === 'IDENTIFIER' || nextType === 'STRING') && peek(2).keyword === 'importa') {
                return parseImportaDeclaration(resolver);
            }

            // Could be for-loop or destructuring - parse and dispatch
            return parseExStatement(resolver);
        }

        // 'de' for for-in loops (iterate over keys)
        // de tabula pro k { } → for-in loop
        if (checkKeyword('de')) {
            return parseDeStatement(resolver);
        }

        // 'in' for mutation blocks
        // in user { } → with-block (mutation)
        if (checkKeyword('in')) {
            return parseInStatement(resolver);
        }

        if (checkKeyword('varia') || checkKeyword('fixum') || checkKeyword('figendum') || checkKeyword('variandum')) {
            return parseVariaDeclaration(resolver);
        }

        if (checkKeyword('functio')) {
            return parseFunctioDeclaration(resolver);
        }

        if (checkKeyword('typus')) {
            return parseTypeAliasDeclaration(resolver);
        }

        if (checkKeyword('ordo')) {
            return parseOrdoDeclaration(resolver);
        }

        if (checkKeyword('genus')) {
            return parseGenusDeclaration(resolver);
        }

        if (checkKeyword('pactum')) {
            return parsePactumDeclaration(resolver);
        }

        if (checkKeyword('discretio')) {
            return parseDiscretioDeclaration(resolver);
        }

        if (checkKeyword('si')) {
            return parseSiStatement(resolver);
        }

        if (checkKeyword('dum')) {
            return parseDumStatement(resolver);
        }

        if (checkKeyword('elige')) {
            return parseEligeStatement(resolver);
        }

        if (checkKeyword('discerne')) {
            return parseDiscerneStatement(resolver);
        }

        if (checkKeyword('custodi')) {
            return parseCustodiStatement(resolver);
        }

        if (checkKeyword('adfirma')) {
            return parseAdfirmaStatement(resolver);
        }

        if (checkKeyword('redde')) {
            return parseReddeStatement(resolver);
        }

        if (checkKeyword('rumpe')) {
            return parseRumpeStatement(resolver);
        }

        if (checkKeyword('perge')) {
            return parsePergeStatement(resolver);
        }

        if (checkKeyword('tacet')) {
            return parseTacetStatement(resolver);
        }

        // WHY: Keywords followed by '(' are treated as function calls, not keyword statements.
        //      This allows user-defined functions with keyword names (e.g., HAL's consolum.scribe).
        //      Keyword syntax: `scribe "hello"` (no parens)
        //      Function call:  `scribe("hello")` (with parens)
        if (checkKeyword('iace') && peek(1).type !== 'LPAREN') {
            return parseIaceStatement(resolver, false);
        }

        if (checkKeyword('mori') && peek(1).type !== 'LPAREN') {
            return parseIaceStatement(resolver, true);
        }

        if (checkKeyword('scribe') && peek(1).type !== 'LPAREN') {
            return parseScribeStatement(resolver, 'log');
        }

        if (checkKeyword('vide') && peek(1).type !== 'LPAREN') {
            return parseScribeStatement(resolver, 'debug');
        }

        if (checkKeyword('mone') && peek(1).type !== 'LPAREN') {
            return parseScribeStatement(resolver, 'warn');
        }

        if (checkKeyword('tempta')) {
            return parseTemptaStatement(resolver);
        }

        // fac { } cape { } is block with optional catch (see parseFacBlockStatement)
        if (checkKeyword('fac') && peek(1).type === 'LBRACE') {
            return parseFacBlockStatement(resolver);
        }

        // Test suite declaration: probandum "name" { ... }
        if (checkKeyword('probandum')) {
            return parseProbandumStatement(resolver);
        }

        // Individual test: proba "name" { ... }
        if (checkKeyword('proba')) {
            return parseProbaStatement(resolver);
        }

        // Dispatch statement
        // ad "target" (args) [binding]? [block]? [cape]?
        if (checkKeyword('ad')) {
            return parseAdStatement(resolver);
        }

        // Test setup/teardown blocks
        // praepara/praeparabit [omnia]? { } - beforeEach/beforeAll
        // postpara/postparabit [omnia]? { } - afterEach/afterAll
        if (checkKeyword('praepara') || checkKeyword('praeparabit') || checkKeyword('postpara') || checkKeyword('postparabit')) {
            return parsePraeparaBlock(resolver);
        }

        // Resource management
        // cura [cede]? <expr> fit <id> { } [cape]? - scoped resources (CuraStatement)
        if (checkKeyword('cura') && peek(1).type !== 'LPAREN') {
            return parseCuraStatement(resolver);
        }

        // Entry point statements: incipit { } (sync) or incipiet { } (async)
        if (checkKeyword('incipit') && peek(1).type !== 'LPAREN') {
            return parseIncipitStatement(resolver);
        }
        if (checkKeyword('incipiet') && peek(1).type !== 'LPAREN') {
            return parseIncipietStatement(resolver);
        }

        if (check('LBRACE')) {
            return parseBlockStatementModule(resolver);
        }

        return parseExpressionStatement(resolver);
    }

    // =============================================================================
    // TYPE ANNOTATION PARSING (delegated to types.ts via resolver)
    // =============================================================================

    // Delegate to types.ts implementation
    const parseTypeAnnotation = () => resolver.typeAnnotation();
    const parseTypeAndParameterList = () => parseTypeAndParameterListImpl(resolver);
    const parseParameterList = () => parseParameterListImpl(resolver);
    const parseParameter = () => parseParameterImpl(resolver);

    // =============================================================================
    // WIRE UP FORWARD DECLARATIONS
    // =============================================================================

    // These assignments connect the resolver to the actual parsing functions
    parseExpressionFn = () => parseExpressionModule(resolver);
    parseStatementFn = parseStatement;
    parseBlockStatementFn = () => parseBlockStatementModule(resolver);
    parseAnnotationsFn = parseAnnotations;

    // =============================================================================
    // MAIN PARSE EXECUTION
    // =============================================================================

    /**
     * Execute the parse.
     *
     * ERROR RECOVERY: Top-level try-catch ensures parser never crashes.
     *                 Returns null program on catastrophic failure.
     */
    try {
        const program = parseProgram();

        return { program, errors };
    }
    catch {
        return { program: null, errors };
    }
}

export * from './ast';
