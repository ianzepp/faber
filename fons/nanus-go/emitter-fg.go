package nanus

import (
	"strings"

	"subsidia"
)

// Glyph mappings from GLYPH.md

// Delimiters (Block Characters U+2580-U+259F)
var fgDelimiters = map[rune]rune{
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
var fgPunctuation = map[rune]rune{
	';': '\u204F', // 	',': '\u2E34', // ⸴
	'.': '\u00B7', // ·
	':': '\u2236', // ∶
}

// Keywords
var fgKeywords = map[string]string{
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
var fgOperators = map[string]string{
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

// EmitFG renders a module to Faber Glyph format.
func EmitFG(mod *subsidia.Modulus) string {
	lines := []string{}
	for _, stmt := range mod.Corpus {
		lines = append(lines, fgEmitStmt(stmt, ""))
	}
	return strings.Join(lines, "\n")
}

func fgEmitStmt(stmt subsidia.Stmt, indent string) string {
	switch s := stmt.(type) {
	case *subsidia.StmtMassa:
		lines := []string{}
		for _, inner := range s.Corpus {
			lines = append(lines, fgEmitStmt(inner, indent+"  "))
		}
		return "▐\n" + strings.Join(lines, "\n") + "\n" + indent + "▌"

	case *subsidia.StmtExpressia:
		return indent + fgEmitExpr(s.Expr)

	case *subsidia.StmtVaria:
		kw := "≔" // varia
		if s.Species == subsidia.VariaFixum {
			kw = "≡" // fixum
		}
		typ := ""
		if s.Typus != nil {
			typ = " ∶ " + fgEmitTypus(s.Typus)
		}
		val := ""
		if s.Valor != nil {
			val = " ← " + fgEmitExpr(s.Valor)
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
			params = append(params, fgEmitParam(param))
		}
		ret := ""
		if s.TypusReditus != nil {
			ret = " → " + fgEmitTypus(s.TypusReditus)
		}
		body := ""
		if s.Corpus != nil {
			body = " " + fgEmitStmt(s.Corpus, indent)
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
				val = " ← " + fgEmitExpr(campo.Valor)
			}
			lines = append(lines, indent+"  "+toBraille(campo.Nomen)+" ∶ "+fgEmitTypus(campo.Typus)+val+"")
		}

		for _, method := range s.Methodi {
			if fn, ok := method.(*subsidia.StmtFunctio); ok {
				lines = append(lines, "")
				lines = append(lines, fgEmitStmt(fn, indent+"  "))
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
				params = append(params, fgEmitParam(param))
			}
			ret := ""
			if method.TypusReditus != nil {
				ret = " → " + fgEmitTypus(method.TypusReditus)
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
					fields = append(fields, toBraille(f.Nomen)+" ∶ "+fgEmitTypus(f.Typus))
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
		code := indent + "∃ " + fgEmitExpr(s.Cond) + " " + fgEmitStmt(s.Cons, indent)
		if s.Alt != nil {
			if _, ok := s.Alt.(*subsidia.StmtSi); ok {
				code += " ∄ " + strings.TrimPrefix(fgEmitStmt(s.Alt, indent), indent+"∃ ")
			} else {
				code += " ∁ " + fgEmitStmt(s.Alt, indent)
			}
		}
		return code

	case *subsidia.StmtDum:
		return indent + "∞ " + fgEmitExpr(s.Cond) + " " + fgEmitStmt(s.Corpus, indent)

	case *subsidia.StmtFacDum:
		return indent + "⊡ " + fgEmitStmt(s.Corpus, indent) + " ∞ " + fgEmitExpr(s.Cond) 

	case *subsidia.StmtIteratio:
		kw := "∋" // de
		if s.Species == "Ex" {
			kw = "∈" // ex
		}
		async := ""
		if s.Asynca {
			async = "⋆ "
		}
		return indent + "∀ " + async + toBraille(s.Binding) + " " + kw + " " + fgEmitExpr(s.Iter) + " " + fgEmitStmt(s.Corpus, indent)

	case *subsidia.StmtElige:
		lines := []string{}
		lines = append(lines, indent+"⋔ "+fgEmitExpr(s.Discrim)+" ▐")
		for _, c := range s.Casus {
			lines = append(lines, indent+"  ↳ "+fgEmitExpr(c.Cond)+" ∴ "+fgEmitStmt(c.Corpus, indent+"  "))
		}
		if s.Default != nil {
			lines = append(lines, indent+"  ↲ "+fgEmitStmt(s.Default, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscerne:
		lines := []string{}
		discrims := []string{}
		for _, d := range s.Discrim {
			discrims = append(discrims, fgEmitExpr(d))
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
			lines = append(lines, indent+"  "+strings.Join(patterns, "⸴ ")+" ∴ "+fgEmitStmt(c.Corpus, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtCustodi:
		lines := []string{}
		for _, c := range s.Clausulae {
			lines = append(lines, indent+"⊧ "+fgEmitExpr(c.Cond)+" "+fgEmitStmt(c.Corpus, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtTempta:
		code := indent + "◇ " + fgEmitStmt(s.Corpus, indent)
		if s.Cape != nil {
			code += " ◆ " + toBraille(s.Cape.Param) + " " + fgEmitStmt(s.Cape.Corpus, indent)
		}
		if s.Demum != nil {
			code += " ◈ " + fgEmitStmt(s.Demum, indent)
		}
		return code

	case *subsidia.StmtRedde:
		if s.Valor != nil {
			return indent + "⊢ " + fgEmitExpr(s.Valor) 
		}
		return indent + "⊢"

	case *subsidia.StmtIace:
		if s.Fatale {
			return indent + "⟂ " + fgEmitExpr(s.Arg) 
		}
		return indent + "↯ " + fgEmitExpr(s.Arg) 

	case *subsidia.StmtScribe:
		glyph := "⊝" // scribe
		if s.Gradus == "Vide" {
			glyph = "⋱" // vide
		} else if s.Gradus == "Mone" {
			glyph = "⋮" // mone
		}
		args := []string{}
		for _, arg := range s.Args {
			args = append(args, fgEmitExpr(arg))
		}
		return indent + glyph + " " + strings.Join(args, "⸴ ") 

	case *subsidia.StmtAdfirma:
		msg := ""
		if s.Msg != nil {
			msg = "⸴ " + fgEmitExpr(s.Msg)
		}
		return indent + "⊩ " + fgEmitExpr(s.Cond) + msg 

	case *subsidia.StmtRumpe:
		return indent + "⊗"

	case *subsidia.StmtPerge:
		return indent + "↻"

	case *subsidia.StmtIncipit:
		glyph := "⟙" // incipit
		if s.Asynca {
			glyph = "⫟" // incipiet
		}
		return indent + glyph + " " + fgEmitStmt(s.Corpus, indent)

	case *subsidia.StmtProbandum:
		lines := []string{}
		lines = append(lines, indent+"⊬ "+toBraille(s.Nomen)+" ▐")
		for _, inner := range s.Corpus {
			lines = append(lines, fgEmitStmt(inner, indent+"  "))
		}
		lines = append(lines, indent+"▌")
		return strings.Join(lines, "\n")

	case *subsidia.StmtProba:
		return indent + "⫞ " + toBraille(s.Nomen) + " " + fgEmitStmt(s.Corpus, indent)

	default:
		return indent + "⌗ unhandled"
	}
}

func fgEmitExpr(expr subsidia.Expr) string {
	switch e := expr.(type) {
	case *subsidia.ExprNomen:
		if kw, ok := fgKeywords[e.Valor]; ok {
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
		if mapped, ok := fgOperators[op]; ok {
			op = mapped
		} else if mapped, ok := fgKeywords[op]; ok {
			op = mapped
		}
		return "▝ " + fgEmitExpr(e.Sin) + " " + op + " " + fgEmitExpr(e.Dex) + " ▘"

	case *subsidia.ExprUnaria:
		op := e.Signum
		if mapped, ok := fgKeywords[op]; ok {
			op = mapped
		} else if mapped, ok := fgOperators[op]; ok {
			op = mapped
		}
		return "▝ " + op + fgEmitExpr(e.Arg) + " ▘"

	case *subsidia.ExprAssignatio:
		op := e.Signum
		if mapped, ok := fgOperators[op]; ok {
			op = mapped
		}
		return fgEmitExpr(e.Sin) + " " + op + " " + fgEmitExpr(e.Dex)

	case *subsidia.ExprCondicio:
		return "▝ " + fgEmitExpr(e.Cond) + " ∴ " + fgEmitExpr(e.Cons) + " ∁ " + fgEmitExpr(e.Alt) + " ▘"

	case *subsidia.ExprVocatio:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, fgEmitExpr(arg))
		}
		return fgEmitExpr(e.Callee) + " ▝ " + strings.Join(args, "⸴ ") + " ▘"

	case *subsidia.ExprMembrum:
		obj := fgEmitExpr(e.Obj)
		if e.Computed {
			return obj + " ▗ " + fgEmitExpr(e.Prop) + " ▖"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = toBraille(lit.Valor)
		} else {
			prop = fgEmitExpr(e.Prop)
		}
		access := "·"
		if e.NonNull {
			access = "¡·"
		}
		return obj + access + prop

	case *subsidia.ExprSeries:
		elems := []string{}
		for _, elem := range e.Elementa {
			elems = append(elems, fgEmitExpr(elem))
		}
		return "▗ " + strings.Join(elems, "⸴ ") + " ▖"

	case *subsidia.ExprObiectum:
		props := []string{}
		for _, p := range e.Props {
			if p.Shorthand {
				if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
					props = append(props, toBraille(lit.Valor))
				} else {
					props = append(props, fgEmitExpr(p.Key))
				}
				continue
			}
			key := ""
			if p.Computed {
				key = "▗ " + fgEmitExpr(p.Key) + " ▖"
			} else if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = toBraille(lit.Valor)
			} else {
				key = fgEmitExpr(p.Key)
			}
			props = append(props, key+" ∶ "+fgEmitExpr(p.Valor))
		}
		return "▐ " + strings.Join(props, "⸴ ") + " ▌"

	case *subsidia.ExprClausura:
		params := []string{}
		for _, param := range e.Params {
			params = append(params, fgEmitParam(param))
		}
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ " + fgEmitStmt(body, "")
		case subsidia.Expr:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ " + fgEmitExpr(body)
		default:
			return "▝ " + strings.Join(params, "⸴ ") + " ▘ ⇒ ▐ ▌"
		}

	case *subsidia.ExprNovum:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, fgEmitExpr(arg))
		}
		code := "⦿ " + fgEmitExpr(e.Callee) + " ▝ " + strings.Join(args, "⸴ ") + " ▘"
		if e.Init != nil {
			code += " " + fgEmitExpr(e.Init)
		}
		return code

	case *subsidia.ExprCede:
		return "⋆ " + fgEmitExpr(e.Arg)

	case *subsidia.ExprQua:
		return fgEmitExpr(e.Expr) + " ⦶ " + fgEmitTypus(e.Typus)

	case *subsidia.ExprInnatum:
		return fgEmitExpr(e.Expr) + " ⦵ " + fgEmitTypus(e.Typus)

	case *subsidia.ExprPostfixNovum:
		return fgEmitExpr(e.Expr) + " ⦿ " + fgEmitTypus(e.Typus)

	case *subsidia.ExprFinge:
		fields := []string{}
		for _, p := range e.Campi {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = toBraille(lit.Valor)
			} else {
				key = fgEmitExpr(p.Key)
			}
			fields = append(fields, key+" ∶ "+fgEmitExpr(p.Valor))
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
				b.WriteString(fgEmitExpr(e.Args[i]))
				b.WriteString("§")
			}
		}
		b.WriteString("▚")
		return b.String()

	case *subsidia.ExprAmbitus:
		start := fgEmitExpr(e.Start)
		end := fgEmitExpr(e.End)
		if e.Inclusive {
			return start + " ≼ " + end
		}
		return start + " ≺ " + end

	default:
		return "⌗ unhandled"
	}
}

func fgEmitTypus(typus subsidia.Typus) string {
	switch t := typus.(type) {
	case *subsidia.TypusNomen:
		if kw, ok := fgKeywords[t.Nomen]; ok {
			return kw
		}
		return toBraille(t.Nomen)
	case *subsidia.TypusNullabilis:
		return fgEmitTypus(t.Inner) + "⸮"
	case *subsidia.TypusGenericus:
		args := []string{}
		for _, arg := range t.Args {
			args = append(args, fgEmitTypus(arg))
		}
		name := t.Nomen
		if kw, ok := fgKeywords[name]; ok {
			name = kw
		} else {
			name = toBraille(name)
		}
		return name + "▀" + strings.Join(args, "⸴ ") + "▄"
	case *subsidia.TypusFunctio:
		args := []string{}
		for _, p := range t.Params {
			args = append(args, fgEmitTypus(p))
		}
		return "▝ " + strings.Join(args, "⸴ ") + " ▘ → " + fgEmitTypus(t.Returns)
	case *subsidia.TypusUnio:
		members := []string{}
		for _, m := range t.Members {
			members = append(members, fgEmitTypus(m))
		}
		return strings.Join(members, " ∨ ")
	case *subsidia.TypusLitteralis:
		return toBraille(t.Valor)
	default:
		return toBraille("ignotum")
	}
}

func fgEmitParam(param subsidia.Param) string {
	rest := ""
	if param.Rest {
		rest = "⋯"
	}
	typ := ""
	if param.Typus != nil {
		typ = " ∶ " + fgEmitTypus(param.Typus)
	}
	def := ""
	if param.Default != nil {
		def = " ← " + fgEmitExpr(param.Default)
	}
	return rest + toBraille(param.Nomen) + typ + def
}
