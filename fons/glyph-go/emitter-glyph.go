package main

import (
	"strings"

	"subsidia"
)

// Glyph mappings from GLYPH.md

// Delimiters (Block Characters U+2580-U+259F)
var glyphDelimiters = map[rune]rune{
	'{': '\u2590', // ▐ Right half block
	'}': '\u258C', // ▌ Left half block
	'(': '\u259D', // ▝ Upper right quadrant
	')': '\u2598', // ▘ Upper left quadrant
	'[': '\u2597', // ▗ Lower right quadrant
	']': '\u2596', // ▖ Lower left quadrant
	'<': '\u2580', // ▀ Upper half block (type params)
	'>': '\u2584', // ▄ Lower half block (type params)
	'"': '\u259A', // ▚ Upper left + lower right
	'\'': '\u259E', // ▞ Upper right + lower left
}

// Punctuation
var glyphPunctuation = map[rune]rune{
	';': '\u204F', // 	',': '\u2E34', // ⸴
	'.': '\u00B7', // ·
	':': '\u2236', // ∶
}

// Keywords
var glyphKeywords = map[string]string{
	// Declarations
	"fixum":     "≡",
	"varia":     "≔",
	"figendum":  "⫢",
	"variandum": "⫤",
	"functio":   "∫",
	"typus":     "⊷",
	"ordo":      "⊞",
	"abstractus": "⊟",

	// Type/Class Family
	"pactum":  "◌",
	"genus":   "◎",
	"ego":     "◉",
	"novum":   "⦿",
	"qua":     "⦶",
	"innatum": "⦵",

	// Tagged Union Family
	"discretio": "⦻",
	"finge":     "⦺",
	"discerne":  "⦼",

	// Class Members
	"sub":     "⊏",
	"implet":  "⊒",
	"generis": "⊺",
	"nexum":   "⊸",

	// Control Flow
	"si":      "∃",
	"sin":     "∄",
	"secus":   "∁",
	"ergo":    "∴",
	"dum":     "∞",
	"ex":      "∈",
	"de":      "∋",
	"pro":     "∀",
	"elige":   "⋔",
	"casu":    "↳",
	"ceterum": "↲",
	"custodi": "⊧",
	"fac":     "⊡",

	// Control Transfer
	"redde":  "⊢",
	"reddit": "⊣",
	"rumpe":  "⊗",
	"perge":  "↻",

	// Error Handling
	"tempta":  "◇",
	"cape":    "◆",
	"demum":   "◈",
	"iace":    "↯",
	"iacit":   "⤋",
	"mori":    "⟂",
	"moritor": "⫫",
	"adfirma": "⊩",

	// Async
	"cede":   "⋆",
	"futura": "⊶",
	"fit":    "→",
	"fiet":   "⇢",
	"fiunt":  "⇉",
	"fient":  "⇶",

	// Boolean and Logic
	"verum":  "⊤",
	"falsum": "⊥",
	"nihil":  "∅",
	"et":     "∧",
	"aut":    "∨",
	"non":    "¬",
	"vel":    "⁇",
	"est":    "≟",

	// Type Conversions
	"numeratum":  "⌊",
	"fractatum":  "⌈",
	"textatum":   "≋",
	"bivalentum": "⊼",

	// Parameters
	"in":     "⊳",
	"ceteri": "⋯",
	"sparge": "⋰",
	"ut":     "↦",

	// Imports
	"importa": "⊲",

	// Output
	"scribe": "⊝",
	"vide":   "⋱",
	"mone":   "⋮",

	// Ranges
	"ante":  "≺",
	"usque": "≼",
	"per":   "⊹",
	"intra": "∈",
	"inter": "≬",

	// Bitwise Keywords
	"sinistratum": "⋘",
	"dextratum":   "⋙",

	// Testing
	"probandum": "⊬",
	"proba":     "⫞",
	"praepara":  "⊰",
	"postpara":  "⊱",
	"omitte":    "⦸",

	// Entry Points
	"incipit":  "⟙",
	"incipiet": "⫟",

	// Resource Management
	"cura": "⦾",
}

// Operators
var glyphOperators = map[string]string{
	// Arithmetic
	"+":  "⊕",
	"-":  "⊖",
	"*":  "⊛",
	"/":  "⊘",
	"%":  "⊜",
	"++": "⧺",
	"--": "⧻",

	// Comparison
	"<":   "≺",
	">":   "≻",
	"<=":  "≼",
	">=":  "≽",
	"==":  "≈",
	"===": "≣",
	"!=":  "≠",
	"!==": "≢",

	// Assignment
	"=":  "←",
	"+=": "↞",
	"-=": "↢",
	"*=": "↩",
	"/=": "↫",
	"&=": "↤",
	"|=": "↜",

	// Bitwise
	"&": "⊓",
	"|": "⊔",
	"^": "⊻",
	"~": "∼",

	// Logical (symbol form)
	"&&": "⋀",
	"||": "⋁",
	"!":  "¬",

	// Other
	"..": "‥",
	"->": "→",
	"=>": "⇒",
}

// toBraille converts an ASCII string to Braille characters
func toBraille(s string) string {
	var b strings.Builder
	for _, r := range s {
		if r >= 0 && r < 256 {
			b.WriteRune(rune(0x2800 + int(r)))
		} else {
			b.WriteRune(r)
		}
	}
	return b.String()
}

// EmitGlyph renders a module to Faber Glyph format.
func EmitGlyph(mod *subsidia.Modulus) string {
	lines := []string{}
	for _, stmt := range mod.Corpus {
		lines = append(lines, glyphEmitStmt(stmt, ""))
	}
	return strings.Join(lines, "\n")
}

func glyphEmitStmt(stmt subsidia.Stmt, indent string) string {
	switch s := stmt.(type) {
	case *subsidia.StmtMassa:
		lines := []string{}
		for _, inner := range s.Corpus {
			lines = append(lines, glyphEmitStmt(inner, indent+"  "))
		}
		return "▐\n" + strings.Join(lines, "\n") + "\n" + indent + "▌"

	case *subsidia.StmtExpressia:
		return indent + glyphEmitExpr(s.Expr)

	case *subsidia.StmtVaria:
		kw := "≔" // varia
		if s.Species == subsidia.VariaFixum {
			kw = "≡" // fixum
		}
		typ := ""
		if s.Typus != nil {
			typ = " ∶ " + glyphEmitTypus(s.Typus)
		}
		val := ""
		if s.Valor != nil {
			val = " ← " + glyphEmitExpr(s.Valor)
		}
		return indent + kw + " " + toBraille(s.Nomen) + typ + val

	case *subsidia.StmtFunctio:
		async := ""
		if s.Asynca {
			async = "⊶ "
		}
		generics := ""
		if len(s.Generics) > 0 {
			glist := []string{}
			for _, g := range s.Generics {
				glist = append(glist, toBraille(g))
			}
			generics = "▀" + strings.Join(glist, "⸴ ") + "▄"
		}
		params := []string{}
		for _, param := range s.Params {
			params = append(params, glyphEmitParam(param))
		}
		ret := ""
		if s.TypusReditus != nil {
			ret = " → " + glyphEmitTypus(s.TypusReditus)
		}
		body := ""
		if s.Corpus != nil {
			body = " " + glyphEmitStmt(s.Corpus, indent)
		}
		return indent + async + "∫ " + toBraille(s.Nomen) + generics + " ▝ " + strings.Join(params, "⸴ ") + " ▘" + ret + body

	case *subsidia.StmtGenus:
		generics := ""
		if len(s.Generics) > 0 {
			glist := []string{}
			for _, g := range s.Generics {
				glist = append(glist, toBraille(g))
			}
			generics = "▀" + strings.Join(glist, "⸴ ") + "▄"
		}
		impl := ""
		if len(s.Implet) > 0 {
			ilist := []string{}
			for _, i := range s.Implet {
				ilist = append(ilist, toBraille(i))
			}
			impl = " ⊒ " + strings.Join(ilist, "⸴ ")
		}
		lines := []string{}
		lines = append(lines, indent+"◎ "+toBraille(s.Nomen)+generics+impl+" ▐")

		for _, campo := range s.Campi {
			val := ""
			if campo.Valor != nil {
				val = " ← " + glyphEmitExpr(campo.Valor)
			}
			lines = append(lines, indent+"  "+toBraille(campo.Nomen)+" ∶ "+glyphEmitTypus(campo.Typus)+val+"")
		}

		for _, method := range s.Methodi {
			if fn, ok := method.(*subsidia.StmtFunctio); ok {
				lines = append(lines, "")
				lines = append(lines, glyphEmitStmt(fn, indent+"  "))
			}
		}

		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtPactum:
		generics := ""
		if len(s.Generics) > 0 {
			glist := []string{}
			for _, g := range s.Generics {
				glist = append(glist, toBraille(g))
			}
			generics = "▀" + strings.Join(glist, "⸴ ") + "▄"
		}
		lines := []string{}
		lines = append(lines, indent+"◌ "+toBraille(s.Nomen)+generics+" ▐")
		for _, method := range s.Methodi {
			params := []string{}
			for _, param := range method.Params {
				params = append(params, glyphEmitParam(param))
			}
			ret := ""
			if method.TypusReditus != nil {
				ret = " → " + glyphEmitTypus(method.TypusReditus)
			}
			lines = append(lines, indent+"  "+toBraille(method.Nomen)+" ▝ "+strings.Join(params, "⸴ ")+" ▘"+ret+"")
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtOrdo:
		members := []string{}
		for _, m := range s.Membra {
			val := ""
			if m.Valor != nil {
				val = " ← " + toBraille(*m.Valor)
			}
			members = append(members, toBraille(m.Nomen)+val)
		}
		return indent + "⊞ " + toBraille(s.Nomen) + " ▐ " + strings.Join(members, "⸴ ") + " ▌"

	case *subsidia.StmtDiscretio:
		generics := ""
		if len(s.Generics) > 0 {
			glist := []string{}
			for _, g := range s.Generics {
				glist = append(glist, toBraille(g))
			}
			generics = "▀" + strings.Join(glist, "⸴ ") + "▄"
		}
		lines := []string{}
		lines = append(lines, indent+"⦻ "+toBraille(s.Nomen)+generics+" ▐")
		for _, v := range s.Variantes {
			if len(v.Campi) == 0 {
				lines = append(lines, indent+"  "+toBraille(v.Nomen)+"")
			} else {
				fields := []string{}
				for _, f := range v.Campi {
					fields = append(fields, toBraille(f.Nomen)+" ∶ "+glyphEmitTypus(f.Typus))
				}
				lines = append(lines, indent+"  "+toBraille(v.Nomen)+" ▝ "+strings.Join(fields, "⸴ ")+" ▘")
			}
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtImporta:
		specs := []string{}
		for _, spec := range s.Specs {
			if spec.Imported == spec.Local {
				specs = append(specs, toBraille(spec.Imported))
			} else {
				specs = append(specs, toBraille(spec.Imported)+" ↦ "+toBraille(spec.Local))
			}
		}
		return indent + "⊲ ▐ " + strings.Join(specs, "⸴ ") + " ▌ ∈ ▚" + toBraille(s.Fons) + "▚"

	case *subsidia.StmtSi:
		code := indent + "∃ " + glyphEmitExpr(s.Cond) + " " + glyphEmitStmt(s.Cons, indent)
		if s.Alt != nil {
			if _, ok := s.Alt.(*subsidia.StmtSi); ok {
				code += " ∄ " + strings.TrimPrefix(glyphEmitStmt(s.Alt, indent), indent+"∃ ")
			} else {
				code += " ∁ " + glyphEmitStmt(s.Alt, indent)
			}
		}
		return code

	case *subsidia.StmtDum:
		return indent + "∞ " + glyphEmitExpr(s.Cond) + " " + glyphEmitStmt(s.Corpus, indent)

	case *subsidia.StmtFacDum:
		return indent + "⊡ " + glyphEmitStmt(s.Corpus, indent) + " ∞ " + glyphEmitExpr(s.Cond) 

	case *subsidia.StmtIteratio:
		kw := "∋" // de
		if s.Species == "Ex" {
			kw = "∈" // ex
		}
		async := ""
		if s.Asynca {
			async = "⋆ "
		}
		return indent + "∀ " + async + toBraille(s.Binding) + " " + kw + " " + glyphEmitExpr(s.Iter) + " " + glyphEmitStmt(s.Corpus, indent)

	case *subsidia.StmtElige:
		lines := []string{}
		lines = append(lines, indent+"⋔ "+glyphEmitExpr(s.Discrim)+" ▐")
		for _, c := range s.Casus {
			lines = append(lines, indent+"  ↳ "+glyphEmitExpr(c.Cond)+" ∴ "+glyphEmitStmt(c.Corpus, indent+"  "))
		}
		if s.Default != nil {
			lines = append(lines, indent+"  ↲ "+glyphEmitStmt(s.Default, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscerne:
		lines := []string{}
		discrims := []string{}
		for _, d := range s.Discrim {
			discrims = append(discrims, glyphEmitExpr(d))
		}
		lines = append(lines, indent+"⦼ "+strings.Join(discrims, "⸴ ")+" ▐")
		for _, c := range s.Casus {
			patterns := []string{}
			for _, p := range c.Patterns {
				if p.Wildcard {
					patterns = append(patterns, "↲")
				} else {
					pat := toBraille(p.Variant)
					if len(p.Bindings) > 0 {
						bindings := []string{}
						for _, b := range p.Bindings {
							bindings = append(bindings, toBraille(b))
						}
						pat += " ▝ " + strings.Join(bindings, "⸴ ") + " ▘"
					}
					if p.Alias != nil {
						pat += " ↦ " + toBraille(*p.Alias)
					}
					patterns = append(patterns, pat)
				}
			}
			lines = append(lines, indent+"  "+strings.Join(patterns, "⸴ ")+" ∴ "+glyphEmitStmt(c.Corpus, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtCustodi:
		lines := []string{}
		for _, c := range s.Clausulae {
			lines = append(lines, indent+"⊧ "+glyphEmitExpr(c.Cond)+" "+glyphEmitStmt(c.Corpus, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtTempta:
		code := indent + "◇ " + glyphEmitStmt(s.Corpus, indent)
		if s.Cape != nil {
			code += " ◆ " + toBraille(s.Cape.Param) + " " + glyphEmitStmt(s.Cape.Corpus, indent)
		}
		if s.Demum != nil {
			code += " ◈ " + glyphEmitStmt(s.Demum, indent)
		}
		return code

	case *subsidia.StmtRedde:
		if s.Valor != nil {
			return indent + "⊢ " + glyphEmitExpr(s.Valor) 
		}
		return indent + "⊢"

	case *subsidia.StmtIace:
		if s.Fatale {
			return indent + "⟂ " + glyphEmitExpr(s.Arg) 
		}
		return indent + "↯ " + glyphEmitExpr(s.Arg) 

	case *subsidia.StmtScribe:
		glyph := "⊝" // scribe
		if s.Gradus == "Vide" {
			glyph = "⋱" // vide
		} else if s.Gradus == "Mone" {
			glyph = "⋮" // mone
		}
		args := []string{}
		for _, arg := range s.Args {
			args = append(args, glyphEmitExpr(arg))
		}
		return indent + glyph + " " + strings.Join(args, "⸴ ") 

	case *subsidia.StmtAdfirma:
		msg := ""
		if s.Msg != nil {
			msg = "⸴ " + glyphEmitExpr(s.Msg)
		}
		return indent + "⊩ " + glyphEmitExpr(s.Cond) + msg 

	case *subsidia.StmtRumpe:
		return indent + "⊗"

	case *subsidia.StmtPerge:
		return indent + "↻"

	case *subsidia.StmtIncipit:
		glyph := "⟙" // incipit
		if s.Asynca {
			glyph = "⫟" // incipiet
		}
		return indent + glyph + " " + glyphEmitStmt(s.Corpus, indent)

	case *subsidia.StmtProbandum:
		lines := []string{}
		lines = append(lines, indent+"⊬ "+toBraille(s.Nomen)+" ▐")
		for _, inner := range s.Corpus {
			lines = append(lines, glyphEmitStmt(inner, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtProba:
		return indent + "⫞ " + toBraille(s.Nomen) + " " + glyphEmitStmt(s.Corpus, indent)

	default:
		return indent + "⌗ unhandled"
	}
}

func glyphEmitExpr(expr subsidia.Expr) string {
	switch e := expr.(type) {
	case *subsidia.ExprNomen:
		if kw, ok := glyphKeywords[e.Valor]; ok {
			return kw
		}
		return toBraille(e.Valor)

	case *subsidia.ExprEgo:
		return "◉"

	case *subsidia.ExprLittera:
		switch e.Species {
		case subsidia.LitteraTextus:
			return "▚" + toBraille(e.Valor) + "▚"
		case subsidia.LitteraVerum:
			return "⊤"
		case subsidia.LitteraFalsum:
			return "⊥"
		case subsidia.LitteraNihil:
			return "∅"
		default:
			return toBraille(e.Valor)
		}

	case *subsidia.ExprBinaria:
		op := e.Signum
		if mapped, ok := glyphOperators[op]; ok {
			op = mapped
		} else if mapped, ok := glyphKeywords[op]; ok {
			op = mapped
		}
		return "▝ " + glyphEmitExpr(e.Sin) + " " + op + " " + glyphEmitExpr(e.Dex) + " ▘"

	case *subsidia.ExprUnaria:
		op := e.Signum
		if mapped, ok := glyphKeywords[op]; ok {
			op = mapped
		} else if mapped, ok := glyphOperators[op]; ok {
			op = mapped
		}
		return "▝ " + op + glyphEmitExpr(e.Arg) + " ▘"

	case *subsidia.ExprAssignatio:
		op := e.Signum
		if mapped, ok := glyphOperators[op]; ok {
			op = mapped
		}
		return glyphEmitExpr(e.Sin) + " " + op + " " + glyphEmitExpr(e.Dex)

	case *subsidia.ExprCondicio:
		return "▝ " + glyphEmitExpr(e.Cond) + " ∴ " + glyphEmitExpr(e.Cons) + " ∁ " + glyphEmitExpr(e.Alt) + " ▘"

	case *subsidia.ExprVocatio:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, glyphEmitExpr(arg))
		}
		return glyphEmitExpr(e.Callee) + " ▝ " + strings.Join(args, "⸴ ") + " ▘"

	case *subsidia.ExprMembrum:
		obj := glyphEmitExpr(e.Obj)
		if e.Computed {
			return obj + " ▗ " + glyphEmitExpr(e.Prop) + " ▖"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = toBraille(lit.Valor)
		} else {
			prop = glyphEmitExpr(e.Prop)
		}
		access := "·"
		if e.NonNull {
			access = "¡·"
		}
		return obj + access + prop

	case *subsidia.ExprSeries:
		elems := []string{}
		for _, elem := range e.Elementa {
			elems = append(elems, glyphEmitExpr(elem))
		}
		return "▗ " + strings.Join(elems, "⸴ ") + " ▖"

	case *subsidia.ExprObiectum:
		props := []string{}
		for _, p := range e.Props {
			if p.Shorthand {
				if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
					props = append(props, toBraille(lit.Valor))
				} else {
					props = append(props, glyphEmitExpr(p.Key))
				}
				continue
			}
			key := ""
			if p.Computed {
				key = "▗ " + glyphEmitExpr(p.Key) + " ▖"
			} else if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = toBraille(lit.Valor)
			} else {
				key = glyphEmitExpr(p.Key)
			}
			props = append(props, key+" ∶ "+glyphEmitExpr(p.Valor))
		}
		return "▐ " + strings.Join(props, "⸴ ") + " ▌"

	case *subsidia.ExprClausura:
		params := []string{}
		for _, param := range e.Params {
			params = append(params, glyphEmitParam(param))
		}
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ " + glyphEmitStmt(body, "")
		case subsidia.Expr:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ " + glyphEmitExpr(body)
		default:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ ▐ ▌"
		}

	case *subsidia.ExprNovum:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, glyphEmitExpr(arg))
		}
		code := "⦿ " + glyphEmitExpr(e.Callee) + " ▝ " + strings.Join(args, "⸴ ") + " ▘"
		if e.Init != nil {
			code += " " + glyphEmitExpr(e.Init)
		}
		return code

	case *subsidia.ExprCede:
		return "⋆ " + glyphEmitExpr(e.Arg)

	case *subsidia.ExprQua:
		return glyphEmitExpr(e.Expr) + " ⦶ " + glyphEmitTypus(e.Typus)

	case *subsidia.ExprInnatum:
		return glyphEmitExpr(e.Expr) + " ⦵ " + glyphEmitTypus(e.Typus)

	case *subsidia.ExprPostfixNovum:
		return glyphEmitExpr(e.Expr) + " ⦿ " + glyphEmitTypus(e.Typus)

	case *subsidia.ExprFinge:
		fields := []string{}
		for _, p := range e.Campi {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = toBraille(lit.Valor)
			} else {
				key = glyphEmitExpr(p.Key)
			}
			fields = append(fields, key+" ∶ "+glyphEmitExpr(p.Valor))
		}
		return "⦺ " + toBraille(e.Variant) + " ▐ " + strings.Join(fields, "⸴ ") + " ▌"

	case *subsidia.ExprScriptum:
		parts := strings.Split(e.Template, "§")
		if len(parts) == 1 {
			return "▚" + toBraille(e.Template) + "▚"
		}
		var b strings.Builder
		b.WriteString("▚")
		for i, part := range parts {
			b.WriteString(toBraille(part))
			if i < len(e.Args) {
				b.WriteString("§")
				b.WriteString(glyphEmitExpr(e.Args[i]))
				b.WriteString("§")
			}
		}
		b.WriteString("▚")
		return b.String()

	case *subsidia.ExprAmbitus:
		start := glyphEmitExpr(e.Start)
		end := glyphEmitExpr(e.End)
		if e.Inclusive {
			return start + " ≼ " + end
		}
		return start + " ≺ " + end

	default:
		return "⌗ unhandled"
	}
}

func glyphEmitTypus(typus subsidia.Typus) string {
	switch t := typus.(type) {
	case *subsidia.TypusNomen:
		if kw, ok := glyphKeywords[t.Nomen]; ok {
			return kw
		}
		return toBraille(t.Nomen)
	case *subsidia.TypusNullabilis:
		return glyphEmitTypus(t.Inner) + "⸮"
	case *subsidia.TypusGenericus:
		args := []string{}
		for _, arg := range t.Args {
			args = append(args, glyphEmitTypus(arg))
		}
		name := t.Nomen
		if kw, ok := glyphKeywords[name]; ok {
			name = kw
		} else {
			name = toBraille(name)
		}
		return name + "▀" + strings.Join(args, "⸴ ") + "▄"
	case *subsidia.TypusFunctio:
		args := []string{}
		for _, p := range t.Params {
			args = append(args, glyphEmitTypus(p))
		}
		return "▝ " + strings.Join(args, "⸴ ") + " ▘ → " + glyphEmitTypus(t.Returns)
	case *subsidia.TypusUnio:
		members := []string{}
		for _, m := range t.Members {
			members = append(members, glyphEmitTypus(m))
		}
		return strings.Join(members, " ∨ ")
	case *subsidia.TypusLitteralis:
		return toBraille(t.Valor)
	default:
		return toBraille("ignotum")
	}
}

func glyphEmitParam(param subsidia.Param) string {
	rest := ""
	if param.Rest {
		rest = "⋯"
	}
	typ := ""
	if param.Typus != nil {
		typ = " ∶ " + glyphEmitTypus(param.Typus)
	}
	def := ""
	if param.Default != nil {
		def = " ← " + glyphEmitExpr(param.Default)
	}
	return rest + toBraille(param.Nomen) + typ + def
}
