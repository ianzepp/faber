; Faber Romanus syntax highlighting queries for Tree-sitter

; ==============================================================================
; Comments
; ==============================================================================

(comment) @comment

; ==============================================================================
; Keywords - Control Flow
; ==============================================================================

"si" @keyword.control.conditional
"aliter" @keyword.control.conditional

"dum" @keyword.control.repeat
"pro" @keyword.control.repeat

"tempta" @keyword.control.exception
"cape" @keyword.control.exception
"demum" @keyword.control.exception
"iace" @keyword.control.exception

"redde" @keyword.control.return
"rumpe" @keyword.control.return
"perge" @keyword.control.return

; ==============================================================================
; Keywords - Declarations
; ==============================================================================

"esto" @keyword.storage
"fixum" @keyword.storage
"functio" @keyword.function
"futura" @keyword.modifier
"novum" @keyword.operator

; ==============================================================================
; Keywords - Imports
; ==============================================================================

"ex" @keyword.import
"importa" @keyword.import
"cum" @keyword.import

; ==============================================================================
; Keywords - Operators (Latin)
; ==============================================================================

"et" @keyword.operator
"aut" @keyword.operator
"non" @keyword.operator
"in" @keyword

; ==============================================================================
; Keywords - Async
; ==============================================================================

"exspecta" @keyword.control

; ==============================================================================
; Literals
; ==============================================================================

(number) @constant.numeric
(string) @string
(template_string) @string
(template_substitution
  "${" @punctuation.special
  "}" @punctuation.special)

; Booleans
(true) @constant.builtin.boolean
(false) @constant.builtin.boolean

; Null
(null) @constant.builtin

; ==============================================================================
; Types
; ==============================================================================

(type_identifier) @type

; ==============================================================================
; Functions
; ==============================================================================

; Function declarations
(function_declaration
  name: (identifier) @function)

; Function calls
(call_expression
  function: (primary_expression (identifier) @function.call))

; Method calls
(call_expression
  function: (member_expression
    property: (identifier) @function.method.call))

; ==============================================================================
; Variables and Parameters
; ==============================================================================

; Variable declarations
(variable_declaration
  name: (identifier) @variable)

; Parameters
(formal_parameter
  name: (identifier) @variable.parameter)

(arrow_function_parameter
  name: (identifier) @variable.parameter)

; Property access
(member_expression
  property: (identifier) @property)

; Object properties
(pair
  key: (identifier) @property)

; ==============================================================================
; Operators
; ==============================================================================

"=" @operator
"==" @operator
"!=" @operator
"<" @operator
"<=" @operator
">" @operator
">=" @operator
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"%" @operator
"!" @operator
"&&" @operator
"||" @operator
"=>" @operator
"?" @operator

; ==============================================================================
; Punctuation
; ==============================================================================

"." @punctuation.delimiter
"," @punctuation.delimiter
":" @punctuation.delimiter
";" @punctuation.delimiter

"(" @punctuation.bracket
")" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"<" @punctuation.bracket
">" @punctuation.bracket
