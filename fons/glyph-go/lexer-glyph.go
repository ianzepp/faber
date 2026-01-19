package main

import (
	"strings"
	"unicode/utf8"

	"subsidia"
)

// Reverse mappings for glyph → Faber

var glyphDelimiterToFaber = map[rune]string{
	'\u2590': "{",  // ▐ Right half block
	'\u258C': "}",  // ▌ Left half block
	'\u259D': "(",  // ▝ Upper right quadrant
	'\u2598': ")",  // ▘ Upper left quadrant
	'\u2597': "[",  // ▗ Lower right quadrant
	'\u2596': "]",  // ▖ Lower left quadrant
	'\u2580': "<",  // ▀ Upper half block (type params)
	'\u2584': ">",  // ▄ Lower half block (type params)
	'\u259A': "\"", // ▚ String delimiter
	'\u259E': "'",  // ▞ Char delimiter
}

var glyphPunctToFaber = map[rune]string{
	'\u204F': ";", // ⁏
	'\u2E34': ",", // ⸴
	'\u00B7': ".", // ·
	'\u2236': ":", // ∶
	'\u2E2E': "?", // ⸮ (optional marker)
	'\u00A1': "!", // ¡ (non-null marker)
	'\u203B': "@", // ※ (annotations)
	'\u2317': "#", // ⌗ (comments)
}

var glyphKeywordToFaber = map[string]string{
	// Declarations
	"≡": "fixum",
	"≔": "varia",
	"⫢": "figendum",
	"⫤": "variandum",
	"∫": "functio",
	"⊷": "typus",
	"⊞": "ordo",
	"⊟": "abstractus",

	// Type/Class Family
	"◌": "pactum",
	"◎": "genus",
	"◉": "ego",
	"⦿": "novum",
	"⦶": "qua",
	"⦵": "innatum",

	// Tagged Union Family
	"⦻": "discretio",
	"⦺": "finge",
	"⦼": "discerne",

	// Class Members
	"⊏": "sub",
	"⊒": "implet",
	"⊺": "generis",
	"⊸": "nexum",

	// Control Flow
	"↳": "si",
	"↴": "sin",
	"↲": "secus",
	"∴": "ergo",
	"∞": "dum",
	"∈": "ex",
	"∋": "de",
	"∀": "pro",
	"⋔": "elige",
	"⌜": "casu",
	"⌟": "ceterum",
	"⊧": "custodi",
	"⊡": "fac",

	// Control Transfer
	"⊢": "redde",
	"⊣": "reddit",
	"⊗": "rumpe",
	"↻": "perge",

	// Error Handling
	"◇": "tempta",
	"◆": "cape",
	"◈": "demum",
	"↯": "iace",
	"⤋": "iacit",
	"⟂": "mori",
	"⫫": "moritor",
	"⊩": "adfirma",

	// Async
	"⋆": "cede",
	"⊶": "futura",

	// Boolean and Logic
	"⊤": "verum",
	"⊥": "falsum",
	"∅": "nihil",
	"∧": "et",
	"∨": "aut",
	"¬": "non",
	"⁇": "vel",
	"≟": "est",

	// Type Conversions
	"⌊": "numeratum",
	"⌈": "fractatum",
	"≋": "textatum",
	"⊼": "bivalentum",

	// Parameters
	"⊳": "in",
	"⋯": "ceteri",
	"⋰": "sparge",
	"↦": "ut",

	// Imports
	"⊲": "importa",

	// Output
	"⊝": "scribe",
	"⋱": "vide",
	"⋮": "mone",

	// Ranges
	"▷": "ante",
	"▶": "usque",
	"▴": "per",
	"≬": "intra",
	"∊": "inter",

	// Bitwise Keywords
	"⋘": "sinistratum",
	"⋙": "dextratum",

	// Testing
	"⊬": "probandum",
	"⫞": "proba",
	"⊰": "praepara",
	"⊱": "postpara",
	"⦸": "omitte",

	// Entry Points
	"⟙": "incipit",
	"⫟": "incipiet",

	// Resource Management
	"⦾": "cura",
}

var glyphOperatorToFaber = map[string]string{
	// Arithmetic
	"⊕": "+",
	"⊖": "-",
	"⊛": "*",
	"⊘": "/",
	"⊜": "%",
	"⧺": "++",
	"⧻": "--",

	// Comparison
	"≺": "<",
	"≻": ">",
	"≼": "<=",
	"≽": ">=",
	"≈": "==",
	"≣": "===",
	"≠": "!=",
	"≢": "!==",

	// Assignment
	"←": "=",
	"↞": "+=",
	"↢": "-=",
	"↩": "*=",
	"↫": "/=",
	"↤": "&=",
	"↜": "|=",

	// Bitwise
	"⊓": "&",
	"⊔": "|",
	"⊻": "^",
	"∼": "~",

	// Logical (symbol form)
	"⋀": "&&",
	"⋁": "||",

	// Other
	"‥": "..",
	"→": "->",
	"⇒": "=>",
}

// fromBraille converts a braille character back to ASCII
func fromBraille(r rune) byte {
	if r >= 0x2800 && r <= 0x28FF {
		return byte(r - 0x2800)
	}
	return 0
}

// isBraille checks if a rune is in the braille range
func isBraille(r rune) bool {
	return r >= 0x2800 && r <= 0x28FF
}

// isBlockDelimiter checks if a rune is a block delimiter
func isBlockDelimiter(r rune) bool {
	_, ok := glyphDelimiterToFaber[r]
	return ok
}

// isGlyphPunct checks if a rune is glyph punctuation
func isGlyphPunct(r rune) bool {
	_, ok := glyphPunctToFaber[r]
	return ok
}

// LexGlyph converts glyph source text into tokens (same format as regular Lex).
func LexGlyph(source string, filename string) []subsidia.Token {
	tokens := []subsidia.Token{}
	runes := []rune(source)
	pos := 0
	linea := 1
	columna := 1

	locus := func() subsidia.Locus {
		return subsidia.Locus{Linea: linea, Columna: columna, Index: pos}
	}

	peek := func(offset int) rune {
		idx := pos + offset
		if idx >= len(runes) {
			return 0
		}
		return runes[idx]
	}

	advance := func() rune {
		r := runes[pos]
		pos++
		if r == '\n' {
			linea++
			columna = 1
		} else {
			columna++
		}
		return r
	}

	skipWhitespace := func() {
		for pos < len(runes) {
			r := peek(0)
			if r == ' ' || r == '\t' || r == '\r' {
				advance()
			} else if r == '\n' {
				loc := locus()
				advance()
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenNewline, Valor: "\n", Locus: loc})
			} else {
				break
			}
		}
	}

	// Read braille sequence and convert to ASCII string
	readBraille := func() string {
		var b strings.Builder
		for pos < len(runes) && isBraille(peek(0)) {
			b.WriteByte(fromBraille(advance()))
		}
		return b.String()
	}

	// Read braille string (between ▚ delimiters)
	readBrailleString := func() string {
		advance() // skip opening ▚
		var b strings.Builder
		for pos < len(runes) && peek(0) != '\u259A' {
			r := peek(0)
			if isBraille(r) {
				b.WriteByte(fromBraille(advance()))
			} else {
				// Handle interpolation markers or other content
				b.WriteRune(advance())
			}
		}
		if pos < len(runes) {
			advance() // skip closing ▚
		}
		return b.String()
	}

	// Try to match a multi-rune glyph keyword/operator
	tryMatchGlyph := func() (string, string, bool) {
		r := peek(0)
		s := string(r)

		// Check keywords first
		if faber, ok := glyphKeywordToFaber[s]; ok {
			return faber, subsidia.TokenKeyword, true
		}

		// Check operators
		if faber, ok := glyphOperatorToFaber[s]; ok {
			return faber, subsidia.TokenOperator, true
		}

		return "", "", false
	}

	for pos < len(runes) {
		skipWhitespace()
		if pos >= len(runes) {
			break
		}

		loc := locus()
		r := peek(0)

		// Comment (⌗)
		if r == '\u2317' {
			advance()
			var b strings.Builder
			for pos < len(runes) && peek(0) != '\n' {
				b.WriteRune(advance())
			}
			tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenComment, Valor: b.String(), Locus: loc})
			continue
		}

		// String literal (▚...▚)
		if r == '\u259A' {
			value := readBrailleString()
			tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenTextus, Valor: value, Locus: loc})
			continue
		}

		// Braille sequence (identifier or number)
		if isBraille(r) {
			value := readBraille()
			// Determine if it's a number or identifier
			if len(value) > 0 && isDigit(value[0]) {
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenNumerus, Valor: value, Locus: loc})
			} else {
				// Check if it's a keyword
				_, isKeyword := keywords[value]
				tag := subsidia.TokenIdentifier
				if isKeyword {
					tag = subsidia.TokenKeyword
				}
				tokens = append(tokens, subsidia.Token{Tag: tag, Valor: value, Locus: loc})
			}
			continue
		}

		// Block delimiters
		if isBlockDelimiter(r) {
			advance()
			faber := glyphDelimiterToFaber[r]
			// < and > are operators in type contexts
			if faber == "<" || faber == ">" {
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenOperator, Valor: faber, Locus: loc})
			} else {
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenPunctuator, Valor: faber, Locus: loc})
			}
			continue
		}

		// Glyph punctuation
		if isGlyphPunct(r) {
			advance()
			faber := glyphPunctToFaber[r]
			tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenPunctuator, Valor: faber, Locus: loc})
			continue
		}

		// Try keyword/operator glyphs
		if faber, tag, ok := tryMatchGlyph(); ok {
			advance()
			tokens = append(tokens, subsidia.Token{Tag: tag, Valor: faber, Locus: loc})
			continue
		}

		// § import sigil (unchanged)
		if r == '§' {
			advance()
			tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenPunctuator, Valor: "§", Locus: loc})
			continue
		}

		// Handle any remaining ASCII that might appear
		if r < 128 {
			ch := byte(r)
			if isDigit(ch) {
				var b strings.Builder
				for pos < len(runes) && peek(0) < 128 && isNumberChar(byte(peek(0))) {
					b.WriteByte(byte(advance()))
				}
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenNumerus, Valor: b.String(), Locus: loc})
				continue
			}
			if isAlpha(ch) || ch == '_' {
				var b strings.Builder
				for pos < len(runes) && peek(0) < 128 && isIdentChar(byte(peek(0))) {
					b.WriteByte(byte(advance()))
				}
				value := b.String()
				_, isKeyword := keywords[value]
				tag := subsidia.TokenIdentifier
				if isKeyword {
					tag = subsidia.TokenKeyword
				}
				tokens = append(tokens, subsidia.Token{Tag: tag, Valor: value, Locus: loc})
				continue
			}
			if _, ok := punctuators[ch]; ok {
				advance()
				tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenPunctuator, Valor: string(ch), Locus: loc})
				continue
			}
		}

		// Unknown character - skip with error
		advance()
		_ = utf8.RuneLen(r) // suppress unused import warning
	}

	tokens = append(tokens, subsidia.Token{Tag: subsidia.TokenEOF, Valor: "", Locus: locus()})
	return tokens
}
