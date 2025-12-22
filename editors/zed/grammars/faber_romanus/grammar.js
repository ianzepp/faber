/**
 * Tree-sitter grammar for Faber Romanus
 *
 * A Latin programming language that compiles to TypeScript and Zig.
 * This is a simplified grammar focused on syntax highlighting.
 */

const PREC = {
  COMMA: -1,
  ASSIGN: 0,
  OR: 1,
  AND: 2,
  EQUAL: 3,
  COMPARE: 4,
  ADD: 5,
  MULT: 6,
  UNARY: 7,
  CALL: 8,
  MEMBER: 9,
};

module.exports = grammar({
  name: "faber_romanus",

  extras: $ => [
    /\s/,
    $.comment,
  ],

  externals: $ => [
    $._automatic_semicolon,
  ],

  supertypes: $ => [
    $._expression,
    $._statement,
  ],

  inline: $ => [
    $._statement,
    $._expressions,
  ],

  conflicts: $ => [
    [$.return_statement],
    [$.primary_expression, $.arrow_function_parameter],
    [$.statement_block, $.object],
    [$.call_expression, $.new_expression],
  ],

  word: $ => $.identifier,

  rules: {
    // ==========================================================================
    // Program
    // ==========================================================================

    source_file: $ => repeat($._statement),

    // ==========================================================================
    // Statements
    // ==========================================================================

    _statement: $ => choice(
      $.import_declaration,
      $.variable_declaration,
      $.function_declaration,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.try_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.throw_statement,
      $.statement_block,
      $.expression_statement,
    ),

    expression_statement: $ => seq(
      $._expression,
      optional($._automatic_semicolon),
    ),

    // Import: ex "module" importa name1, name2
    import_declaration: $ => seq(
      "ex",
      field("source", $.string),
      "importa",
      choice(
        seq("*", optional(seq("cum", field("alias", $.identifier)))),
        sep1(field("specifier", $.identifier), ","),
      ),
    ),

    // Variable: esto/fixum name: Type = value
    variable_declaration: $ => seq(
      field("kind", choice("esto", "fixum")),
      field("name", $.identifier),
      optional(seq(":", field("type", $.type))),
      optional(seq("=", field("value", $._expression))),
    ),

    // Function: [futura] functio name(params): ReturnType { body }
    function_declaration: $ => seq(
      optional(field("async", "futura")),
      "functio",
      field("name", $.identifier),
      field("parameters", $.formal_parameters),
      optional(seq(":", field("return_type", $.type))),
      field("body", $.statement_block),
    ),

    formal_parameters: $ => seq(
      "(",
      sep($.formal_parameter, ","),
      ")",
    ),

    formal_parameter: $ => seq(
      field("name", $.identifier),
      optional(seq(":", field("type", $.type))),
    ),

    // Control flow
    if_statement: $ => prec.right(seq(
      "si",
      field("condition", $._expression),
      field("consequence", $.statement_block),
      optional(field("alternative", $.else_clause)),
    )),

    else_clause: $ => seq(
      "aliter",
      choice($.if_statement, $.statement_block),
    ),

    while_statement: $ => seq(
      "dum",
      field("condition", $._expression),
      field("body", $.statement_block),
    ),

    for_statement: $ => seq(
      "pro",
      field("iterator", $.identifier),
      "in",
      field("iterable", $._expression),
      field("body", $.statement_block),
    ),

    try_statement: $ => seq(
      "tempta",
      field("body", $.statement_block),
      optional($.catch_clause),
      optional($.finally_clause),
    ),

    catch_clause: $ => seq(
      "cape",
      optional(field("parameter", $.identifier)),
      field("body", $.statement_block),
    ),

    finally_clause: $ => seq(
      "demum",
      field("body", $.statement_block),
    ),

    return_statement: $ => seq(
      "redde",
      optional($._expression),
    ),

    break_statement: $ => "rumpe",

    continue_statement: $ => "perge",

    throw_statement: $ => seq(
      "iace",
      $._expression,
    ),

    statement_block: $ => seq(
      "{",
      repeat($._statement),
      "}",
    ),

    // ==========================================================================
    // Types
    // ==========================================================================

    type: $ => seq(
      field("name", $.type_identifier),
      optional(field("arguments", $.type_arguments)),
      optional("?"),
    ),

    type_identifier: $ => /[A-Z][a-zA-Z0-9_]*/,

    type_arguments: $ => seq(
      "<",
      sep1($.type, ","),
      ">",
    ),

    // ==========================================================================
    // Expressions
    // ==========================================================================

    _expression: $ => choice(
      $.assignment_expression,
      $.binary_expression,
      $.unary_expression,
      $.await_expression,
      $.call_expression,
      $.member_expression,
      $.subscript_expression,
      $.arrow_function,
      $.new_expression,
      $.parenthesized_expression,
      $.primary_expression,
    ),

    primary_expression: $ => choice(
      $.identifier,
      $.number,
      $.string,
      $.template_string,
      $.true,
      $.false,
      $.null,
      $.array,
      $.object,
    ),

    _expressions: $ => choice(
      $._expression,
      $.sequence_expression,
    ),

    sequence_expression: $ => prec(PREC.COMMA, seq(
      $._expression,
      ",",
      choice($.sequence_expression, $._expression),
    )),

    assignment_expression: $ => prec.right(PREC.ASSIGN, seq(
      field("left", choice($.identifier, $.member_expression, $.subscript_expression)),
      "=",
      field("right", $._expression),
    )),

    binary_expression: $ => choice(
      ...[
        [choice("aut", "||"), PREC.OR],
        [choice("et", "&&"), PREC.AND],
        [choice("==", "!="), PREC.EQUAL],
        [choice("<", "<=", ">", ">="), PREC.COMPARE],
        [choice("+", "-"), PREC.ADD],
        [choice("*", "/", "%"), PREC.MULT],
      ].map(([op, p]) =>
        prec.left(p, seq($._expression, op, $._expression))
      ),
    ),

    unary_expression: $ => prec.right(PREC.UNARY, seq(
      field("operator", choice("!", "non", "-", "+")),
      field("argument", $._expression),
    )),

    await_expression: $ => prec.right(PREC.UNARY, seq(
      "exspecta",
      $._expression,
    )),

    call_expression: $ => prec(PREC.CALL, seq(
      field("function", $._expression),
      field("arguments", $.arguments),
    )),

    arguments: $ => seq(
      "(",
      sep($._expression, ","),
      ")",
    ),

    member_expression: $ => prec(PREC.MEMBER, seq(
      field("object", $._expression),
      ".",
      field("property", $.identifier),
    )),

    subscript_expression: $ => prec(PREC.MEMBER, seq(
      field("object", $._expression),
      "[",
      field("index", $._expression),
      "]",
    )),

    arrow_function: $ => prec.right(seq(
      choice(
        $.arrow_function_parameter,
        seq("(", sep($.arrow_function_parameter, ","), ")"),
      ),
      "=>",
      field("body", choice($._expression, $.statement_block)),
    )),

    arrow_function_parameter: $ => seq(
      field("name", $.identifier),
      optional(seq(":", field("type", $.type))),
    ),

    new_expression: $ => prec.right(PREC.CALL, seq(
      "novum",
      $._expression,
      optional($.arguments),
    )),

    parenthesized_expression: $ => seq(
      "(",
      $._expression,
      ")",
    ),

    // ==========================================================================
    // Literals
    // ==========================================================================

    identifier: $ => /[a-z_][a-zA-Z0-9_]*/,

    number: $ => /\d+(\.\d+)?/,

    string: $ => choice(
      seq('"', repeat(choice(/[^"\\]/, /\\./)), '"'),
      seq("'", repeat(choice(/[^'\\]/, /\\./)), "'"),
    ),

    template_string: $ => seq(
      "`",
      repeat(choice(
        $._template_chars,
        $.template_substitution,
      )),
      "`",
    ),

    _template_chars: $ => token(prec(-1, /[^`$]+/)),

    template_substitution: $ => seq(
      "${",
      $._expression,
      "}",
    ),

    true: $ => "verum",
    false: $ => "falsum",
    null: $ => "nihil",

    array: $ => seq(
      "[",
      sep($._expression, ","),
      "]",
    ),

    object: $ => seq(
      "{",
      sep($.pair, ","),
      "}",
    ),

    pair: $ => seq(
      field("key", choice($.identifier, $.string)),
      ":",
      field("value", $._expression),
    ),

    // ==========================================================================
    // Comments
    // ==========================================================================

    comment: $ => token(choice(
      seq("//", /.*/),
      seq("/*", /[^*]*\*+([^/*][^*]*\*+)*/, "/"),
    )),
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
