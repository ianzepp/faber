package subsidia

// Analyze performs semantic analysis on a parsed module.
// It returns a SemanticContext containing resolved types, symbols, and any errors.
func Analyze(mod *Modulus) *SemanticContext {
	ctx := NovumSemanticContext()

	// Pass 1: Collect all type declarations (genus, ordo, discretio, pactum)
	// This allows forward references to work
	for _, stmt := range mod.Corpus {
		collectDeclaration(ctx, stmt)
	}

	// Pass 2: Resolve types in all declarations and bodies
	for _, stmt := range mod.Corpus {
		analyzeStatement(ctx, stmt)
	}

	return ctx
}

// collectDeclaration registers type declarations in the first pass
func collectDeclaration(ctx *SemanticContext, stmt Stmt) {
	switch s := stmt.(type) {
	case *StmtGenus:
		collectGenus(ctx, s)
	case *StmtOrdo:
		collectOrdo(ctx, s)
	case *StmtDiscretio:
		collectDiscretio(ctx, s)
	case *StmtPactum:
		collectPactum(ctx, s)
	case *StmtFunctio:
		collectFunctio(ctx, s)
	case *StmtVaria:
		// Variables collected in pass 2
	}
}

func collectGenus(ctx *SemanticContext, s *StmtGenus) {
	genus := &SemGenus{
		Nomen:   s.Nomen,
		Agri:    make(map[string]SemanticTypus),
		Methodi: make(map[string]*SemFunctio),
	}

	// Collect fields
	for _, campo := range s.Campi {
		if campo.Typus != nil {
			genus.Agri[campo.Nomen] = resolveTypusAnnotatio(ctx, campo.Typus)
		} else {
			genus.Agri[campo.Nomen] = IGNOTUM
		}
	}

	// Collect method signatures
	for _, method := range s.Methodi {
		if fn, ok := method.(*StmtFunctio); ok {
			genus.Methodi[fn.Nomen] = resolveFunctioTypus(ctx, fn)
		}
	}

	ctx.GenusRegistry[s.Nomen] = genus
	ctx.RegisterTypus(s.Nomen, genus)

	// Add to symbol table
	ctx.Definie(&Symbolum{
		Nomen:   s.Nomen,
		Typus:   genus,
		Species: SymbolGenus,
		Locus:   s.Locus,
		Node:    s,
	})
}

func collectOrdo(ctx *SemanticContext, s *StmtOrdo) {
	ordo := &SemOrdo{
		Nomen:  s.Nomen,
		Membra: make(map[string]int64),
	}

	// Collect members with their values
	for i, m := range s.Membra {
		ordo.Membra[m.Nomen] = int64(i)
	}

	ctx.OrdoRegistry[s.Nomen] = ordo
	ctx.RegisterTypus(s.Nomen, ordo)

	// Add to symbol table
	ctx.Definie(&Symbolum{
		Nomen:   s.Nomen,
		Typus:   ordo,
		Species: SymbolOrdo,
		Locus:   s.Locus,
		Node:    s,
	})
}

func collectDiscretio(ctx *SemanticContext, s *StmtDiscretio) {
	disc := &SemDiscretio{
		Nomen:     s.Nomen,
		Variantes: make(map[string]*SemGenus),
	}

	// Collect variants
	for _, v := range s.Variantes {
		variant := &SemGenus{
			Nomen: v.Nomen,
			Agri:  make(map[string]SemanticTypus),
		}
		for _, f := range v.Campi {
			if f.Typus != nil {
				variant.Agri[f.Nomen] = resolveTypusAnnotatio(ctx, f.Typus)
			} else {
				variant.Agri[f.Nomen] = IGNOTUM
			}
		}
		disc.Variantes[v.Nomen] = variant

		// Also register variant as a symbol
		ctx.Definie(&Symbolum{
			Nomen:   v.Nomen,
			Typus:   variant,
			Species: SymbolVarians,
			Locus:   v.Locus,
			Node:    &v,
		})
	}

	ctx.DiscRegistry[s.Nomen] = disc
	ctx.RegisterTypus(s.Nomen, disc)

	// Add to symbol table
	ctx.Definie(&Symbolum{
		Nomen:   s.Nomen,
		Typus:   disc,
		Species: SymbolDiscretio,
		Locus:   s.Locus,
		Node:    s,
	})
}

func collectPactum(ctx *SemanticContext, s *StmtPactum) {
	pactum := &SemPactum{
		Nomen:   s.Nomen,
		Methodi: make(map[string]*SemFunctio),
	}

	for _, m := range s.Methodi {
		pactum.Methodi[m.Nomen] = resolvePactumMethodTypus(ctx, &m)
	}

	ctx.RegisterTypus(s.Nomen, pactum)

	ctx.Definie(&Symbolum{
		Nomen:   s.Nomen,
		Typus:   pactum,
		Species: SymbolPactum,
		Locus:   s.Locus,
		Node:    s,
	})
}

func collectFunctio(ctx *SemanticContext, s *StmtFunctio) {
	if s.Externa {
		// External functions are declared but not defined
		return
	}

	funcTypus := resolveFunctioTypus(ctx, s)

	ctx.Definie(&Symbolum{
		Nomen:   s.Nomen,
		Typus:   funcTypus,
		Species: SymbolFunctio,
		Locus:   s.Locus,
		Node:    s,
	})
}

func resolveFunctioTypus(ctx *SemanticContext, s *StmtFunctio) *SemFunctio {
	params := make([]SemanticTypus, len(s.Params))
	for i, p := range s.Params {
		if p.Typus != nil {
			params[i] = resolveTypusAnnotatio(ctx, p.Typus)
		} else {
			params[i] = IGNOTUM
		}
	}

	var reditus SemanticTypus
	if s.TypusReditus != nil {
		reditus = resolveTypusAnnotatio(ctx, s.TypusReditus)
	}

	return &SemFunctio{
		Params: params,
		Reditus:   reditus,
	}
}

func resolvePactumMethodTypus(ctx *SemanticContext, m *PactumMethodus) *SemFunctio {
	params := make([]SemanticTypus, len(m.Params))
	for i, p := range m.Params {
		if p.Typus != nil {
			params[i] = resolveTypusAnnotatio(ctx, p.Typus)
		} else {
			params[i] = IGNOTUM
		}
	}

	var reditus SemanticTypus
	if m.TypusReditus != nil {
		reditus = resolveTypusAnnotatio(ctx, m.TypusReditus)
	}

	return &SemFunctio{
		Params: params,
		Reditus:   reditus,
	}
}

// resolveTypusAnnotatio converts an AST type annotation to a SemanticTypus
func resolveTypusAnnotatio(ctx *SemanticContext, typus Typus) SemanticTypus {
	if typus == nil {
		return IGNOTUM
	}

	switch t := typus.(type) {
	case *TypusNomen:
		return ctx.ResolveTypusNomen(t.Nomen)

	case *TypusNullabilis:
		inner := resolveTypusAnnotatio(ctx, t.Inner)
		return Nullabilis(inner)

	case *TypusGenericus:
		base := t.Nomen
		switch base {
		case "lista":
			var elem SemanticTypus = IGNOTUM
			if len(t.Args) > 0 {
				elem = resolveTypusAnnotatio(ctx, t.Args[0])
			}
			return &SemLista{Elementum: elem}
		case "tabula":
			var clavis SemanticTypus = TEXTUS
			var valor SemanticTypus = IGNOTUM
			if len(t.Args) > 0 {
				clavis = resolveTypusAnnotatio(ctx, t.Args[0])
			}
			if len(t.Args) > 1 {
				valor = resolveTypusAnnotatio(ctx, t.Args[1])
			}
			return &SemTabula{Clavis: clavis, Valor: valor}
		case "copia", "collectio":
			var elem SemanticTypus = IGNOTUM
			if len(t.Args) > 0 {
				elem = resolveTypusAnnotatio(ctx, t.Args[0])
			}
			return &SemCopia{Elementum: elem}
		default:
			// Generic user type - return as reference for now
			return &SemUsitatum{Nomen: base}
		}

	case *TypusFunctio:
		params := make([]SemanticTypus, len(t.Params))
		for i, p := range t.Params {
			params[i] = resolveTypusAnnotatio(ctx, p)
		}
		var reditus SemanticTypus
		if t.Returns != nil {
			reditus = resolveTypusAnnotatio(ctx, t.Returns)
		}
		return &SemFunctio{Params: params, Reditus: reditus}

	case *TypusUnio:
		membra := make([]SemanticTypus, len(t.Members))
		for i, m := range t.Members {
			membra[i] = resolveTypusAnnotatio(ctx, m)
		}
		return &SemUnio{Membra: membra}

	case *TypusLitteralis:
		// Literal types (like "success" | "error") - treat as string for now
		return TEXTUS

	default:
		return IGNOTUM
	}
}

// analyzeStatement performs semantic analysis on a statement
func analyzeStatement(ctx *SemanticContext, stmt Stmt) {
	switch s := stmt.(type) {
	case *StmtMassa:
		ctx.IntraScopum(ScopusMassa, "")
		for _, inner := range s.Corpus {
			analyzeStatement(ctx, inner)
		}
		ctx.ExiScopum()

	case *StmtVaria:
		analyzeVaria(ctx, s)

	case *StmtFunctio:
		analyzeFunctio(ctx, s)

	case *StmtGenus:
		analyzeGenus(ctx, s)

	case *StmtSi:
		analyzeExpression(ctx, s.Cond)
		analyzeStatement(ctx, s.Cons)
		if s.Alt != nil {
			analyzeStatement(ctx, s.Alt)
		}

	case *StmtDum:
		analyzeExpression(ctx, s.Cond)
		analyzeStatement(ctx, s.Corpus)

	case *StmtFacDum:
		analyzeStatement(ctx, s.Corpus)
		analyzeExpression(ctx, s.Cond)

	case *StmtIteratio:
		analyzeExpression(ctx, s.Iter)
		ctx.IntraScopum(ScopusMassa, "")
		// Add loop binding to scope
		iterType := ctx.GetExprType(s.Iter)
		var elemType SemanticTypus = IGNOTUM
		if lista, ok := iterType.(*SemLista); ok {
			elemType = lista.Elementum
		}
		ctx.Definie(&Symbolum{
			Nomen:   s.Binding,
			Typus:   elemType,
			Species: SymbolVariabilis,
		})
		analyzeStatement(ctx, s.Corpus)
		ctx.ExiScopum()

	case *StmtElige:
		analyzeExpression(ctx, s.Discrim)
		for _, c := range s.Casus {
			analyzeExpression(ctx, c.Cond)
			analyzeStatement(ctx, c.Corpus)
		}
		if s.Default != nil {
			analyzeStatement(ctx, s.Default)
		}

	case *StmtDiscerne:
		for _, d := range s.Discrim {
			analyzeExpression(ctx, d)
		}
		for _, c := range s.Casus {
			ctx.IntraScopum(ScopusMassa, "")
			// Add pattern bindings to scope
			for _, p := range c.Patterns {
				analyzePattern(ctx, p, s.Discrim)
			}
			analyzeStatement(ctx, c.Corpus)
			ctx.ExiScopum()
		}

	case *StmtRedde:
		if s.Valor != nil {
			analyzeExpression(ctx, s.Valor)
		}

	case *StmtExpressia:
		analyzeExpression(ctx, s.Expr)

	case *StmtScribe:
		for _, arg := range s.Args {
			analyzeExpression(ctx, arg)
		}

	case *StmtAdfirma:
		analyzeExpression(ctx, s.Cond)
		if s.Msg != nil {
			analyzeExpression(ctx, s.Msg)
		}

	case *StmtIace:
		analyzeExpression(ctx, s.Arg)

	case *StmtCustodi:
		for _, c := range s.Clausulae {
			analyzeExpression(ctx, c.Cond)
			analyzeStatement(ctx, c.Corpus)
		}

	case *StmtIncipit:
		analyzeStatement(ctx, s.Corpus)

	// Type declarations already handled in pass 1
	case *StmtOrdo, *StmtDiscretio, *StmtPactum:
		// Already collected

	case *StmtImporta:
		// TODO: handle imports

	case *StmtTempta:
		analyzeStatement(ctx, s.Corpus)
		if s.Cape != nil {
			analyzeStatement(ctx, s.Cape.Corpus)
		}
		if s.Demum != nil {
			analyzeStatement(ctx, s.Demum)
		}

	case *StmtTypusAlias:
		// Register type alias
		targetType := resolveTypusAnnotatio(ctx, s.Typus)
		ctx.RegisterTypus(s.Nomen, targetType)
		ctx.Definie(&Symbolum{
			Nomen:   s.Nomen,
			Typus:   targetType,
			Species: SymbolTypus,
			Locus:   s.Locus,
			Node:    s,
		})

	case *StmtIn:
		// Mutation block - analyze expression and body
		analyzeExpression(ctx, s.Expr)
		analyzeStatement(ctx, s.Corpus)
	}
}

func analyzeVaria(ctx *SemanticContext, s *StmtVaria) {
	if s.Externa {
		return
	}

	var varType SemanticTypus = IGNOTUM

	// Get type from annotation if present
	if s.Typus != nil {
		varType = resolveTypusAnnotatio(ctx, s.Typus)
	}

	// Analyze initializer if present
	if s.Valor != nil {
		analyzeExpression(ctx, s.Valor)
		initType := ctx.GetExprType(s.Valor)

		// If no annotation, infer from initializer
		if s.Typus == nil {
			varType = initType
		}
	}

	ctx.Definie(&Symbolum{
		Nomen:     s.Nomen,
		Typus:     varType,
		Species:   SymbolVariabilis,
		Mutabilis: s.Species == VariaVaria,
		Locus:     s.Locus,
		Node:      s,
	})
}

func analyzeFunctio(ctx *SemanticContext, s *StmtFunctio) {
	if s.Externa || s.Corpus == nil {
		return
	}

	ctx.IntraScopum(ScopusFunctio, s.Nomen)

	// Add parameters to scope
	for _, p := range s.Params {
		var paramType SemanticTypus = IGNOTUM
		if p.Typus != nil {
			paramType = resolveTypusAnnotatio(ctx, p.Typus)
		}
		ctx.Definie(&Symbolum{
			Nomen:   p.Nomen,
			Typus:   paramType,
			Species: SymbolParametrum,
		})
	}

	analyzeStatement(ctx, s.Corpus)

	ctx.ExiScopum()
}

func analyzeGenus(ctx *SemanticContext, s *StmtGenus) {
	// Analyze method bodies
	for _, method := range s.Methodi {
		if fn, ok := method.(*StmtFunctio); ok {
			ctx.IntraScopum(ScopusGenus, s.Nomen)

			// Add 'ego' (self) to scope
			genus := ctx.GenusRegistry[s.Nomen]
			ctx.Definie(&Symbolum{
				Nomen:   "ego",
				Typus:   genus,
				Species: SymbolVariabilis,
			})

			analyzeFunctio(ctx, fn)

			ctx.ExiScopum()
		}
	}
}

func analyzePattern(ctx *SemanticContext, p VariansPattern, discrim []Expr) {
	if p.Wildcard {
		return
	}

	// Look up the variant type
	if len(discrim) > 0 {
		discrimType := ctx.GetExprType(discrim[0])
		if disc, ok := discrimType.(*SemDiscretio); ok {
			if variant, ok := disc.Variantes[p.Variant]; ok {
				// Add field bindings
				for _, b := range p.Bindings {
					var fieldType SemanticTypus = IGNOTUM
					if ft, ok := variant.Agri[b]; ok {
						fieldType = ft
					}
					ctx.Definie(&Symbolum{
						Nomen:   b,
						Typus:   fieldType,
						Species: SymbolVariabilis,
					})
				}
			}
		}
	}

	// Add alias binding if present
	if p.Alias != nil {
		ctx.Definie(&Symbolum{
			Nomen:   *p.Alias,
			Typus:   IGNOTUM, // TODO: resolve from discriminant
			Species: SymbolVariabilis,
		})
	}
}
