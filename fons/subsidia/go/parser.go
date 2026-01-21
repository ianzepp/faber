package subsidia

import (
	"strconv"
	"strings"
)

// Operator precedence for Pratt parser.
var Precedence = map[string]int{
	"=":       1,
	"+=":      1,
	"-=":      1,
	"*=":      1,
	"/=":      1,
	"vel":     2,
	"??":      2,
	"aut":     3,
	"||":      3,
	"et":      4,
	"&&":      4,
	"==":      5,
	"!=":      5,
	"===":     5,
	"!==":     5,
	"<":       6,
	">":       6,
	"<=":      6,
	">=":      6,
	"inter":   6,
	"intra":   6,
	"+":       7,
	"-":       7,
	"*":       8,
	"/":       8,
	"%":       8,
	"qua":     9,
	"innatum": 9,
	"novum":   9,
}

var UnaryOps = map[string]struct{}{
	"-": {}, "!": {}, "non": {}, "nihil": {}, "nonnihil": {}, "positivum": {},
}

var AssignOps = map[string]struct{}{
	"=": {}, "+=": {}, "-=": {}, "*=": {}, "/=": {},
}

// Parser for Faber source.
type Parser struct {
	tokens   []Token
	pos      int
	filename string
}

func NewParser(tokens []Token, filename string) *Parser {
	return &Parser{tokens: tokens, filename: filename}
}

func (p *Parser) peek(offset int) Token {
	idx := p.pos + offset
	if idx >= len(p.tokens) {
		return p.tokens[len(p.tokens)-1]
	}
	return p.tokens[idx]
}

func (p *Parser) advance() Token {
	tok := p.tokens[p.pos]
	p.pos++
	return tok
}

func (p *Parser) check(tag string, valor ...string) bool {
	tok := p.peek(0)
	if tok.Tag != tag {
		return false
	}
	if len(valor) > 0 && tok.Valor != valor[0] {
		return false
	}
	return true
}

func (p *Parser) match(tag string, valor ...string) *Token {
	if p.check(tag, valor...) {
		tok := p.advance()
		return &tok
	}
	return nil
}

func (p *Parser) expect(tag string, valor ...string) Token {
	tok := p.match(tag, valor...)
	if tok == nil {
		got := p.peek(0)
		msg := tag
		if len(valor) > 0 {
			msg = valor[0]
		}
		panic(p.error("expected " + msg + ", got '" + got.Valor + "'"))
	}
	return *tok
}

func (p *Parser) error(msg string) *CompileError {
	return &CompileError{Message: msg, Locus: p.peek(0).Locus, Filename: p.filename}
}

// Accept identifier OR keyword as a name.
func (p *Parser) expectName() Token {
	tok := p.peek(0)
	if tok.Tag == TokenIdentifier || tok.Tag == TokenKeyword {
		return p.advance()
	}
	panic(p.error("expected identifier, got '" + tok.Valor + "'"))
}

func (p *Parser) checkName() bool {
	tok := p.peek(0)
	return tok.Tag == TokenIdentifier || tok.Tag == TokenKeyword
}

// Parse entry point.
func (p *Parser) Parse() *Modulus {
	corpus := []Stmt{}
	for !p.check(TokenEOF) {
		corpus = append(corpus, p.parseStmt())
	}
	return &Modulus{Locus: Locus{Linea: 1, Columna: 1, Index: 0}, Corpus: corpus}
}

func (p *Parser) parseStmt() Stmt {
	publica := false
	futura := false
	externa := false

	for p.match(TokenPunctuator, "@") != nil {
		tok := p.peek(0)
		if tok.Tag != TokenIdentifier && tok.Tag != TokenKeyword {
			panic(p.error("expected annotation name"))
		}
		anno := p.advance().Valor
		switch anno {
		case "publicum", "publica":
			publica = true
		case "futura":
			futura = true
		case "externa":
			externa = true
		default:
			for !p.check(TokenEOF) && !p.check(TokenPunctuator, "@") && !p.check(TokenPunctuator, "§") && !p.isDeclarationKeyword() {
				p.advance()
			}
		}
	}

	if p.match(TokenPunctuator, "§") != nil {
		return p.parseImport()
	}

	tok := p.peek(0)
	if tok.Tag == TokenKeyword {
		switch tok.Valor {
		case "varia", "fixum", "figendum":
			return p.parseVaria(publica, externa)
		case "ex":
			return p.parseExStmt(publica)
		case "functio":
			return p.parseFunctio(publica, futura, externa)
		case "genus":
			return p.parseGenus(publica)
		case "pactum":
			return p.parsePactum(publica)
		case "ordo":
			return p.parseOrdo(publica)
		case "discretio":
			return p.parseDiscretio(publica)
		case "si":
			return p.parseSi()
		case "dum":
			return p.parseDum()
		case "fac":
			return p.parseFac()
		case "elige":
			return p.parseElige()
		case "discerne":
			return p.parseDiscerne()
		case "custodi":
			return p.parseCustodi()
		case "tempta":
			return p.parseTempta()
		case "redde":
			return p.parseRedde()
		case "iace", "mori":
			return p.parseIace()
		case "scribe", "vide", "mone":
			return p.parseScribe()
		case "adfirma":
			return p.parseAdfirma()
		case "rumpe":
			return p.parseRumpe()
		case "perge":
			return p.parsePerge()
		case "incipit", "incipiet":
			return p.parseIncipit()
		case "probandum":
			return p.parseProbandum()
		case "proba":
			return p.parseProba()
		}
	}

	if p.check(TokenPunctuator, "{") {
		return p.parseMassa()
	}

	return p.parseExpressiaStmt()
}

func (p *Parser) parseImport() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "ex")
	fons := p.expect(TokenTextus).Valor
	p.expect(TokenKeyword, "importa")

	specs := []ImportSpec{}
	for {
		loc := p.peek(0).Locus
		imported := p.expect(TokenIdentifier).Valor
		local := imported
		if p.match(TokenKeyword, "ut") != nil {
			local = p.expect(TokenIdentifier).Valor
		}
		specs = append(specs, ImportSpec{Locus: loc, Imported: imported, Local: local})
		if p.match(TokenPunctuator, ",") == nil {
			break
		}
	}

	return &StmtImporta{Tag: "Importa", Locus: locus, Fons: fons, Specs: specs, Totum: false, Alias: nil}
}

func (p *Parser) parseVaria(publica bool, externa bool) Stmt {
	locus := p.peek(0).Locus
	kw := p.advance().Valor
	species := VariaVaria
	if kw == "figendum" {
		species = VariaFigendum
	} else if kw == "fixum" {
		species = VariaFixum
	}

	var typus Typus
	var nomen string

	first := p.expectName().Valor

	if p.check(TokenOperator, "<") {
		args := []Typus{}
		p.advance()
		for {
			args = append(args, p.parseTypus())
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
		typus = &TypusGenericus{Tag: "Genericus", Nomen: first, Args: args}

		if p.match(TokenPunctuator, "?") != nil {
			typus = &TypusNullabilis{Tag: "Nullabilis", Inner: typus}
		}

		nomen = p.expectName().Valor
	} else if p.match(TokenPunctuator, "?") != nil {
		typus = &TypusNullabilis{Tag: "Nullabilis", Inner: &TypusNomen{Tag: "Nomen", Nomen: first}}
		nomen = p.expectName().Valor
	} else if p.checkName() {
		typus = &TypusNomen{Tag: "Nomen", Nomen: first}
		nomen = p.expectName().Valor
	} else {
		nomen = first
	}

	var valor Expr
	if p.match(TokenOperator, "=") != nil {
		valor = p.parseExpr(0)
	}

	return &StmtVaria{Tag: "Varia", Locus: locus, Species: species, Nomen: nomen, Typus: typus, Valor: valor, Publica: publica, Externa: externa}
}

func (p *Parser) parseExStmt(_ bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "ex")

	expr := p.parseExpr(0)

	if p.check(TokenKeyword, "fixum") || p.check(TokenKeyword, "varia") {
		species := "Ex"
		asynca := false
		p.advance()
		binding := p.expect(TokenIdentifier).Valor
		corpus := p.parseMassa()
		return &StmtIteratio{Tag: "Iteratio", Locus: locus, Species: species, Binding: binding, Iter: expr, Corpus: corpus, Asynca: asynca}
	}

	panic(p.error("destructuring not supported in nanus"))
}

func (p *Parser) parseFunctio(publica bool, futura bool, externa bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "functio")
	asynca := futura

	nomen := p.expectName().Valor

	generics := []string{}
	if p.match(TokenOperator, "<") != nil {
		for {
			generics = append(generics, p.expect(TokenIdentifier).Valor)
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
	}

	p.expect(TokenPunctuator, "(")
	params := p.parseParams()
	p.expect(TokenPunctuator, ")")

	var typusReditus Typus
	if p.match(TokenOperator, "->") != nil {
		typusReditus = p.parseTypus()
	}

	var corpus Stmt
	if p.check(TokenPunctuator, "{") {
		corpus = p.parseMassa()
	}

	return &StmtFunctio{Tag: "Functio", Locus: locus, Nomen: nomen, Params: params, TypusReditus: typusReditus, Corpus: corpus, Asynca: asynca, Publica: publica, Generics: generics, Externa: externa}
}

func (p *Parser) parseParams() []Param {
	params := []Param{}
	if p.check(TokenPunctuator, ")") {
		return params
	}

	for {
		locus := p.peek(0).Locus
		rest := false
		if p.match(TokenKeyword, "ceteri") != nil {
			rest = true
		}

		optional := false
		if p.match(TokenKeyword, "si") != nil {
			optional = true
		}

		// Check for ownership preposition: ex/de/in
		var ownership string
		if p.match(TokenKeyword, "ex") != nil {
			ownership = "ex"
		} else if p.match(TokenKeyword, "de") != nil {
			ownership = "de"
		} else if p.match(TokenKeyword, "in") != nil {
			ownership = "in"
		}

		var typus Typus
		var nomen string

		if p.checkName() {
			first := p.expectName().Valor

			if p.match(TokenOperator, "<") != nil {
				args := []Typus{}
				for {
					args = append(args, p.parseTypus())
					if p.match(TokenPunctuator, ",") == nil {
						break
					}
				}
				p.expect(TokenOperator, ">")
				typus = &TypusGenericus{Tag: "Genericus", Nomen: first, Args: args}

				if p.match(TokenPunctuator, "?") != nil {
					typus = &TypusNullabilis{Tag: "Nullabilis", Inner: typus}
				}

				nomen = p.expectName().Valor
			} else if p.match(TokenPunctuator, "?") != nil {
				typus = &TypusNullabilis{Tag: "Nullabilis", Inner: &TypusNomen{Tag: "Nomen", Nomen: first}}
				nomen = p.expectName().Valor
			} else if p.checkName() {
				typus = &TypusNomen{Tag: "Nomen", Nomen: first}
				nomen = p.expectName().Valor
			} else {
				nomen = first
			}
		} else {
			panic(p.error("expected parameter name"))
		}

		if optional && typus != nil {
			if _, ok := typus.(*TypusNullabilis); !ok {
				typus = &TypusNullabilis{Tag: "Nullabilis", Inner: typus}
			}
		}

		var def Expr
		if p.match(TokenOperator, "=") != nil {
			def = p.parseExpr(0)
		}

		params = append(params, Param{Locus: locus, Nomen: nomen, Typus: typus, Default: def, Rest: rest, Optional: optional, Ownership: ownership})

		if p.match(TokenPunctuator, ",") == nil {
			break
		}
	}

	return params
}

func (p *Parser) parseGenus(publica bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "genus")
	nomen := p.expect(TokenIdentifier).Valor

	generics := []string{}
	if p.match(TokenOperator, "<") != nil {
		for {
			generics = append(generics, p.expect(TokenIdentifier).Valor)
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
	}

	implet := []string{}
	if p.match(TokenKeyword, "implet") != nil {
		for {
			implet = append(implet, p.expect(TokenIdentifier).Valor)
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
	}

	p.expect(TokenPunctuator, "{")

	campi := []CampusDecl{}
	methodi := []Stmt{}

	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		for p.match(TokenPunctuator, "@") != nil {
			tok := p.peek(0)
			if tok.Tag != TokenIdentifier && tok.Tag != TokenKeyword {
				panic(p.error("expected annotation name"))
			}
			p.advance()
		}

		visibilitas := "Publica"
		if p.match(TokenKeyword, "privata") != nil || p.match(TokenKeyword, "privatus") != nil {
			visibilitas = "Privata"
		} else if p.match(TokenKeyword, "protecta") != nil || p.match(TokenKeyword, "protectus") != nil {
			visibilitas = "Protecta"
		}

		if p.check(TokenKeyword, "functio") {
			methodi = append(methodi, p.parseFunctio(false, false, false))
		} else {
			loc := p.peek(0).Locus
			first := p.expectName().Valor
			var fieldTypus Typus
			var fieldNomen string

			if p.match(TokenOperator, "<") != nil {
				args := []Typus{}
				for {
					args = append(args, p.parseTypus())
					if p.match(TokenPunctuator, ",") == nil {
						break
					}
				}
				p.expect(TokenOperator, ">")
				fieldTypus = &TypusGenericus{Tag: "Genericus", Nomen: first, Args: args}

				if p.match(TokenPunctuator, "?") != nil {
					fieldTypus = &TypusNullabilis{Tag: "Nullabilis", Inner: fieldTypus}
				}

				fieldNomen = p.expectName().Valor
			} else {
				nullable := false
				if p.match(TokenPunctuator, "?") != nil {
					nullable = true
				}

				if p.checkName() {
					fieldTypus = &TypusNomen{Tag: "Nomen", Nomen: first}
					if nullable {
						fieldTypus = &TypusNullabilis{Tag: "Nullabilis", Inner: fieldTypus}
					}
					fieldNomen = p.expectName().Valor
				} else {
					panic(p.error("expected field type or name"))
				}
			}

			var valor Expr
			if p.match(TokenOperator, "=") != nil {
				valor = p.parseExpr(0)
			}

			campi = append(campi, CampusDecl{Locus: loc, Nomen: fieldNomen, Typus: fieldTypus, Valor: valor, Visibilitas: visibilitas})
		}
	}

	p.expect(TokenPunctuator, "}")
	return &StmtGenus{Tag: "Genus", Locus: locus, Nomen: nomen, Campi: campi, Methodi: methodi, Implet: implet, Generics: generics, Publica: publica}
}

func (p *Parser) parsePactum(publica bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "pactum")
	nomen := p.expect(TokenIdentifier).Valor

	generics := []string{}
	if p.match(TokenOperator, "<") != nil {
		for {
			generics = append(generics, p.expect(TokenIdentifier).Valor)
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
	}

	p.expect(TokenPunctuator, "{")

	methodi := []PactumMethodus{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		loc := p.peek(0).Locus
		p.expect(TokenKeyword, "functio")
		asynca := false
		if p.match(TokenKeyword, "asynca") != nil {
			asynca = true
		}
		name := p.expect(TokenIdentifier).Valor
		p.expect(TokenPunctuator, "(")
		params := p.parseParams()
		p.expect(TokenPunctuator, ")")
		var typusReditus Typus
		if p.match(TokenOperator, "->") != nil {
			typusReditus = p.parseTypus()
		}
		methodi = append(methodi, PactumMethodus{Locus: loc, Nomen: name, Params: params, TypusReditus: typusReditus, Asynca: asynca})
	}

	p.expect(TokenPunctuator, "}")
	return &StmtPactum{Tag: "Pactum", Locus: locus, Nomen: nomen, Methodi: methodi, Generics: generics, Publica: publica}
}

func (p *Parser) parseOrdo(publica bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "ordo")
	nomen := p.expect(TokenIdentifier).Valor
	p.expect(TokenPunctuator, "{")

	membra := []OrdoMembrum{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		loc := p.peek(0).Locus
		name := p.expect(TokenIdentifier).Valor
		var valor *string
		if p.match(TokenOperator, "=") != nil {
			tok := p.peek(0)
			if tok.Tag == TokenTextus {
				v := strconv.Quote(tok.Valor)
				valor = &v
			} else {
				v := tok.Valor
				valor = &v
			}
			p.advance()
		}
		membra = append(membra, OrdoMembrum{Locus: loc, Nomen: name, Valor: valor})
		p.match(TokenPunctuator, ",")
	}

	p.expect(TokenPunctuator, "}")
	return &StmtOrdo{Tag: "Ordo", Locus: locus, Nomen: nomen, Membra: membra, Publica: publica}
}

func (p *Parser) parseDiscretio(publica bool) Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "discretio")
	nomen := p.expect(TokenIdentifier).Valor

	generics := []string{}
	if p.match(TokenOperator, "<") != nil {
		for {
			generics = append(generics, p.expect(TokenIdentifier).Valor)
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
	}

	p.expect(TokenPunctuator, "{")

	variantes := []VariansDecl{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		loc := p.peek(0).Locus
		name := p.expect(TokenIdentifier).Valor
		campi := []VariansCampus{}

		if p.match(TokenPunctuator, "{") != nil {
			for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
				typNomen := p.expectName().Valor
				var fieldTypus Typus

				if p.match(TokenOperator, "<") != nil {
					args := []Typus{}
					for {
						args = append(args, p.parseTypus())
						if p.match(TokenPunctuator, ",") == nil {
							break
						}
					}
					p.expect(TokenOperator, ">")
					fieldTypus = &TypusGenericus{Tag: "Genericus", Nomen: typNomen, Args: args}
				} else {
					fieldTypus = &TypusNomen{Tag: "Nomen", Nomen: typNomen}
				}

				if p.match(TokenPunctuator, "?") != nil {
					fieldTypus = &TypusNullabilis{Tag: "Nullabilis", Inner: fieldTypus}
				}

				fieldNomen := p.expectName().Valor
				campi = append(campi, VariansCampus{Nomen: fieldNomen, Typus: fieldTypus})
			}
			p.expect(TokenPunctuator, "}")
		}

		variantes = append(variantes, VariansDecl{Locus: loc, Nomen: name, Campi: campi})
	}

	p.expect(TokenPunctuator, "}")
	return &StmtDiscretio{Tag: "Discretio", Locus: locus, Nomen: nomen, Variantes: variantes, Generics: generics, Publica: publica}
}

func (p *Parser) parseMassa() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenPunctuator, "{")
	corpus := []Stmt{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		corpus = append(corpus, p.parseStmt())
	}
	p.expect(TokenPunctuator, "}")
	return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: corpus}
}

func (p *Parser) parseBody() Stmt {
	locus := p.peek(0).Locus

	if p.check(TokenPunctuator, "{") {
		return p.parseMassa()
	}

	if p.match(TokenKeyword, "ergo") != nil {
		stmt := p.parseStmt()
		return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: []Stmt{stmt}}
	}

	if p.match(TokenKeyword, "reddit") != nil {
		valor := p.parseExpr(0)
		return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: []Stmt{&StmtRedde{Tag: "Redde", Locus: locus, Valor: valor}}}
	}

	if p.match(TokenKeyword, "iacit") != nil {
		arg := p.parseExpr(0)
		return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: []Stmt{&StmtIace{Tag: "Iace", Locus: locus, Arg: arg, Fatale: false}}}
	}

	if p.match(TokenKeyword, "moritor") != nil {
		arg := p.parseExpr(0)
		return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: []Stmt{&StmtIace{Tag: "Iace", Locus: locus, Arg: arg, Fatale: true}}}
	}

	if p.match(TokenKeyword, "tacet") != nil {
		return &StmtMassa{Tag: "Massa", Locus: locus, Corpus: []Stmt{}}
	}

	return p.parseMassa()
}

func (p *Parser) parseSi() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "si")
	return p.parseSiBody(locus)
}

func (p *Parser) parseSiBody(locus Locus) Stmt {
	cond := p.parseExpr(0)
	cons := p.parseBody()
	var alt Stmt
	if p.match(TokenKeyword, "sin") != nil {
		sinLocus := p.peek(0).Locus
		alt = p.parseSiBody(sinLocus)
	} else if p.match(TokenKeyword, "secus") != nil {
		if p.check(TokenKeyword, "si") {
			alt = p.parseSi()
		} else {
			alt = p.parseBody()
		}
	}
	return &StmtSi{Tag: "Si", Locus: locus, Cond: cond, Cons: cons, Alt: alt}
}

func (p *Parser) parseDum() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "dum")
	cond := p.parseExpr(0)
	corpus := p.parseBody()
	return &StmtDum{Tag: "Dum", Locus: locus, Cond: cond, Corpus: corpus}
}

func (p *Parser) parseFac() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "fac")
	corpus := p.parseMassa()
	p.expect(TokenKeyword, "dum")
	cond := p.parseExpr(0)
	return &StmtFacDum{Tag: "FacDum", Locus: locus, Corpus: corpus, Cond: cond}
}

func (p *Parser) parseElige() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "elige")
	discrim := p.parseExpr(0)
	p.expect(TokenPunctuator, "{")

	casus := []EligeCasus{}
	var def Stmt

	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		if p.match(TokenKeyword, "ceterum") != nil {
			if p.check(TokenPunctuator, "{") {
				def = p.parseMassa()
			} else if p.match(TokenKeyword, "reddit") != nil {
				redLoc := p.peek(0).Locus
				valor := p.parseExpr(0)
				def = &StmtMassa{Tag: "Massa", Locus: redLoc, Corpus: []Stmt{&StmtRedde{Tag: "Redde", Locus: redLoc, Valor: valor}}}
			} else {
				panic(p.error("expected { or reddit after ceterum"))
			}
		} else {
			p.expect(TokenKeyword, "casu")
			loc := p.peek(0).Locus
			cond := p.parseExpr(0)
			var corpus Stmt
			if p.check(TokenPunctuator, "{") {
				corpus = p.parseMassa()
			} else if p.match(TokenKeyword, "reddit") != nil {
				redLoc := p.peek(0).Locus
				valor := p.parseExpr(0)
				corpus = &StmtMassa{Tag: "Massa", Locus: redLoc, Corpus: []Stmt{&StmtRedde{Tag: "Redde", Locus: redLoc, Valor: valor}}}
			} else {
				panic(p.error("expected { or reddit after casu condition"))
			}
			casus = append(casus, EligeCasus{Locus: loc, Cond: cond, Corpus: corpus})
		}
	}

	p.expect(TokenPunctuator, "}")
	return &StmtElige{Tag: "Elige", Locus: locus, Discrim: discrim, Casus: casus, Default: def}
}

func (p *Parser) parseDiscerne() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "discerne")
	discrim := []Expr{p.parseExpr(0)}
	for p.match(TokenPunctuator, ",") != nil {
		discrim = append(discrim, p.parseExpr(0))
	}
	p.expect(TokenPunctuator, "{")

	casus := []DiscerneCasus{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		loc := p.peek(0).Locus

		if p.match(TokenKeyword, "ceterum") != nil {
			patterns := []VariansPattern{{Locus: loc, Variant: "_", Bindings: []string{}, Alias: nil, Wildcard: true}}
			corpus := p.parseMassa()
			casus = append(casus, DiscerneCasus{Locus: loc, Patterns: patterns, Corpus: corpus})
			continue
		}

		p.expect(TokenKeyword, "casu")
		patterns := []VariansPattern{}

		for {
			pLoc := p.peek(0).Locus
			variant := p.expect(TokenIdentifier).Valor
			var alias *string
			bindings := []string{}
			wildcard := variant == "_"

			if p.match(TokenKeyword, "ut") != nil {
				name := p.expectName().Valor
				alias = &name
			} else if p.match(TokenKeyword, "pro") != nil || p.match(TokenKeyword, "fixum") != nil {
				for {
					bindings = append(bindings, p.expectName().Valor)
					if p.match(TokenPunctuator, ",") == nil {
						break
					}
				}
			}

			patterns = append(patterns, VariansPattern{Locus: pLoc, Variant: variant, Bindings: bindings, Alias: alias, Wildcard: wildcard})

			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}

		corpus := p.parseMassa()
		casus = append(casus, DiscerneCasus{Locus: loc, Patterns: patterns, Corpus: corpus})
	}

	p.expect(TokenPunctuator, "}")
	return &StmtDiscerne{Tag: "Discerne", Locus: locus, Discrim: discrim, Casus: casus}
}

func (p *Parser) parseCustodi() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "custodi")
	p.expect(TokenPunctuator, "{")

	clausulae := []CustodiClausula{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		loc := p.peek(0).Locus
		p.expect(TokenKeyword, "si")
		cond := p.parseExpr(0)
		corpus := p.parseMassa()
		clausulae = append(clausulae, CustodiClausula{Locus: loc, Cond: cond, Corpus: corpus})
	}

	p.expect(TokenPunctuator, "}")
	return &StmtCustodi{Tag: "Custodi", Locus: locus, Clausulae: clausulae}
}

func (p *Parser) parseTempta() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "tempta")
	corpus := p.parseMassa()

	var cape *CapeClausula
	if p.match(TokenKeyword, "cape") != nil {
		loc := p.peek(0).Locus
		param := p.expect(TokenIdentifier).Valor
		body := p.parseMassa()
		cape = &CapeClausula{Locus: loc, Param: param, Corpus: body}
	}

	var demum Stmt
	if p.match(TokenKeyword, "demum") != nil {
		demum = p.parseMassa()
	}

	return &StmtTempta{Tag: "Tempta", Locus: locus, Corpus: corpus, Cape: cape, Demum: demum}
}

func (p *Parser) parseRedde() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "redde")
	var valor Expr
	if !p.check(TokenEOF) && !p.check(TokenPunctuator, "}") && !p.isStatementKeyword() {
		valor = p.parseExpr(0)
	}
	return &StmtRedde{Tag: "Redde", Locus: locus, Valor: valor}
}

func (p *Parser) isStatementKeyword() bool {
	if !p.check(TokenKeyword) {
		return false
	}
	kw := p.peek(0).Valor
	stmtKeywords := map[string]struct{}{
		"si": {}, "sin": {}, "secus": {}, "dum": {}, "fac": {}, "ex": {}, "de": {}, "elige": {}, "discerne": {}, "custodi": {},
		"tempta": {}, "cape": {}, "demum": {}, "redde": {}, "rumpe": {}, "perge": {}, "iace": {}, "mori": {},
		"scribe": {}, "vide": {}, "mone": {}, "adfirma": {}, "functio": {}, "genus": {}, "pactum": {}, "ordo": {},
		"discretio": {}, "varia": {}, "fixum": {}, "figendum": {}, "incipit": {}, "probandum": {}, "proba": {},
		"casu": {}, "ceterum": {}, "reddit": {}, "ergo": {}, "tacet": {}, "iacit": {}, "moritor": {},
	}
	_, ok := stmtKeywords[kw]
	return ok
}

func (p *Parser) isDeclarationKeyword() bool {
	if !p.check(TokenKeyword) {
		return false
	}
	kw := p.peek(0).Valor
	declKeywords := map[string]struct{}{
		"functio": {}, "genus": {}, "pactum": {}, "ordo": {}, "discretio": {},
		"varia": {}, "fixum": {}, "figendum": {}, "incipit": {}, "probandum": {},
	}
	_, ok := declKeywords[kw]
	return ok
}

func (p *Parser) parseIace() Stmt {
	locus := p.peek(0).Locus
	fatale := p.advance().Valor == "mori"
	arg := p.parseExpr(0)
	return &StmtIace{Tag: "Iace", Locus: locus, Arg: arg, Fatale: fatale}
}

func (p *Parser) parseScribe() Stmt {
	locus := p.peek(0).Locus
	kw := p.advance().Valor
	gradus := "Scribe"
	if kw == "vide" {
		gradus = "Vide"
	} else if kw == "mone" {
		gradus = "Mone"
	}
	args := []Expr{}
	if !p.check(TokenEOF) && !p.check(TokenPunctuator, "}") && !p.isStatementKeyword() {
		for {
			args = append(args, p.parseExpr(0))
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
	}
	return &StmtScribe{Tag: "Scribe", Locus: locus, Gradus: gradus, Args: args}
}

func (p *Parser) parseAdfirma() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "adfirma")
	cond := p.parseExpr(0)
	var msg Expr
	if p.match(TokenPunctuator, ",") != nil {
		msg = p.parseExpr(0)
	}
	return &StmtAdfirma{Tag: "Adfirma", Locus: locus, Cond: cond, Msg: msg}
}

func (p *Parser) parseRumpe() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "rumpe")
	return &StmtRumpe{Tag: "Rumpe", Locus: locus}
}

func (p *Parser) parsePerge() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "perge")
	return &StmtPerge{Tag: "Perge", Locus: locus}
}

func (p *Parser) parseIncipit() Stmt {
	locus := p.peek(0).Locus
	kw := p.advance().Valor
	asynca := kw == "incipiet"
	corpus := p.parseMassa()
	return &StmtIncipit{Tag: "Incipit", Locus: locus, Corpus: corpus, Asynca: asynca}
}

func (p *Parser) parseProbandum() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "probandum")
	nomen := p.expect(TokenTextus).Valor
	p.expect(TokenPunctuator, "{")

	corpus := []Stmt{}
	for !p.check(TokenPunctuator, "}") && !p.check(TokenEOF) {
		corpus = append(corpus, p.parseStmt())
	}

	p.expect(TokenPunctuator, "}")
	return &StmtProbandum{Tag: "Probandum", Locus: locus, Nomen: nomen, Corpus: corpus}
}

func (p *Parser) parseProba() Stmt {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "proba")
	nomen := p.expect(TokenTextus).Valor
	corpus := p.parseMassa()
	return &StmtProba{Tag: "Proba", Locus: locus, Nomen: nomen, Corpus: corpus}
}

func (p *Parser) parseExpressiaStmt() Stmt {
	locus := p.peek(0).Locus
	expr := p.parseExpr(0)
	return &StmtExpressia{Tag: "Expressia", Locus: locus, Expr: expr}
}

func (p *Parser) parseTypus() Typus {
	typus := p.parseTypusPrimary()

	if p.match(TokenPunctuator, "?") != nil {
		typus = &TypusNullabilis{Tag: "Nullabilis", Inner: typus}
	}

	if p.match(TokenOperator, "|") != nil {
		members := []Typus{typus}
		for {
			members = append(members, p.parseTypusPrimary())
			if p.match(TokenOperator, "|") == nil {
				break
			}
		}
		typus = &TypusUnio{Tag: "Unio", Members: members}
	}

	return typus
}

func (p *Parser) parseTypusPrimary() Typus {
	nomen := p.expect(TokenIdentifier).Valor

	if p.match(TokenOperator, "<") != nil {
		args := []Typus{}
		for {
			args = append(args, p.parseTypus())
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
		p.expect(TokenOperator, ">")
		return &TypusGenericus{Tag: "Genericus", Nomen: nomen, Args: args}
	}

	return &TypusNomen{Tag: "Nomen", Nomen: nomen}
}

func (p *Parser) parseExpr(minPrec int) Expr {
	left := p.parseUnary()

	for {
		tok := p.peek(0)
		op := tok.Valor
		prec, ok := Precedence[op]
		if !ok || prec < minPrec {
			break
		}

		p.advance()

		if op == "qua" {
			typus := p.parseTypus()
			left = &ExprQua{Tag: "Qua", Locus: tok.Locus, Expr: left, Typus: typus}
			continue
		}
		if op == "innatum" {
			typus := p.parseTypus()
			left = &ExprInnatum{Tag: "Innatum", Locus: tok.Locus, Expr: left, Typus: typus}
			continue
		}
		if op == "novum" {
			typus := p.parseTypus()
			left = &ExprPostfixNovum{Tag: "PostfixNovum", Locus: tok.Locus, Expr: left, Typus: typus}
			continue
		}

		right := p.parseExpr(prec + 1)

		if _, ok := AssignOps[op]; ok {
			left = &ExprAssignatio{Tag: "Assignatio", Locus: tok.Locus, Signum: op, Sin: left, Dex: right}
		} else {
			left = &ExprBinaria{Tag: "Binaria", Locus: tok.Locus, Signum: op, Sin: left, Dex: right}
		}
	}

	if p.match(TokenKeyword, "sic") != nil {
		cons := p.parseExpr(0)
		p.expect(TokenKeyword, "secus")
		alt := p.parseExpr(0)
		left = &ExprCondicio{Tag: "Condicio", Locus: ExprLocus(left), Cond: left, Cons: cons, Alt: alt}
	}

	return left
}

func (p *Parser) parseUnary() Expr {
	tok := p.peek(0)

	if tok.Tag == TokenOperator || tok.Tag == TokenKeyword {
		if _, ok := UnaryOps[tok.Valor]; ok {
			nonExpr := map[string]struct{}{
				"qua": {}, "innatum": {}, "et": {}, "aut": {}, "vel": {}, "sic": {}, "secus": {}, "inter": {}, "intra": {},
				"perge": {}, "rumpe": {}, "redde": {}, "reddit": {}, "iace": {}, "mori": {},
				"si": {}, "secussi": {}, "dum": {}, "ex": {}, "de": {}, "elige": {}, "discerne": {}, "custodi": {}, "tempta": {},
				"functio": {}, "genus": {}, "pactum": {}, "ordo": {}, "discretio": {},
				"casu": {}, "ceterum": {}, "importa": {}, "incipit": {}, "incipiet": {}, "probandum": {}, "proba": {},
			}
			next := p.peek(1)
			canBeUnary := next.Tag == TokenIdentifier || (next.Tag == TokenKeyword && !containsKey(nonExpr, next.Valor)) ||
				next.Tag == TokenNumerus || next.Tag == TokenTextus || next.Valor == "(" || next.Valor == "[" || next.Valor == "{" ||
				containsKey(UnaryOps, next.Valor)

			if canBeUnary {
				p.advance()
				arg := p.parseUnary()
				return &ExprUnaria{Tag: "Unaria", Locus: tok.Locus, Signum: tok.Valor, Arg: arg}
			}
		}
	}

	if p.match(TokenKeyword, "cede") != nil {
		arg := p.parseUnary()
		return &ExprCede{Tag: "Cede", Locus: tok.Locus, Arg: arg}
	}

	return p.parsePostfix()
}

func (p *Parser) parsePostfix() Expr {
	expr := p.parsePrimary()

	for {
		tok := p.peek(0)

		if p.match(TokenPunctuator, "(") != nil {
			args := p.parseArgs()
			p.expect(TokenPunctuator, ")")
			expr = &ExprVocatio{Tag: "Vocatio", Locus: tok.Locus, Callee: expr, Args: args}
			continue
		}

		if p.match(TokenPunctuator, ".") != nil {
			prop := &ExprLittera{Tag: "Littera", Locus: p.peek(0).Locus, Species: LitteraTextus, Valor: p.expectName().Valor}
			expr = &ExprMembrum{Tag: "Membrum", Locus: tok.Locus, Obj: expr, Prop: prop, Computed: false, NonNull: false}
			continue
		}

		if p.match(TokenOperator, "!.") != nil || (tok.Valor == "!" && p.peek(1).Valor == ".") {
			if tok.Valor == "!" {
				p.advance()
				p.advance()
			}
			prop := &ExprLittera{Tag: "Littera", Locus: p.peek(0).Locus, Species: LitteraTextus, Valor: p.expectName().Valor}
			expr = &ExprMembrum{Tag: "Membrum", Locus: tok.Locus, Obj: expr, Prop: prop, Computed: false, NonNull: true}
			continue
		}

		if tok.Valor == "!" && p.peek(1).Valor == "[" {
			p.advance()
			p.advance()
			prop := p.parseExpr(0)
			p.expect(TokenPunctuator, "]")
			expr = &ExprMembrum{Tag: "Membrum", Locus: tok.Locus, Obj: expr, Prop: prop, Computed: true, NonNull: true}
			continue
		}

		if p.match(TokenPunctuator, "[") != nil {
			prop := p.parseExpr(0)
			p.expect(TokenPunctuator, "]")
			expr = &ExprMembrum{Tag: "Membrum", Locus: tok.Locus, Obj: expr, Prop: prop, Computed: true, NonNull: false}
			continue
		}

		break
	}

	return expr
}

func (p *Parser) parsePrimary() Expr {
	tok := p.peek(0)

	if p.match(TokenPunctuator, "(") != nil {
		expr := p.parseExpr(0)
		p.expect(TokenPunctuator, ")")
		return expr
	}

	if p.match(TokenPunctuator, "[") != nil {
		elementa := []Expr{}
		if !p.check(TokenPunctuator, "]") {
			for {
				elementa = append(elementa, p.parseExpr(0))
				if p.match(TokenPunctuator, ",") == nil {
					break
				}
			}
		}
		p.expect(TokenPunctuator, "]")
		return &ExprSeries{Tag: "Series", Locus: tok.Locus, Elementa: elementa}
	}

	if p.match(TokenPunctuator, "{") != nil {
		props := []ObiectumProp{}
		if !p.check(TokenPunctuator, "}") {
			for {
				loc := p.peek(0).Locus
				var key Expr
				computed := false

				if p.match(TokenPunctuator, "[") != nil {
					key = p.parseExpr(0)
					p.expect(TokenPunctuator, "]")
					computed = true
				} else if p.check(TokenTextus) {
					strKey := p.advance().Valor
					key = &ExprLittera{Tag: "Littera", Locus: loc, Species: LitteraTextus, Valor: strKey}
				} else {
					name := p.expectName().Valor
					key = &ExprLittera{Tag: "Littera", Locus: loc, Species: LitteraTextus, Valor: name}
				}

				var valor Expr
				shorthand := false

				if p.match(TokenPunctuator, ":") != nil {
					valor = p.parseExpr(0)
				} else {
					shorthand = true
					keyName := key.(*ExprLittera).Valor
					valor = &ExprNomen{Tag: "Nomen", Locus: loc, Valor: keyName}
				}

				props = append(props, ObiectumProp{Locus: loc, Key: key, Valor: valor, Shorthand: shorthand, Computed: computed})

				if p.match(TokenPunctuator, ",") == nil {
					break
				}
			}
		}
		p.expect(TokenPunctuator, "}")
		return &ExprObiectum{Tag: "Obiectum", Locus: tok.Locus, Props: props}
	}

	if tok.Tag == TokenKeyword {
		switch tok.Valor {
		case "verum":
			p.advance()
			return &ExprLittera{Tag: "Littera", Locus: tok.Locus, Species: LitteraVerum, Valor: "true"}
		case "falsum":
			p.advance()
			return &ExprLittera{Tag: "Littera", Locus: tok.Locus, Species: LitteraFalsum, Valor: "false"}
		case "nihil":
			p.advance()
			return &ExprLittera{Tag: "Littera", Locus: tok.Locus, Species: LitteraNihil, Valor: "null"}
		case "ego":
			p.advance()
			return &ExprEgo{Tag: "Ego", Locus: tok.Locus}
		case "novum":
			return p.parseNovum()
		case "finge":
			return p.parseFinge()
		case "clausura":
			return p.parseClausura()
		case "scriptum":
			return p.parseScriptum()
		default:
			p.advance()
			return &ExprNomen{Tag: "Nomen", Locus: tok.Locus, Valor: tok.Valor}
		}
	}

	if tok.Tag == TokenNumerus {
		p.advance()
		species := LitteraNumerus
		if strings.Contains(tok.Valor, ".") {
			species = LitteraFractus
		}
		return &ExprLittera{Tag: "Littera", Locus: tok.Locus, Species: species, Valor: tok.Valor}
	}

	if tok.Tag == TokenTextus {
		p.advance()
		return &ExprLittera{Tag: "Littera", Locus: tok.Locus, Species: LitteraTextus, Valor: tok.Valor}
	}

	if tok.Tag == TokenIdentifier {
		p.advance()
		return &ExprNomen{Tag: "Nomen", Locus: tok.Locus, Valor: tok.Valor}
	}

	panic(p.error("unexpected token '" + tok.Valor + "'"))
}

func (p *Parser) parseArgs() []Expr {
	args := []Expr{}
	if p.check(TokenPunctuator, ")") {
		return args
	}

	for {
		args = append(args, p.parseExpr(0))
		if p.match(TokenPunctuator, ",") == nil {
			break
		}
	}

	return args
}

func (p *Parser) parseNovum() Expr {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "novum")
	callee := p.parsePrimary()
	args := []Expr{}
	if p.match(TokenPunctuator, "(") != nil {
		args = p.parseArgs()
		p.expect(TokenPunctuator, ")")
	}
	var init Expr
	if p.check(TokenPunctuator, "{") {
		init = p.parsePrimary()
	}
	return &ExprNovum{Tag: "Novum", Locus: locus, Callee: callee, Args: args, Init: init}
}

func (p *Parser) parseFinge() Expr {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "finge")
	variant := p.expect(TokenIdentifier).Valor
	p.expect(TokenPunctuator, "{")

	campi := []ObiectumProp{}
	if !p.check(TokenPunctuator, "}") {
		for {
			loc := p.peek(0).Locus
			name := p.expectName().Valor
			key := &ExprLittera{Tag: "Littera", Locus: loc, Species: LitteraTextus, Valor: name}
			p.expect(TokenPunctuator, ":")
			valor := p.parseExpr(0)
			campi = append(campi, ObiectumProp{Locus: loc, Key: key, Valor: valor, Shorthand: false, Computed: false})
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
	}
	p.expect(TokenPunctuator, "}")

	var typus Typus
	if p.match(TokenKeyword, "qua") != nil {
		typus = p.parseTypus()
	}

	return &ExprFinge{Tag: "Finge", Locus: locus, Variant: variant, Campi: campi, Typus: typus}
}

func (p *Parser) parseClausura() Expr {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "clausura")

	params := []Param{}
	if p.check(TokenIdentifier) {
		for {
			loc := p.peek(0).Locus
			nomen := p.expect(TokenIdentifier).Valor
			var typus Typus
			if p.match(TokenPunctuator, ":") != nil {
				typus = p.parseTypus()
			}
			params = append(params, Param{Locus: loc, Nomen: nomen, Typus: typus, Default: nil, Rest: false})
			if p.match(TokenPunctuator, ",") == nil {
				break
			}
		}
	}

	var corpus interface{}
	if p.check(TokenPunctuator, "{") {
		corpus = p.parseMassa()
	} else {
		p.expect(TokenPunctuator, ":")
		corpus = p.parseExpr(0)
	}

	return &ExprClausura{Tag: "Clausura", Locus: locus, Params: params, Corpus: corpus}
}

func (p *Parser) parseScriptum() Expr {
	locus := p.peek(0).Locus
	p.expect(TokenKeyword, "scriptum")
	p.expect(TokenPunctuator, "(")
	template := p.expect(TokenTextus).Valor
	args := []Expr{}
	for p.match(TokenPunctuator, ",") != nil {
		args = append(args, p.parseExpr(0))
	}
	p.expect(TokenPunctuator, ")")
	return &ExprScriptum{Tag: "Scriptum", Locus: locus, Template: template, Args: args}
}

func containsKey[T any](m map[string]T, key string) bool {
	_, ok := m[key]
	return ok
}

// ExprLocus extracts the location from an expression.
func ExprLocus(expr Expr) Locus {
	switch e := expr.(type) {
	case *ExprNomen:
		return e.Locus
	case *ExprEgo:
		return e.Locus
	case *ExprLittera:
		return e.Locus
	case *ExprBinaria:
		return e.Locus
	case *ExprUnaria:
		return e.Locus
	case *ExprAssignatio:
		return e.Locus
	case *ExprCondicio:
		return e.Locus
	case *ExprVocatio:
		return e.Locus
	case *ExprMembrum:
		return e.Locus
	case *ExprSeries:
		return e.Locus
	case *ExprObiectum:
		return e.Locus
	case *ExprClausura:
		return e.Locus
	case *ExprNovum:
		return e.Locus
	case *ExprCede:
		return e.Locus
	case *ExprQua:
		return e.Locus
	case *ExprInnatum:
		return e.Locus
	case *ExprPostfixNovum:
		return e.Locus
	case *ExprFinge:
		return e.Locus
	case *ExprScriptum:
		return e.Locus
	case *ExprAmbitus:
		return e.Locus
	default:
		return Locus{}
	}
}

// Prepare filters out comments and newlines.
func Prepare(tokens []Token) []Token {
	out := make([]Token, 0, len(tokens))
	for _, tok := range tokens {
		if tok.Tag == TokenComment || tok.Tag == TokenNewline {
			continue
		}
		out = append(out, tok)
	}
	return out
}

// Parse tokens into a module.
func Parse(tokens []Token, filename string) *Modulus {
	return NewParser(tokens, filename).Parse()
}
