package main

import (
	"strconv"
	"strings"

	"subsidia"
)

// Go binary operators (Latin -> Go)
var goBinaryOps = map[string]string{
	"et":  "&&",
	"aut": "||",
}

// Go unary operators (Latin -> Go)
var goUnaryOps = map[string]string{
	"non":       "!",
	"nihil":     "!",       // nil check
	"nonnihil":  "",        // handled specially
	"positivum": "+",
}

// Go method mappings - methods that translate to Go stdlib calls
// Format: Latin name -> Go equivalent pattern
var goMethodMap = map[string]string{
	"longitudo": "len",
	"adde":      "append",
	"coniunge":  "strings.Join",
	"continet":  "strings.Contains",
	"initium":   "strings.HasPrefix",
	"finis":     "strings.HasSuffix",
	"maiuscula": "strings.ToUpper",
	"minuscula": "strings.ToLower",
	"recide":    "strings.TrimSpace",
	"divide":    "strings.Split",
	"muta":      "strings.ReplaceAll",
}

// Properties that are accessed as len() in Go
var goPropertyAsFunc = map[string]struct{}{
	"longitudo": {},
}

// EmitGo renders a module to Go code.
func EmitGo(mod *subsidia.Modulus) string {
	var b strings.Builder

	// Package declaration
	b.WriteString("package main\n\n")

	// Collect imports
	imports := collectImports(mod)
	if len(imports) > 0 {
		b.WriteString("import (\n")
		for _, imp := range imports {
			b.WriteString("\t\"" + imp + "\"\n")
		}
		b.WriteString(")\n\n")
	}

	// Emit statements
	for _, stmt := range mod.Corpus {
		code := emitGoStmt(stmt, "")
		if code != "" {
			b.WriteString(code)
			b.WriteString("\n")
		}
	}

	return b.String()
}

// collectImports scans the AST and determines which Go imports are needed
func collectImports(mod *subsidia.Modulus) []string {
	imports := map[string]struct{}{}

	// Always need fmt for scribe, os for stdout/stderr
	imports["fmt"] = struct{}{}
	imports["os"] = struct{}{}
	imports["strings"] = struct{}{}

	// TODO: scan AST for specific needs
	// For now, include common ones

	result := []string{}
	for imp := range imports {
		result = append(result, imp)
	}
	return result
}

func emitGoStmt(stmt subsidia.Stmt, indent string) string {
	switch s := stmt.(type) {
	case *subsidia.StmtMassa:
		lines := []string{}
		for _, inner := range s.Corpus {
			lines = append(lines, emitGoStmt(inner, indent+"\t"))
		}
		return "{\n" + strings.Join(lines, "\n") + "\n" + indent + "}"

	case *subsidia.StmtExpressia:
		return indent + emitGoExpr(s.Expr)

	case *subsidia.StmtVaria:
		if s.Externa {
			// External declarations - emit as comment
			return indent + "// extern: " + s.Nomen
		}
		kw := "var"
		if s.Species == subsidia.VariaFixum {
			// Go doesn't have const for complex types, use var
			kw = "var"
		}
		typ := ""
		if s.Typus != nil {
			typ = " " + emitGoTypus(s.Typus)
		}
		val := ""
		if s.Valor != nil {
			val = " = " + emitGoExpr(s.Valor)
		}
		return indent + kw + " " + s.Nomen + typ + val

	case *subsidia.StmtFunctio:
		if s.Externa {
			return indent + "// extern: " + s.Nomen
		}
		generics := ""
		if len(s.Generics) > 0 {
			params := []string{}
			for _, g := range s.Generics {
				params = append(params, g+" any")
			}
			generics = "[" + strings.Join(params, ", ") + "]"
		}
		params := []string{}
		for _, param := range s.Params {
			params = append(params, emitGoParam(param))
		}
		ret := ""
		if s.TypusReditus != nil {
			ret = " " + emitGoTypus(s.TypusReditus)
		}
		body := " {}"
		if s.Corpus != nil {
			body = " " + emitGoStmt(s.Corpus, indent)
		}
		return indent + "func " + s.Nomen + generics + "(" + strings.Join(params, ", ") + ")" + ret + body

	case *subsidia.StmtGenus:
		lines := []string{}
		// Struct definition
		lines = append(lines, indent+"type "+s.Nomen+" struct {")
		for _, campo := range s.Campi {
			fieldName := capitalize(campo.Nomen)
			lines = append(lines, indent+"\t"+fieldName+" "+emitGoTypus(campo.Typus))
		}
		lines = append(lines, indent+"}")

		// Constructor
		if len(s.Campi) > 0 {
			lines = append(lines, "")
			params := []string{}
			for _, campo := range s.Campi {
				params = append(params, campo.Nomen+" "+emitGoTypus(campo.Typus))
			}
			lines = append(lines, indent+"func New"+s.Nomen+"("+strings.Join(params, ", ")+") *"+s.Nomen+" {")
			lines = append(lines, indent+"\treturn &"+s.Nomen+"{")
			for _, campo := range s.Campi {
				lines = append(lines, indent+"\t\t"+capitalize(campo.Nomen)+": "+campo.Nomen+",")
			}
			lines = append(lines, indent+"\t}")
			lines = append(lines, indent+"}")
		}

		// Methods
		for _, method := range s.Methodi {
			if fn, ok := method.(*subsidia.StmtFunctio); ok {
				lines = append(lines, "")
				params := []string{}
				for _, param := range fn.Params {
					params = append(params, emitGoParam(param))
				}
				ret := ""
				if fn.TypusReditus != nil {
					ret = " " + emitGoTypus(fn.TypusReditus)
				}
				body := " {}"
				if fn.Corpus != nil {
					body = " " + emitGoStmt(fn.Corpus, indent)
				}
				lines = append(lines, indent+"func (self *"+s.Nomen+") "+fn.Nomen+"("+strings.Join(params, ", ")+")"+ret+body)
			}
		}

		return strings.Join(lines, "\n")

	case *subsidia.StmtPactum:
		lines := []string{}
		lines = append(lines, indent+"type "+s.Nomen+" interface {")
		for _, method := range s.Methodi {
			params := []string{}
			for _, param := range method.Params {
				params = append(params, emitGoParam(param))
			}
			ret := ""
			if method.TypusReditus != nil {
				ret = " " + emitGoTypus(method.TypusReditus)
			}
			lines = append(lines, indent+"\t"+method.Nomen+"("+strings.Join(params, ", ")+")"+ret)
		}
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtOrdo:
		lines := []string{}
		lines = append(lines, indent+"type "+s.Nomen+" int")
		lines = append(lines, indent+"const (")
		for i, m := range s.Membra {
			if i == 0 {
				lines = append(lines, indent+"\t"+s.Nomen+m.Nomen+" "+s.Nomen+" = iota")
			} else {
				lines = append(lines, indent+"\t"+s.Nomen+m.Nomen)
			}
		}
		lines = append(lines, indent+")")
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscretio:
		lines := []string{}
		// Interface for the discriminated union
		lines = append(lines, indent+"type "+s.Nomen+" interface {")
		lines = append(lines, indent+"\tis"+s.Nomen+"()")
		lines = append(lines, indent+"}")
		lines = append(lines, "")

		// Each variant is a struct
		for _, v := range s.Variantes {
			lines = append(lines, indent+"type "+v.Nomen+" struct {")
			for _, f := range v.Campi {
				lines = append(lines, indent+"\t"+capitalize(f.Nomen)+" "+emitGoTypus(f.Typus))
			}
			lines = append(lines, indent+"}")
			lines = append(lines, indent+"func ("+v.Nomen+") is"+s.Nomen+"() {}")
			lines = append(lines, "")
		}

		return strings.Join(lines, "\n")

	case *subsidia.StmtImporta:
		// Go imports are collected at top, skip inline
		return ""

	case *subsidia.StmtSi:
		code := indent + "if " + emitGoExpr(s.Cond) + " " + emitGoStmt(s.Cons, indent)
		if s.Alt != nil {
			code += " else " + emitGoStmt(s.Alt, indent)
		}
		return code

	case *subsidia.StmtDum:
		return indent + "for " + emitGoExpr(s.Cond) + " " + emitGoStmt(s.Corpus, indent)

	case *subsidia.StmtFacDum:
		// Go doesn't have do-while, emulate with for + break
		lines := []string{}
		lines = append(lines, indent+"for {")
		if block, ok := s.Corpus.(*subsidia.StmtMassa); ok {
			for _, inner := range block.Corpus {
				lines = append(lines, emitGoStmt(inner, indent+"\t"))
			}
		} else {
			lines = append(lines, emitGoStmt(s.Corpus, indent+"\t"))
		}
		lines = append(lines, indent+"\tif !("+emitGoExpr(s.Cond)+") { break }")
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtIteratio:
		binding := s.Binding
		iter := emitGoExpr(s.Iter)
		if s.Species == "De" {
			// for key in map
			return indent + "for " + binding + " := range " + iter + " " + emitGoStmt(s.Corpus, indent)
		}
		// for value in slice
		return indent + "for _, " + binding + " := range " + iter + " " + emitGoStmt(s.Corpus, indent)

	case *subsidia.StmtElige:
		lines := []string{}
		lines = append(lines, indent+"switch "+emitGoExpr(s.Discrim)+" {")
		for _, c := range s.Casus {
			lines = append(lines, indent+"case "+emitGoExpr(c.Cond)+":")
			if block, ok := c.Corpus.(*subsidia.StmtMassa); ok {
				for _, inner := range block.Corpus {
					lines = append(lines, emitGoStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, emitGoStmt(c.Corpus, indent+"\t"))
			}
		}
		if s.Default != nil {
			lines = append(lines, indent+"default:")
			if block, ok := s.Default.(*subsidia.StmtMassa); ok {
				for _, inner := range block.Corpus {
					lines = append(lines, emitGoStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, emitGoStmt(s.Default, indent+"\t"))
			}
		}
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscerne:
		lines := []string{}
		discrim := emitGoExpr(s.Discrim[0])
		lines = append(lines, indent+"switch _v := "+discrim+".(type) {")

		for _, c := range s.Casus {
			pattern := c.Patterns[0]
			if pattern.Wildcard {
				lines = append(lines, indent+"default:")
			} else {
				lines = append(lines, indent+"case "+pattern.Variant+":")
			}

			// Bind fields
			if pattern.Alias != nil {
				lines = append(lines, indent+"\t"+*pattern.Alias+" := _v")
			}
			for _, b := range pattern.Bindings {
				lines = append(lines, indent+"\t"+b+" := _v."+capitalize(b))
			}

			if block, ok := c.Corpus.(*subsidia.StmtMassa); ok {
				for _, inner := range block.Corpus {
					lines = append(lines, emitGoStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, emitGoStmt(c.Corpus, indent+"\t"))
			}
		}

		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtCustodi:
		lines := []string{}
		for _, c := range s.Clausulae {
			lines = append(lines, indent+"if "+emitGoExpr(c.Cond)+" "+emitGoStmt(c.Corpus, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtTempta:
		// Go doesn't have try/catch - emit as defer/recover or just the body
		// For now, just emit the body (rivus removed try/catch anyway)
		return emitGoStmt(s.Corpus, indent)

	case *subsidia.StmtRedde:
		if s.Valor != nil {
			return indent + "return " + emitGoExpr(s.Valor)
		}
		return indent + "return"

	case *subsidia.StmtIace:
		// mori -> panic
		return indent + "panic(" + emitGoExpr(s.Arg) + ")"

	case *subsidia.StmtScribe:
		target := "os.Stdout"
		if s.Gradus == "Mone" {
			target = "os.Stderr"
		}
		args := []string{}
		for _, arg := range s.Args {
			args = append(args, emitGoExpr(arg))
		}
		return indent + "fmt.Fprintln(" + target + ", " + strings.Join(args, ", ") + ")"

	case *subsidia.StmtAdfirma:
		msg := `"assertion failed"`
		if s.Msg != nil {
			msg = emitGoExpr(s.Msg)
		}
		return indent + "if !(" + emitGoExpr(s.Cond) + ") { panic(" + msg + ") }"

	case *subsidia.StmtRumpe:
		return indent + "break"

	case *subsidia.StmtPerge:
		return indent + "continue"

	case *subsidia.StmtIncipit:
		// Entry point - emit as main() or init block
		return indent + "func main() " + emitGoStmt(s.Corpus, indent)

	default:
		return indent + "/* unhandled statement */"
	}
}

func emitGoExpr(expr subsidia.Expr) string {
	switch e := expr.(type) {
	case *subsidia.ExprNomen:
		return e.Valor

	case *subsidia.ExprEgo:
		return "self"

	case *subsidia.ExprLittera:
		switch e.Species {
		case subsidia.LitteraTextus:
			return strconv.Quote(e.Valor)
		case subsidia.LitteraVerum:
			return "true"
		case subsidia.LitteraFalsum:
			return "false"
		case subsidia.LitteraNihil:
			return "nil"
		default:
			return e.Valor
		}

	case *subsidia.ExprBinaria:
		op := e.Signum
		if mapped, ok := goBinaryOps[op]; ok {
			op = mapped
		}
		// Handle vel (nullish coalescing) specially
		if e.Signum == "vel" {
			// Go doesn't have ??, emit as: func() T { if l := left; l != nil { return l }; return right }()
			// For simplicity, just emit left for now (TODO: proper handling)
			return emitGoExpr(e.Sin)
		}
		return "(" + emitGoExpr(e.Sin) + " " + op + " " + emitGoExpr(e.Dex) + ")"

	case *subsidia.ExprUnaria:
		op := e.Signum
		if e.Signum == "nonnihil" {
			return "(" + emitGoExpr(e.Arg) + " != nil)"
		}
		if e.Signum == "nihil" {
			return "(" + emitGoExpr(e.Arg) + " == nil)"
		}
		if mapped, ok := goUnaryOps[op]; ok {
			op = mapped
		}
		return "(" + op + emitGoExpr(e.Arg) + ")"

	case *subsidia.ExprAssignatio:
		return emitGoExpr(e.Sin) + " " + e.Signum + " " + emitGoExpr(e.Dex)

	case *subsidia.ExprCondicio:
		// Go doesn't have ternary - emit as inline func
		return "func() interface{} { if " + emitGoExpr(e.Cond) + " { return " + emitGoExpr(e.Cons) + " }; return " + emitGoExpr(e.Alt) + " }()"

	case *subsidia.ExprVocatio:
		// Check for method calls that need translation
		if m, ok := e.Callee.(*subsidia.ExprMembrum); ok && !m.Computed {
			if prop, ok := m.Prop.(*subsidia.ExprLittera); ok {
				propName := prop.Valor
				obj := emitGoExpr(m.Obj)
				args := []string{}
				for _, arg := range e.Args {
					args = append(args, emitGoExpr(arg))
				}

				// Handle special method translations
				switch propName {
				case "longitudo":
					return "len(" + obj + ")"
				case "adde":
					return obj + " = append(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "coniunge":
					return "strings.Join(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "continet":
					return "strings.Contains(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "initium":
					return "strings.HasPrefix(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "finis":
					return "strings.HasSuffix(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "maiuscula":
					return "strings.ToUpper(" + obj + ")"
				case "minuscula":
					return "strings.ToLower(" + obj + ")"
				case "recide":
					return "strings.TrimSpace(" + obj + ")"
				case "divide":
					return "strings.Split(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "muta":
					return "strings.ReplaceAll(" + obj + ", " + strings.Join(args, ", ") + ")"
				case "sectio":
					if len(args) == 2 {
						return obj + "[" + args[0] + ":" + args[1] + "]"
					} else if len(args) == 1 {
						return obj + "[" + args[0] + ":]"
					}
				}
			}
		}

		args := []string{}
		for _, arg := range e.Args {
			args = append(args, emitGoExpr(arg))
		}
		return emitGoExpr(e.Callee) + "(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprMembrum:
		obj := emitGoExpr(e.Obj)
		if e.Computed {
			return obj + "[" + emitGoExpr(e.Prop) + "]"
		}
		prop := ""
		if lit, ok := e.Prop.(*subsidia.ExprLittera); ok {
			prop = lit.Valor
		} else {
			prop = emitGoExpr(e.Prop)
		}

		// Handle special properties
		if prop == "longitudo" {
			return "len(" + obj + ")"
		}
		if prop == "primus" {
			return obj + "[0]"
		}
		if prop == "ultimus" {
			return obj + "[len(" + obj + ")-1]"
		}

		return obj + "." + capitalize(prop)

	case *subsidia.ExprSeries:
		elems := []string{}
		for _, elem := range e.Elementa {
			elems = append(elems, emitGoExpr(elem))
		}
		// Infer type from elements if possible
		return "[]interface{}{" + strings.Join(elems, ", ") + "}"

	case *subsidia.ExprObiectum:
		props := []string{}
		for _, p := range e.Props {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = strconv.Quote(lit.Valor)
			} else {
				key = emitGoExpr(p.Key)
			}
			props = append(props, key+": "+emitGoExpr(p.Valor))
		}
		return "map[string]interface{}{" + strings.Join(props, ", ") + "}"

	case *subsidia.ExprClausura:
		params := []string{}
		for _, param := range e.Params {
			typ := "interface{}"
			if param.Typus != nil {
				typ = emitGoTypus(param.Typus)
			}
			params = append(params, param.Nomen+" "+typ)
		}
		ret := "interface{}"
		switch body := e.Corpus.(type) {
		case subsidia.Stmt:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " " + emitGoStmt(body, "")
		case subsidia.Expr:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " { return " + emitGoExpr(body) + " }"
		default:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " {}"
		}

	case *subsidia.ExprNovum:
		args := []string{}
		for _, arg := range e.Args {
			args = append(args, emitGoExpr(arg))
		}
		callee := emitGoExpr(e.Callee)
		if e.Init != nil {
			// Struct literal with initializer: novum Type { field: value }
			if obj, ok := e.Init.(*subsidia.ExprObiectum); ok {
				fields := []string{}
				for _, p := range obj.Props {
					key := ""
					if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
						key = capitalize(lit.Valor)
					} else {
						key = emitGoExpr(p.Key)
					}
					fields = append(fields, key+": "+emitGoExpr(p.Valor))
				}
				return "&" + callee + "{" + strings.Join(fields, ", ") + "}"
			}
			return "&" + callee + "{" + emitGoExpr(e.Init) + "}"
		}
		return "New" + callee + "(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprCede:
		// No await in Go - just emit the expression
		return emitGoExpr(e.Arg)

	case *subsidia.ExprQua:
		// Type assertion
		return emitGoExpr(e.Expr) + ".(" + emitGoTypus(e.Typus) + ")"

	case *subsidia.ExprInnatum:
		// Type assertion/conversion
		return emitGoTypus(e.Typus) + "(" + emitGoExpr(e.Expr) + ")"

	case *subsidia.ExprPostfixNovum:
		return "&" + emitGoTypus(e.Typus) + "{" + emitGoExpr(e.Expr) + "}"

	case *subsidia.ExprFinge:
		fields := []string{}
		for _, p := range e.Campi {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = capitalize(lit.Valor)
			} else {
				key = emitGoExpr(p.Key)
			}
			fields = append(fields, key + ": " + emitGoExpr(p.Valor))
		}
		return e.Variant + "{" + strings.Join(fields, ", ") + "}"

	case *subsidia.ExprScriptum:
		// Format string: scriptum("Hello §", name) -> fmt.Sprintf("Hello %v", name)
		template := strings.ReplaceAll(e.Template, "§", "%v")
		args := []string{strconv.Quote(template)}
		for _, arg := range e.Args {
			args = append(args, emitGoExpr(arg))
		}
		return "fmt.Sprintf(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprAmbitus:
		// Range expression - emit as helper
		start := emitGoExpr(e.Start)
		end := emitGoExpr(e.End)
		if e.Inclusive {
			return "rangeInclusive(" + start + ", " + end + ")"
		}
		return "rangeExclusive(" + start + ", " + end + ")"

	default:
		return "/* unhandled expression */"
	}
}

func emitGoTypus(typus subsidia.Typus) string {
	switch t := typus.(type) {
	case *subsidia.TypusNomen:
		return goMapTypeName(t.Nomen)
	case *subsidia.TypusNullabilis:
		return "*" + emitGoTypus(t.Inner)
	case *subsidia.TypusGenericus:
		base := goMapTypeName(t.Nomen)
		if base == "[]" {
			// lista<T> -> []T
			if len(t.Args) > 0 {
				return "[]" + emitGoTypus(t.Args[0])
			}
			return "[]interface{}"
		}
		if base == "map" {
			// tabula<K,V> -> map[K]V
			if len(t.Args) >= 2 {
				return "map[" + emitGoTypus(t.Args[0]) + "]" + emitGoTypus(t.Args[1])
			}
			return "map[string]interface{}"
		}
		if base == "set" {
			// copia<T> -> map[T]struct{}
			if len(t.Args) > 0 {
				return "map[" + emitGoTypus(t.Args[0]) + "]struct{}"
			}
			return "map[interface{}]struct{}"
		}
		// Generic type with parameters
		args := []string{}
		for _, arg := range t.Args {
			args = append(args, emitGoTypus(arg))
		}
		return base + "[" + strings.Join(args, ", ") + "]"
	case *subsidia.TypusFunctio:
		params := []string{}
		for _, p := range t.Params {
			params = append(params, emitGoTypus(p))
		}
		ret := ""
		if t.Returns != nil {
			ret = " " + emitGoTypus(t.Returns)
		}
		return "func(" + strings.Join(params, ", ") + ")" + ret
	case *subsidia.TypusUnio:
		// Go doesn't have union types - use interface{}
		return "interface{}"
	case *subsidia.TypusLitteralis:
		return t.Valor
	default:
		return "interface{}"
	}
}

func goMapTypeName(name string) string {
	mapping := map[string]string{
		"textus":    "string",
		"numerus":   "int64",
		"fractus":   "float64",
		"bivalens":  "bool",
		"nihil":     "nil",
		"vacuum":    "",
		"vacuus":    "",
		"ignotum":   "interface{}",
		"quodlibet": "interface{}",
		"quidlibet": "interface{}",
		"lista":     "[]",
		"tabula":    "map",
		"collectio": "set",
		"copia":     "set",
	}
	if mapped, ok := mapping[name]; ok {
		return mapped
	}
	return name
}

func emitGoParam(param subsidia.Param) string {
	typ := "interface{}"
	if param.Typus != nil {
		typ = emitGoTypus(param.Typus)
	}
	if param.Rest {
		return param.Nomen + " ..." + typ
	}
	return param.Nomen + " " + typ
}

func capitalize(s string) string {
	if len(s) == 0 {
		return s
	}
	return strings.ToUpper(s[:1]) + s[1:]
}
