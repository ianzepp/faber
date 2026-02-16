/**
 * Declaration Parsing
 *
 * Handles parsing of type declarations: functions, type aliases, enums,
 * discriminated unions, structs, and interfaces.
 *
 * GRAMMAR: See `EBNF.md` "Declarations" section
 *
 * @module parser/statements/declarations
 */

import type { Resolver } from '../resolver';
import type {
    FunctioDeclaration,
    TypeAliasDeclaration,
    OrdoDeclaration,
    OrdoMember,
    DiscretioDeclaration,
    VariantDeclaration,
    VariantField,
    GenusDeclaration,
    FieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    TypeAnnotation,
    BlockStatement,
    Identifier,
    Literal,
    Expression,
    ReturnVerb,
    FunctioModifier,
    TypeParameterDeclaration,
} from '../ast';
import { ParserErrorCode } from '../errors';
import { parseTypeAndParameterList, parseParameterList } from '../types';

// =============================================================================
// FUNCTION DECLARATIONS
// =============================================================================

function parseFunctionModifiers(r: Resolver): FunctioModifier[] | undefined {
    const ctx = r.ctx();
    const modifiers: FunctioModifier[] = [];

    while (!ctx.isAtEnd()) {
        if (ctx.checkKeyword('curata')) {
            const position = ctx.peek().position;
            ctx.advance();
            const name = ctx.parseIdentifier();
            modifiers.push({ type: 'CurataModifier', name, position });
            continue;
        }

        if (ctx.checkKeyword('errata')) {
            const position = ctx.peek().position;
            ctx.advance();
            const name = ctx.parseIdentifier();
            modifiers.push({ type: 'ErrataModifier', name, position });
            continue;
        }

        if (ctx.checkKeyword('exitus')) {
            const position = ctx.peek().position;
            ctx.advance();
            // Parse either IDENTIFIER or NUMBER literal
            const next = ctx.peek();
            if (next.type === 'IDENTIFIER') {
                const code = ctx.parseIdentifier();
                modifiers.push({ type: 'ExitusModifier', code, position });
            }
            else if (next.type === 'NUMBER') {
                const codeToken = ctx.advance();
                const code: Literal = {
                    type: 'Literal',
                    value: codeToken.value,
                    raw: String(codeToken.value),
                    position: codeToken.position,
                };
                modifiers.push({ type: 'ExitusModifier', code, position });
            }
            else {
                ctx.reportError(ParserErrorCode.UnexpectedToken, 'Expected identifier or number after exitus');
            }
            continue;
        }

        if (ctx.checkKeyword('immutata')) {
            const position = ctx.peek().position;
            ctx.advance();
            modifiers.push({ type: 'ImmutataModifier', position });
            continue;
        }

        if (ctx.checkKeyword('iacit')) {
            const position = ctx.peek().position;
            ctx.advance();
            modifiers.push({ type: 'IacitModifier', position });
            continue;
        }

        if (ctx.checkKeyword('optiones')) {
            const position = ctx.peek().position;
            ctx.advance();
            const name = ctx.parseIdentifier();
            modifiers.push({ type: 'OptionesModifier', name, position });
            continue;
        }

        break;
    }

    return modifiers.length > 0 ? modifiers : undefined;
}

/**
 * Parse function declaration.
 *
 * GRAMMAR:
 *   funcDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
 *   paramList := (typeParamDecl ',')* (parameter (',' parameter)*)?
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'exitus' (IDENTIFIER | NUMBER)
 *                | 'immutata'
 *                | 'iacit'
 *   returnClause := '->' typeAnnotation
 *
 * NOTE: fit/fiet/fiunt/fient return syntax disabled pending Go/Zig backend support.
 *
 * WHY: Top-level function declaration. Body is optional for @ externa declarations.
 */
export function parseFunctioDeclaration(r: Resolver): FunctioDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

    // WHY: Allow keywords as function names (e.g., stdlib's `functio lege()`)
    const name = ctx.parseIdentifierOrKeyword();

    ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

    // Parse type parameters and regular parameters
    const { typeParams, params } = parseTypeAndParameterList(r);

    ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

    const modifiers = parseFunctionModifiers(r);

    let returnType: TypeAnnotation | undefined;
    let returnVerb: ReturnVerb | undefined;

    // Parse return type with arrow only
    // NOTE: fit/fiet/fiunt/fient return syntax is disabled pending Go/Zig backend support.
    // The Responsum protocol machinery remains in codegen but is dormant until re-enabled here.
    if (ctx.match('THIN_ARROW')) {
        returnType = r.typeAnnotation();
        returnVerb = 'arrow';
    }

    // WHY: async/generator now derived from annotations (@ futura / @ cursor) not return verbs
    const async: boolean = false;
    const generator: boolean = false;

    // WHY: Body is optional for @ externa declarations (external functions have no body)
    // If no opening brace, function has no body (validated in semantic phase)
    let body: BlockStatement | undefined;
    if (ctx.check('LBRACE')) {
        body = r.block();
    }

    return {
        type: 'FunctioDeclaration',
        name,
        typeParams: typeParams.length > 0 ? typeParams : undefined,
        params,
        returnType,
        body,
        async,
        generator,
        modifiers,
        returnVerb,
        position,
    };
}

// =============================================================================
// TYPE ALIAS DECLARATIONS
// =============================================================================

/**
 * Parse type alias declaration.
 *
 * GRAMMAR:
 *   typeAliasDecl := 'typus' IDENTIFIER '=' typeAnnotation
 *
 * WHY: Enables creating named type aliases for complex types.
 *
 * Examples:
 *   typus ID = textus
 *   typus UserID = numerus<32, Naturalis>
 *   typus ConfigTypus = typus config    // typeof
 */
export function parseTypeAliasDeclaration(r: Resolver): TypeAliasDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('typus', ParserErrorCode.ExpectedKeywordTypus);

    // WHY: Allow keywords as type names for consistency with genus methods
    const name = ctx.parseIdentifierOrKeyword();

    ctx.expect('EQUAL', ParserErrorCode.ExpectedEqual);

    // Check for typeof: `typus X = typus y`
    if (ctx.checkKeyword('typus')) {
        ctx.advance(); // consume 'typus'
        const typeofTarget = ctx.parseIdentifier();
        // WHY: When RHS is `typus identifier`, we extract the type of a value.
        // typeAnnotation is set to a placeholder; codegen uses typeofTarget.
        return {
            type: 'TypeAliasDeclaration',
            name,
            typeAnnotation: { type: 'TypeAnnotation', name: 'ignotum', position },
            typeofTarget,
            position,
        };
    }

    const typeAnnotation = r.typeAnnotation();

    return { type: 'TypeAliasDeclaration', name, typeAnnotation, position };
}

// =============================================================================
// ENUM (ORDO) DECLARATIONS
// =============================================================================

/**
 * Parse enum declaration.
 *
 * GRAMMAR:
 *   enumDecl := 'ordo' IDENTIFIER '{' enumMember (',' enumMember)* ','? '}'
 *   enumMember := IDENTIFIER ('=' ('-'? NUMBER | STRING))?
 *
 * WHY: Latin 'ordo' (order/rank) for enumerated constants.
 *
 * Examples:
 *   ordo color { rubrum, viridis, caeruleum }
 *   ordo status { pendens = 0, actum = 1, finitum = 2 }
 *   ordo offset { ante = -1, ad = 0, post = 1 }
 */
export function parseOrdoDeclaration(r: Resolver): OrdoDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('ordo', ParserErrorCode.ExpectedKeywordOrdo);

    const name = ctx.parseIdentifier();

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const members: OrdoMember[] = [];

    while (!ctx.check('RBRACE') && !ctx.isAtEnd()) {
        const memberPosition = ctx.peek().position;
        // WHY: Keywords like Oct, Dec can be used as enum member names
        const memberName = ctx.parseIdentifierOrKeyword();

        let value: Literal | undefined;

        if (ctx.match('EQUAL')) {
            // Expect a literal value (number or string), with optional leading minus
            const valuePosition = ctx.peek().position;
            const isNegative = ctx.match('MINUS');
            const valueTok = ctx.advance();

            if (valueTok.type === 'NUMBER') {
                const numValue = Number(valueTok.value);
                value = {
                    type: 'Literal',
                    value: isNegative ? -numValue : numValue,
                    raw: isNegative ? `-${valueTok.value}` : valueTok.value,
                    position: valuePosition,
                };
            }
            else if (valueTok.type === 'STRING') {
                if (isNegative) {
                    ctx.reportError(
                        ParserErrorCode.UnexpectedToken,
                        `Cannot use minus sign with string enum value`
                    );
                }
                value = {
                    type: 'Literal',
                    value: valueTok.value,
                    raw: valueTok.value,
                    position: valueTok.position,
                };
            }
            else {
                ctx.reportError(
                    ParserErrorCode.UnexpectedToken,
                    `Expected number or string for enum value, got ${valueTok.type}`
                );
            }
        }

        members.push({
            type: 'OrdoMember',
            name: memberName,
            value,
            position: memberPosition,
        });

        // Allow trailing comma
        if (!ctx.check('RBRACE')) {
            ctx.match('COMMA');
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'OrdoDeclaration', name, members, position };
}

// =============================================================================
// DISCRETIO (TAGGED UNION) DECLARATIONS
// =============================================================================

/**
 * Parse discretio (tagged union) declaration.
 *
 * GRAMMAR:
 *   discretioDecl := 'discretio' IDENTIFIER typeParams? '{' variant (',' variant)* ','? '}'
 *   variant := IDENTIFIER ('{' variantFields '}')?
 *   variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
 *
 * WHY: Latin 'discretio' (distinction) for tagged unions.
 *      Each variant has a compiler-managed tag for exhaustive pattern matching.
 *
 * Examples:
 *   discretio Event {
 *       Click { numerus x, numerus y }
 *       Keypress { textus key }
 *       Quit
 *   }
 *
 *   discretio Option<T> {
 *       Some { T value }
 *       None
 *   }
 */
export function parseDiscretioDeclaration(r: Resolver): DiscretioDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('discretio', ParserErrorCode.ExpectedKeywordDiscretio);

    const name = ctx.parseIdentifier();

    // Parse optional type parameters <T, U>
    let typeParameters: Identifier[] | undefined;

    if (ctx.match('LESS')) {
        typeParameters = [];

        do {
            typeParameters.push(ctx.parseIdentifier());
        } while (ctx.match('COMMA'));

        ctx.expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
    }

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const variants: VariantDeclaration[] = [];

    while (!ctx.check('RBRACE') && !ctx.isAtEnd()) {
        variants.push(parseVariantDeclaration(r));

        // Allow trailing comma or no separator between variants
        ctx.match('COMMA');
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'DiscretioDeclaration', name, typeParameters, variants, position };
}

/**
 * Parse a single variant within a discretio.
 *
 * GRAMMAR:
 *   variant := IDENTIFIER ('{' variantFields '}')?
 *   variantFields := (typeAnnotation IDENTIFIER (',' typeAnnotation IDENTIFIER)*)?
 *
 * WHY: Variant names are capitalized by convention (like type names).
 *      Fields use type-first syntax like genus fields.
 *
 * Examples:
 *   Click { numerus x, numerus y }  -> fields with payload
 *   Quit                            -> unit variant (no payload)
 */
export function parseVariantDeclaration(r: Resolver): VariantDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    const name = ctx.parseIdentifier();

    const fields: VariantField[] = [];

    // Check for payload: Variant { fields }
    if (ctx.match('LBRACE')) {
        // Parse fields until closing brace
        while (!ctx.check('RBRACE') && !ctx.isAtEnd()) {
            const fieldPosition = ctx.peek().position;
            const fieldType = r.typeAnnotation();
            const fieldName = ctx.parseIdentifierOrKeyword();

            fields.push({
                type: 'VariantField',
                name: fieldName,
                fieldType,
                position: fieldPosition,
            });

            // Allow comma between fields
            if (!ctx.check('RBRACE')) {
                ctx.match('COMMA');
            }
        }

        ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);
    }

    return { type: 'VariantDeclaration', name, fields, position };
}

// =============================================================================
// GENUS (STRUCT) DECLARATIONS
// =============================================================================

/**
 * Synchronize within genus body after parse error.
 *
 * WHY: Looks for field types, method declarations, or closing brace
 *      to recover from malformed syntax like `fixum name: textus` (TS-style).
 */
function synchronizeGenusMember(r: Resolver): void {
    const ctx = r.ctx();
    ctx.advance();
    let braceDepth = 0;

    while (!ctx.isAtEnd()) {
        if (ctx.check('LBRACE')) {
            braceDepth++;
            ctx.advance();
            continue;
        }

        if (ctx.check('RBRACE')) {
            if (braceDepth === 0) {
                return;
            }
            braceDepth--;
            ctx.advance();
            continue;
        }

        // Stop at tokens that could start a new member (only at genus-body depth)
        if (
            braceDepth === 0 &&
            (ctx.check('AT') ||
                ctx.checkKeyword('functio') ||
                ctx.checkKeyword('publicus') ||
                ctx.checkKeyword('privatus') ||
                ctx.checkKeyword('protectus') ||
                ctx.checkKeyword('abstractus') ||
                ctx.checkKeyword('generis') ||
                // Type annotations may begin with borrow prepositions or a type
                ctx.checkKeyword('de') ||
                ctx.checkKeyword('in') ||
                ctx.check('LPAREN') ||
                ctx.isTypeName(ctx.peek()) ||
                ctx.check('IDENTIFIER'))
        ) {
            return;
        }

        ctx.advance();
    }
}

/**
 * Parse genus (struct) declaration.
 *
 * GRAMMAR:
 *   genusDecl := 'abstractus'? 'genus' IDENTIFIER typeParams? ('sub' IDENTIFIER)? ('implet' IDENTIFIER (',' IDENTIFIER)*)? '{' genusMember* '}'
 *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
 *   genusMember := fieldDecl | methodDecl
 *
 * WHY: Latin 'genus' (kind/type) for data structures.
 *      'sub' (under) for inheritance - child is under parent.
 *      'implet' (fulfills) for implementing pactum interfaces.
 *      'abstractus' for abstract classes that cannot be instantiated.
 */
export function parseGenusDeclaration(r: Resolver): GenusDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('genus', ParserErrorCode.ExpectedKeywordGenus);

    const name = ctx.parseIdentifier();

    // Parse optional type parameters <T, U>
    let typeParameters: Identifier[] | undefined;

    if (ctx.match('LESS')) {
        typeParameters = [];

        do {
            typeParameters.push(ctx.parseIdentifier());
        } while (ctx.match('COMMA'));

        ctx.expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
    }

    // Parse optional 'sub' clause (inheritance)
    let extendsClause: Identifier | undefined;

    if (ctx.matchKeyword('sub')) {
        extendsClause = ctx.parseIdentifier();
    }

    // Parse optional 'implet' clause
    let implementsList: Identifier[] | undefined;

    if (ctx.matchKeyword('implet')) {
        implementsList = [];

        do {
            implementsList.push(ctx.parseIdentifier());
        } while (ctx.match('COMMA'));
    }

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const fields: FieldDeclaration[] = [];
    const methods: FunctioDeclaration[] = [];
    let constructorMethod: FunctioDeclaration | undefined;

    // True while there are unparsed members (not at '}' or EOF)
    const hasMoreMembers = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreMembers()) {
        const startPosition = ctx.current;
        const member = parseGenusMember(r);

        // EDGE: If parser didn't advance, synchronize to avoid infinite loop.
        // This happens with malformed syntax like `fixum name: textus` (TS-style).
        if (ctx.current === startPosition) {
            synchronizeGenusMember(r);
            continue;
        }

        switch (member.type) {
            case 'FieldDeclaration':
                fields.push(member);
                break;

            case 'FunctioDeclaration':
                if (member.isConstructor) {
                    constructorMethod = member;
                }
                else {
                    methods.push(member);
                }
                break;

            default: {
                const _exhaustive: never = member;
                throw new Error(`Unknown genus member type: ${(_exhaustive as any).type}`);
            }
        }
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return {
        type: 'GenusDeclaration',
        name,
        typeParameters,
        extends: extendsClause,
        implements: implementsList,
        isAbstract: false, // Semantic analyzer extracts from annotations
        fields,
        constructor: constructorMethod,
        methods,
        position,
    };
}

/**
 * Parse a member of a genus (field or method).
 *
 * GRAMMAR:
 *   genusMember := annotation* (fieldDecl | methodDecl)
 *   annotation := '@' IDENTIFIER+
 *   fieldDecl := 'generis'? 'nexum'? typeAnnotation IDENTIFIER (':' expression)?
 *   methodDecl := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause? blockStmt?
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'immutata'
 *                | 'iacit'
 *   returnClause := '->' typeAnnotation
 *
 * NOTE: fit/fiet/fiunt/fient return syntax disabled pending Go/Zig backend support.
 *
 * WHY: Distinguishes between fields and methods by looking for 'functio' keyword.
 * WHY: Fields are public by default (struct semantics).
 * WHY: Use annotations for visibility: @ privatum, @ protectum.
 * WHY: Use annotations for abstract methods: @ abstracta (no body, must be overridden).
 */
export function parseGenusMember(r: Resolver): FieldDeclaration | FunctioDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Parse any leading annotations (visibility, async, abstract, etc.)
    const annotations = r.annotations();

    // Parse inline modifiers that remain in signature
    let isStatic = false;

    if (ctx.matchKeyword('generis')) {
        isStatic = true;
    }

    // If we see 'functio', it's a method
    if (ctx.checkKeyword('functio')) {
        ctx.expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

        // WHY: Method names can be keywords in unambiguous contexts
        const methodName = ctx.parseIdentifierOrKeyword();

        ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

        const params = parseParameterList(r);

        ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

        const modifiers = parseFunctionModifiers(r);

        let returnType: TypeAnnotation | undefined;
        let returnVerb: ReturnVerb | undefined;

        // Parse return type with arrow only
        // NOTE: fit/fiet/fiunt/fient return syntax is disabled pending Go/Zig backend support.
        if (ctx.match('THIN_ARROW')) {
            returnType = r.typeAnnotation();
            returnVerb = 'arrow';
        }

        // Abstract methods (from annotation) have no body
        const isAbstract = annotations.some(
            a => a.name === 'abstractum' || a.name === 'abstracta' || a.name === 'abstractus'
        );

        let body: BlockStatement | undefined;
        if (!isAbstract) {
            body = r.block();
        }

        const method: FunctioDeclaration = {
            type: 'FunctioDeclaration',
            name: methodName,
            params,
            returnType,
            body,
            async: false,
            generator: false,
            modifiers,
            isAbstract: isAbstract || undefined,
            position,
            annotations: annotations.length > 0 ? annotations : undefined,
        };

        if (methodName.name === 'creo') {
            method.isConstructor = true;
        }

        return method;
    }

    // Otherwise it's a field: type name with optional ':' default
    const fieldType = r.typeAnnotation();
    const fieldName = ctx.parseIdentifierOrKeyword();

    let init: Expression | undefined;

    // WHY: Field defaults use ':' (declarative "has value") not '=' (imperative "assign")
    // This aligns with object literal syntax: { nomen: "Marcus" }
    if (ctx.match('COLON')) {
        init = r.expression();
    }

    return {
        type: 'FieldDeclaration',
        name: fieldName,
        fieldType,
        init,
        visibility: 'public', // Default; semantic analyzer extracts from annotations
        isStatic,
        position,
        annotations: annotations.length > 0 ? annotations : undefined,
    };
}

// =============================================================================
// PACTUM (INTERFACE) DECLARATIONS
// =============================================================================

/**
 * Parse interface declaration.
 *
 * GRAMMAR:
 *   pactumDecl := 'pactum' IDENTIFIER typeParams? '{' pactumMethod* '}'
 *   typeParams := '<' IDENTIFIER (',' IDENTIFIER)* '>'
 *
 * WHY: Latin 'pactum' (agreement/contract) for interfaces.
 *      Defines method signatures that genus types can implement via 'implet'.
 *
 * Examples:
 *   pactum Legibilis { functio lege() -> textus }
 *   pactum Mappabilis<T, U> { functio mappa(T valor) -> U }
 */
export function parsePactumDeclaration(r: Resolver): PactumDeclaration {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    ctx.expectKeyword('pactum', ParserErrorCode.ExpectedKeywordPactum);

    const name = ctx.parseIdentifier();

    let typeParameters: Identifier[] | undefined;

    if (ctx.match('LESS')) {
        typeParameters = [];

        do {
            typeParameters.push(ctx.parseIdentifier());
        } while (ctx.match('COMMA'));

        ctx.expect('GREATER', ParserErrorCode.ExpectedClosingAngle);
    }

    ctx.expect('LBRACE', ParserErrorCode.ExpectedOpeningBrace);

    const methods: PactumMethod[] = [];

    // True while there are unparsed methods (not at '}' or EOF)
    const hasMoreMethods = () => !ctx.check('RBRACE') && !ctx.isAtEnd();

    while (hasMoreMethods()) {
        methods.push(parsePactumMethod(r));
    }

    ctx.expect('RBRACE', ParserErrorCode.ExpectedClosingBrace);

    return { type: 'PactumDeclaration', name, typeParameters, methods, position };
}

/**
 * Parse interface method signature.
 *
 * GRAMMAR:
 *   pactumMethod := 'functio' IDENTIFIER '(' paramList ')' funcModifier* returnClause?
 *   funcModifier := 'curata' IDENTIFIER
 *                | 'errata' IDENTIFIER
 *                | 'immutata'
 *                | 'iacit'
 *   returnClause := '->' typeAnnotation
 *
 * NOTE: fit/fiet/fiunt/fient return syntax disabled pending Go/Zig backend support.
 *
 * WHY: Method signatures without bodies. Same syntax as function declarations
 *      but terminates after return type (no block).
 */
export function parsePactumMethod(r: Resolver): PactumMethod {
    const ctx = r.ctx();
    const position = ctx.peek().position;

    // Parse any leading annotations
    const annotations = r.annotations();

    ctx.expectKeyword('functio', ParserErrorCode.ExpectedKeywordFunctio);

    // WHY: Allow keywords as method names for consistency with genus methods
    const name = ctx.parseIdentifierOrKeyword();

    ctx.expect('LPAREN', ParserErrorCode.ExpectedOpeningParen);

    const params = parseParameterList(r);

    ctx.expect('RPAREN', ParserErrorCode.ExpectedClosingParen);

    const modifiers = parseFunctionModifiers(r);

    let returnType: TypeAnnotation | undefined;

    // Parse return type with arrow only
    // NOTE: fit/fiet/fiunt/fient return syntax is disabled pending Go/Zig backend support.
    if (ctx.match('THIN_ARROW')) {
        returnType = r.typeAnnotation();
    }

    return {
        type: 'PactumMethod',
        name,
        params,
        returnType,
        async: false,
        generator: false,
        modifiers,
        position,
        annotations: annotations.length > 0 ? annotations : undefined,
    };
}
