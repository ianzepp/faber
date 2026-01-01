/**
 * Tree-sitter grammar for Faber Romanus
 *
 * A Latin programming language that compiles to TypeScript and Zig.
 * Aligned with fons/parser/ast.ts
 */

const PREC = {
    COMMA: -1,
    ASSIGN: 0,
    TERNARY: 1,
    OR: 2,
    AND: 3,
    EQUAL: 4,
    COMPARE: 5,
    RANGE: 6,
    ADD: 7,
    MULT: 8,
    UNARY: 9,
    CALL: 10,
    MEMBER: 11,
};

module.exports = grammar({
    name: 'faber_romanus',

    extras: $ => [/\s/, $.comment],

    externals: $ => [$._automatic_semicolon],

    supertypes: $ => [$._expression, $._statement],

    inline: $ => [$._statement, $._expressions],

    conflicts: $ => [
        [$.return_statement],
        [$.primary_expression, $.arrow_function_parameter],
        [$.block_statement, $.object_expression],
        [$.type_annotation],
    ],

    word: $ => $.identifier,

    rules: {
        // ==========================================================================
        // Program
        // ==========================================================================

        program: $ => repeat($._statement),

        // ==========================================================================
        // Statements
        // ==========================================================================

        _statement: $ =>
            choice(
                $.import_declaration,
                $.variable_declaration,
                $.function_declaration,
                $.type_alias_declaration,
                $.enum_declaration,
                $.genus_declaration,
                $.pactum_declaration,
                $.if_statement,
                $.while_statement,
                $.for_statement,
                $.switch_statement,
                $.guard_statement,
                $.assert_statement,
                $.try_statement,
                $.return_statement,
                $.break_statement,
                $.continue_statement,
                $.throw_statement,
                $.scribe_statement,
                $.fac_block_statement,
                $.block_statement,
                $.expression_statement,
            ),

        expression_statement: $ => seq($._expression, optional($._automatic_semicolon)),

        // Import: ex module importa name1, name2 (or *)
        import_declaration: $ => seq('ex', field('source', $.identifier), 'importa', choice('*', sep1(field('specifier', $.identifier), ','))),

        // Variable: varia/fixum [type] name = value (type-first, optional)
        variable_declaration: $ =>
            seq(
                field('kind', choice('varia', 'fixum', 'figendum', 'variandum')),
                choice(
                    // Type-first: fixum textus name
                    seq(field('type', $.type_annotation), field('name', $.identifier)),
                    // Type inference: fixum name
                    field('name', $.identifier),
                    // Destructuring: fixum { a, b }
                    field('pattern', $.object_pattern),
                ),
                optional(seq('=', field('value', $._expression))),
            ),

        object_pattern: $ => seq('{', sep($.object_pattern_property, ','), '}'),

        object_pattern_property: $ => seq(field('key', $.identifier), optional(seq(':', field('value', $.identifier)))),

        // Function: [futura] functio name(params) [-> ReturnType] { body }
        function_declaration: $ =>
            seq(
                optional(field('async', 'futura')),
                optional(field('generator', 'cursor')),
                'functio',
                field('name', $.identifier),
                field('parameters', $.formal_parameters),
                optional(seq('->', field('return_type', $.type_annotation))),
                field('body', $.block_statement),
            ),

        formal_parameters: $ => seq('(', sep($.parameter, ','), ')'),

        // Parameter: [preposition] [Type] name
        parameter: $ =>
            seq(
                optional(field('preposition', choice('ad', 'de', 'in', 'ex'))),
                optional(field('type', $.type_annotation)),
                field('name', $.identifier),
            ),

        // Type alias: typus Name = Type
        type_alias_declaration: $ => seq('typus', field('name', $.identifier), '=', field('type', $.type_annotation)),

        // Enum: ordo Name { member1, member2 = value }
        enum_declaration: $ => seq('ordo', field('name', $.identifier), '{', sep($.enum_member, ','), optional(','), '}'),

        enum_member: $ => seq(field('name', $.identifier), optional(seq('=', field('value', $.literal)))),

        // Genus (class/struct): genus Name<T> implet Interface { fields, methods }
        genus_declaration: $ =>
            seq(
                'genus',
                field('name', $.identifier),
                optional(field('type_parameters', $.type_parameters)),
                optional(seq('implet', sep1($.identifier, ','))),
                '{',
                repeat(choice($.field_declaration, $.computed_field_declaration, $.function_declaration)),
                '}',
            ),

        type_parameters: $ => seq('<', sep1($.identifier, ','), '>'),

        // Field: [publicus] [generis] [nexum] Type name [: default]
        field_declaration: $ =>
            seq(
                optional('publicus'),
                optional('generis'),
                optional('nexum'),
                field('type', $.type_annotation),
                field('name', $.identifier),
                optional(seq(':', field('default', $._expression))),
            ),

        // Computed field: [publicus] [generis] Type name => expression
        computed_field_declaration: $ =>
            seq(
                optional('publicus'),
                optional('generis'),
                field('type', $.type_annotation),
                field('name', $.identifier),
                '=>',
                field('expression', $._expression),
            ),

        // Pactum (interface): pactum Name<T> { method signatures }
        pactum_declaration: $ =>
            seq('pactum', field('name', $.identifier), optional(field('type_parameters', $.type_parameters)), '{', repeat($.pactum_method), '}'),

        // Pactum method signature (no body)
        pactum_method: $ =>
            seq(
                optional(field('async', 'futura')),
                optional(field('generator', 'cursor')),
                'functio',
                field('name', $.identifier),
                field('parameters', $.formal_parameters),
                optional(seq('->', field('return_type', $.type_annotation))),
            ),

        // If: si condition { } [cape err { }] [secus { }]
        // Or: si condition ergo statement [secus ...]
        if_statement: $ =>
            prec.right(
                choice(
                    // Block form with optional catch
                    seq(
                        'si',
                        field('condition', $._expression),
                        field('consequence', $.block_statement),
                        optional(field('catch', $.catch_clause)),
                        optional(field('alternative', $.else_clause)),
                    ),
                    // One-liner form (no catch)
                    seq(
                        'si',
                        field('condition', $._expression),
                        'ergo',
                        field('consequence', $._statement),
                        optional(field('alternative', $.else_clause)),
                    ),
                ),
            ),

        else_clause: $ => seq('secus', choice($.if_statement, $.block_statement, $._statement)),

        // While: dum condition { } [cape err { }]
        // Or: dum condition ergo statement
        while_statement: $ =>
            choice(
                seq('dum', field('condition', $._expression), field('body', $.block_statement), optional(field('catch', $.catch_clause))),
                seq('dum', field('condition', $._expression), 'ergo', field('body', $._statement)),
            ),

        // For: ex/in iterable pro/fit/fiet variable { } [cape err { }]
        // Or: ex/in iterable pro variable ergo statement
        for_statement: $ =>
            choice(
                seq(
                    field('kind', choice('ex', 'in')),
                    field('iterable', $._expression),
                    field('binding', choice('pro', 'fit', 'fiet')),
                    field('variable', $.identifier),
                    field('body', $.block_statement),
                    optional(field('catch', $.catch_clause)),
                ),
                seq(
                    field('kind', choice('ex', 'in')),
                    field('iterable', $._expression),
                    'pro',
                    field('variable', $.identifier),
                    'ergo',
                    field('body', $._statement),
                ),
            ),

        // Switch: elige expr { si val ergo/{ } ... secus { } }
        switch_statement: $ =>
            seq(
                'elige',
                field('discriminant', $._expression),
                '{',
                repeat($.switch_case),
                optional($.switch_default),
                '}',
                optional(field('catch', $.catch_clause)),
            ),

        switch_case: $ =>
            choice(
                seq('si', field('test', $._expression), field('consequent', $.block_statement)),
                seq('si', field('test', $._expression), 'ergo', field('consequent', $._statement)),
            ),

        switch_default: $ => seq('secus', choice($.block_statement, $._statement)),

        // Guard: custodi { si cond { } ... }
        guard_statement: $ => seq('custodi', '{', repeat($.guard_clause), '}'),

        guard_clause: $ => seq('si', field('test', $._expression), field('consequent', $.block_statement)),

        // Assert: adfirma condition [, message]
        assert_statement: $ => seq('adfirma', field('test', $._expression), optional(seq(',', field('message', $._expression)))),

        // Try: tempta { } [cape err { }] [demum { }]
        try_statement: $ =>
            seq(
                'tempta',
                field('body', $.block_statement),
                optional(field('handler', $.catch_clause)),
                optional(field('finalizer', $.finally_clause)),
            ),

        catch_clause: $ => seq('cape', field('parameter', $.identifier), field('body', $.block_statement)),

        finally_clause: $ => seq('demum', field('body', $.block_statement)),

        return_statement: $ => seq('redde', optional($._expression)),

        break_statement: $ => 'rumpe',

        continue_statement: $ => 'perge',

        throw_statement: $ => seq(choice('iace', 'mori'), $._expression),

        // Scribe: scribe/vide/mone expr [, expr, ...]
        scribe_statement: $ => seq(choice('scribe', 'vide', 'mone'), sep1($._expression, ',')),

        // Fac block: fac { } [cape err { }]
        fac_block_statement: $ => seq('fac', field('body', $.block_statement), optional(field('catch', $.catch_clause))),

        block_statement: $ => seq('{', repeat($._statement), '}'),

        // ==========================================================================
        // Types
        // ==========================================================================

        type_annotation: $ =>
            prec.right(
                seq(
                    optional(field('preposition', choice('de', 'in'))),
                    field('name', $.type_identifier),
                    optional(field('arguments', $.type_arguments)),
                    optional('?'),
                    optional(seq('|', $.type_annotation)),
                ),
            ),

        // Type names: case-insensitive (textus, textus, TEXTUS all work)
        // Convention: lowercase preferred (Latin had no case distinction)
        type_identifier: $ => /[A-Za-z][a-zA-Z0-9_]*/,

        type_arguments: $ => seq('<', sep1(choice($.type_annotation, $.number, $.type_modifier), ','), '>'),

        type_modifier: $ => choice('naturalis', 'proprius', 'alienus', 'mutabilis'),

        // ==========================================================================
        // Expressions
        // ==========================================================================

        _expression: $ =>
            choice(
                $.assignment_expression,
                $.conditional_expression,
                $.binary_expression,
                $.unary_expression,
                $.await_expression,
                $.call_expression,
                $.member_expression,
                $.subscript_expression,
                $.arrow_function_expression,
                $.fac_expression,
                $.new_expression,
                $.range_expression,
                $.parenthesized_expression,
                $.primary_expression,
            ),

        primary_expression: $ => choice($.identifier, $.this_expression, $.literal, $.template_literal, $.array_expression, $.object_expression),

        _expressions: $ => choice($._expression, $.sequence_expression),

        sequence_expression: $ => prec(PREC.COMMA, seq($._expression, ',', choice($.sequence_expression, $._expression))),

        assignment_expression: $ =>
            prec.right(
                PREC.ASSIGN,
                seq(field('left', choice($.identifier, $.member_expression, $.subscript_expression)), '=', field('right', $._expression)),
            ),

        // Ternary: condition sic consequent secus alternate
        // Or: condition ? consequent : alternate
        conditional_expression: $ =>
            prec.right(
                PREC.TERNARY,
                choice(
                    seq(field('test', $._expression), 'sic', field('consequent', $._expression), 'secus', field('alternate', $._expression)),
                    seq(field('test', $._expression), '?', field('consequent', $._expression), ':', field('alternate', $._expression)),
                ),
            ),

        binary_expression: $ =>
            choice(
                ...[
                    [choice('aut', '||'), PREC.OR],
                    [choice('et', '&&'), PREC.AND],
                    [choice('==', '!=', 'est'), PREC.EQUAL],
                    [choice('<', '<=', '>', '>='), PREC.COMPARE],
                    [choice('+', '-'), PREC.ADD],
                    [choice('*', '/', '%'), PREC.MULT],
                ].map(([op, p]) => prec.left(p, seq(field('left', $._expression), field('operator', op), field('right', $._expression)))),
            ),

        unary_expression: $ =>
            prec.right(
                PREC.UNARY,
                seq(field('operator', choice('!', 'non', '-', '+', 'nulla', 'nonnulla', 'negativum', 'positivum')), field('argument', $._expression)),
            ),

        await_expression: $ => prec.right(PREC.UNARY, seq('cede', $._expression)),

        call_expression: $ => prec(PREC.CALL, seq(field('function', $._expression), field('arguments', $.arguments))),

        arguments: $ => seq('(', sep($._expression, ','), ')'),

        member_expression: $ => prec(PREC.MEMBER, seq(field('object', $._expression), '.', field('property', $.identifier))),

        subscript_expression: $ => prec(PREC.MEMBER, seq(field('object', $._expression), '[', field('index', $._expression), ']')),

        // Arrow function: (params) => body
        arrow_function_expression: $ =>
            prec.right(
                seq(
                    optional(field('async', 'futura')),
                    choice($.arrow_function_parameter, seq('(', sep($.arrow_function_parameter, ','), ')')),
                    '=>',
                    field('body', choice($._expression, $.block_statement)),
                ),
            ),

        arrow_function_parameter: $ => seq(optional(field('type', $.type_annotation)), field('name', $.identifier)),

        // Fac expression (lambda): pro params redde expr
        fac_expression: $ => prec.right(seq('pro', optional(sep1($.identifier, ',')), 'redde', field('body', $._expression))),

        new_expression: $ =>
            prec.right(
                PREC.CALL,
                seq('novum', field('callee', $.identifier), optional($.arguments), optional(seq('de', field('with', $.object_expression)))),
            ),

        // Range: start..end [per step]
        range_expression: $ =>
            prec.left(
                PREC.RANGE,
                seq(field('start', $._expression), '..', field('end', $._expression), optional(seq('per', field('step', $._expression)))),
            ),

        parenthesized_expression: $ => seq('(', $._expression, ')'),

        // ==========================================================================
        // Literals
        // ==========================================================================

        identifier: $ => /[a-z_][a-zA-Z0-9_]*/,

        this_expression: $ => 'ego',

        // Unified literal for numbers, strings, booleans, null
        literal: $ => choice($.number, $.string, $.true, $.false, $.null),

        number: $ => /\d+(\.\d+)?/,

        string: $ => choice(seq('"', repeat(choice(/[^"\\]/, /\\./)), '"'), seq("'", repeat(choice(/[^'\\]/, /\\./)), "'")),

        template_literal: $ => seq('`', repeat(choice($._template_chars, $.template_substitution)), '`'),

        _template_chars: $ => token(prec(-1, /[^`$]+/)),

        template_substitution: $ => seq('${', $._expression, '}'),

        true: $ => 'verum',
        false: $ => 'falsum',
        null: $ => 'nihil',

        array_expression: $ => seq('[', sep($._expression, ','), ']'),

        object_expression: $ => seq('{', sep($.object_property, ','), '}'),

        object_property: $ => seq(field('key', choice($.identifier, $.string)), ':', field('value', $._expression)),

        // ==========================================================================
        // Comments
        // ==========================================================================

        comment: $ => token(choice(seq('//', /.*/), seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/'))),
    },
});

// Helper: separated list (zero or more)
function sep(rule, separator) {
    return optional(sep1(rule, separator));
}

// Helper: separated list (one or more)
function sep1(rule, separator) {
    return seq(rule, repeat(seq(separator, rule)));
}
