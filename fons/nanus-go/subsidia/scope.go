package subsidia

// SymbolSpecies indicates what kind of symbol this is
type SymbolSpecies int

const (
	SymbolVariabilis SymbolSpecies = iota // variable or constant
	SymbolFunctio                          // function
	SymbolParametrum                       // function parameter
	SymbolTypus                            // type alias
	SymbolGenus                            // class/struct
	SymbolOrdo                             // enum
	SymbolDiscretio                        // discriminated union
	SymbolPactum                           // interface/protocol
	SymbolVarians                          // union variant
)

func (s SymbolSpecies) String() string {
	switch s {
	case SymbolVariabilis:
		return "variabilis"
	case SymbolFunctio:
		return "functio"
	case SymbolParametrum:
		return "parametrum"
	case SymbolTypus:
		return "typus"
	case SymbolGenus:
		return "genus"
	case SymbolOrdo:
		return "ordo"
	case SymbolDiscretio:
		return "discretio"
	case SymbolPactum:
		return "pactum"
	case SymbolVarians:
		return "varians"
	default:
		return "ignotum"
	}
}

// ScopusSpecies indicates what kind of scope this is
type ScopusSpecies int

const (
	ScopusGlobal ScopusSpecies = iota
	ScopusFunctio
	ScopusMassa // block scope
	ScopusGenus // class scope
)

// Symbolum represents a named entity in the symbol table
type Symbolum struct {
	Nomen     string
	Typus     SemanticTypus
	Species   SymbolSpecies
	Mutabilis bool  // true for var, false for fixum
	Locus     Locus
	Node      interface{} // reference to AST node for this symbol
}

// Scopus represents a lexical scope with a symbol table
type Scopus struct {
	Parent  *Scopus
	Symbola map[string]*Symbolum
	Species ScopusSpecies
	Nomen   string // for named scopes (functions, classes)
}

// NovumScopus creates a new scope with the given parent
func NovumScopus(parent *Scopus, species ScopusSpecies, nomen string) *Scopus {
	return &Scopus{
		Parent:  parent,
		Symbola: make(map[string]*Symbolum),
		Species: species,
		Nomen:   nomen,
	}
}

// Definie adds a symbol to this scope
func (s *Scopus) Definie(sym *Symbolum) {
	s.Symbola[sym.Nomen] = sym
}

// Quaere looks up a symbol in this scope and parent scopes
func (s *Scopus) Quaere(nomen string) *Symbolum {
	if sym, ok := s.Symbola[nomen]; ok {
		return sym
	}
	if s.Parent != nil {
		return s.Parent.Quaere(nomen)
	}
	return nil
}

// QuaereLocalis looks up a symbol only in this scope (not parents)
func (s *Scopus) QuaereLocalis(nomen string) *Symbolum {
	return s.Symbola[nomen]
}

// QuaereTypus looks up a type symbol (genus, ordo, discretio, pactum)
func (s *Scopus) QuaereTypus(nomen string) *Symbolum {
	sym := s.Quaere(nomen)
	if sym == nil {
		return nil
	}
	switch sym.Species {
	case SymbolGenus, SymbolOrdo, SymbolDiscretio, SymbolPactum, SymbolTypus:
		return sym
	default:
		return nil
	}
}

// SemanticContext holds the state during semantic analysis
type SemanticContext struct {
	Global        *Scopus                    // global scope
	Current       *Scopus                    // current scope
	Typi          map[string]SemanticTypus   // type registry (resolved types)
	OrdoRegistry  map[string]*SemOrdo      // enum registry
	DiscRegistry  map[string]*SemDiscretio // discriminated union registry
	GenusRegistry map[string]*SemGenus     // class/struct registry
	Errores       []SemanticError
	ExprTypes     map[Expr]SemanticTypus     // expression -> resolved type
}

// SemanticError represents an error found during semantic analysis
type SemanticError struct {
	Nuntius string
	Locus   Locus
}

// NovumSemanticContext creates a new semantic analysis context
func NovumSemanticContext() *SemanticContext {
	global := NovumScopus(nil, ScopusGlobal, "")
	return &SemanticContext{
		Global:        global,
		Current:       global,
		Typi:          make(map[string]SemanticTypus),
		OrdoRegistry:  make(map[string]*SemOrdo),
		DiscRegistry:  make(map[string]*SemDiscretio),
		GenusRegistry: make(map[string]*SemGenus),
		Errores:       []SemanticError{},
		ExprTypes:     make(map[Expr]SemanticTypus),
	}
}

// IntraScopum enters a new scope
func (ctx *SemanticContext) IntraScopum(species ScopusSpecies, nomen string) {
	ctx.Current = NovumScopus(ctx.Current, species, nomen)
}

// ExiScopum exits the current scope, returning to parent
func (ctx *SemanticContext) ExiScopum() {
	if ctx.Current.Parent != nil {
		ctx.Current = ctx.Current.Parent
	}
}

// Definie adds a symbol to the current scope
func (ctx *SemanticContext) Definie(sym *Symbolum) {
	ctx.Current.Definie(sym)
}

// Quaere looks up a symbol in the current scope chain
func (ctx *SemanticContext) Quaere(nomen string) *Symbolum {
	return ctx.Current.Quaere(nomen)
}

// Error records a semantic error
func (ctx *SemanticContext) Error(nuntius string, locus Locus) {
	ctx.Errores = append(ctx.Errores, SemanticError{
		Nuntius: nuntius,
		Locus:   locus,
	})
}

// RegisterTypus registers a resolved type by name
func (ctx *SemanticContext) RegisterTypus(nomen string, typus SemanticTypus) {
	ctx.Typi[nomen] = typus
}

// ResolveTypusNomen resolves a type name to its semantic type
func (ctx *SemanticContext) ResolveTypusNomen(nomen string) SemanticTypus {
	// Check primitive types first
	switch nomen {
	case "textus":
		return TEXTUS
	case "numerus":
		return NUMERUS
	case "fractus":
		return FRACTUS
	case "bivalens":
		return BIVALENS
	case "nihil":
		return NIHIL
	case "vacuum", "vacuus":
		return VACUUM
	case "ignotum", "quodlibet", "quidlibet":
		return IGNOTUM
	}

	// Check registered types
	if t, ok := ctx.Typi[nomen]; ok {
		return t
	}

	// Check registries
	if t, ok := ctx.OrdoRegistry[nomen]; ok {
		return t
	}
	if t, ok := ctx.DiscRegistry[nomen]; ok {
		return t
	}
	if t, ok := ctx.GenusRegistry[nomen]; ok {
		return t
	}

	// Return unresolved reference
	return &SemUsitatum{Nomen: nomen}
}

// SetExprType records the resolved type for an expression
func (ctx *SemanticContext) SetExprType(expr Expr, typus SemanticTypus) {
	ctx.ExprTypes[expr] = typus
}

// GetExprType retrieves the resolved type for an expression
func (ctx *SemanticContext) GetExprType(expr Expr) SemanticTypus {
	if t, ok := ctx.ExprTypes[expr]; ok {
		return t
	}
	return IGNOTUM
}
