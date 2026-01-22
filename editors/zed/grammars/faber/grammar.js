// Faber tree-sitter grammar - highlighting only
// No structural parsing, just tokenization for syntax highlighting

module.exports = grammar({
    name: 'faber',

    extras: $ => [/\s+/],

    rules: {
        program: $ => repeat($._token),

        _token: $ =>
            choice(
                $.comment,
                $.annotation,
                $.directive,
                $.keyword_control,
                $.keyword_declaration,
                $.keyword_other,
                $.builtin_type,
                $.boolean,
                $.string,
                $.number,
                $.identifier,
                $.operator,
                $.punctuation,
            ),

        // Comments
        comment: $ => token(choice(seq('#', /.*/), seq('//', /.*/))),

        // Annotations: @ ... (line-based)
        annotation: $ => token(seq('@', /[^\n]*/)),

        // Directives: § ... (line-based)
        directive: $ => token(seq('§', /[^\n]*/)),

        // Control flow keywords
        keyword_control: $ =>
            choice(
                // Conditionals
                'si',
                'sin',
                'secus',
                'ergo',
                // Switch/match
                'elige',
                'discerne',
                'casu',
                'ceterum',
                // Loops
                'dum',
                'fac',
                'ex',
                'de',
                'pro',
                // Error handling
                'tempta',
                'cape',
                'demum',
                // Entry points
                'incipit',
                'incipiet',
                // Return/transfer
                'redde',
                'reddit',
                'rumpe',
                'perge',
                // Throw/panic
                'iace',
                'iacit',
                'mori',
                'moritor',
                // Guard
                'custodi',
                // Resource management
                'cura',
                // Mutation block
                'in',
                // Collection DSL
                'ab',
                'ubi',
                'prima',
                'ultima',
                'summa',
                // Endpoint dispatch
                'ad',
                'fit',
                'fiet',
                'fiunt',
                'fient',
                // No-op
                'tacet',
            ),

        // Declaration keywords
        keyword_declaration: $ =>
            choice(
                // Function
                'functio',
                'clausura',
                // Variables
                'fixum',
                'varia',
                'figendum',
                'variandum',
                // Types
                'genus',
                'pactum',
                'ordo',
                'discretio',
                'typus',
                // Modifiers
                'abstractus',
                'sub',
                'implet',
                'generis',
                'nexum',
                // Parameters
                'prae',
                'ceteri',
                // Testing
                'probandum',
                'proba',
                'praepara',
                'postpara',
                'praeparabit',
                'postparabit',
                // Imports
                'importa',
            ),

        // Other keywords
        keyword_other: $ =>
            choice(
                // Objects
                'novum',
                'ego',
                'finge',
                // Async
                'cede',
                'futura',
                // Spread
                'sparge',
                // Assert
                'adfirma',
                // Output
                'scribe',
                'vide',
                'mone',
                // Boolean operators
                'vel',
                'et',
                'aut',
                'non',
                'est',
                // Type casting
                'qua',
                'innatum',
                // Type conversion
                'numeratum',
                'fractatum',
                'textatum',
                'bivalentum',
                // Bitwise shift
                'sinistratum',
                'dextratum',
                // Unary operators
                'nulla',
                'nonnulla',
                'negativum',
                'positivum',
                // Format strings
                'scriptum',
                'praefixum',
                // Alias
                'ut',
                // Range
                'ante',
                'usque',
                'per',
                'intra',
                'inter',
                // Input
                'lege',
                'lineam',
                // Regex
                'sed',
                // Ternary
                'sic',
            ),

        // Built-in types
        builtin_type: $ =>
            choice(
                // Primitives
                'textus',
                'numerus',
                'fractus',
                'bivalens',
                'vacuum',
                'nihil',
                'ignotum',
                'numquam',
                'octeti',
                // Collections
                'lista',
                'tabula',
                'copia',
                'promissum',
                'cursor',
                'unio',
                // Radix types
                'Hex',
                'Oct',
                'Bin',
                'Dec',
            ),

        // Literals
        boolean: $ => choice('verum', 'falsum'),

        string: $ => choice(seq('"', repeat(choice(/[^"\\]/, $.escape_sequence)), '"'), seq("'", repeat(choice(/[^'\\]/, $.escape_sequence)), "'")),
        escape_sequence: $ => token.immediate(seq('\\', /./)),

        number: $ => /\d+(\.\d+)?([eE][+-]?\d+)?/,

        identifier: $ => /[a-zA-Z_][a-zA-Z0-9_]*/,

        operator: $ =>
            token(
                choice(
                    '->',
                    '?.',
                    '!.',
                    '==',
                    '!=',
                    '===',
                    '!==',
                    '<=',
                    '>=',
                    '&&',
                    '||',
                    '+=',
                    '-=',
                    '*=',
                    '/=',
                    '+',
                    '-',
                    '*',
                    '/',
                    '%',
                    '<',
                    '>',
                    '=',
                    '!',
                    '?',
                    ':',
                    '.',
                    '&',
                    '|',
                    '^',
                    '~',
                ),
            ),

        punctuation: $ => choice('(', ')', '[', ']', '{', '}', ',', ';'),
    },
});
