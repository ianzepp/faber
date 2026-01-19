package subsidia

// Source location for error reporting.
type Locus struct {
	Linea   int `json:"linea"`
	Columna int `json:"columna"`
	Index   int `json:"index"`
}

// Tokens produced by lexer.
type Token struct {
	Tag   string `json:"tag"`
	Valor string `json:"valor"`
	Locus Locus  `json:"locus"`
}

// Token tag constants.
const (
	TokenEOF        = "EOF"
	TokenNewline    = "Newline"
	TokenIdentifier = "Identifier"
	TokenNumerus    = "Numerus"
	TokenTextus     = "Textus"
	TokenOperator   = "Operator"
	TokenPunctuator = "Punctuator"
	TokenKeyword    = "Keyword"
	TokenComment    = "Comment"
)

// Type annotations.
type Typus interface {
	typusNode()
}

type TypusNomen struct {
	Tag   string `json:"tag"`
	Nomen string `json:"nomen"`
}

type TypusNullabilis struct {
	Tag   string `json:"tag"`
	Inner Typus  `json:"inner"`
}

type TypusGenericus struct {
	Tag   string  `json:"tag"`
	Nomen string  `json:"nomen"`
	Args  []Typus `json:"args"`
}

type TypusFunctio struct {
	Tag     string  `json:"tag"`
	Params  []Typus `json:"params"`
	Returns Typus   `json:"returns"`
}

type TypusUnio struct {
	Tag     string  `json:"tag"`
	Members []Typus `json:"members"`
}

type TypusLitteralis struct {
	Tag   string `json:"tag"`
	Valor string `json:"valor"`
}

func (*TypusNomen) typusNode()      {}
func (*TypusNullabilis) typusNode() {}
func (*TypusGenericus) typusNode()  {}
func (*TypusFunctio) typusNode()    {}
func (*TypusUnio) typusNode()       {}
func (*TypusLitteralis) typusNode() {}

// Expressions.
type Expr interface {
	exprNode()
}

type LitteraSpecies string

const (
	LitteraNumerus LitteraSpecies = "Numerus"
	LitteraFractus LitteraSpecies = "Fractus"
	LitteraTextus  LitteraSpecies = "Textus"
	LitteraVerum   LitteraSpecies = "Verum"
	LitteraFalsum  LitteraSpecies = "Falsum"
	LitteraNihil   LitteraSpecies = "Nihil"
)

type ExprNomen struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Valor string `json:"valor"`
}

type ExprEgo struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
}

type ExprLittera struct {
	Tag     string         `json:"tag"`
	Locus   Locus          `json:"locus"`
	Species LitteraSpecies `json:"species"`
	Valor   string         `json:"valor"`
}

type ExprBinaria struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Signum string `json:"signum"`
	Sin    Expr   `json:"sin"`
	Dex    Expr   `json:"dex"`
}

type ExprUnaria struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Signum string `json:"signum"`
	Arg    Expr   `json:"arg"`
}

type ExprAssignatio struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Signum string `json:"signum"`
	Sin    Expr   `json:"sin"`
	Dex    Expr   `json:"dex"`
}

type ExprCondicio struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Cond  Expr   `json:"cond"`
	Cons  Expr   `json:"cons"`
	Alt   Expr   `json:"alt"`
}

type ExprVocatio struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Callee Expr   `json:"callee"`
	Args   []Expr `json:"args"`
}

type ExprMembrum struct {
	Tag      string `json:"tag"`
	Locus    Locus  `json:"locus"`
	Obj      Expr   `json:"obj"`
	Prop     Expr   `json:"prop"`
	Computed bool   `json:"computed"`
	NonNull  bool   `json:"nonNull"`
}

type ExprSeries struct {
	Tag      string `json:"tag"`
	Locus    Locus  `json:"locus"`
	Elementa []Expr `json:"elementa"`
}

type ExprObiectum struct {
	Tag   string         `json:"tag"`
	Locus Locus          `json:"locus"`
	Props []ObiectumProp `json:"props"`
}

type ExprClausura struct {
	Tag    string      `json:"tag"`
	Locus  Locus       `json:"locus"`
	Params []Param     `json:"params"`
	Corpus interface{} `json:"corpus"`
}

type ExprNovum struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Callee Expr   `json:"callee"`
	Args   []Expr `json:"args"`
	Init   Expr   `json:"init"`
}

type ExprCede struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Arg   Expr   `json:"arg"`
}

type ExprQua struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Expr  Expr   `json:"expr"`
	Typus Typus  `json:"typus"`
}

type ExprInnatum struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Expr  Expr   `json:"expr"`
	Typus Typus  `json:"typus"`
}

type ExprPostfixNovum struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Expr  Expr   `json:"expr"`
	Typus Typus  `json:"typus"`
}

type ExprFinge struct {
	Tag     string         `json:"tag"`
	Locus   Locus          `json:"locus"`
	Variant string         `json:"variant"`
	Campi   []ObiectumProp `json:"campi"`
	Typus   Typus          `json:"typus"`
}

type ExprScriptum struct {
	Tag      string `json:"tag"`
	Locus    Locus  `json:"locus"`
	Template string `json:"template"`
	Args     []Expr `json:"args"`
}

type ExprAmbitus struct {
	Tag       string `json:"tag"`
	Locus     Locus  `json:"locus"`
	Start     Expr   `json:"start"`
	End       Expr   `json:"end"`
	Inclusive bool   `json:"inclusive"`
}

func (*ExprNomen) exprNode()        {}
func (*ExprEgo) exprNode()          {}
func (*ExprLittera) exprNode()      {}
func (*ExprBinaria) exprNode()      {}
func (*ExprUnaria) exprNode()       {}
func (*ExprAssignatio) exprNode()   {}
func (*ExprCondicio) exprNode()     {}
func (*ExprVocatio) exprNode()      {}
func (*ExprMembrum) exprNode()      {}
func (*ExprSeries) exprNode()       {}
func (*ExprObiectum) exprNode()     {}
func (*ExprClausura) exprNode()     {}
func (*ExprNovum) exprNode()        {}
func (*ExprCede) exprNode()         {}
func (*ExprQua) exprNode()          {}
func (*ExprInnatum) exprNode()      {}
func (*ExprPostfixNovum) exprNode() {}
func (*ExprFinge) exprNode()        {}
func (*ExprScriptum) exprNode()     {}
func (*ExprAmbitus) exprNode()      {}

// Object literal properties.
type ObiectumProp struct {
	Locus     Locus `json:"locus"`
	Key       Expr  `json:"key"`
	Valor     Expr  `json:"valor"`
	Shorthand bool  `json:"shorthand"`
	Computed  bool  `json:"computed"`
}

// Statements.
type Stmt interface {
	stmtNode()
}

type VariaSpecies string

const (
	VariaVaria    VariaSpecies = "Varia"
	VariaFixum    VariaSpecies = "Fixum"
	VariaFigendum VariaSpecies = "Figendum"
)

type StmtMassa struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Corpus []Stmt `json:"corpus"`
}

type StmtExpressia struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Expr  Expr   `json:"expr"`
}

type StmtVaria struct {
	Tag     string       `json:"tag"`
	Locus   Locus        `json:"locus"`
	Species VariaSpecies `json:"species"`
	Nomen   string       `json:"nomen"`
	Typus   Typus        `json:"typus"`
	Valor   Expr         `json:"valor"`
	Publica bool         `json:"publica"`
	Externa bool         `json:"externa"`
}

type StmtFunctio struct {
	Tag          string   `json:"tag"`
	Locus        Locus    `json:"locus"`
	Nomen        string   `json:"nomen"`
	Params       []Param  `json:"params"`
	TypusReditus Typus    `json:"typusReditus"`
	Corpus       Stmt     `json:"corpus"`
	Asynca       bool     `json:"asynca"`
	Publica      bool     `json:"publica"`
	Generics     []string `json:"generics"`
	Externa      bool     `json:"externa"`
}

type StmtGenus struct {
	Tag      string       `json:"tag"`
	Locus    Locus        `json:"locus"`
	Nomen    string       `json:"nomen"`
	Campi    []CampusDecl `json:"campi"`
	Methodi  []Stmt       `json:"methodi"`
	Implet   []string     `json:"implet"`
	Generics []string     `json:"generics"`
	Publica  bool         `json:"publica"`
}

type StmtPactum struct {
	Tag      string           `json:"tag"`
	Locus    Locus            `json:"locus"`
	Nomen    string           `json:"nomen"`
	Methodi  []PactumMethodus `json:"methodi"`
	Generics []string         `json:"generics"`
	Publica  bool             `json:"publica"`
}

type StmtOrdo struct {
	Tag     string        `json:"tag"`
	Locus   Locus         `json:"locus"`
	Nomen   string        `json:"nomen"`
	Membra  []OrdoMembrum `json:"membra"`
	Publica bool          `json:"publica"`
}

type StmtDiscretio struct {
	Tag       string        `json:"tag"`
	Locus     Locus         `json:"locus"`
	Nomen     string        `json:"nomen"`
	Variantes []VariansDecl `json:"variantes"`
	Generics  []string      `json:"generics"`
	Publica   bool          `json:"publica"`
}

type StmtImporta struct {
	Tag   string       `json:"tag"`
	Locus Locus        `json:"locus"`
	Fons  string       `json:"fons"`
	Specs []ImportSpec `json:"specs"`
	Totum bool         `json:"totum"`
	Alias *string      `json:"alias"`
}

type StmtSi struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Cond  Expr   `json:"cond"`
	Cons  Stmt   `json:"cons"`
	Alt   Stmt   `json:"alt"`
}

type StmtDum struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Cond   Expr   `json:"cond"`
	Corpus Stmt   `json:"corpus"`
}

type StmtFacDum struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Corpus Stmt   `json:"corpus"`
	Cond   Expr   `json:"cond"`
}

type StmtIteratio struct {
	Tag     string `json:"tag"`
	Locus   Locus  `json:"locus"`
	Species string `json:"species"`
	Binding string `json:"binding"`
	Iter    Expr   `json:"iter"`
	Corpus  Stmt   `json:"corpus"`
	Asynca  bool   `json:"asynca"`
}

type StmtElige struct {
	Tag     string       `json:"tag"`
	Locus   Locus        `json:"locus"`
	Discrim Expr         `json:"discrim"`
	Casus   []EligeCasus `json:"casus"`
	Default Stmt         `json:"default_"`
}

type StmtDiscerne struct {
	Tag     string          `json:"tag"`
	Locus   Locus           `json:"locus"`
	Discrim []Expr          `json:"discrim"`
	Casus   []DiscerneCasus `json:"casus"`
}

type StmtCustodi struct {
	Tag       string            `json:"tag"`
	Locus     Locus             `json:"locus"`
	Clausulae []CustodiClausula `json:"clausulae"`
}

type StmtTempta struct {
	Tag    string        `json:"tag"`
	Locus  Locus         `json:"locus"`
	Corpus Stmt          `json:"corpus"`
	Cape   *CapeClausula `json:"cape"`
	Demum  Stmt          `json:"demum"`
}

type StmtRedde struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Valor Expr   `json:"valor"`
}

type StmtIace struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Arg    Expr   `json:"arg"`
	Fatale bool   `json:"fatale"`
}

type StmtScribe struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Gradus string `json:"gradus"`
	Args   []Expr `json:"args"`
}

type StmtAdfirma struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
	Cond  Expr   `json:"cond"`
	Msg   Expr   `json:"msg"`
}

type StmtRumpe struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
}

type StmtPerge struct {
	Tag   string `json:"tag"`
	Locus Locus  `json:"locus"`
}

type StmtIncipit struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Corpus Stmt   `json:"corpus"`
	Asynca bool   `json:"asynca"`
}

type StmtProbandum struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Nomen  string `json:"nomen"`
	Corpus []Stmt `json:"corpus"`
}

type StmtProba struct {
	Tag    string `json:"tag"`
	Locus  Locus  `json:"locus"`
	Nomen  string `json:"nomen"`
	Corpus Stmt   `json:"corpus"`
}

func (*StmtMassa) stmtNode()     {}
func (*StmtExpressia) stmtNode() {}
func (*StmtVaria) stmtNode()     {}
func (*StmtFunctio) stmtNode()   {}
func (*StmtGenus) stmtNode()     {}
func (*StmtPactum) stmtNode()    {}
func (*StmtOrdo) stmtNode()      {}
func (*StmtDiscretio) stmtNode() {}
func (*StmtImporta) stmtNode()   {}
func (*StmtSi) stmtNode()        {}
func (*StmtDum) stmtNode()       {}
func (*StmtFacDum) stmtNode()    {}
func (*StmtIteratio) stmtNode()  {}
func (*StmtElige) stmtNode()     {}
func (*StmtDiscerne) stmtNode()  {}
func (*StmtCustodi) stmtNode()   {}
func (*StmtTempta) stmtNode()    {}
func (*StmtRedde) stmtNode()     {}
func (*StmtIace) stmtNode()      {}
func (*StmtScribe) stmtNode()    {}
func (*StmtAdfirma) stmtNode()   {}
func (*StmtRumpe) stmtNode()     {}
func (*StmtPerge) stmtNode()     {}
func (*StmtIncipit) stmtNode()   {}
func (*StmtProbandum) stmtNode() {}
func (*StmtProba) stmtNode()     {}

// Supporting types.
type Param struct {
	Locus   Locus  `json:"locus"`
	Nomen   string `json:"nomen"`
	Typus   Typus  `json:"typus"`
	Default Expr   `json:"default_"`
	Rest    bool   `json:"rest"`
}

type CampusDecl struct {
	Locus       Locus  `json:"locus"`
	Nomen       string `json:"nomen"`
	Typus       Typus  `json:"typus"`
	Valor       Expr   `json:"valor"`
	Visibilitas string `json:"visibilitas"`
}

type PactumMethodus struct {
	Locus        Locus   `json:"locus"`
	Nomen        string  `json:"nomen"`
	Params       []Param `json:"params"`
	TypusReditus Typus   `json:"typusReditus"`
	Asynca       bool    `json:"asynca"`
}

type OrdoMembrum struct {
	Locus Locus   `json:"locus"`
	Nomen string  `json:"nomen"`
	Valor *string `json:"valor"`
}

type VariansDecl struct {
	Locus Locus           `json:"locus"`
	Nomen string          `json:"nomen"`
	Campi []VariansCampus `json:"campi"`
}

type VariansCampus struct {
	Nomen string `json:"nomen"`
	Typus Typus  `json:"typus"`
}

type ImportSpec struct {
	Locus    Locus  `json:"locus"`
	Imported string `json:"imported"`
	Local    string `json:"local"`
}

type EligeCasus struct {
	Locus  Locus `json:"locus"`
	Cond   Expr  `json:"cond"`
	Corpus Stmt  `json:"corpus"`
}

type DiscerneCasus struct {
	Locus    Locus            `json:"locus"`
	Patterns []VariansPattern `json:"patterns"`
	Corpus   Stmt             `json:"corpus"`
}

type VariansPattern struct {
	Locus    Locus    `json:"locus"`
	Variant  string   `json:"variant"`
	Bindings []string `json:"bindings"`
	Alias    *string  `json:"alias"`
	Wildcard bool     `json:"wildcard"`
}

type CustodiClausula struct {
	Locus  Locus `json:"locus"`
	Cond   Expr  `json:"cond"`
	Corpus Stmt  `json:"corpus"`
}

type CapeClausula struct {
	Locus  Locus  `json:"locus"`
	Param  string `json:"param"`
	Corpus Stmt   `json:"corpus"`
}

// Top-level compilation unit.
type Modulus struct {
	Locus  Locus  `json:"locus"`
	Corpus []Stmt `json:"corpus"`
}
