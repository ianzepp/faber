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
    ExpectedIdentifier = 'P010',
    ExpectedTypeName = 'P011',
    ExpectedThinArrow = 'P012',
    ExpectedArrow = 'P013',

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
    ExpectedKeywordCustodi = 'P031',
    ExpectedKeywordAdfirma = 'P032',
    ExpectedKeywordRedde = 'P033',
    ExpectedKeywordIace = 'P034',
    ExpectedKeywordScribe = 'P035',
    ExpectedKeywordTempta = 'P036',
    ExpectedKeywordCape = 'P037',
    ExpectedKeywordAliter = 'P038',
    ExpectedKeywordSecus = 'P039',
    ExpectedKeywordFac = 'P040',
    ExpectedKeywordFit = 'P041',
    ExpectedKeywordOrdo = 'P042',
    ExpectedKeywordProbandum = 'P044',
    ExpectedKeywordProba = 'P045',
    ExpectedKeywordCura = 'P046',
    ExpectedKeywordAnteOrPost = 'P047',

    // Module/source errors (P050-P059)
    ExpectedModuleName = 'P050',
    ExpectedString = 'P051',

    // Invalid construct errors (P100-P199)
    InvalidAssignmentTarget = 'P100',
    InvalidForLoopStart = 'P101',
    InvalidSwitchCaseStart = 'P102',
    InvalidGuardClauseStart = 'P103',
    UnexpectedToken = 'P104',

    // Generic errors (P190-P199)
    GenericError = 'P190',

    // Declaration errors (P200-P299)
    MissingFunctionName = 'P200',
    MissingParameterName = 'P201',
    MissingGenusName = 'P202',
    MissingPactumName = 'P203',
    MissingTypeAliasName = 'P204',
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
    [ParserErrorCode.ExpectedArrow]: {
        text: "Expected '=>'",
        help: 'Arrow functions require => between parameters and body (exempli gratia: (x) => x + 1).',
    },

    // Keyword expectation errors
    [ParserErrorCode.ExpectedKeywordEx]: {
        text: "Expected 'ex'",
        help: "For-of loops start with 'ex' (from/out of): ex lista pro item { ... }",
    },
    [ParserErrorCode.ExpectedKeywordImporta]: {
        text: "Expected 'importa'",
        help: "Import declarations use 'importa' after module name: ex norma importa scribe",
    },
    [ParserErrorCode.ExpectedKeywordFunctio]: {
        text: "Expected 'functio'",
        help: "Function declarations require 'functio' keyword (optionally preceded by 'futura' for async).",
    },
    [ParserErrorCode.ExpectedKeywordGenus]: {
        text: "Expected 'genus'",
        help: "Struct/class declarations use 'genus' keyword: genus Persona { ... }",
    },
    [ParserErrorCode.ExpectedKeywordPactum]: {
        text: "Expected 'pactum'",
        help: "Interface declarations use 'pactum' keyword: pactum Iterabilis { ... }",
    },
    [ParserErrorCode.ExpectedKeywordTypus]: {
        text: "Expected 'typus'",
        help: "Type alias declarations use 'typus' keyword: typus ID = textus",
    },
    [ParserErrorCode.ExpectedKeywordSi]: {
        text: "Expected 'si'",
        help: "Conditional statements and switch cases use 'si' (if): si x > 0 { ... }",
    },
    [ParserErrorCode.ExpectedKeywordDum]: {
        text: "Expected 'dum'",
        help: "While loops use 'dum' keyword: dum verum { ... }",
    },
    [ParserErrorCode.ExpectedKeywordPro]: {
        text: "Expected 'pro'",
        help: "For loops use 'pro' (for each) after iterable: ex lista pro item { ... }",
    },
    [ParserErrorCode.ExpectedKeywordIn]: {
        text: "Expected 'in'",
        help: "Mutation blocks use 'in' (into) keyword: in object { prop = value }",
    },
    [ParserErrorCode.ExpectedKeywordDe]: {
        text: "Expected 'de'",
        help: "For-in loops use 'de' (from/concerning) keyword: de tabula pro clavis { ... }",
    },
    [ParserErrorCode.ExpectedKeywordElige]: {
        text: "Expected 'elige'",
        help: "Switch statements use 'elige' (choose) keyword: elige status { si 'active' { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordCustodi]: {
        text: "Expected 'custodi'",
        help: "Guard statements use 'custodi' (guard) keyword: custodi { si error { ... } }",
    },
    [ParserErrorCode.ExpectedKeywordAdfirma]: {
        text: "Expected 'adfirma'",
        help: "Assert statements use 'adfirma' (affirm) keyword: adfirma x > 0",
    },
    [ParserErrorCode.ExpectedKeywordRedde]: {
        text: "Expected 'redde'",
        help: "Return statements use 'redde' (give back) keyword: redde valor",
    },
    [ParserErrorCode.ExpectedKeywordIace]: {
        text: "Expected 'iace'",
        help: "Throw statements use 'iace' (throw/hurl) keyword: iace erratum",
    },
    [ParserErrorCode.ExpectedKeywordScribe]: {
        text: "Expected 'scribe'",
        help: "Print statements use 'scribe' (write) keyword: scribe 'hello'",
    },
    [ParserErrorCode.ExpectedKeywordTempta]: {
        text: "Expected 'tempta'",
        help: "Try-catch blocks use 'tempta' (attempt) keyword: tempta { ... } cape e { ... }",
    },
    [ParserErrorCode.ExpectedKeywordCape]: {
        text: "Expected 'cape'",
        help: "Catch clauses use 'cape' (seize/capture) keyword: cape erratum { ... }",
    },
    [ParserErrorCode.ExpectedKeywordAliter]: {
        text: "Expected 'aliter'",
        help: "Else clauses and switch defaults use 'aliter' (otherwise): si x { ... } aliter { ... }",
    },
    [ParserErrorCode.ExpectedKeywordSecus]: {
        text: "Expected 'secus'",
        help: "Latin ternary expressions use 'secus' (otherwise) after 'sic' (thus): verum sic 1 secus 0",
    },
    [ParserErrorCode.ExpectedKeywordFac]: {
        text: "Expected 'fac'",
        help: "Block scopes use 'fac' (do) keyword: fac { ... } cape { ... }",
    },
    [ParserErrorCode.ExpectedKeywordFit]: {
        text: "Expected 'fit'",
        help: "Return type uses 'fit' (becomes): functio f() fit textus { }",
    },
    [ParserErrorCode.ExpectedKeywordOrdo]: {
        text: "Expected 'ordo'",
        help: "Enum declarations use 'ordo' (order): ordo color { rubrum, viridis, caeruleum }",
    },
    [ParserErrorCode.ExpectedKeywordProbandum]: {
        text: "Expected 'probandum'",
        help: 'Test suites use \'probandum\' (that which must be tested): probandum "Tokenizer" { ... }',
    },
    [ParserErrorCode.ExpectedKeywordProba]: {
        text: "Expected 'proba'",
        help: 'Test cases use \'proba\' (test!): proba "parses numbers" { ... }',
    },
    [ParserErrorCode.ExpectedKeywordCura]: {
        text: "Expected 'cura'",
        help: "Resource management blocks use 'cura' (care): cura ante { ... } or cura post { ... }",
    },
    [ParserErrorCode.ExpectedKeywordAnteOrPost]: {
        text: "Expected 'ante' or 'post'",
        help: "After 'cura', specify timing: 'ante' (before) or 'post' (after)",
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

    // Invalid construct errors
    [ParserErrorCode.InvalidAssignmentTarget]: {
        text: 'Invalid assignment target',
        help: 'Only identifiers and member expressions can be assigned to. Literals and operators cannot be assignment targets.',
    },
    [ParserErrorCode.InvalidForLoopStart]: {
        text: "Expected 'ex' or 'de' to start for loop",
        help: "For loops start with 'ex' (for-of) or 'de' (for-in): ex items pro item { ... } or de tabula pro clavis { ... }",
    },
    [ParserErrorCode.InvalidSwitchCaseStart]: {
        text: "Expected 'si' or 'aliter' in switch block",
        help: "Switch cases start with 'si' for conditions or 'aliter' for default: elige x { si 1 { ... } aliter { ... } }",
    },
    [ParserErrorCode.InvalidGuardClauseStart]: {
        text: "Expected 'si' in guard block",
        help: "Guard clauses must start with 'si' condition: custodi { si error { redde nihil } }",
    },
    [ParserErrorCode.UnexpectedToken]: {
        text: 'Unexpected token',
        help: 'Token not expected in this context. Check for syntax errors or missing operators/delimiters.',
    },

    // Generic errors
    [ParserErrorCode.GenericError]: {
        text: 'Syntax error',
        help: 'Check the code structure for syntax errors.',
    },

    // Declaration errors
    [ParserErrorCode.MissingFunctionName]: {
        text: 'Expected function name',
        help: 'Function declarations require a name: functio salve() { ... }',
    },
    [ParserErrorCode.MissingParameterName]: {
        text: 'Expected parameter name',
        help: 'Function parameters require names. Type-first syntax: functio f(textus nomen) or type inference: functio f(nomen)',
    },
    [ParserErrorCode.MissingGenusName]: {
        text: 'Expected genus name',
        help: 'Genus declarations require a name: genus Persona { ... }',
    },
    [ParserErrorCode.MissingPactumName]: {
        text: 'Expected pactum name',
        help: 'Pactum declarations require a name: pactum Iterabilis { ... }',
    },
    [ParserErrorCode.MissingTypeAliasName]: {
        text: 'Expected type alias name',
        help: 'Type alias declarations require a name: typus ID = textus',
    },
    [ParserErrorCode.MissingVariableName]: {
        text: 'Expected variable name',
        help: "Variable declarations require a name: varia x = 5 or fixum nomen = 'Marcus'",
    },
} as const;
