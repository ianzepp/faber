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
                'si',
                'sin',
                'secus',
                'ergo',
                'elige',
                'discerne',
                'casu',
                'ceterum',
                'dum',
                'fac',
                'ex',
                'de',
                'pro',
                'tempta',
                'cape',
                'demum',
                'incipit',
                'incipiet',
                'redde',
                'reddit',
                'rumpe',
                'perge',
                'iace',
                'iacit',
                'mori',
                'moritor',
            ),

        // Declaration keywords
        keyword_declaration: $ =>
            choice(
                'functio',
                'fixum',
                'varia',
                'figendum',
                'variandum',
                'genus',
                'pactum',
                'ordo',
                'discretio',
                'typus',
                'abstractus',
                'sub',
                'implet',
                'generis',
                'nexum',
                'probandum',
                'proba',
                'praepara',
                'postpara',
            ),

        // Other keywords
        keyword_other: $ =>
            choice(
                'novum',
                'ego',
                'finge',
                'cede',
                'sparge',
                'adfirma',
                'scribe',
                'vide',
                'mone',
                'vel',
                'et',
                'aut',
                'non',
                'est',
                'qua',
                'innatum',
            ),

        // Built-in types
        builtin_type: $ =>
            choice(
                'textus',
                'numerus',
                'fractus',
                'bivalens',
                'vacuum',
                'nihil',
                'ignotum',
                'numquam',
                'octeti',
                'lista',
                'tabula',
                'copia',
                'promissum',
                'cursor',
                'unio',
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
