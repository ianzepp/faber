package main

import (
	"strings"

	"subsidia"
)

// EmitFaber converts an AST back to Faber source code.
func EmitFaber(mod *subsidia.Modulus) string {
	var b strings.Builder
	for i, stmt := range mod.Corpus {
		if i > 0 {
			b.WriteString("\n")
		}
		b.WriteString(faberEmitStmt(stmt, ""))
	}
	return b.String()
}

func faberEmitStmt(s subsidia.Stmt, indent string) string {
	switch s := s.(type) {
	case *subsidia.StmtVaria:
		return faberEmitVaria(s, indent)
	case *subsidia.StmtFunctio:
		return faberEmitFunctio(s, indent)
	case *subsidia.StmtGenus:
		return faberEmitGenus(s, indent)
	case *subsidia.StmtPactum:
		return faberEmitPactum(s, indent)
	case *subsidia.StmtOrdo:
		return faberEmitOrdo(s, indent)
	case *subsidia.StmtDiscretio:
		return faberEmitDiscretio(s, indent)
	case *subsidia.StmtImporta:
		return faberEmitImporta(s, indent)
	case *subsidia.StmtRedde:
		return faberEmitRedde(s, indent)
	case *subsidia.StmtSi:
		return faberEmitSi(s, indent)
	case *subsidia.StmtDum:
		return faberEmitDum(s, indent)
	case *subsidia.StmtFacDum:
		return faberEmitFacDum(s, indent)
	case *subsidia.StmtIteratio:
		return faberEmitIteratio(s, indent)
	case *subsidia.StmtElige:
		return faberEmitElige(s, indent)
	case *subsidia.StmtDiscerne:
		return faberEmitDiscerne(s, indent)
	case *subsidia.StmtCustodi:
		return faberEmitCustodi(s, indent)
	case *subsidia.StmtTempta:
		return faberEmitTempta(s, indent)
	case *subsidia.StmtIace:
		if s.Fatale {
			if s.Arg != nil {
				return indent + "mori " + faberEmitExpr(s.Arg)
			}
			return indent + "mori"
		}
		return indent + "iace " + faberEmitExpr(s.Arg)
	case *subsidia.StmtRumpe:
		return indent + "rumpe"
	case *subsidia.StmtPerge:
		return indent + "perge"
	case *subsidia.StmtScribe:
		return faberEmitScribe(s, indent)
	case *subsidia.StmtAdfirma:
		code := indent + "adfirma " + faberEmitExpr(s.Cond)
		if s.Msg != nil {
			code += ", " + faberEmitExpr(s.Msg)
		}
		return code
	case *subsidia.StmtExpressia:
		return indent + faberEmitExpr(s.Expr)
	case *subsidia.StmtMassa:
		return faberEmitMassa(s, indent)
	case *subsidia.StmtIncipit:
		return faberEmitIncipit(s, indent)
	case *subsidia.StmtProbandum:
		return faberEmitProbandum(s, indent)
	case *subsidia.StmtProba:
		return faberEmitProba(s, indent)
	default:
		return indent + "/* unknown stmt */"
	}
}

func faberEmitMassa(s *subsidia.StmtMassa, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("{\n")
	for _, stmt := range s.Corpus {
		b.WriteString(faberEmitStmt(stmt, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitVaria(s *subsidia.StmtVaria, indent string) string {
	var keyword string
	switch s.Species {
	case subsidia.VariaFixum:
		keyword = "fixum"
	case subsidia.VariaFigendum:
		keyword = "figendum"
	default:
		keyword = "varia"
	}
	code := indent + keyword + " " + s.Nomen
	if s.Typus != nil {
		code += ": " + faberEmitTypus(s.Typus)
	}
	if s.Valor != nil {
		code += " = " + faberEmitExpr(s.Valor)
	}
	return code
}

func faberEmitFunctio(s *subsidia.StmtFunctio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Asynca {
		b.WriteString("asynca ")
	}
	b.WriteString("functio ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	b.WriteString("(")
	for i, p := range s.Params {
		if i > 0 {
			b.WriteString(", ")
		}
		b.WriteString(faberEmitParam(p))
	}
	b.WriteString(")")
	if s.TypusReditus != nil {
		b.WriteString(": ")
		b.WriteString(faberEmitTypus(s.TypusReditus))
	}
	if s.Corpus != nil {
		b.WriteString(" ")
		b.WriteString(faberEmitStmt(s.Corpus, ""))
	}
	return b.String()
}

func faberEmitGenus(s *subsidia.StmtGenus, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("genus ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	if len(s.Implet) > 0 {
		b.WriteString(" implet ")
		b.WriteString(strings.Join(s.Implet, ", "))
	}
	b.WriteString(" {\n")
	for _, c := range s.Campi {
		b.WriteString(faberEmitCampus(c, indent+"\t"))
		b.WriteString("\n")
	}
	for _, m := range s.Methodi {
		b.WriteString(faberEmitStmt(m, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitCampus(c subsidia.CampusDecl, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString(c.Nomen)
	if c.Typus != nil {
		b.WriteString(": ")
		b.WriteString(faberEmitTypus(c.Typus))
	}
	if c.Valor != nil {
		b.WriteString(" = ")
		b.WriteString(faberEmitExpr(c.Valor))
	}
	return b.String()
}

func faberEmitPactum(s *subsidia.StmtPactum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("pactum ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	b.WriteString(" {\n")
	for _, m := range s.Methodi {
		b.WriteString(indent)
		b.WriteString("\t")
		if m.Asynca {
			b.WriteString("asynca ")
		}
		b.WriteString("functio ")
		b.WriteString(m.Nomen)
		b.WriteString("(")
		for i, p := range m.Params {
			if i > 0 {
				b.WriteString(", ")
			}
			b.WriteString(faberEmitParam(p))
		}
		b.WriteString(")")
		if m.TypusReditus != nil {
			b.WriteString(": ")
			b.WriteString(faberEmitTypus(m.TypusReditus))
		}
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitOrdo(s *subsidia.StmtOrdo, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("ordo ")
	b.WriteString(s.Nomen)
	b.WriteString(" {\n")
	for _, m := range s.Membra {
		b.WriteString(indent)
		b.WriteString("\t")
		b.WriteString(m.Nomen)
		if m.Valor != nil {
			b.WriteString(" = ")
			b.WriteString(*m.Valor)
		}
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitDiscretio(s *subsidia.StmtDiscretio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("discretio ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	b.WriteString(" {\n")
	for _, v := range s.Variantes {
		b.WriteString(indent)
		b.WriteString("\t")
		b.WriteString(v.Nomen)
		if len(v.Campi) > 0 {
			b.WriteString("(")
			for i, f := range v.Campi {
				if i > 0 {
					b.WriteString(", ")
				}
				b.WriteString(f.Nomen)
				b.WriteString(": ")
				b.WriteString(faberEmitTypus(f.Typus))
			}
			b.WriteString(")")
		}
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitImporta(s *subsidia.StmtImporta, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("§ ex \"")
	b.WriteString(s.Fons)
	b.WriteString("\" importa ")
	if s.Totum {
		if s.Alias != nil {
			b.WriteString("* ut ")
			b.WriteString(*s.Alias)
		} else {
			b.WriteString("*")
		}
	} else {
		for i, spec := range s.Specs {
			if i > 0 {
				b.WriteString(", ")
			}
			b.WriteString(spec.Imported)
			if spec.Local != "" && spec.Local != spec.Imported {
				b.WriteString(" ut ")
				b.WriteString(spec.Local)
			}
		}
	}
	return b.String()
}

func faberEmitRedde(s *subsidia.StmtRedde, indent string) string {
	if s.Valor == nil {
		return indent + "redde"
	}
	return indent + "redde " + faberEmitExpr(s.Valor)
}

func faberEmitSi(s *subsidia.StmtSi, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("si ")
	b.WriteString(faberEmitExpr(s.Cond))
	b.WriteString(" ")
	b.WriteString(faberEmitStmt(s.Cons, ""))
	if s.Alt != nil {
		b.WriteString(" secus ")
		b.WriteString(faberEmitStmt(s.Alt, ""))
	}
	return b.String()
}

func faberEmitDum(s *subsidia.StmtDum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("dum ")
	b.WriteString(faberEmitExpr(s.Cond))
	b.WriteString(" ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	return b.String()
}

func faberEmitFacDum(s *subsidia.StmtFacDum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("fac ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	b.WriteString(" dum ")
	b.WriteString(faberEmitExpr(s.Cond))
	return b.String()
}

func faberEmitIteratio(s *subsidia.StmtIteratio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("ex ")
	b.WriteString(faberEmitExpr(s.Iter))
	b.WriteString(" fixum ")
	b.WriteString(s.Binding)
	b.WriteString(" ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	return b.String()
}

func faberEmitElige(s *subsidia.StmtElige, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("elige ")
	b.WriteString(faberEmitExpr(s.Discrim))
	b.WriteString(" {\n")
	for _, c := range s.Casus {
		b.WriteString(indent)
		b.WriteString("\tcasu ")
		b.WriteString(faberEmitExpr(c.Cond))
		b.WriteString(" ")
		b.WriteString(faberEmitStmt(c.Corpus, ""))
		b.WriteString("\n")
	}
	if s.Default != nil {
		b.WriteString(indent)
		b.WriteString("\tceterum ")
		b.WriteString(faberEmitStmt(s.Default, ""))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitDiscerne(s *subsidia.StmtDiscerne, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("discerne ")
	for i, d := range s.Discrim {
		if i > 0 {
			b.WriteString(", ")
		}
		b.WriteString(faberEmitExpr(d))
	}
	b.WriteString(" {\n")
	for _, c := range s.Casus {
		b.WriteString(indent)
		b.WriteString("\tcasu ")
		for i, p := range c.Patterns {
			if i > 0 {
				b.WriteString(", ")
			}
			if p.Wildcard {
				b.WriteString("_")
			} else {
				b.WriteString(p.Variant)
				if len(p.Bindings) > 0 {
					b.WriteString("(")
					b.WriteString(strings.Join(p.Bindings, ", "))
					b.WriteString(")")
				}
			}
		}
		b.WriteString(" ")
		b.WriteString(faberEmitStmt(c.Corpus, ""))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitCustodi(s *subsidia.StmtCustodi, indent string) string {
	var b strings.Builder
	for i, c := range s.Clausulae {
		if i > 0 {
			b.WriteString("\n")
		}
		b.WriteString(indent)
		b.WriteString("custodi ")
		b.WriteString(faberEmitExpr(c.Cond))
		b.WriteString(" ")
		b.WriteString(faberEmitStmt(c.Corpus, ""))
	}
	return b.String()
}

func faberEmitTempta(s *subsidia.StmtTempta, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("tempta ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	if s.Cape != nil {
		b.WriteString(" cape ")
		b.WriteString(s.Cape.Param)
		b.WriteString(" ")
		b.WriteString(faberEmitStmt(s.Cape.Corpus, ""))
	}
	if s.Demum != nil {
		b.WriteString(" demum ")
		b.WriteString(faberEmitStmt(s.Demum, ""))
	}
	return b.String()
}

func faberEmitScribe(s *subsidia.StmtScribe, indent string) string {
	keyword := "scribe"
	switch s.Gradus {
	case "vide":
		keyword = "vide"
	case "mone":
		keyword = "mone"
	}
	args := []string{}
	for _, a := range s.Args {
		args = append(args, faberEmitExpr(a))
	}
	return indent + keyword + " " + strings.Join(args, ", ")
}

func faberEmitIncipit(s *subsidia.StmtIncipit, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Asynca {
		b.WriteString("incipiet")
	} else {
		b.WriteString("incipit")
	}
	b.WriteString(" ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	return b.String()
}

func faberEmitProbandum(s *subsidia.StmtProbandum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("probandum \"")
	b.WriteString(s.Nomen)
	b.WriteString("\" {\n")
	for _, stmt := range s.Corpus {
		b.WriteString(faberEmitStmt(stmt, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func faberEmitProba(s *subsidia.StmtProba, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("proba \"")
	b.WriteString(s.Nomen)
	b.WriteString("\" ")
	b.WriteString(faberEmitStmt(s.Corpus, ""))
	return b.String()
}

func faberEmitExpr(e subsidia.Expr) string {
	if e == nil {
		return ""
	}
	switch e := e.(type) {
	case *subsidia.ExprNomen:
		return e.Valor
	case *subsidia.ExprEgo:
		return "ego"
	case *subsidia.ExprLittera:
		switch e.Species {
		case subsidia.LitteraTextus:
			return "\"" + escapeString(e.Valor) + "\""
		case subsidia.LitteraVerum:
			return "verum"
		case subsidia.LitteraFalsum:
			return "falsum"
		case subsidia.LitteraNihil:
			return "nihil"
		default:
			return e.Valor
		}
	case *subsidia.ExprBinaria:
		return faberEmitExpr(e.Sin) + " " + e.Signum + " " + faberEmitExpr(e.Dex)
	case *subsidia.ExprUnaria:
		// Keyword operators need a space, symbol operators don't
		if e.Signum == "nihil" || e.Signum == "non" || e.Signum == "nonnihil" {
			return e.Signum + " " + faberEmitExpr(e.Arg)
		}
		return e.Signum + faberEmitExpr(e.Arg)
	case *subsidia.ExprAssignatio:
		return faberEmitExpr(e.Sin) + " " + e.Signum + " " + faberEmitExpr(e.Dex)
	case *subsidia.ExprVocatio:
		args := []string{}
		for _, a := range e.Args {
			args = append(args, faberEmitExpr(a))
		}
		return faberEmitExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"
	case *subsidia.ExprMembrum:
		obj := faberEmitExpr(e.Obj)
		if e.Computed {
			return obj + "[" + faberEmitExpr(e.Prop) + "]"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = lit.Valor
		} else {
			prop = faberEmitExpr(e.Prop)
		}
		if e.NonNull {
			return obj + "!." + prop
		}
		return obj + "." + prop
	case *subsidia.ExprCondicio:
		return faberEmitExpr(e.Cond) + " ? " + faberEmitExpr(e.Cons) + " : " + faberEmitExpr(e.Alt)
	case *subsidia.ExprSeries:
		items := []string{}
		for _, i := range e.Elementa {
			items = append(items, faberEmitExpr(i))
		}
		return "[" + strings.Join(items, ", ") + "]"
	case *subsidia.ExprObiectum:
		pairs := []string{}
		for _, p := range e.Props {
			if p.Shorthand {
				pairs = append(pairs, faberEmitExpr(p.Key))
			} else {
				pairs = append(pairs, faberEmitExpr(p.Key)+": "+faberEmitExpr(p.Valor))
			}
		}
		return "{" + strings.Join(pairs, ", ") + "}"
	case *subsidia.ExprClausura:
		params := []string{}
		for _, p := range e.Params {
			params = append(params, faberEmitParam(p))
		}
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "(" + strings.Join(params, ", ") + ") => " + faberEmitStmt(body, "")
		case subsidia.Expr:
			return "(" + strings.Join(params, ", ") + ") => " + faberEmitExpr(body)
		default:
			return "(" + strings.Join(params, ", ") + ") => {}"
		}
	case *subsidia.ExprNovum:
		args := []string{}
		for _, a := range e.Args {
			args = append(args, faberEmitExpr(a))
		}
		code := "novum " + faberEmitExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"
		if e.Init != nil {
			code += " " + faberEmitExpr(e.Init)
		}
		return code
	case *subsidia.ExprQua:
		return faberEmitExpr(e.Expr) + " qua " + faberEmitTypus(e.Typus)
	case *subsidia.ExprInnatum:
		return "innatum " + faberEmitExpr(e.Expr)
	case *subsidia.ExprCede:
		return "cede " + faberEmitExpr(e.Arg)
	case *subsidia.ExprFinge:
		pairs := []string{}
		for _, p := range e.Campi {
			if p.Shorthand {
				pairs = append(pairs, faberEmitExpr(p.Key))
			} else {
				pairs = append(pairs, faberEmitExpr(p.Key)+": "+faberEmitExpr(p.Valor))
			}
		}
		return "finge " + e.Variant + " {" + strings.Join(pairs, ", ") + "}"
	case *subsidia.ExprScriptum:
		return "`" + e.Template + "`"
	case *subsidia.ExprAmbitus:
		if e.Inclusive {
			return faberEmitExpr(e.Start) + " usque " + faberEmitExpr(e.End)
		}
		return faberEmitExpr(e.Start) + " ante " + faberEmitExpr(e.End)
	case *subsidia.ExprPostfixNovum:
		return faberEmitExpr(e.Expr) + "!"
	default:
		return "/* unknown expr */"
	}
}

func faberEmitTypus(t subsidia.Typus) string {
	if t == nil {
		return ""
	}
	switch t := t.(type) {
	case *subsidia.TypusNomen:
		return t.Nomen
	case *subsidia.TypusGenericus:
		args := []string{}
		for _, a := range t.Args {
			args = append(args, faberEmitTypus(a))
		}
		return t.Nomen + "<" + strings.Join(args, ", ") + ">"
	case *subsidia.TypusFunctio:
		params := []string{}
		for _, p := range t.Params {
			params = append(params, faberEmitTypus(p))
		}
		return "(" + strings.Join(params, ", ") + ") -> " + faberEmitTypus(t.Returns)
	case *subsidia.TypusNullabilis:
		return faberEmitTypus(t.Inner) + "?"
	case *subsidia.TypusUnio:
		parts := []string{}
		for _, p := range t.Members {
			parts = append(parts, faberEmitTypus(p))
		}
		return strings.Join(parts, " | ")
	case *subsidia.TypusLitteralis:
		return t.Valor
	default:
		return "/* unknown type */"
	}
}

func faberEmitParam(p subsidia.Param) string {
	var b strings.Builder
	if p.Rest {
		b.WriteString("...")
	}
	b.WriteString(p.Nomen)
	if p.Typus != nil {
		b.WriteString(": ")
		b.WriteString(faberEmitTypus(p.Typus))
	}
	if p.Default != nil {
		b.WriteString(" = ")
		b.WriteString(faberEmitExpr(p.Default))
	}
	return b.String()
}

func escapeString(s string) string {
	var b strings.Builder
	for _, r := range s {
		switch r {
		case '\n':
			b.WriteString("\\n")
		case '\t':
			b.WriteString("\\t")
		case '\r':
			b.WriteString("\\r")
		case '\\':
			b.WriteString("\\\\")
		case '"':
			b.WriteString("\\\"")
		default:
			b.WriteRune(r)
		}
	}
	return b.String()
}
