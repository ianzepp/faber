package subsidia

// SemanticTypus represents a resolved type in the Faber type system.
// These are distinct from the AST Typus types which are syntax nodes.
type SemanticTypus interface {
	isSemanticTypus()
	String() string
}

// Primitive types

type SemPrimitivus struct {
	Species    string // textus, numerus, fractus, bivalens, nihil, vacuum
	Nullabilis bool
}

func (t *SemPrimitivus) isSemanticTypus() {}
func (t *SemPrimitivus) String() string {
	s := t.Species
	if t.Nullabilis {
		s += "?"
	}
	return s
}

// Collection types

type SemLista struct {
	Elementum  SemanticTypus
	Nullabilis bool
}

func (t *SemLista) isSemanticTypus() {}
func (t *SemLista) String() string {
	s := "lista<" + t.Elementum.String() + ">"
	if t.Nullabilis {
		s += "?"
	}
	return s
}

type SemTabula struct {
	Clavis     SemanticTypus
	Valor      SemanticTypus
	Nullabilis bool
}

func (t *SemTabula) isSemanticTypus() {}
func (t *SemTabula) String() string {
	s := "tabula<" + t.Clavis.String() + ", " + t.Valor.String() + ">"
	if t.Nullabilis {
		s += "?"
	}
	return s
}

type SemCopia struct {
	Elementum  SemanticTypus
	Nullabilis bool
}

func (t *SemCopia) isSemanticTypus() {}
func (t *SemCopia) String() string {
	s := "copia<" + t.Elementum.String() + ">"
	if t.Nullabilis {
		s += "?"
	}
	return s
}

// Function type

type SemFunctio struct {
	Params     []SemanticTypus
	Reditus    SemanticTypus // nil for void
	Nullabilis bool
}

func (t *SemFunctio) isSemanticTypus() {}
func (t *SemFunctio) String() string {
	s := "functio("
	for i, p := range t.Params {
		if i > 0 {
			s += ", "
		}
		s += p.String()
	}
	s += ")"
	if t.Reditus != nil {
		s += " -> " + t.Reditus.String()
	}
	if t.Nullabilis {
		s += "?"
	}
	return s
}

// User-defined types

type SemGenus struct {
	Nomen      string
	Agri       map[string]SemanticTypus // field name -> type
	Methodi    map[string]*SemFunctio   // method name -> signature
	Nullabilis bool
}

func (t *SemGenus) isSemanticTypus() {}
func (t *SemGenus) String() string {
	s := t.Nomen
	if t.Nullabilis {
		s += "?"
	}
	return s
}

type SemOrdo struct {
	Nomen  string
	Membra map[string]int64 // member name -> value
}

func (t *SemOrdo) isSemanticTypus() {}
func (t *SemOrdo) String() string {
	return t.Nomen
}

type SemDiscretio struct {
	Nomen     string
	Variantes map[string]*SemGenus // variant name -> fields as genus
}

func (t *SemDiscretio) isSemanticTypus() {}
func (t *SemDiscretio) String() string {
	return t.Nomen
}

type SemPactum struct {
	Nomen   string
	Methodi map[string]*SemFunctio // method name -> signature
}

func (t *SemPactum) isSemanticTypus() {}
func (t *SemPactum) String() string {
	return t.Nomen
}

// Reference to a user-defined type (unresolved or for simple references)
type SemUsitatum struct {
	Nomen      string
	Nullabilis bool
}

func (t *SemUsitatum) isSemanticTypus() {}
func (t *SemUsitatum) String() string {
	s := t.Nomen
	if t.Nullabilis {
		s += "?"
	}
	return s
}

// Union type (A | B | C)
type SemUnio struct {
	Membra     []SemanticTypus
	Nullabilis bool
}

func (t *SemUnio) isSemanticTypus() {}
func (t *SemUnio) String() string {
	s := ""
	for i, m := range t.Membra {
		if i > 0 {
			s += " | "
		}
		s += m.String()
	}
	if t.Nullabilis {
		s += "?"
	}
	return s
}

// Generic type parameter (e.g., T in lista<T>)
type SemParametrum struct {
	Nomen string
}

func (t *SemParametrum) isSemanticTypus() {}
func (t *SemParametrum) String() string {
	return t.Nomen
}

// Unknown/error type for unresolved cases
type SemIgnotum struct {
	Ratio string // reason for unknown
}

func (t *SemIgnotum) isSemanticTypus() {}
func (t *SemIgnotum) String() string {
	return "ignotum"
}

// Primitive type constants
var (
	TEXTUS   = &SemPrimitivus{Species: "textus"}
	NUMERUS  = &SemPrimitivus{Species: "numerus"}
	FRACTUS  = &SemPrimitivus{Species: "fractus"}
	BIVALENS = &SemPrimitivus{Species: "bivalens"}
	NIHIL    = &SemPrimitivus{Species: "nihil"}
	VACUUM   = &SemPrimitivus{Species: "vacuum"}
	IGNOTUM  = &SemIgnotum{Ratio: "unresolved"}
)

// Helper to make a type nullable
func Nullabilis(t SemanticTypus) SemanticTypus {
	switch typ := t.(type) {
	case *SemPrimitivus:
		return &SemPrimitivus{Species: typ.Species, Nullabilis: true}
	case *SemLista:
		return &SemLista{Elementum: typ.Elementum, Nullabilis: true}
	case *SemTabula:
		return &SemTabula{Clavis: typ.Clavis, Valor: typ.Valor, Nullabilis: true}
	case *SemCopia:
		return &SemCopia{Elementum: typ.Elementum, Nullabilis: true}
	case *SemFunctio:
		return &SemFunctio{Params: typ.Params, Reditus: typ.Reditus, Nullabilis: true}
	case *SemGenus:
		return &SemGenus{Nomen: typ.Nomen, Agri: typ.Agri, Methodi: typ.Methodi, Nullabilis: true}
	case *SemUsitatum:
		return &SemUsitatum{Nomen: typ.Nomen, Nullabilis: true}
	case *SemUnio:
		return &SemUnio{Membra: typ.Membra, Nullabilis: true}
	default:
		return t
	}
}
