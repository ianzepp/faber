package main

import (
	"strings"

	"subsidia"
)

// EmitFaber formats an AST back to canonical Faber source.
func EmitFaber(mod *subsidia.Modulus) string {
	var b strings.Builder
	for i, stmt := range mod.Corpus {
		if i > 0 {
			b.WriteString("\n")
		}
		b.WriteString(fabStmt(stmt, ""))
	}
	return b.String()
}

func fabStmt(s subsidia.Stmt, indent string) string {
	switch s := s.(type) {
	case *subsidia.StmtVaria:
		return fabVaria(s, indent)
	case *subsidia.StmtFunctio:
		return fabFunctio(s, indent)
	case *subsidia.StmtGenus:
		return fabGenus(s, indent)
	case *subsidia.StmtPactum:
		return fabPactum(s, indent)
	case *subsidia.StmtOrdo:
		return fabOrdo(s, indent)
	case *subsidia.StmtDiscretio:
		return fabDiscretio(s, indent)
	case *subsidia.StmtImporta:
		return fabImporta(s, indent)
	case *subsidia.StmtRedde:
		return fabRedde(s, indent)
	case *subsidia.StmtSi:
		return fabSi(s, indent)
	case *subsidia.StmtDum:
		return fabDum(s, indent)
	case *subsidia.StmtFacDum:
		return fabFacDum(s, indent)
	case *subsidia.StmtIteratio:
		return fabIteratio(s, indent)
	case *subsidia.StmtElige:
		return fabElige(s, indent)
	case *subsidia.StmtDiscerne:
		return fabDiscerne(s, indent)
	case *subsidia.StmtCustodi:
		return fabCustodi(s, indent)
	case *subsidia.StmtTempta:
		return fabTempta(s, indent)
	case *subsidia.StmtIace:
		if s.Fatale {
			if s.Arg != nil {
				return indent + "mori " + fabExpr(s.Arg)
			}
			return indent + "mori"
		}
		return indent + "iace " + fabExpr(s.Arg)
	case *subsidia.StmtRumpe:
		return indent + "rumpe"
	case *subsidia.StmtPerge:
		return indent + "perge"
	case *subsidia.StmtScribe:
		return fabScribe(s, indent)
	case *subsidia.StmtAdfirma:
		code := indent + "adfirma " + fabExpr(s.Cond)
		if s.Msg != nil {
			code += ", " + fabExpr(s.Msg)
		}
		return code
	case *subsidia.StmtExpressia:
		return indent + fabExpr(s.Expr)
	case *subsidia.StmtMassa:
		return fabMassa(s, indent)
	case *subsidia.StmtIncipit:
		return fabIncipit(s, indent)
	case *subsidia.StmtProbandum:
		return fabProbandum(s, indent)
	case *subsidia.StmtProba:
		return fabProba(s, indent)
	default:
		return indent + "# unknown statement"
	}
}

func fabMassa(s *subsidia.StmtMassa, indent string) string {
	var b strings.Builder
	b.WriteString("{\n")
	for _, stmt := range s.Corpus {
		b.WriteString(fabStmt(stmt, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabVaria(s *subsidia.StmtVaria, indent string) string {
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
		code += ": " + fabTypus(s.Typus)
	}
	if s.Valor != nil {
		code += " = " + fabExpr(s.Valor)
	}
	return code
}

func fabFunctio(s *subsidia.StmtFunctio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Publica {
		b.WriteString("publica ")
	}
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
		b.WriteString(fabParam(p))
	}
	b.WriteString(")")
	if s.TypusReditus != nil {
		b.WriteString(": ")
		b.WriteString(fabTypus(s.TypusReditus))
	}
	if s.Corpus != nil {
		b.WriteString(" ")
		b.WriteString(fabStmt(s.Corpus, indent))
	}
	return b.String()
}

func fabGenus(s *subsidia.StmtGenus, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Publica {
		b.WriteString("publica ")
	}
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
		b.WriteString(fabCampus(c, indent+"\t"))
		b.WriteString("\n")
	}
	for _, m := range s.Methodi {
		b.WriteString(fabStmt(m, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabCampus(c subsidia.CampusDecl, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString(c.Nomen)
	if c.Typus != nil {
		b.WriteString(": ")
		b.WriteString(fabTypus(c.Typus))
	}
	if c.Valor != nil {
		b.WriteString(" = ")
		b.WriteString(fabExpr(c.Valor))
	}
	return b.String()
}

func fabPactum(s *subsidia.StmtPactum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Publica {
		b.WriteString("publica ")
	}
	b.WriteString("pactum ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	b.WriteString(" {\n")
	for _, m := range s.Methodi {
		b.WriteString(indent + "\t")
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
			b.WriteString(fabParam(p))
		}
		b.WriteString(")")
		if m.TypusReditus != nil {
			b.WriteString(": ")
			b.WriteString(fabTypus(m.TypusReditus))
		}
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabOrdo(s *subsidia.StmtOrdo, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Publica {
		b.WriteString("publica ")
	}
	b.WriteString("ordo ")
	b.WriteString(s.Nomen)
	b.WriteString(" {\n")
	for _, m := range s.Membra {
		b.WriteString(indent + "\t")
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

func fabDiscretio(s *subsidia.StmtDiscretio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Publica {
		b.WriteString("publica ")
	}
	b.WriteString("discretio ")
	b.WriteString(s.Nomen)
	if len(s.Generics) > 0 {
		b.WriteString("<")
		b.WriteString(strings.Join(s.Generics, ", "))
		b.WriteString(">")
	}
	b.WriteString(" {\n")
	for _, v := range s.Variantes {
		b.WriteString(indent + "\t")
		b.WriteString(v.Nomen)
		if len(v.Campi) > 0 {
			b.WriteString("(")
			for i, f := range v.Campi {
				if i > 0 {
					b.WriteString(", ")
				}
				b.WriteString(f.Nomen)
				b.WriteString(": ")
				b.WriteString(fabTypus(f.Typus))
			}
			b.WriteString(")")
		}
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabImporta(s *subsidia.StmtImporta, indent string) string {
	var b strings.Builder
	// Always emit new syntax: § importa ex "path" bindings
	b.WriteString(indent)
	b.WriteString("§ importa ex \"")
	b.WriteString(s.Fons)
	b.WriteString("\" ")
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

func fabRedde(s *subsidia.StmtRedde, indent string) string {
	if s.Valor == nil {
		return indent + "redde"
	}
	return indent + "redde " + fabExpr(s.Valor)
}

func fabSi(s *subsidia.StmtSi, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("si ")
	b.WriteString(fabExpr(s.Cond))
	b.WriteString(" ")
	b.WriteString(fabStmt(s.Cons, indent))
	if s.Alt != nil {
		b.WriteString(" secus ")
		b.WriteString(fabStmt(s.Alt, indent))
	}
	return b.String()
}

func fabDum(s *subsidia.StmtDum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("dum ")
	b.WriteString(fabExpr(s.Cond))
	b.WriteString(" ")
	b.WriteString(fabStmt(s.Corpus, indent))
	return b.String()
}

func fabFacDum(s *subsidia.StmtFacDum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("fac ")
	b.WriteString(fabStmt(s.Corpus, indent))
	b.WriteString(" dum ")
	b.WriteString(fabExpr(s.Cond))
	return b.String()
}

func fabIteratio(s *subsidia.StmtIteratio, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Asynca {
		b.WriteString("cede ")
	}
	b.WriteString("pro ")
	b.WriteString(s.Binding)
	b.WriteString(" de ")
	b.WriteString(fabExpr(s.Iter))
	b.WriteString(" ")
	b.WriteString(fabStmt(s.Corpus, indent))
	return b.String()
}

func fabElige(s *subsidia.StmtElige, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("elige ")
	b.WriteString(fabExpr(s.Discrim))
	b.WriteString(" {\n")
	for _, c := range s.Casus {
		b.WriteString(indent + "\tcasu ")
		b.WriteString(fabExpr(c.Cond))
		b.WriteString(" ")
		b.WriteString(fabStmt(c.Corpus, indent+"\t"))
		b.WriteString("\n")
	}
	if s.Default != nil {
		b.WriteString(indent + "\tceterum ")
		b.WriteString(fabStmt(s.Default, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabDiscerne(s *subsidia.StmtDiscerne, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("discerne ")
	for i, d := range s.Discrim {
		if i > 0 {
			b.WriteString(", ")
		}
		b.WriteString(fabExpr(d))
	}
	b.WriteString(" {\n")
	for _, c := range s.Casus {
		b.WriteString(indent + "\tcasu ")
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
				if p.Alias != nil {
					b.WriteString(" ut ")
					b.WriteString(*p.Alias)
				}
			}
		}
		b.WriteString(" ")
		b.WriteString(fabStmt(c.Corpus, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabCustodi(s *subsidia.StmtCustodi, indent string) string {
	var b strings.Builder
	for i, c := range s.Clausulae {
		if i > 0 {
			b.WriteString("\n")
		}
		b.WriteString(indent)
		b.WriteString("custodi ")
		b.WriteString(fabExpr(c.Cond))
		b.WriteString(" ")
		b.WriteString(fabStmt(c.Corpus, indent))
	}
	return b.String()
}

func fabTempta(s *subsidia.StmtTempta, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("tempta ")
	b.WriteString(fabStmt(s.Corpus, indent))
	if s.Cape != nil {
		b.WriteString(" cape ")
		b.WriteString(s.Cape.Param)
		b.WriteString(" ")
		b.WriteString(fabStmt(s.Cape.Corpus, indent))
	}
	if s.Demum != nil {
		b.WriteString(" demum ")
		b.WriteString(fabStmt(s.Demum, indent))
	}
	return b.String()
}

func fabScribe(s *subsidia.StmtScribe, indent string) string {
	keyword := "scribe"
	switch s.Gradus {
	case "Vide":
		keyword = "vide"
	case "Mone":
		keyword = "mone"
	}
	args := []string{}
	for _, a := range s.Args {
		args = append(args, fabExpr(a))
	}
	return indent + keyword + " " + strings.Join(args, ", ")
}

func fabIncipit(s *subsidia.StmtIncipit, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	if s.Asynca {
		b.WriteString("incipiet")
	} else {
		b.WriteString("incipit")
	}
	b.WriteString(" ")
	b.WriteString(fabStmt(s.Corpus, indent))
	return b.String()
}

func fabProbandum(s *subsidia.StmtProbandum, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("probandum \"")
	b.WriteString(s.Nomen)
	b.WriteString("\" {\n")
	for _, stmt := range s.Corpus {
		b.WriteString(fabStmt(stmt, indent+"\t"))
		b.WriteString("\n")
	}
	b.WriteString(indent)
	b.WriteString("}")
	return b.String()
}

func fabProba(s *subsidia.StmtProba, indent string) string {
	var b strings.Builder
	b.WriteString(indent)
	b.WriteString("proba \"")
	b.WriteString(s.Nomen)
	b.WriteString("\" ")
	b.WriteString(fabStmt(s.Corpus, indent))
	return b.String()
}

func fabExpr(e subsidia.Expr) string {
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
		return fabExpr(e.Sin) + " " + e.Signum + " " + fabExpr(e.Dex)
	case *subsidia.ExprUnaria:
		if e.Signum == "nihil" || e.Signum == "non" || e.Signum == "nonnihil" {
			return e.Signum + " " + fabExpr(e.Arg)
		}
		return e.Signum + fabExpr(e.Arg)
	case *subsidia.ExprAssignatio:
		return fabExpr(e.Sin) + " " + e.Signum + " " + fabExpr(e.Dex)
	case *subsidia.ExprVocatio:
		args := []string{}
		for _, a := range e.Args {
			args = append(args, fabExpr(a))
		}
		return fabExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"
	case *subsidia.ExprMembrum:
		obj := fabExpr(e.Obj)
		if e.Computed {
			return obj + "[" + fabExpr(e.Prop) + "]"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = lit.Valor
		} else {
			prop = fabExpr(e.Prop)
		}
		if e.NonNull {
			return obj + "!." + prop
		}
		return obj + "." + prop
	case *subsidia.ExprCondicio:
		return fabExpr(e.Cond) + " ? " + fabExpr(e.Cons) + " : " + fabExpr(e.Alt)
	case *subsidia.ExprSeries:
		items := []string{}
		for _, i := range e.Elementa {
			items = append(items, fabExpr(i))
		}
		return "[" + strings.Join(items, ", ") + "]"
	case *subsidia.ExprObiectum:
		pairs := []string{}
		for _, p := range e.Props {
			if p.Shorthand {
				pairs = append(pairs, fabExpr(p.Key))
			} else {
				pairs = append(pairs, fabExpr(p.Key)+": "+fabExpr(p.Valor))
			}
		}
		return "{ " + strings.Join(pairs, ", ") + " }"
	case *subsidia.ExprClausura:
		params := []string{}
		for _, p := range e.Params {
			params = append(params, fabParam(p))
		}
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "(" + strings.Join(params, ", ") + ") => " + fabStmt(body, "")
		case subsidia.Expr:
			return "(" + strings.Join(params, ", ") + ") => " + fabExpr(body)
		default:
			return "(" + strings.Join(params, ", ") + ") => {}"
		}
	case *subsidia.ExprNovum:
		args := []string{}
		for _, a := range e.Args {
			args = append(args, fabExpr(a))
		}
		code := "novum " + fabExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"
		if e.Init != nil {
			code += " " + fabExpr(e.Init)
		}
		return code
	case *subsidia.ExprQua:
		return fabExpr(e.Expr) + " qua " + fabTypus(e.Typus)
	case *subsidia.ExprInnatum:
		return "innatum " + fabExpr(e.Expr)
	case *subsidia.ExprCede:
		return "cede " + fabExpr(e.Arg)
	case *subsidia.ExprFinge:
		pairs := []string{}
		for _, p := range e.Campi {
			if p.Shorthand {
				pairs = append(pairs, fabExpr(p.Key))
			} else {
				pairs = append(pairs, fabExpr(p.Key)+": "+fabExpr(p.Valor))
			}
		}
		return "finge " + e.Variant + " { " + strings.Join(pairs, ", ") + " }"
	case *subsidia.ExprScriptum:
		return "scriptum(\"" + e.Template + "\")"
	case *subsidia.ExprAmbitus:
		if e.Inclusive {
			return fabExpr(e.Start) + " usque " + fabExpr(e.End)
		}
		return fabExpr(e.Start) + " ante " + fabExpr(e.End)
	case *subsidia.ExprPostfixNovum:
		return fabExpr(e.Expr) + " novum " + fabTypus(e.Typus)
	default:
		return "# unknown expr"
	}
}

func fabTypus(t subsidia.Typus) string {
	if t == nil {
		return ""
	}
	switch t := t.(type) {
	case *subsidia.TypusNomen:
		return t.Nomen
	case *subsidia.TypusGenericus:
		args := []string{}
		for _, a := range t.Args {
			args = append(args, fabTypus(a))
		}
		return t.Nomen + "<" + strings.Join(args, ", ") + ">"
	case *subsidia.TypusFunctio:
		params := []string{}
		for _, p := range t.Params {
			params = append(params, fabTypus(p))
		}
		return "(" + strings.Join(params, ", ") + ") -> " + fabTypus(t.Returns)
	case *subsidia.TypusNullabilis:
		return fabTypus(t.Inner) + "?"
	case *subsidia.TypusUnio:
		parts := []string{}
		for _, p := range t.Members {
			parts = append(parts, fabTypus(p))
		}
		return strings.Join(parts, " | ")
	case *subsidia.TypusLitteralis:
		return t.Valor
	default:
		return "# unknown type"
	}
}

func fabParam(p subsidia.Param) string {
	var b strings.Builder
	if p.Rest {
		b.WriteString("...")
	}
	b.WriteString(p.Nomen)
	if p.Typus != nil {
		b.WriteString(": ")
		b.WriteString(fabTypus(p.Typus))
	}
	if p.Default != nil {
		b.WriteString(" = ")
		b.WriteString(fabExpr(p.Default))
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
