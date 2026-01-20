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

// GoEmitter holds state for Go code generation
type GoEmitter struct {
	ctx *subsidia.SemanticContext
}

// EmitGo renders a module to Go code.
func EmitGo(mod *subsidia.Modulus, pkg string) string {
	// Run semantic analysis first
	ctx := subsidia.Analyze(mod)

	e := &GoEmitter{ctx: ctx}

	var b strings.Builder

	// Package declaration
	b.WriteString("package " + pkg + "\n\n")

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
		code := e.emitStmt(stmt, "")
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

func (e *GoEmitter) emitStmt(stmt subsidia.Stmt, indent string) string {
	switch s := stmt.(type) {
	case *subsidia.StmtMassa:
		lines := []string{}
		for _, inner := range s.Corpus {
			lines = append(lines, e.emitStmt(inner, indent+"\t"))
		}
		return "{\n" + strings.Join(lines, "\n") + "\n" + indent + "}"

	case *subsidia.StmtExpressia:
		return indent + e.emitExpr(s.Expr)

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
			typ = " " + e.emitTypus(s.Typus)
		}
		val := ""
		if s.Valor != nil {
			val = " = " + e.emitExpr(s.Valor)
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
			params = append(params, e.emitParam(param))
		}
		ret := ""
		if s.TypusReditus != nil {
			ret = " " + e.emitTypus(s.TypusReditus)
		}
		body := " {}"
		if s.Corpus != nil {
			body = " " + e.emitStmt(s.Corpus, indent)
		}
		return indent + "func " + s.Nomen + generics + "(" + strings.Join(params, ", ") + ")" + ret + body

	case *subsidia.StmtGenus:
		lines := []string{}
		// Struct definition
		lines = append(lines, indent+"type "+s.Nomen+" struct {")
		for _, campo := range s.Campi {
			fieldName := capitalize(campo.Nomen)
			lines = append(lines, indent+"\t"+fieldName+" "+e.emitTypus(campo.Typus))
		}
		lines = append(lines, indent+"}")

		// Constructor
		if len(s.Campi) > 0 {
			lines = append(lines, "")
			params := []string{}
			for _, campo := range s.Campi {
				params = append(params, campo.Nomen+" "+e.emitTypus(campo.Typus))
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
					params = append(params, e.emitParam(param))
				}
				ret := ""
				if fn.TypusReditus != nil {
					ret = " " + e.emitTypus(fn.TypusReditus)
				}
				body := " {}"
				if fn.Corpus != nil {
					body = " " + e.emitStmt(fn.Corpus, indent)
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
				params = append(params, e.emitParam(param))
			}
			ret := ""
			if method.TypusReditus != nil {
				ret = " " + e.emitTypus(method.TypusReditus)
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
				lines = append(lines, indent+"\t"+capitalize(f.Nomen)+" "+e.emitTypus(f.Typus))
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
		code := indent + "if " + e.emitExpr(s.Cond) + " " + e.emitStmt(s.Cons, indent)
		if s.Alt != nil {
			code += " else " + e.emitStmt(s.Alt, indent)
		}
		return code

	case *subsidia.StmtDum:
		return indent + "for " + e.emitExpr(s.Cond) + " " + e.emitStmt(s.Corpus, indent)

	case *subsidia.StmtFacDum:
		// Go doesn't have do-while, emulate with for + break
		lines := []string{}
		lines = append(lines, indent+"for {")
		if block, ok := s.Corpus.(*subsidia.StmtMassa); ok {
			for _, inner := range block.Corpus {
				lines = append(lines, e.emitStmt(inner, indent+"\t"))
			}
		} else {
			lines = append(lines, e.emitStmt(s.Corpus, indent+"\t"))
		}
		lines = append(lines, indent+"\tif !("+e.emitExpr(s.Cond)+") { break }")
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtIteratio:
		binding := s.Binding
		iter := e.emitExpr(s.Iter)
		if s.Species == "De" {
			// for key in map
			return indent + "for " + binding + " := range " + iter + " " + e.emitStmt(s.Corpus, indent)
		}
		// for value in slice
		return indent + "for _, " + binding + " := range " + iter + " " + e.emitStmt(s.Corpus, indent)

	case *subsidia.StmtElige:
		lines := []string{}
		lines = append(lines, indent+"switch "+e.emitExpr(s.Discrim)+" {")
		for _, c := range s.Casus {
			lines = append(lines, indent+"case "+e.emitExpr(c.Cond)+":")
			if block, ok := c.Corpus.(*subsidia.StmtMassa); ok {
				for _, inner := range block.Corpus {
					lines = append(lines, e.emitStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, e.emitStmt(c.Corpus, indent+"\t"))
			}
		}
		if s.Default != nil {
			lines = append(lines, indent+"default:")
			if block, ok := s.Default.(*subsidia.StmtMassa); ok {
				for _, inner := range block.Corpus {
					lines = append(lines, e.emitStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, e.emitStmt(s.Default, indent+"\t"))
			}
		}
		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtDiscerne:
		lines := []string{}
		discrim := e.emitExpr(s.Discrim[0])
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
					lines = append(lines, e.emitStmt(inner, indent+"\t"))
				}
			} else {
				lines = append(lines, e.emitStmt(c.Corpus, indent+"\t"))
			}
		}

		lines = append(lines, indent+"}")
		return strings.Join(lines, "\n")

	case *subsidia.StmtCustodi:
		lines := []string{}
		for _, c := range s.Clausulae {
			lines = append(lines, indent+"if "+e.emitExpr(c.Cond)+" "+e.emitStmt(c.Corpus, indent))
		}
		return strings.Join(lines, "\n")

	case *subsidia.StmtTempta:
		// Go doesn't have try/catch - emit as defer/recover or just the body
		// For now, just emit the body (rivus removed try/catch anyway)
		return e.emitStmt(s.Corpus, indent)

	case *subsidia.StmtRedde:
		if s.Valor != nil {
			return indent + "return " + e.emitExpr(s.Valor)
		}
		return indent + "return"

	case *subsidia.StmtIace:
		// mori -> panic
		return indent + "panic(" + e.emitExpr(s.Arg) + ")"

	case *subsidia.StmtScribe:
		target := "os.Stdout"
		if s.Gradus == "Mone" {
			target = "os.Stderr"
		}
		args := []string{}
		for _, arg := range s.Args {
			args = append(args, e.emitExpr(arg))
		}
		return indent + "fmt.Fprintln(" + target + ", " + strings.Join(args, ", ") + ")"

	case *subsidia.StmtAdfirma:
		msg := `"assertion failed"`
		if s.Msg != nil {
			msg = e.emitExpr(s.Msg)
		}
		return indent + "if !(" + e.emitExpr(s.Cond) + ") { panic(" + msg + ") }"

	case *subsidia.StmtRumpe:
		return indent + "break"

	case *subsidia.StmtPerge:
		return indent + "continue"

	case *subsidia.StmtIncipit:
		// Entry point - emit as main() or init block
		return indent + "func main() " + e.emitStmt(s.Corpus, indent)

	default:
		return indent + "/* unhandled statement */"
	}
}

func (e *GoEmitter) emitExpr(expr subsidia.Expr) string {
	switch x := expr.(type) {
	case *subsidia.ExprNomen:
		return x.Valor

	case *subsidia.ExprEgo:
		return "self"

	case *subsidia.ExprLittera:
		switch x.Species {
		case subsidia.LitteraTextus:
			return strconv.Quote(x.Valor)
		case subsidia.LitteraVerum:
			return "true"
		case subsidia.LitteraFalsum:
			return "false"
		case subsidia.LitteraNihil:
			return "nil"
		default:
			return x.Valor
		}

	case *subsidia.ExprBinaria:
		op := x.Signum
		if mapped, ok := goBinaryOps[op]; ok {
			op = mapped
		}
		// Handle vel (nullish coalescing) specially
		if x.Signum == "vel" {
			// Go doesn't have ??, emit as: func() T { if l := left; l != nil { return l }; return right }()
			// For simplicity, just emit left for now (TODO: proper handling)
			return e.emitExpr(x.Sin)
		}
		// Handle inter (membership test) - value inter collection
		if x.Signum == "inter" {
			val := e.emitExpr(x.Sin)
			// For array literals, expand to equality checks
			if series, ok := x.Dex.(*subsidia.ExprSeries); ok {
				if len(series.Elementa) == 0 {
					return "false"
				}
				checks := []string{}
				for _, elem := range series.Elementa {
					checks = append(checks, val+" == "+e.emitExpr(elem))
				}
				return "(" + strings.Join(checks, " || ") + ")"
			}
			// For other expressions, use sliceContains helper
			return "sliceContains(" + e.emitExpr(x.Dex) + ", " + val + ")"
		}
		return "(" + e.emitExpr(x.Sin) + " " + op + " " + e.emitExpr(x.Dex) + ")"

	case *subsidia.ExprUnaria:
		op := x.Signum
		if x.Signum == "nonnihil" {
			return "(" + e.emitExpr(x.Arg) + " != nil)"
		}
		if x.Signum == "nihil" {
			return "(" + e.emitExpr(x.Arg) + " == nil)"
		}
		if mapped, ok := goUnaryOps[op]; ok {
			op = mapped
		}
		return "(" + op + e.emitExpr(x.Arg) + ")"

	case *subsidia.ExprAssignatio:
		return e.emitExpr(x.Sin) + " " + x.Signum + " " + e.emitExpr(x.Dex)

	case *subsidia.ExprCondicio:
		// Go doesn't have ternary - emit as inline func
		return "func() interface{} { if " + e.emitExpr(x.Cond) + " { return " + e.emitExpr(x.Cons) + " }; return " + e.emitExpr(x.Alt) + " }()"

	case *subsidia.ExprVocatio:
		// Check for method calls that need translation
		if m, ok := x.Callee.(*subsidia.ExprMembrum); ok && !m.Computed {
			if prop, ok := m.Prop.(*subsidia.ExprLittera); ok {
				propName := prop.Valor
				obj := e.emitExpr(m.Obj)
				args := []string{}
				for _, arg := range x.Args {
					args = append(args, e.emitExpr(arg))
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
		for _, arg := range x.Args {
			args = append(args, e.emitExpr(arg))
		}
		return e.emitExpr(x.Callee) + "(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprMembrum:
		obj := e.emitExpr(x.Obj)
		if x.Computed {
			return obj + "[" + e.emitExpr(x.Prop) + "]"
		}
		prop := ""
		if lit, ok := x.Prop.(*subsidia.ExprLittera); ok {
			prop = lit.Valor
		} else {
			prop = e.emitExpr(x.Prop)
		}

		// Check if this is an enum member access
		if nomen, ok := x.Obj.(*subsidia.ExprNomen); ok {
			if ordo, ok := e.ctx.OrdoRegistry[nomen.Valor]; ok {
				// This is Enum.Member - emit as EnumMember (prefixed constant)
				if _, exists := ordo.Membra[prop]; exists {
					return nomen.Valor + capitalize(prop)
				}
			}
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
		for _, elem := range x.Elementa {
			elems = append(elems, e.emitExpr(elem))
		}
		// Infer type from elements if possible
		return "[]interface{}{" + strings.Join(elems, ", ") + "}"

	case *subsidia.ExprObiectum:
		props := []string{}
		for _, p := range x.Props {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = strconv.Quote(lit.Valor)
			} else {
				key = e.emitExpr(p.Key)
			}
			props = append(props, key+": "+e.emitExpr(p.Valor))
		}
		return "map[string]interface{}{" + strings.Join(props, ", ") + "}"

	case *subsidia.ExprClausura:
		params := []string{}
		for _, param := range x.Params {
			typ := "interface{}"
			if param.Typus != nil {
				typ = e.emitTypus(param.Typus)
			}
			params = append(params, param.Nomen+" "+typ)
		}
		ret := "interface{}"
		switch body := x.Corpus.(type) {
		case subsidia.Stmt:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " " + e.emitStmt(body, "")
		case subsidia.Expr:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " { return " + e.emitExpr(body) + " }"
		default:
			return "func(" + strings.Join(params, ", ") + ") " + ret + " {}"
		}

	case *subsidia.ExprNovum:
		args := []string{}
		for _, arg := range x.Args {
			args = append(args, e.emitExpr(arg))
		}
		callee := e.emitExpr(x.Callee)
		if x.Init != nil {
			// Struct literal with initializer: novum Type { field: value }
			if obj, ok := x.Init.(*subsidia.ExprObiectum); ok {
				fields := []string{}
				for _, p := range obj.Props {
					key := ""
					if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
						key = capitalize(lit.Valor)
					} else {
						key = e.emitExpr(p.Key)
					}
					fields = append(fields, key+": "+e.emitExpr(p.Valor))
				}
				return "&" + callee + "{" + strings.Join(fields, ", ") + "}"
			}
			return "&" + callee + "{" + e.emitExpr(x.Init) + "}"
		}
		return "New" + callee + "(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprPostfixNovum:
		// Postfix syntax: { field: value } novum Type
		typeName := e.emitTypus(x.Typus)
		// Strip pointer prefix since we add & ourselves
		if strings.HasPrefix(typeName, "*") {
			typeName = typeName[1:]
		}
		if obj, ok := x.Expr.(*subsidia.ExprObiectum); ok {
			fields := []string{}
			for _, p := range obj.Props {
				key := ""
				if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
					key = capitalize(lit.Valor)
				} else {
					key = e.emitExpr(p.Key)
				}
				fields = append(fields, key+": "+e.emitExpr(p.Valor))
			}
			return "&" + typeName + "{" + strings.Join(fields, ", ") + "}"
		}
		return "&" + typeName + "{" + e.emitExpr(x.Expr) + "}"

	case *subsidia.ExprCede:
		// No await in Go - just emit the expression
		return e.emitExpr(x.Arg)

	case *subsidia.ExprQua:
		// Type assertion
		return e.emitExpr(x.Expr) + ".(" + e.emitTypus(x.Typus) + ")"

	case *subsidia.ExprInnatum:
		// Type assertion/conversion
		return e.emitTypus(x.Typus) + "(" + e.emitExpr(x.Expr) + ")"

	case *subsidia.ExprFinge:
		fields := []string{}
		for _, p := range x.Campi {
			key := ""
			if lit, ok := p.Key.(*subsidia.ExprLittera); ok {
				key = capitalize(lit.Valor)
			} else {
				key = e.emitExpr(p.Key)
			}
			fields = append(fields, key+": "+e.emitExpr(p.Valor))
		}
		return x.Variant + "{" + strings.Join(fields, ", ") + "}"

	case *subsidia.ExprScriptum:
		// Format string: scriptum("Hello §", name) -> fmt.Sprintf("Hello %v", name)
		template := strings.ReplaceAll(x.Template, "§", "%v")
		args := []string{strconv.Quote(template)}
		for _, arg := range x.Args {
			args = append(args, e.emitExpr(arg))
		}
		return "fmt.Sprintf(" + strings.Join(args, ", ") + ")"

	case *subsidia.ExprAmbitus:
		// Range expression - emit as helper
		start := e.emitExpr(x.Start)
		end := e.emitExpr(x.End)
		if x.Inclusive {
			return "rangeInclusive(" + start + ", " + end + ")"
		}
		return "rangeExclusive(" + start + ", " + end + ")"

	default:
		return "/* unhandled expression */"
	}
}

func (e *GoEmitter) emitTypus(typus subsidia.Typus) string {
	switch t := typus.(type) {
	case *subsidia.TypusNomen:
		name := goMapTypeName(t.Nomen)
		// Genus types (structs) are always pointers in Go
		if _, isGenus := e.ctx.GenusRegistry[t.Nomen]; isGenus {
			return "*" + name
		}
		return name
	case *subsidia.TypusNullabilis:
		// For genus types, already emitted as pointer, so no extra *
		if nomen, ok := t.Inner.(*subsidia.TypusNomen); ok {
			if _, isGenus := e.ctx.GenusRegistry[nomen.Nomen]; isGenus {
				return e.emitTypus(t.Inner)
			}
		}
		// For primitives, add pointer
		return "*" + e.emitTypus(t.Inner)
	case *subsidia.TypusGenericus:
		base := goMapTypeName(t.Nomen)
		if base == "[]" {
			// lista<T> -> []T
			if len(t.Args) > 0 {
				return "[]" + e.emitTypus(t.Args[0])
			}
			return "[]interface{}"
		}
		if base == "map" {
			// tabula<K,V> -> map[K]V
			if len(t.Args) >= 2 {
				return "map[" + e.emitTypus(t.Args[0]) + "]" + e.emitTypus(t.Args[1])
			}
			return "map[string]interface{}"
		}
		if base == "set" {
			// copia<T> -> map[T]struct{}
			if len(t.Args) > 0 {
				return "map[" + e.emitTypus(t.Args[0]) + "]struct{}"
			}
			return "map[interface{}]struct{}"
		}
		// Generic type with parameters
		args := []string{}
		for _, arg := range t.Args {
			args = append(args, e.emitTypus(arg))
		}
		return base + "[" + strings.Join(args, ", ") + "]"
	case *subsidia.TypusFunctio:
		params := []string{}
		for _, p := range t.Params {
			params = append(params, e.emitTypus(p))
		}
		ret := ""
		if t.Returns != nil {
			ret = " " + e.emitTypus(t.Returns)
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

func (e *GoEmitter) emitParam(param subsidia.Param) string {
	typ := "interface{}"
	if param.Typus != nil {
		typ = e.emitTypus(param.Typus)
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
