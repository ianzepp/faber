/**
 * Parser Error Catalog - Centralized error definitions for syntactic analysis
 *
 * COMPILER PHASE
 * ==============
 * syntactic
 *
 * ARCHITECTURE
 * ============
 * This module centralizes all parser error messages into a typed catalog. Each error
 * has a unique P-prefixed code, descriptive text, and optional help for remediation.
 *
 * Error codes follow a logical grouping:
 * P001-P099: Token expectation errors (missing delimiters, keywords)
 * P100-P199: Invalid syntax errors (assignment targets, expressions)
 * P200-P299: Declaration errors (missing names, types)
 * P300-P399: Statement structure errors (incomplete constructs)
 *
 * WHY: Centralized errors enable:
 * - Testable error codes (assert on codes, not brittle strings)
 * - Consistent messaging across the parser
 * - Easy localization/customization of messages
 * - IDE tooling (hover for help text)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (constant definitions only)
 * OUTPUT: ParserErrorCode enum and PARSER_ERRORS catalog exported
 * ERRORS: N/A (compile-time type checking only)
 *
 * INVARIANTS
 * ==========
 * INV-1: Every ParserErrorCode has entry in PARSER_ERRORS catalog
 * INV-2: Error codes are P-prefixed and sequentially numbered
 * INV-3: text field is concise, help field provides guidance
 * INV-4: Catalog is const-asserted for type inference
 *
 * @module parser/errors
 */

// =============================================================================
// ERROR CODES
// =============================================================================

/**
 * Parser error codes with P prefix.
 *
 * DESIGN: Grouped by category for maintainability.
 */
export enum ParserErrorCode {
    // Token expectation errors (P001-P099)
    ExpectedClosingParen = 'P001',
    ExpectedClosingBrace = 'P002',
    ExpectedClosingBracket = 'P003',
    ExpectedClosingAngle = 'P004',
    ExpectedComma = 'P005',
    ExpectedColon = 'P006',
    ExpectedEqual = 'P007',
    ExpectedOpeningBrace = 'P008',
    ExpectedOpeningParen = 'P009',
    ExpectedOpeningBracket = 'P014',
    ExpectedIdentifier = 'P010',
    ExpectedTypeName = 'P011',
    ExpectedThinArrow = 'P012',

    // Keyword expectation errors (P020-P049)
    ExpectedKeywordEx = 'P020',
    ExpectedKeywordImporta = 'P021',
    ExpectedKeywordFunctio = 'P022',
    ExpectedKeywordGenus = 'P023',
    ExpectedKeywordPactum = 'P024',
    ExpectedKeywordTypus = 'P025',
    ExpectedKeywordSi = 'P026',
    ExpectedKeywordDum = 'P027',
    ExpectedKeywordPro = 'P028',
    ExpectedKeywordIn = 'P029',
    ExpectedKeywordDe = 'P043',
    ExpectedKeywordElige = 'P030',
    ExpectedKeywordCasu = 'P038',
    ExpectedKeywordCeterum = 'P056',
    ExpectedKeywordCustodi = 'P031',
    ExpectedKeywordAdfirma = 'P032',
    ExpectedKeywordRedde = 'P033',
    ExpectedKeywordIace = 'P034',
    ExpectedKeywordScribe = 'P035',
    ExpectedKeywordTempta = 'P036',
    ExpectedKeywordCape = 'P037',
    ExpectedKeywordSecus = 'P039',
    ExpectedKeywordFac = 'P040',
    ExpectedKeywordFit = 'P041',
    ExpectedKeywordOrdo = 'P042',
    ExpectedKeywordDiscretio = 'P048',
    ExpectedKeywordDiscerne = 'P049',
    ExpectedKeywordProbandum = 'P044',
    ExpectedKeywordProba = 'P045',
    ExpectedKeywordCura = 'P046',
    ExpectedKeywordAnteOrPost = 'P047',
    ExpectedKeywordAd = 'P052',
    ExpectedKeywordIncipit = 'P054',
    ExpectedKeywordIncipiet = 'P055',

    // Module/source errors (P050-P059)
    ExpectedModuleName = 'P050',
    ExpectedString = 'P051',
    ExpectedStringAfterSed = 'P053',

    // Invalid construct errors (P100-P199)
    InvalidAssignmentTarget = 'P100',
    InvalidExDeStart = 'P101',
    InvalidEligeCaseStart = 'P102',
    InvalidDiscerneCaseStart = 'P106',
    InvalidCustodiClauseStart = 'P103',
    UnexpectedToken = 'P104',
    ExpectedOpeningBraceOrParen = 'P105',

    // Generic errors (P190-P199)
    GenericError = 'P190',
    PrefixVerbConflict = 'P191',

    // Declaration errors (P200-P299)
    MissingFunctioName = 'P200',
    MissingParameterName = 'P201',
    MissingGenusName = 'P202',
    MissingPactumName = 'P203',
    MissingTypusName = 'P204',
    MissingVariableName = 'P205',
}

// =============================================================================
// ERROR CATALOG
// =============================================================================

/**
 * Parser error catalog with text and help for each code.
 *
 * DESIGN: Using 'as const' for type inference and exhaustiveness checking.
 *
 * INVARIANT: Every ParserErrorCode must have an entry here.
 */
export const PARSER_ERRORS = {
    // Token expectation errors
    [ParserErrorCode.ExpectedClosingParen]: {
        text: "Expected ')'",
        help: 'Parentheses must be balanced. Check for missing closing paren in function calls or grouped expressions.',
    },
    [ParserErrorCode.ExpectedClosingBrace]: {
        text: "Expected '}'",
        help: 'Braces must be balanced. Check for missing closing brace in blocks, objects, or declarations.',
    },
    [ParserErrorCode.ExpectedClosingBracket]: {
        text: "Expected ']'",
        help: 'Brackets must be balanced. Check for missing closing bracket in arrays or computed member access.',
    },
    [ParserErrorCode.ExpectedClosingAngle]: {
        text: "Expected '>'",
        help: 'Type parameters must be closed (exempli gratia: genus Arca<T>, lista<numerus>).',
    },
    [ParserErrorCode.ExpectedComma]: {
        text: "Expected ','",
        help: 'Use commas to separate items in lists (parameters, arguments, array elements, object properties).',
    },
    [ParserErrorCode.ExpectedColon]: {
        text: "Expected ':'",
        help: 'Use colon to separate keys from values in object literals, default field values, or as the alternate separator in ternary expressions: condition ? truthy : falsy',
    },
    [ParserErrorCode.ExpectedEqual]: {
        text: "Expected '='",
        help: 'Use equals to assign values in declarations or type aliases.',
    },
    [ParserErrorCode.ExpectedOpeningBrace]: {
        text: "Expected '{'",
        help: 'Block statements, function bodies, and type declarations require opening brace.',
    },
    [ParserErrorCode.ExpectedOpeningParen]: {
        text: "Expected '('",
        help: 'Function declarations and calls require opening parenthesis for parameters/arguments.',
    },
    [ParserErrorCode.ExpectedOpeningBracket]: {
        text: "Expected '['",
        help: 'Array destructuring patterns require opening bracket: fixum [a, b] = arr',
    },
    [ParserErrorCode.ExpectedIdentifier]: {
        text: 'Expected identifier',
        help: 'Identifier required here (variable name, function name, property name, etc.).',
    },
    [ParserErrorCode.ExpectedTypeName]: {
        text: 'Expected type name',
        help: 'Type annotation required (textus, numerus, bivalens, or user-defined type).',
    },
    [ParserErrorCode.ExpectedThinArrow]: {
        text: "Expected '->'",
        help: 'Use thin arrow (->) to specify return type (exempli gratia: functio salve() -> textus).',
    },

    // Keyword expectation errors
    [ParserErrorCode.ExpectedKeywordEx]: {
        text: "Expected 'ex' (from)",
        help: "For-of loops start with 'ex' (from/out of): ex lista pro item { ... }",
    },
    [ParserErrorCode.ExpectedKeywordImporta]: {
        text: "Expected import keyword 'importa'",
        help: "Import declarations use 'importa' after module name: ex norma importa scribe",
    },
    [ParserErrorCode.ExpectedKeywordFunctio]: {
        text: "Expected function keyword 'functio'",
        help: "Function declarations require 'functio' keyword (optionally preceded by 'futura' for async).",
    },
    [ParserErrorCode.ExpectedKeywordGenus]: {
        text: "Expected struct keyword 'genus'",
        help: "Struct/class declarations use 'genus' keyword: genus Persona { ... }",
    },
    [ParserErrorCode.ExpectedKeywordPactum]: {
        text: "Expected interface keyword 'pactum'",
        help: "Interface declarations use 'pactum' keyword: pactum Iterabilis { ... }",
    },
    [ParserErrorCode.ExpectedKeywordTypus]: {
        text: "Expected type alias keyword 'typus'",
        help: "Type alias declarations use 'typus' keyword: typus ID = textus",
    },
    [ParserErrorCode.ExpectedKeywordSi]: {
        text: "Expected condition keyword 'si' (if)",
        help: "Conditional statements use 'si' (if): si x > 0 { ... }",
    },
    [ParserErrorCode.ExpectedKeywordDum]: {
        text: "Expected while keyword 'dum'",
        help: "While loops use 'dum' keyword: dum verum { ... }",
    },
    [ParserErrorCode.ExpectedKeywordPro]: {
        text: "Expected 'pro' (for each)",
        help: "For loops use 'pro' (for each) after iterable: ex lista pro item { ... }",
    },
    [ParserErrorCode.ExpectedKeywordIn]: {
        text: "Expected mutation keyword 'in' (into)",
        help: "Mutation blocks use 'in' (into) keyword: in object { prop = value }",
    },
    [ParserErrorCode.ExpectedKeywordDe]: {
        text: "Expected 'de' (concerning)",
        help: "For-in loops use 'de' (from/concerning) keyword: de tabula pro clavis { ... }",
    },
    [ParserErrorCode.ExpectedKeywordElige]: {
        text: "Expected switch keyword 'elige' (choose)",
        help: "Switch statements use 'elige' (choose) keyword: elige status { casu 'active' { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordCasu]: {
        text: "Expected case keyword 'casu'",
        help: "Switch cases and match arms use 'casu': elige status { casu 'active' { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordCeterum]: {
        text: "Expected default keyword 'ceterum' (otherwise)",
        help: "Switch defaults use 'ceterum': elige status { casu 'active' { ... } ceterum { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordCustodi]: {
        text: "Expected guard keyword 'custodi'",
        help: "Guard statements use 'custodi' (guard) keyword: custodi { si error { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordAdfirma]: {
        text: "Expected assert keyword 'adfirma' (affirm)",
        help: "Assert statements use 'adfirma' (affirm) keyword: adfirma x > 0",
    },
    [ParserErrorCode.ExpectedKeywordRedde]: {
        text: "Expected return keyword 'redde'",
        help: "Return statements use 'redde' (give back) keyword: redde valor",
    },
    [ParserErrorCode.ExpectedKeywordIace]: {
        text: "Expected throw keyword 'iace'",
        help: "Throw statements use 'iace' (throw/hurl) keyword: iace erratum",
    },
    [ParserErrorCode.ExpectedKeywordScribe]: {
        text: "Expected print keyword 'scribe' (write)",
        help: "Print statements use 'scribe' (write) keyword: scribe 'hello'",
    },
    [ParserErrorCode.ExpectedKeywordTempta]: {
        text: "Expected try keyword 'tempta' (attempt)",
        help: "Try-catch blocks use 'tempta' (attempt) keyword: tempta { ... } cape e { ... }",
    },
    [ParserErrorCode.ExpectedKeywordCape]: {
        text: "Expected catch keyword 'cape' (seize)",
        help: "Catch clauses use 'cape' (seize/capture) keyword: cape erratum { ... }",
    },
    [ParserErrorCode.ExpectedKeywordSecus]: {
        text: "Expected 'secus' (otherwise)",
        help: "Use 'secus' for else clauses (si x { } secus { }) and ternary alternates (sic x secus y)",
    },
    [ParserErrorCode.ExpectedKeywordFac]: {
        text: "Expected do keyword 'fac'",
        help: "Block scopes use 'fac' (do) keyword: fac { ... } cape { ... }",
    },
    [ParserErrorCode.ExpectedKeywordFit]: {
        text: "Expected 'fit' (becomes)",
        help: "Return type uses 'fit' (becomes): functio f() fit textus { }",
    },
    [ParserErrorCode.ExpectedKeywordOrdo]: {
        text: "Expected enum keyword 'ordo' (order)",
        help: "Enum declarations use 'ordo' (order): ordo color { rubrum, viridis, caeruleum }",
    },
    [ParserErrorCode.ExpectedKeywordDiscretio]: {
        text: "Expected tagged union keyword 'discretio'",
        help: "Tagged union declarations use 'discretio' (distinction): discretio Event { Click { numerus x } }",
    },
    [ParserErrorCode.ExpectedKeywordDiscerne]: {
        text: "Expected match keyword 'discerne' (distinguish)",
        help: "Variant matching uses 'discerne' (distinguish!): discerne event { casu Click pro x, y { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordProbandum]: {
        text: "Expected test suite keyword 'probandum'",
        help: 'Test suites use \'probandum\' (that which must be tested): probandum "Tokenizer" { ... }',
    },
    [ParserErrorCode.ExpectedKeywordProba]: {
        text: "Expected test keyword 'proba'",
        help: 'Test cases use \'proba\' (test!): proba "parses numbers" { ... }',
    },
    [ParserErrorCode.ExpectedKeywordCura]: {
        text: "Expected resource keyword 'cura' (care)",
        help: "Resource management blocks use 'cura' (care): cura ante { ... } or cura post { ... }",
    },
    [ParserErrorCode.ExpectedKeywordAnteOrPost]: {
        text: "Expected 'ante' (before) or 'post' (after)",
        help: "After 'cura', specify timing: 'ante' (before) or 'post' (after)",
    },
    [ParserErrorCode.ExpectedKeywordAd]: {
        text: "Expected dispatch keyword 'ad' (to/toward)",
        help: 'Dispatch statements use \'ad\': ad "target" (args) fit Type pro name { ... }',
    },
    [ParserErrorCode.ExpectedKeywordIncipit]: {
        text: "Expected entry point keyword 'incipit' (it begins)",
        help: 'Entry point blocks use \'incipit\': incipit { scribe "Hello" }',
    },
    [ParserErrorCode.ExpectedKeywordIncipiet]: {
        text: "Expected async entry point keyword 'incipiet' (it will begin)",
        help: "Async entry point blocks use 'incipiet': incipiet { cede fetchData() }",
    },

    // Module/source errors
    [ParserErrorCode.ExpectedModuleName]: {
        text: "Expected module name after 'ex'",
        help: 'Import declarations require module name: ex norma importa scribe',
    },
    [ParserErrorCode.ExpectedString]: {
        text: 'Expected string literal',
        help: 'A quoted string is required here (exempli gratia: "test name").',
    },
    [ParserErrorCode.ExpectedStringAfterSed]: {
        text: "Expected pattern string after 'sed'",
        help: 'Regex literals require a pattern string: sed "\\\\d+" or sed "hello" i',
    },

    // Invalid construct errors
    [ParserErrorCode.InvalidAssignmentTarget]: {
        text: 'Invalid assignment target',
        help: 'Only identifiers and member expressions can be assigned to. Literals and operators cannot be assignment targets.',
    },
    [ParserErrorCode.InvalidExDeStart]: {
        text: "For loop 'pro' requires 'ex' (for-of) or 'de' (for-in)",
        help: "For loops start with 'ex' (for-of) or 'de' (for-in): ex items pro item { ... } or de tabula pro clavis { ... }",
    },
    [ParserErrorCode.InvalidEligeCaseStart]: {
        text: "Switch 'elige' cases must start with 'casu' or 'ceterum'",
        help: "Switch cases start with 'casu' for value matching or 'ceterum' for default: elige x { casu 1 { ... } ceterum { ... } }. For variant matching, use 'discerne' instead.",
    },
    [ParserErrorCode.InvalidDiscerneCaseStart]: {
        text: "Match 'discerne' cases must start with 'casu'",
        help: "Variant cases start with 'casu' followed by variant name: discerne event { casu Click pro x, y { ... } casu Quit { ... } }",
    },
    [ParserErrorCode.InvalidCustodiClauseStart]: {
        text: "Guard 'custodi' clauses must start with 'si'",
        help: "Guard clauses must start with 'si' condition: custodi { si error { redde nihil } }",
    },
    [ParserErrorCode.UnexpectedToken]: {
        text: 'Unexpected token',
        help: 'Token not expected in this context. Check for syntax errors or missing operators/delimiters.',
    },
    [ParserErrorCode.ExpectedOpeningBraceOrParen]: {
        text: "Expected '{' or '('",
        help: 'praefixum requires block { } or expression ( ): praefixum { ... } or praefixum(expr)',
    },

    // Generic errors
    [ParserErrorCode.GenericError]: {
        text: 'Syntax error',
        help: 'Check the code structure for syntax errors.',
    },
    [ParserErrorCode.PrefixVerbConflict]: {
        text: 'Cannot combine futura/cursor with fit/fiet/fiunt/fient',
        help: "Verbs (fit/fiet/fiunt/fient) use the stream protocol and cannot be combined with futura/cursor prefixes. Use '->' arrow with futura/cursor for traditional async/generator returns.",
    },

    // Declaration errors
    [ParserErrorCode.MissingFunctioName]: {
        text: "Function 'functio' requires a name",
        help: 'Function declarations require a name: functio salve() { ... }',
    },
    [ParserErrorCode.MissingParameterName]: {
        text: 'Expected parameter name',
        help: 'Function parameters require names. Type-first syntax: functio f(textus nomen) or type inference: functio f(nomen)',
    },
    [ParserErrorCode.MissingGenusName]: {
        text: "Struct 'genus' requires a name",
        help: 'Genus declarations require a name: genus Persona { ... }',
    },
    [ParserErrorCode.MissingPactumName]: {
        text: "Interface 'pactum' requires a name",
        help: 'Pactum declarations require a name: pactum Iterabilis { ... }',
    },
    [ParserErrorCode.MissingTypusName]: {
        text: "Type alias 'typus' requires a name",
        help: 'Type alias declarations require a name: typus ID = textus',
    },
    [ParserErrorCode.MissingVariableName]: {
        text: "Variable 'varia'/'fixum' requires a name",
        help: "Variable declarations require a name: varia x = 5 or fixum nomen = 'Marcus'",
    },
} as const;
