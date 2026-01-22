; Faber Romanus syntax highlighting for Zed
; Token-based highlighting (no structural parsing)

; Comments
(comment) @comment

; Annotations and directives (entire line)
(annotation) @attribute
(directive) @attribute

; Control flow keywords
(keyword_control) @keyword

; Declaration keywords  
(keyword_declaration) @keyword

; Other keywords
(keyword_other) @keyword

; Built-in types
(builtin_type) @type.builtin

; Literals
(boolean) @boolean
(string) @string
(escape_sequence) @string.escape
(number) @number

; Identifiers (fallback)
(identifier) @variable

; Operators
(operator) @operator

; Punctuation
(punctuation) @punctuation.bracket
