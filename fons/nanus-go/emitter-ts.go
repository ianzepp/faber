package main

import (
	"strconv"
	"strings"

	"subsidia"
)

var binaryOps = map[string]string{
	"et":    "&&",
	"aut":   "||",
	"vel":   "??",
	"inter": "in",
	"intra": "instanceof",
}

var unaryOpsEmit = map[string]string{
	"non":       "!",
	"nihil":     "!",
	"nonnihil":  "!!",
	"positivum": "+",
}

var methodMap = map[string]string{
	"adde":          "push",
	"praepone":      "unshift",
	"remove":        "pop",
	"decapita":      "shift",
	"coniunge":      "join",
	"continet":      "includes",
	"indiceDe":      "indexOf",
	"inveni":        "find",
	"inveniIndicem": "findIndex",
	"omnes":         "every",
	"aliquis":       "some",
	"filtrata":      "filter",
	"mappata":       "map",
	"explanata":     "flatMap",
	"plana":         "flat",
	"sectio":        "slice",
	"reducta":       "reduce",
	"perambula":     "forEach",
	"inverte":       "reverse",
	"ordina":        "sort",
	"pone":          "set",
	"accipe":        "get",
	"habet":         "has",
	"dele":          "delete",
	"purga":         "clear",
	"claves":        "keys",
	"valores":       "values",
	"paria":         "entries",
	"initium":       "startsWith",
	"finis":         "endsWith",
	"maiuscula":     "toUpperCase",
	"minuscula":     "toLowerCase",
	"recide":        "trim",
	"divide":        "split",
	"muta":          "replaceAll",
	"longitudo":     "length",
}

var propertyOnly = map[string]struct{}{
	"longitudo": {},
	"primus":    {},
	"ultimus":   {},
}

// EmitTS renders a module to TypeScript.
func EmitTS(mod *subsidia.Modulus) string {
	lines := []string{}
	for _, stmt := range mod.Corpus {
		lines = append(lines, emitStmt(stmt, ""))
	}
	return strings.Join(lines, "\n")
}

func emitStmt(stmt subsidia.Stmt, indent string) string {
	switch s := stmt.(type) {
	case *subsidia.StmtMassa:
		lines := []string{}
		for _, inner := range s.Corpus {
			lines = append(lines, emitStmt(inner, indent+"  "))
		}
		return "{\n" + strings.Join(lines, "\n") + "\n" + indent + "}"

	case *subsidia.StmtExpressia:
		return indent + emitExpr(s.Expr) + ";"

	case *subsidia.StmtVaria:
		decl := ""
		if s.Externa {
			decl = "declare "
		}
		kw := "let"
		if s.Species == subsidia.VariaFixum {
			kw = "const"
		}
		typ := ""
		if s.Typus != nil {
			if s.Externa {
				if t, ok := s.Typus.(*subsidia.TypusNomen); ok && t.Nomen == "ignotum" {
					typ = ": any"
				} else {
					typ = ": " + emitTypus(s.Typus)
				}
			} else {
				typ = ": " + emitTypus(s.Typus)
			}
		}
		val := ""
		if s.Valor != nil && !s.Externa {
			val = " = " + emitExpr(s.Valor)
		}
		exp := ""
		if s.Publica {
			exp = "export "
		}
		return indent + exp + decl + kw + " " + s.Nomen + typ + val + ";"

	case *subsidia.StmtFunctio:
		decl := ""
		if s.Externa {
			decl = "declare "
		}
		exp := ""
		if s.Publica {
			exp = "export "
		}
		async := ""
		if s.Asynca {
			async = "async "
		}
		generics := ""
		if len(s.Generics) > 0 {
			generics = "<" + strings.Join(s.Generics, ", ") + ">"
		}
		params := []string{}
		for _, param := range s.Params {
			params = append(params, emitParam(param))
		}
		ret := ""
		if s.TypusReditus != nil {
			ret = ": " + emitTypus(s.TypusReditus)
		}
		body := ";"
		if s.Corpus != nil && !s.Externa {
			body = " " + emitStmt(s.Corpus, indent)
		}
		return indent + exp + decl + async + "function " + s.Nomen + generics + "(" + strings.Join(params, ", ") + ")" + ret + body

	case *subsidia.StmtGenus:
		exp := ""
		if s.Publica {
			exp = "export "
		}
		generics := ""
		if len(s.Generics) > 0 {
			generics = "<" + strings.Join(s.Generics, ", ") + ">"
		}
		impl := ""
		if len(s.Implet) > 0 {
			impl = " implements " + strings.Join(s.Implet, ", ")
		}
		lines := []string{}
		lines = append(lines, indent+exp+"class "+s.Nomen+generics+impl+" {")

		for _, campo := range s.Campi {
			vis := "private "
			if campo.Visibilitas == "Protecta" {
				vis = "protected "
			}
			val := ""
			if campo.Valor != nil {
				val = " = " + emitExpr(campo.Valor)
			}
			lines = append(lines, indent+"  "+vis+campo.Nomen+": "+emitTypus(campo.Typus)+val+";")
		}

		if len(s.Campi) > 0 {
			lines = append(lines, "")
			fields := []string{}
			for _, campo := range s.Campi {
				fields = append(fields, campo.Nomen+"?: "+emitTypus(campo.Typus))
			}
			lines = append(lines, indent+"  constructor(overrides: { "+strings.Join(fields, ", ")+" } = {}) {")
			for _, campo := range s.Campi {
				lines = append(lines, indent+"    if (overrides."+campo.Nomen+" !== undefined) { this."+campo.Nomen+" = overrides."+campo.Nomen+"; }")
			}
			lines = append(lines, indent+"  }")
		}

		for _, method := range s.Methodi {
			if fn, ok := method.(*subsidia.StmtFunctio); ok {
				lines = append(lines, "")
				vis := "private "
				if fn.Publica {
					vis = ""
				}
				async := ""
				if fn.Asynca {
					async = "async "
				}
				params := []string{}
				for _, param := range fn.Params {
					params = append(params, emitParam(param))
				}
				ret := ""
				if fn.TypusReditus != nil {
					ret = ": " + emitTypus(fn.TypusReditus)
				}
				body := ";"
				if fn.Corpus != nil {
					body = " " + emitStmt(fn.Corpus, indent+"  ")
				}
				lines = append(lines, indent+"  "+vis+async+fn.Nomen+"("+strings.Join(params, ", ")+")"+ret+body)
			}
		}

		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtPactum:
		exp := ""
		if s.Publica {
			exp = "export "
		}
		generics := ""
		if len(s.Generics) > 0 {
			generics = "<" + strings.Join(s.Generics, ", ") + ">"
		}
		lines := []string{}
		lines = append(lines, indent+exp+"interface "+s.Nomen+generics+" {")
		for _, method := range s.Methodi {
			params := []string{}
			for _, param := range method.Params {
				params = append(params, emitParam(param))
			}
			ret := ""
			if method.TypusReditus != nil {
				ret = ": " + emitTypus(method.TypusReditus)
			}
			lines = append(lines, indent+"  "+method.Nomen+"("+strings.Join(params, ", ")+")"+ret+";")
		}
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtOrdo:
		exp := ""
		if s.Publica {
			exp = "export "
		}
		members := []string{}
		for _, m := range s.Membra {
			val := ""
			if m.Valor != nil {
				val = " = " + *m.Valor
			}
			members = append(members, m.Nomen+val)
		}
		return indent + exp + "enum " + s.Nomen + " { " + strings.Join(members, ", ") + " }"

	case *subsidia.StmtDiscretio:
		exp := ""
		if s.Publica {
			exp = "export "
		}
		generics := ""
		if len(s.Generics) > 0 {
			generics = "<" + strings.Join(s.Generics, ", ") + ">"
		}
		lines := []string{}
		variantNames := []string{}
		for _, v := range s.Variantes {
			variantNames = append(variantNames, v.Nomen)
			if len(v.Campi) == 0 {
				lines = append(lines, indent+exp+"type "+v.Nomen+" = { tag: '"+v.Nomen+"' };")
			} else {
				fields := []string{}
				for _, f := range v.Campi {
					fields = append(fields, f.Nomen+": "+emitTypus(f.Typus))
				}
				lines = append(lines, indent+exp+"type "+v.Nomen+" = { tag: '"+v.Nomen+"'; "+strings.Join(fields, "; ")+" };")
			}
		}
		lines = append(lines, indent+exp+"type "+s.Nomen+generics+" = "+strings.Join(variantNames, " | ")+";")
		return strings.Join(lines, "\n")

	case *subsidia.StmtImporta:
		specs := []string{}
		for _, spec := range s.Specs {
			if spec.Imported == spec.Local {
				specs = append(specs, spec.Imported)
			} else {
				specs = append(specs, spec.Imported+" as "+spec.Local)
			}
		}
		return indent + "import { " + strings.Join(specs, ", ") + " } from \"" + s.Fons + "\";"

	case *subsidia.StmtSi:
		code := indent + "if (" + emitExpr(s.Cond) + ") " + emitStmt(s.Cons, indent)
		if s.Alt != nil {
			code += " else " + emitStmt(s.Alt, indent)
		}
		return code

	case *subsidia.StmtDum:
		return indent + "while (" + emitExpr(s.Cond) + ") " + emitStmt(s.Corpus, indent)

	case *subsidia.StmtFacDum:
		return indent + "do " + emitStmt(s.Corpus, indent) + " while (" + emitExpr(s.Cond) + ");"

	case *subsidia.StmtIteratio:
		kw := "of"
		if s.Species == "De" {
			kw = "in"
		}
		async := ""
		if s.Asynca {
			async = "await "
		}
		return indent + "for " + async + "(const " + s.Binding + " " + kw + " " + emitExpr(s.Iter) + ") " + emitStmt(s.Corpus, indent)

	case *subsidia.StmtElige:
		discrim := emitExpr(s.Discrim)
		lines := []string{}
		for i, c := range s.Casus {
			kw := "if"
			if i > 0 {
				kw = "else if"
			}
			lines = append(lines, indent+kw+" ("+discrim+" === "+emitExpr(c.Cond)+") "+emitStmt(c.Corpus, indent))
		}
		if s.Default != nil {
			lines = append(lines, indent+"else "+emitStmt(s.Default, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscerne:
		lines := []string{}
		numDiscrim := len(s.Discrim)
		discrimVars := []string{}
		if numDiscrim == 1 {
			discrimVars = append(discrimVars, emitExpr(s.Discrim[0]))
		} else {
			for i := 0; i < numDiscrim; i++ {
				varName := "discriminant_" + strconv.Itoa(i)
				discrimVars = append(discrimVars, varName)
				lines = append(lines, indent+"const "+varName+" = "+emitExpr(s.Discrim[i])+";")
			}
		}

		for ci, c := range s.Casus {
			firstPattern := c.Patterns[0]
			kw := "if"
			if ci > 0 {
				kw = "else if"
			}
			if firstPattern.Wildcard {
				lines = append(lines, indent+"else {")
			} else {
				lines = append(lines, indent+kw+" ("+discrimVars[0]+".tag === '"+firstPattern.Variant+"') {")
			}

			for i := 0; i < len(c.Patterns) && i < numDiscrim; i++ {
				pattern := c.Patterns[i]
				discrimVar := discrimVars[i]

				if pattern.Alias != nil {
					lines = append(lines, indent+"  const "+*pattern.Alias+" = "+discrimVar+";")
				}
				for _, b := range pattern.Bindings {
					lines = append(lines, indent+"  const "+b+" = "+discrimVar+"."+b+";")
				}
			}

			if block, ok := c.Corpus.(*subsidia.StmtMassa); ok {
				for _, s := range block.Corpus {
					lines = append(lines, emitStmt(s, indent+"  "))
				}
			} else {
				lines = append(lines, emitStmt(c.Corpus, indent+"  "))
			}
			lines = append(lines, indent+"}")
		}

		return strings.Join(lines, "\n")

	case *subsidia.StmtCustodi:
		lines := []string{}
		for _, c := range s.Clausulae {
			lines = append(lines, indent+"if ("+emitExpr(c.Cond)+") "+emitStmt(c.Corpus, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtTempta:
		code := indent + "try " + emitStmt(s.Corpus, indent)
		if s.Cape != nil {
			code += " catch (" + s.Cape.Param + ") " + emitStmt(s.Cape.Corpus, indent)
		}
		if s.Demum != nil {
			code += " finally " + emitStmt(s.Demum, indent)
		}
		return code

	case *subsidia.StmtRedde:
		if s.Valor != nil {
			return indent + "return " + emitExpr(s.Valor) + ";"
		}
		return indent + "return;"

	case *subsidia.StmtIace:
		if s.Fatale {
			return indent + "throw new Error(" + emitExpr(s.Arg) + ");"
		}
		return indent + "throw " + emitExpr(s.Arg) + ";"

	case *subsidia.StmtScribe:
		method := "log"
		if s.Gradus == "Vide" {
			method = "debug"
		} else if s.Gradus == "Mone" {
			method = "warn"
		}
		args := []string{}
		for _, arg := range s.Args {
			args = append(args, emitExpr(arg))
		}
		return indent + "console." + method + "(" + strings.Join(args, ", ") + ");"

	case *subsidia.StmtAdfirma:
		msg := ""
		if s.Msg != nil {
			msg = ", " + emitExpr(s.Msg)
		}
		return indent + "console.assert(" + emitExpr(s.Cond) + msg + ");"

	case *subsidia.StmtRumpe:
		return indent + "break;"

	case *subsidia.StmtPerge:
		return indent + "continue;"

	case *subsidia.StmtIncipit:
		async := ""
		if s.Asynca {
			async = "async "
		}
		return indent + "(" + async + "function() " + emitStmt(s.Corpus, indent) + ")();"

	case *subsidia.StmtProbandum:
		lines := []string{}
		lines = append(lines, indent+"describe("+strconv.Quote(s.Nomen)+", () => {")
		for _, inner := range s.Corpus {
			lines = append(lines, emitStmt(inner, indent+"  "))
		}
		lines = append(lines, indent+"});")
		return strings.Join(lines, "\n")

	case *subsidia.StmtProba:
		return indent + "it(" + strconv.Quote(s.Nomen) + ", () => " + emitStmt(s.Corpus, indent) + ");"

	default:
		return indent + "/* unhandled */"
	}
}

func emitExpr(expr subsidia.Expr) string {
	switch e := expr.(type) {
	case *subsidia.ExprNomen:
		return e.Valor

	case *subsidia.ExprEgo:
		return "this"

	case *subsidia.ExprLittera:
		switch e.Species {
		case subsidia.LitteraTextus:
			return strconv.Quote(e.Valor)
		case subsidia.LitteraVerum:
			return "true"
		case subsidia.LitteraFalsum:
			return "false"
		case subsidia.LitteraNihil:
			return "null"
		default:
			return e.Valor
		}

	case *subsidia.ExprBinaria:
		op := e.Signum
		if mapped, ok := binaryOps[op]; ok {
			op = mapped
		}
		return "(" + emitExpr(e.Sin) + " " + op + " " + emitExpr(e.Dex) + ")"

	case *subsidia.ExprUnaria:
		op := e.Signum
		if mapped, ok := unaryOpsEmit[op]; ok {
			op = mapped
		}
		return "(" + op + emitExpr(e.Arg) + ")"

	case *subsidia.ExprAssignatio:
		return emitExpr(e.Sin) + " " + e.Signum + " " + emitBareExpr(e.Dex)

	case *subsidia.ExprCondicio:
		return "(" + emitExpr(e.Cond) + " ? " + emitExpr(e.Cons) + " : " + emitExpr(e.Alt) + ")"

	case *subsidia.ExprVocatio:
		if m, ok := e.Callee.(*subsidia.ExprMembrum); ok && !m.Computed {
			if prop, ok := m.Prop.(*subsidia.ExprLittera); ok {
				propName := prop.Valor
				if _, ok := propertyOnly[propName]; ok {
					return emitExpr(m)
				}
				if translated, ok := methodMap[propName]; ok {
					obj := emitExpr(m.Obj)
					access := "."
					if m.NonNull {
						access = "!."
					}
					args := []string{}
					for _, arg := range e.Args {
						args = append(args, emitExpr(arg))
					}
					return obj + access + translated + "(" + strings.Join(args, ", ") + ")"
				}
			}
		}
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, emitExpr(arg))
		}
		return emitExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprMembrum:
		obj := emitExpr(e.Obj)
		if e.Computed {
			return obj + "[" + emitExpr(e.Prop) + "]"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = lit.Valor
		} else {
			prop = emitExpr(e.Prop)
		}
		if prop == "primus" {
			return obj + "[0]"
		}
		if prop == "ultimus" {
			return obj + ".at(-1)"
		}
		if _, ok := propertyOnly[prop]; ok {
			if mapped, ok := methodMap[prop]; ok {
				prop = mapped
			}
		}
		access := "."
		if e.NonNull {
			access = "!."
		}
		return obj + access + prop

	case *subsidia.ExprSeries:
		elems := []string{}
		for _, elem := range e.Elementa {
			elems = append(elems, emitExpr(elem))
		}
		return "[" + strings.Join(elems, ", ") + "]"

	case *subsidia.ExprObiectum:
		props := []string{}
		for _, p := range e.Props {
			if p.Shorthand {
				if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
					props = append(props, lit.Valor)
				} else {
					props = append(props, emitExpr(p.Key))
				}
				continue
			}
			key := ""
			if p.Computed {
				key = "[" + emitExpr(p.Key) + "]"
			} else if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = lit.Valor
			} else {
				key = emitExpr(p.Key)
			}
			props = append(props, key+": "+emitExpr(p.Valor))
		}
		return "{ " + strings.Join(props, ", ") + " }"

	case *subsidia.ExprClausura:
		params := []string{}
		for _, param := range e.Params {
			if param.Typus != nil {
				params = append(params, param.Nomen+": "+emitTypus(param.Typus))
			} else {
				params = append(params, param.Nomen)
			}
		}
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "(" + strings.Join(params, ", ") + ") => " + emitStmt(body, "")
		case subsidia.Expr:
			return "(" + strings.Join(params, ", ") + ") => " + emitExpr(body)
		default:
			return "(" + strings.Join(params, ", ") + ") => {}"
		}

	case *subsidia.ExprNovum:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, emitExpr(arg))
		}
		code := "new " + emitExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"
		if e.Init != nil {
			code = "Object.assign(" + code + ", " + emitExpr(e.Init) + ")"
		}
		return code

	case *subsidia.ExprCede:
		return "await " + emitExpr(e.Arg)

	case *subsidia.ExprQua:
		return "(" + emitExpr(e.Expr) + " as " + emitTypus(e.Typus) + ")"

	case *subsidia.ExprInnatum:
		return "(" + emitExpr(e.Expr) + " as " + emitTypus(e.Typus) + ")"

	case *subsidia.ExprPostfixNovum:
		return "new " + emitTypus(e.Typus) + "(" + emitExpr(e.Expr) + ")"

	case *subsidia.ExprFinge:
		fields := []string{}
		for _, p := range e.Campi {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = lit.Valor
			} else {
				key = emitExpr(p.Key)
			}
			fields = append(fields, key+": "+emitExpr(p.Valor))
		}
		cast := ""
		if e.Typus != nil {
			cast = " as " + emitTypus(e.Typus)
		}
		return "{ tag: '" + e.Variant + "', " + strings.Join(fields, ", ") + " }" + cast

	case *subsidia.ExprScriptum:
		parts := strings.Split(e.Template, "§")
		if len(parts) == 1 {
			return strconv.Quote(e.Template)
		}
		var b strings.Builder
		b.WriteString("`")
		for i, part := range parts {
			b.WriteString(strings.ReplaceAll(part, "`", "\\`"))
			if i < len(e.Args) {
				b.WriteString("${" + emitExpr(e.Args[i]) + "}")
			}
		}
		b.WriteString("`")
		return b.String()

	case *subsidia.ExprAmbitus:
		start := emitExpr(e.Start)
		end := emitExpr(e.End)
		if e.Inclusive {
			return "Array.from({ length: " + end + " - " + start + " + 1 }, (_, i) => " + start + " + i)"
		}
		return "Array.from({ length: " + end + " - " + start + " }, (_, i) => " + start + " + i)"

	default:
		return "/* unhandled */"
	}
}

func emitBareExpr(expr subsidia.Expr) string {
	if e, ok := expr.(*subsidia.ExprBinaria); ok {
		op := e.Signum
		if mapped, ok := binaryOps[op]; ok {
			op = mapped
		}
		return emitBareExpr(e.Sin) + " " + op + " " + emitBareExpr(e.Dex)
	}
	return emitExpr(expr)
}

func emitTypus(typus subsidia.Typus) string {
	switch t := typus.(type) {
	case *subsidia.TypusNomen:
		return mapTypeName(t.Nomen)
	case *subsidia.TypusNullabilis:
		return emitTypus(t.Inner) + " | null"
	case *subsidia.TypusGenericus:
		args := []string{}
		for _, arg := range t.Args {
			args = append(args, emitTypus(arg))
		}
		return mapTypeName(t.Nomen) + "<" + strings.Join(args, ", ") + ">"
	case *subsidia.TypusFunctio:
		args := []string{}
		for i, p := range t.Params {
			args = append(args, "arg"+strconv.Itoa(i)+": "+emitTypus(p))
		}
		return "(" + strings.Join(args, ", ") + ") => " + emitTypus(t.Returns)
	case *subsidia.TypusUnio:
		members := []string{}
		for _, m := range t.Members {
			members = append(members, emitTypus(m))
		}
		return strings.Join(members, " | ")
	case *subsidia.TypusLitteralis:
		return t.Valor
	default:
		return "unknown"
	}
}

func mapTypeName(name string) string {
	mapping := map[string]string{
		"textus":    "string",
		"numerus":   "number",
		"fractus":   "number",
		"bivalens":  "boolean",
		"nihil":     "null",
		"vacuum":    "void",
		"vacuus":    "void",
		"ignotum":   "unknown",
		"quodlibet": "any",
		"quidlibet": "any",
		"lista":     "Array",
		"tabula":    "Map",
		"collectio": "Set",
		"copia":     "Set",
	}
	if mapped, ok := mapping[name]; ok {
		return mapped
	}
	return name
}

func emitParam(param subsidia.Param) string {
	rest := ""
	if param.Rest {
		rest = "..."
	}
	typ := ""
	if param.Typus != nil {
		typ = ": " + emitTypus(param.Typus)
	}
	def := ""
	if param.Default != nil {
		def = " = " + emitExpr(param.Default)
	} else {
		if _, ok := param.Typus.(*subsidia.TypusNullabilis); ok {
			def = " = null"
		}
	}
	return rest + param.Nomen + typ + def
}
