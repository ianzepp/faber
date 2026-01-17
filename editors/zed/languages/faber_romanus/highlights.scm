; Faber Romanus syntax highlighting queries for Tree-sitter

; ==============================================================================
; Comments
; ==============================================================================

(comment) @comment

; ==============================================================================
; Keywords - Control Flow
; ==============================================================================

"si" @keyword.control.conditional
"secus" @keyword.control.conditional
"ergo" @keyword.control.conditional

"dum" @keyword.control.repeat
"ex" @keyword.control.repeat
"in" @keyword.control.repeat
"pro" @keyword.control.repeat

"elige" @keyword.control.conditional
"custodi" @keyword.control.conditional

"tempta" @keyword.control.exception
"cape" @keyword.control.exception
"demum" @keyword.control.exception
"iace" @keyword.control.exception

"redde" @keyword.control.return
(break_statement) @keyword.control.return
(continue_statement) @keyword.control.return

; ==============================================================================
; Keywords - Declarations
; ==============================================================================

"varia" @keyword.storage
"fixum" @keyword.storage
"functio" @keyword.function
"futura" @keyword.modifier
"novum" @keyword.operator
"typus" @keyword.storage

; ==============================================================================
; Keywords - Imports
; ==============================================================================

"importa" @keyword.import

; ==============================================================================
; Keywords - Operators (Latin)
; ==============================================================================

"et" @keyword.operator
"aut" @keyword.operator
"non" @keyword.operator
"nulla" @keyword.operator
"nonnulla" @keyword.operator
"per" @keyword.operator

; ==============================================================================
; Keywords - Other
; ==============================================================================

"adfirma" @keyword
"scribe" @keyword.function
"cede" @keyword.control
"clausura" @keyword.function

; ==============================================================================
; Literals
; ==============================================================================

(number) @constant.numeric
(string) @string
(template_string) @string

; Booleans
(true) @constant.builtin.boolean
(false) @constant.builtin.boolean

; Null and self
(null) @constant.builtin
(self) @variable.builtin

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
; Identifiers (fallback)
; ==============================================================================

(identifier) @variable

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
"->" @operator
".." @operator

; ==============================================================================
; Punctuation
; ==============================================================================

"." @punctuation.delimiter
"," @punctuation.delimiter
":" @punctuation.delimiter

"(" @punctuation.bracket
")" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
