package subsidia

// analyzeExpression resolves the type of an expression and records it
func analyzeExpression(ctx *SemanticContext, expr Expr) SemanticTypus {
	if expr == nil {
		return IGNOTUM
	}

	var result SemanticTypus

	switch e := expr.(type) {
	case *ExprLittera:
		result = analyzeLittera(ctx, e)

	case *ExprNomen:
		result = analyzeNomen(ctx, e)

	case *ExprEgo:
		// 'ego' (self) - look up in scope
		if sym := ctx.Quaere("ego"); sym != nil {
			result = sym.Typus
		} else {
			result = IGNOTUM
		}

	case *ExprBinaria:
		result = analyzeBinaria(ctx, e)

	case *ExprUnaria:
		result = analyzeUnaria(ctx, e)

	case *ExprAssignatio:
		analyzeExpression(ctx, e.Sin)
		analyzeExpression(ctx, e.Dex)
		result = ctx.GetExprType(e.Sin)

	case *ExprCondicio:
		analyzeExpression(ctx, e.Cond)
		consType := analyzeExpression(ctx, e.Cons)
		analyzeExpression(ctx, e.Alt)
		result = consType // TODO: union of cons and alt types

	case *ExprVocatio:
		result = analyzeVocatio(ctx, e)

	case *ExprMembrum:
		result = analyzeMembrum(ctx, e)

	case *ExprSeries:
		result = analyzeSeries(ctx, e)

	case *ExprObiectum:
		result = analyzeObiectum(ctx, e)

	case *ExprClausura:
		result = analyzeClausura(ctx, e)

	case *ExprNovum:
		result = analyzeNovum(ctx, e)

	case *ExprFinge:
		result = analyzeFinge(ctx, e)

	case *ExprCede:
		// await - type is the inner expression type
		result = analyzeExpression(ctx, e.Arg)

	case *ExprQua:
		// type assertion - type is the asserted type
		result = resolveTypusAnnotatio(ctx, e.Typus)

	case *ExprInnatum:
		// type cast - type is the target type
		result = resolveTypusAnnotatio(ctx, e.Typus)

	case *ExprPostfixNovum:
		result = resolveTypusAnnotatio(ctx, e.Typus)

	case *ExprScriptum:
		// string interpolation
		for _, arg := range e.Args {
			analyzeExpression(ctx, arg)
		}
		result = TEXTUS

	case *ExprAmbitus:
		analyzeExpression(ctx, e.Start)
		analyzeExpression(ctx, e.End)
		result = &SemLista{Elementum: NUMERUS}

	case *ExprConversio:
		analyzeExpression(ctx, e.Expr)
		if e.Fallback != nil {
			analyzeExpression(ctx, e.Fallback)
		}
		switch e.Species {
		case "numeratum":
			result = NUMERUS
		case "fractatum":
			result = FRACTUS
		case "textatum":
			result = TEXTUS
		case "bivalentum":
			result = BIVALENS
		default:
			result = IGNOTUM
		}

	default:
		result = IGNOTUM
	}

	ctx.SetExprType(expr, result)
	return result
}

func analyzeLittera(ctx *SemanticContext, e *ExprLittera) SemanticTypus {
	switch e.Species {
	case LitteraTextus:
		return TEXTUS
	case LitteraNumerus:
		return NUMERUS
	case LitteraFractus:
		return FRACTUS
	case LitteraVerum, LitteraFalsum:
		return BIVALENS
	case LitteraNihil:
		return NIHIL
	default:
		return IGNOTUM
	}
}

func analyzeNomen(ctx *SemanticContext, e *ExprNomen) SemanticTypus {
	sym := ctx.Quaere(e.Valor)
	if sym != nil {
		return sym.Typus
	}

	// Check if it's a type name (for enum access like Color.Red)
	if t := ctx.ResolveTypusNomen(e.Valor); t != nil {
		if _, ok := t.(*SemUsitatum); !ok {
			// It's a real type, return it
			return t
		}
	}

	ctx.Error("undefined identifier: "+e.Valor, e.Locus)
	return IGNOTUM
}

func analyzeBinaria(ctx *SemanticContext, e *ExprBinaria) SemanticTypus {
	leftType := analyzeExpression(ctx, e.Sin)
	rightType := analyzeExpression(ctx, e.Dex)

	switch e.Signum {
	case "+", "-", "*", "/", "%":
		// Arithmetic - result is numeric type
		if isNumeric(leftType) && isNumeric(rightType) {
			// If either is fractus, result is fractus
			if isFractus(leftType) || isFractus(rightType) {
				return FRACTUS
			}
			return NUMERUS
		}
		// String concatenation
		if e.Signum == "+" && isTextus(leftType) {
			return TEXTUS
		}
		return IGNOTUM

	case "==", "!=", "<", ">", "<=", ">=":
		return BIVALENS

	case "et", "aut", "&&", "||":
		return BIVALENS

	case "vel":
		// Nullish coalescing - return non-null type
		return leftType

	default:
		return IGNOTUM
	}
}

func analyzeUnaria(ctx *SemanticContext, e *ExprUnaria) SemanticTypus {
	argType := analyzeExpression(ctx, e.Arg)

	switch e.Signum {
	case "non", "!":
		return BIVALENS
	case "nihil", "nonnihil", "nulla", "nonnulla":
		return BIVALENS
	case "-", "+", "~", "positivum", "negativum":
		return argType
	default:
		return argType
	}
}

func analyzeVocatio(ctx *SemanticContext, e *ExprVocatio) SemanticTypus {
	// Analyze arguments
	for _, arg := range e.Args {
		analyzeExpression(ctx, arg)
	}

	// Analyze callee
	calleeType := analyzeExpression(ctx, e.Callee)

	// If callee is a function, return its return type
	if fn, ok := calleeType.(*SemFunctio); ok {
		if fn.Reditus != nil {
			return fn.Reditus
		}
		return VACUUM
	}

	// If callee is a genus (constructor call via member), check for method
	if membrum, ok := e.Callee.(*ExprMembrum); ok {
		objType := ctx.GetExprType(membrum.Obj)
		if genus, ok := objType.(*SemGenus); ok {
			if lit, ok := membrum.Prop.(*ExprLittera); ok {
				if method, ok := genus.Methodi[lit.Valor]; ok {
					if method.Reditus != nil {
						return method.Reditus
					}
					return VACUUM
				}
			}
		}
	}

	// Check if it's a constructor
	if nomen, ok := e.Callee.(*ExprNomen); ok {
		if genus, ok := ctx.GenusRegistry[nomen.Valor]; ok {
			return genus
		}
	}

	return IGNOTUM
}

func analyzeMembrum(ctx *SemanticContext, e *ExprMembrum) SemanticTypus {
	objType := analyzeExpression(ctx, e.Obj)

	// Handle computed access (array/map indexing)
	if e.Computed {
		analyzeExpression(ctx, e.Prop)
		switch t := objType.(type) {
		case *SemLista:
			return t.Elementum
		case *SemTabula:
			return t.Valor
		case *SemCopia:
			return BIVALENS // set membership check
		}
		return IGNOTUM
	}

	// Get property name
	propName := ""
	if lit, ok := e.Prop.(*ExprLittera); ok {
		propName = lit.Valor
	} else {
		return IGNOTUM
	}

	// Handle built-in properties
	switch propName {
	case "longitudo":
		switch objType.(type) {
		case *SemLista, *SemTabula, *SemCopia:
			return NUMERUS
		}
		if isTextus(objType) {
			return NUMERUS
		}
	case "primus", "ultimus":
		if lista, ok := objType.(*SemLista); ok {
			return lista.Elementum
		}
	}

	// Handle genus field access
	switch t := objType.(type) {
	case *SemGenus:
		if fieldType, ok := t.Agri[propName]; ok {
			return fieldType
		}
		if method, ok := t.Methodi[propName]; ok {
			return method
		}

	case *SemUsitatum:
		// Resolve the type reference
		if genus, ok := ctx.GenusRegistry[t.Nomen]; ok {
			if fieldType, ok := genus.Agri[propName]; ok {
				return fieldType
			}
			if method, ok := genus.Methodi[propName]; ok {
				return method
			}
		}

	case *SemOrdo:
		// Enum member access - returns the enum type itself
		if _, ok := t.Membra[propName]; ok {
			return t
		}

	case *SemDiscretio:
		// Check if accessing a variant constructor
		if variant, ok := t.Variantes[propName]; ok {
			return variant
		}
	}

	return IGNOTUM
}

func analyzeSeries(ctx *SemanticContext, e *ExprSeries) SemanticTypus {
	var elemType SemanticTypus = IGNOTUM

	for i, elem := range e.Elementa {
		t := analyzeExpression(ctx, elem)
		if i == 0 {
			elemType = t
		}
		// TODO: check type consistency or compute union
	}

	return &SemLista{Elementum: elemType}
}

func analyzeObiectum(ctx *SemanticContext, e *ExprObiectum) SemanticTypus {
	fields := make(map[string]SemanticTypus)

	for _, p := range e.Props {
		valueType := analyzeExpression(ctx, p.Valor)
		if lit, ok := p.Key.(*ExprLittera); ok {
			fields[lit.Valor] = valueType
		}
	}

	// Object literals without a type annotation are anonymous structs
	// The emitter will need context to know if this should be a struct literal
	return &SemGenus{
		Nomen: "", // anonymous
		Agri:  fields,
	}
}

func analyzeClausura(ctx *SemanticContext, e *ExprClausura) SemanticTypus {
	params := make([]SemanticTypus, len(e.Params))

	ctx.IntraScopum(ScopusFunctio, "")

	for i, p := range e.Params {
		var paramType SemanticTypus = IGNOTUM
		if p.Typus != nil {
			paramType = resolveTypusAnnotatio(ctx, p.Typus)
		}
		params[i] = paramType

		ctx.Definie(&Symbolum{
			Nomen:   p.Nomen,
			Typus:   paramType,
			Species: SymbolParametrum,
		})
	}

	// Analyze body
	var reditus SemanticTypus
	switch body := e.Corpus.(type) {
	case Stmt:
		analyzeStatement(ctx, body)
		// TODO: infer return type from return statements
	case Expr:
		reditus = analyzeExpression(ctx, body)
	}

	ctx.ExiScopum()

	return &SemFunctio{
		Params: params,
		Reditus:   reditus,
	}
}

func analyzeNovum(ctx *SemanticContext, e *ExprNovum) SemanticTypus {
	// Analyze arguments
	for _, arg := range e.Args {
		analyzeExpression(ctx, arg)
	}

	// Analyze initializer if present
	if e.Init != nil {
		analyzeExpression(ctx, e.Init)
	}

	// Get the type being constructed
	if nomen, ok := e.Callee.(*ExprNomen); ok {
		if genus, ok := ctx.GenusRegistry[nomen.Valor]; ok {
			return genus
		}
		// Could be a variant
		if sym := ctx.Quaere(nomen.Valor); sym != nil && sym.Species == SymbolVarians {
			return sym.Typus
		}
		return &SemUsitatum{Nomen: nomen.Valor}
	}

	return IGNOTUM
}

func analyzeFinge(ctx *SemanticContext, e *ExprFinge) SemanticTypus {
	// Analyze field values
	for _, p := range e.Campi {
		analyzeExpression(ctx, p.Valor)
	}

	// Look up variant type
	if sym := ctx.Quaere(e.Variant); sym != nil && sym.Species == SymbolVarians {
		return sym.Typus
	}

	// Check in discretio registry
	for _, disc := range ctx.DiscRegistry {
		if variant, ok := disc.Variantes[e.Variant]; ok {
			return variant
		}
	}

	return &SemUsitatum{Nomen: e.Variant}
}

// Helper functions

func isNumeric(t SemanticTypus) bool {
	if p, ok := t.(*SemPrimitivus); ok {
		return p.Species == "numerus" || p.Species == "fractus"
	}
	return false
}

func isFractus(t SemanticTypus) bool {
	if p, ok := t.(*SemPrimitivus); ok {
		return p.Species == "fractus"
	}
	return false
}

func isTextus(t SemanticTypus) bool {
	if p, ok := t.(*SemPrimitivus); ok {
		return p.Species == "textus"
	}
	return false
}
