/**
 * Tree-sitter grammar for Faber Romanus
 *
 * A Latin programming language that compiles to TypeScript and Zig.
 * Aligned with fons/parser/index.ts
 */

const PREC = {
  COMMA: -1,
  ASSIGN: 0,
  OR: 1,
  AND: 2,
  EQUAL: 3,
  COMPARE: 4,
  RANGE: 5,
  ADD: 6,
  MULT: 7,
  UNARY: 8,
  CALL: 9,
  MEMBER: 10,
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
      $.type_alias_declaration,
      $.if_statement,
      $.while_statement,
      $.for_statement,
      $.with_statement,
      $.switch_statement,
      $.guard_statement,
      $.assert_statement,
      $.try_statement,
      $.return_statement,
      $.break_statement,
      $.continue_statement,
      $.throw_statement,
      $.scribe_statement,
      $.statement_block,
      $.expression_statement,
    ),

    expression_statement: $ => seq(
      $._expression,
      optional($._automatic_semicolon),
    ),

    // Import: ex module importa name1, name2 (or *)
    import_declaration: $ => seq(
      "ex",
      field("source", $.identifier),
      "importa",
      choice(
        "*",
        sep1(field("specifier", $.identifier), ","),
      ),
    ),

    // Variable: esto/fixum [type] name = value (type-first, optional)
    variable_declaration: $ => seq(
      field("kind", choice("esto", "fixum")),
      choice(
        // Type-first: fixum textus name
        seq(
          field("type", $.type),
          field("name", $.identifier),
        ),
        // Type inference: fixum name
        field("name", $.identifier),
        // Destructuring: fixum { a, b }
        field("pattern", $.object_pattern),
      ),
      optional(seq("=", field("value", $._expression))),
    ),

    object_pattern: $ => seq(
      "{",
      sep($.object_pattern_property, ","),
      "}",
    ),

    object_pattern_property: $ => seq(
      field("key", $.identifier),
      optional(seq(":", field("value", $.identifier))),
    ),

    // Function: [futura] functio name(params) [-> ReturnType] { body }
    function_declaration: $ => seq(
      optional(field("async", "futura")),
      "functio",
      field("name", $.identifier),
      field("parameters", $.formal_parameters),
      optional(seq("->", field("return_type", $.type))),
      field("body", $.statement_block),
    ),

    formal_parameters: $ => seq(
      "(",
      sep($.formal_parameter, ","),
      ")",
    ),

    // Parameter: [preposition] [Type] name
    formal_parameter: $ => seq(
      optional(field("preposition", choice("ad", "cum", "in", "ex"))),
      optional(field("type", $.type)),
      field("name", $.identifier),
    ),

    // Type alias: typus Name = Type
    type_alias_declaration: $ => seq(
      "typus",
      field("name", $.identifier),
      "=",
      field("type", $.type),
    ),

    // If: si condition { } [cape err { }] [aliter { }]
    // Or: si condition ergo statement [aliter ...]
    if_statement: $ => prec.right(choice(
      // Block form with optional catch
      seq(
        "si",
        field("condition", $._expression),
        field("consequence", $.statement_block),
        optional(field("catch", $.catch_clause)),
        optional(field("alternative", $.else_clause)),
      ),
      // One-liner form (no catch)
      seq(
        "si",
        field("condition", $._expression),
        "ergo",
        field("consequence", $._statement),
        optional(field("alternative", $.else_clause)),
      ),
    )),

    else_clause: $ => seq(
      "aliter",
      choice(
        $.if_statement,
        $.statement_block,
        $._statement,
      ),
    ),

    // While: dum condition { } [cape err { }]
    // Or: dum condition ergo statement
    while_statement: $ => choice(
      seq(
        "dum",
        field("condition", $._expression),
        field("body", $.statement_block),
        optional(field("catch", $.catch_clause)),
      ),
      seq(
        "dum",
        field("condition", $._expression),
        "ergo",
        field("body", $._statement),
      ),
    ),

    // For: ex/in iterable pro variable { } [cape err { }]
    // Or: ex/in iterable pro variable ergo statement
    for_statement: $ => choice(
      seq(
        field("kind", choice("ex", "in")),
        field("iterable", $._expression),
        "pro",
        field("variable", $.identifier),
        field("body", $.statement_block),
        optional(field("catch", $.catch_clause)),
      ),
      seq(
        field("kind", choice("ex", "in")),
        field("iterable", $._expression),
        "pro",
        field("variable", $.identifier),
        "ergo",
        field("body", $._statement),
      ),
    ),

    // With: cum object { }
    with_statement: $ => seq(
      "cum",
      field("object", $._expression),
      field("body", $.statement_block),
    ),

    // Switch: elige expr { si val ergo/{ } ... aliter { } }
    switch_statement: $ => seq(
      "elige",
      field("discriminant", $._expression),
      "{",
      repeat($.switch_case),
      optional($.switch_default),
      "}",
      optional(field("catch", $.catch_clause)),
    ),

    switch_case: $ => choice(
      seq(
        "si",
        field("test", $._expression),
        field("consequent", $.statement_block),
      ),
      seq(
        "si",
        field("test", $._expression),
        "ergo",
        field("consequent", $._statement),
      ),
    ),

    switch_default: $ => seq(
      "aliter",
      choice(
        $.statement_block,
        $._statement,
      ),
    ),

    // Guard: custodi { si cond { } ... }
    guard_statement: $ => seq(
      "custodi",
      "{",
      repeat($.guard_clause),
      "}",
    ),

    guard_clause: $ => seq(
      "si",
      field("test", $._expression),
      field("consequent", $.statement_block),
    ),

    // Assert: adfirma condition [, message]
    assert_statement: $ => seq(
      "adfirma",
      field("test", $._expression),
      optional(seq(",", field("message", $._expression))),
    ),

    // Try: tempta { } [cape err { }] [demum { }]
    try_statement: $ => seq(
      "tempta",
      field("body", $.statement_block),
      optional(field("handler", $.catch_clause)),
      optional(field("finalizer", $.finally_clause)),
    ),

    catch_clause: $ => seq(
      "cape",
      field("parameter", $.identifier),
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

    // Scribe: scribe expr [, expr, ...]
    scribe_statement: $ => seq(
      "scribe",
      sep1($._expression, ","),
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

    // Type names: case-insensitive (textus, Textus, TEXTUS all work)
    // Convention: lowercase preferred (Latin had no case distinction)
    type_identifier: $ => /[A-Za-z][a-zA-Z0-9_]*/,

    type_arguments: $ => seq(
      "<",
      sep1(choice($.type, $.number, $.type_modifier), ","),
      ">",
    ),

    type_modifier: $ => choice(
      "naturalis",
      "proprius",
      "alienus",
      "mutabilis",
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
      $.range_expression,
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
      $.self,
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
        prec.left(p, seq(
          field("left", $._expression),
          field("operator", op),
          field("right", $._expression),
        ))
      ),
    ),

    unary_expression: $ => prec.right(PREC.UNARY, seq(
      field("operator", choice("!", "non", "-", "+", "nulla", "nonnulla")),
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
      optional(field("type", $.type)),
      field("name", $.identifier),
    ),

    new_expression: $ => prec.right(PREC.CALL, seq(
      "novum",
      field("callee", $.identifier),
      optional($.arguments),
    )),

    // Range: start..end [per step]
    range_expression: $ => prec.left(PREC.RANGE, seq(
      field("start", $._expression),
      "..",
      field("end", $._expression),
      optional(seq("per", field("step", $._expression))),
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
    self: $ => "ego",

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
