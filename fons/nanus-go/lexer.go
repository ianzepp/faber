package nanus

import "strings"

var keywords = map[string]struct{}{
	// Declarations
	"varia": {}, "fixum": {}, "figendum": {},
	"functio": {}, "genus": {}, "pactum": {}, "ordo": {}, "discretio": {},
	"ex": {}, "importa": {}, "ut": {},
	// Modifiers
	"publica": {}, "privata": {}, "protecta": {}, "generis": {}, "implet": {}, "sub": {},
	// Control flow
	"si": {}, "sin": {}, "secus": {}, "dum": {}, "fac": {}, "elige": {}, "casu": {}, "ceterum": {}, "discerne": {}, "custodi": {},
	"de": {}, "pro": {}, "omnia": {},
	// Actions
	"redde": {}, "reddit": {}, "rumpe": {}, "perge": {}, "iace": {}, "mori": {}, "tempta": {}, "cape": {}, "demum": {},
	"scribe": {}, "vide": {}, "mone": {}, "adfirma": {},
	// Expressions
	"cede": {}, "novum": {}, "clausura": {}, "qua": {}, "innatum": {}, "finge": {},
	"sic": {}, "scriptum": {},
	// Operators (word-form)
	"et": {}, "aut": {}, "vel": {}, "inter": {}, "intra": {},
	"non": {}, "nihil": {}, "nonnihil": {}, "positivum": {},
	// Literals
	"verum": {}, "falsum": {}, "ego": {},
	// Entry
	"incipit": {}, "incipiet": {},
	// Test
	"probandum": {}, "proba": {},
	// Type
	"usque": {},
	// Annotations
	"publicum": {}, "externa": {},
}

var punctuators = map[byte]struct{}{
	'(': {}, ')': {}, '{': {}, '}': {}, '[': {}, ']': {},
	',': {}, '.': {}, ':': {}, ';': {}, '@': {}, '#': {},
	'?': {}, '!': {},
}

var operators = []string{
	// Multi-char first (greedy match)
	"===", "!==", "==", "!=", "<=", ">=", "&&", "||", "??",
	"+=", "-=", "*=", "/=",
	"->", "..",
	// Single-char
	"+", "-", "*", "/", "%",
	"<", ">", "=",
	"&", "|", "^", "~",
}

const (
	tokenEOF        = "EOF"
	tokenNewline    = "Newline"
	tokenIdentifier = "Identifier"
	tokenNumerus    = "Numerus"
	tokenTextus     = "Textus"
	tokenOperator   = "Operator"
	tokenPunctuator = "Punctuator"
	tokenKeyword    = "Keyword"
	tokenComment    = "Comment"
)

// Lex converts source text into tokens.
func Lex(source string, filename string) []Token {
	tokens := []Token{}
	pos := 0
	linea := 1
	lineStart := 0

	locus := func() Locus {
		return Locus{Linea: linea, Columna: pos - lineStart + 1, Index: pos}
	}

	peek := func(offset int) byte {
		idx := pos + offset
		if idx >= len(source) {
			return 0
		}
		return source[idx]
	}

	advance := func() byte {
		ch := source[pos]
		pos++
		if ch == '\n' {
			linea++
			lineStart = pos
		}
		return ch
	}

	match := func(str string) bool {
		if strings.HasPrefix(source[pos:], str) {
			for i := 0; i < len(str); i++ {
				advance()
			}
			return true
		}
		return false
	}

	skipWhitespace := func() {
		for pos < len(source) {
			ch := peek(0)
			if ch == ' ' || ch == '\t' || ch == '\r' {
				advance()
			} else if ch == '\n' {
				loc := locus()
				advance()
				tokens = append(tokens, Token{Tag: tokenNewline, Valor: "\n", Locus: loc})
			} else {
				break
			}
		}
	}

	readString := func(quote byte) string {
		var b strings.Builder
		advance()
		for pos < len(source) && peek(0) != quote {
			if peek(0) == '\\' {
				advance()
				esc := advance()
				switch esc {
				case 'n':
					b.WriteByte('\n')
				case 't':
					b.WriteByte('\t')
				case 'r':
					b.WriteByte('\r')
				case '\\':
					b.WriteByte('\\')
				case '"':
					b.WriteByte('"')
				case '\'':
					b.WriteByte('\'')
				default:
					b.WriteByte(esc)
				}
			} else {
				b.WriteByte(advance())
			}
		}
		advance()
		return b.String()
	}

	readTripleString := func() string {
		advance()
		advance()
		advance()

		if peek(0) == '\n' {
			advance()
		}

		var b strings.Builder
		for pos < len(source) {
			if peek(0) == '"' && peek(1) == '"' && peek(2) == '"' {
				value := b.String()
				if strings.HasSuffix(value, "\n") {
					value = strings.TrimSuffix(value, "\n")
				}
				advance()
				advance()
				advance()
				return value
			}
			b.WriteByte(advance())
		}
		return b.String()
	}

	readNumber := func() string {
		var b strings.Builder
		for pos < len(source) && isNumberChar(peek(0)) {
			b.WriteByte(advance())
		}
		return b.String()
	}

	readIdentifier := func() string {
		var b strings.Builder
		for pos < len(source) && isIdentChar(peek(0)) {
			b.WriteByte(advance())
		}
		return b.String()
	}

	readComment := func() string {
		var b strings.Builder
		advance()
		for pos < len(source) && peek(0) != '\n' {
			b.WriteByte(advance())
		}
		return b.String()
	}

	for pos < len(source) {
		skipWhitespace()
		if pos >= len(source) {
			break
		}

		loc := locus()
		ch := peek(0)

		if ch == '#' {
			value := readComment()
			tokens = append(tokens, Token{Tag: tokenComment, Valor: value, Locus: loc})
			continue
		}

		if ch == '"' && peek(1) == '"' && peek(2) == '"' {
			value := readTripleString()
			tokens = append(tokens, Token{Tag: tokenTextus, Valor: value, Locus: loc})
			continue
		}
		if ch == '"' || ch == '\'' {
			value := readString(ch)
			tokens = append(tokens, Token{Tag: tokenTextus, Valor: value, Locus: loc})
			continue
		}

		if isDigit(ch) {
			value := readNumber()
			tokens = append(tokens, Token{Tag: tokenNumerus, Valor: value, Locus: loc})
			continue
		}

		if isAlpha(ch) || ch == '_' {
			value := readIdentifier()
			_, isKeyword := keywords[value]
			tag := tokenIdentifier
			if isKeyword {
				tag = tokenKeyword
			}
			tokens = append(tokens, Token{Tag: tag, Valor: value, Locus: loc})
			continue
		}

		matched := false
		for _, op := range operators {
			if match(op) {
				tokens = append(tokens, Token{Tag: tokenOperator, Valor: op, Locus: loc})
				matched = true
				break
			}
		}
		if matched {
			continue
		}

		// § is multi-byte UTF-8, handle via string match
		if match("§") {
			tokens = append(tokens, Token{Tag: tokenPunctuator, Valor: "§", Locus: loc})
			continue
		}

		if _, ok := punctuators[ch]; ok {
			advance()
			tokens = append(tokens, Token{Tag: tokenPunctuator, Valor: string(ch), Locus: loc})
			continue
		}

		panic(&CompileError{Message: "unexpected character '" + string(ch) + "'", Locus: loc, Filename: filename})
	}

	tokens = append(tokens, Token{Tag: tokenEOF, Valor: "", Locus: locus()})
	return tokens
}

// Prepare filters out comments and newlines.
func Prepare(tokens []Token) []Token {
	out := make([]Token, 0, len(tokens))
	for _, tok := range tokens {
		if tok.Tag == tokenComment || tok.Tag == tokenNewline {
			continue
		}
		out = append(out, tok)
	}
	return out
}

func isDigit(ch byte) bool {
	return ch >= '0' && ch <= '9'
}

func isAlpha(ch byte) bool {
	return (ch >= 'a' && ch <= 'z') || (ch >= 'A' && ch <= 'Z')
}

func isIdentChar(ch byte) bool {
	return isAlpha(ch) || isDigit(ch) || ch == '_'
}

func isNumberChar(ch byte) bool {
	return isDigit(ch) || ch == '.' || ch == '_'
}
