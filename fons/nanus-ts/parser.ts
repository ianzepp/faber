/**
 * NANUS - Minimal Faber Compiler
 *
 * Parser: tokens → AST
 * Pratt parser for expressions, recursive descent for statements.
 *
 * Supports the subset of Faber syntax needed to compile rivus.
 */

import type {
    Token,
    Locus,
    Typus,
    Expr,
    Stmt,
    Param,
    ObiectumProp,
    LitteraSpecies,
    VariaSpecies,
    CampusDecl,
    PactumMethodus,
    OrdoMembrum,
    VariansDecl,
    ImportSpec,
    EligeCasus,
    DiscerneCasus,
    VariansPattern,
    CustodiClausula,
    CapeClausula,
    Modulus,
} from './ast';
import { CompileError } from './errors';

// Operator precedence for Pratt parser
const PRECEDENCE: Record<string, number> = {
    '=': 1,
    '+=': 1,
    '-=': 1,
    '*=': 1,
    '/=': 1,
    vel: 2,
    '??': 2,
    aut: 3,
    '||': 3,
    et: 4,
    '&&': 4,
    '==': 5,
    '!=': 5,
    '===': 5,
    '!==': 5,
    '<': 6,
    '>': 6,
    '<=': 6,
    '>=': 6,
    inter: 6,
    intra: 6,
    '+': 7,
    '-': 7,
    '*': 8,
    '/': 8,
    '%': 8,
    qua: 9,
    innatum: 9,
    novum: 9,
    numeratum: 9,
    fractatum: 9,
    textatum: 9,
    bivalentum: 9,
};

const UNARY_OPS = new Set(['-', '!', '~', 'non', 'nihil', 'nonnihil', 'positivum', 'negativum', 'nulla', 'nonnulla']);
const ASSIGN_OPS = new Set(['=', '+=', '-=', '*=', '/=']);

export class Parser {
    private tokens: Token[];
    private pos = 0;
    private filename: string;

    constructor(tokens: Token[], filename = '<stdin>') {
        this.tokens = tokens;
        this.filename = filename;
    }

    // Token access
    private peek(offset = 0): Token {
        return this.tokens[this.pos + offset] ?? this.tokens[this.tokens.length - 1];
    }

    private advance(): Token {
        return this.tokens[this.pos++];
    }

    private check(tag: string, valor?: string): boolean {
        const tok = this.peek();
        if (tok.tag !== tag) return false;
        if (valor !== undefined && tok.valor !== valor) return false;
        return true;
    }

    private match(tag: string, valor?: string): Token | null {
        if (this.check(tag, valor)) return this.advance();
        return null;
    }

    private expect(tag: string, valor?: string): Token {
        const tok = this.match(tag, valor);
        if (!tok) {
            const got = this.peek();
            throw this.error(`expected ${valor ?? tag}, got '${got.valor}'`);
        }
        return tok;
    }

    private error(msg: string): CompileError {
        return new CompileError(msg, this.peek().locus, this.filename);
    }

    // Accept identifier OR keyword as a name (for field names, param names that are keywords)
    private expectName(): Token {
        const tok = this.peek();
        if (tok.tag === 'Identifier' || tok.tag === 'Keyword') {
            return this.advance();
        }
        throw this.error(`expected identifier, got '${tok.valor}'`);
    }

    private checkName(): boolean {
        const tok = this.peek();
        return tok.tag === 'Identifier' || tok.tag === 'Keyword';
    }

    // Main entry point
    parse(): Modulus {
        const corpus: Stmt[] = [];
        while (!this.check('EOF')) {
            corpus.push(this.parseStmt());
        }
        return { locus: { linea: 1, columna: 1, index: 0 }, corpus };
    }

    // Statements
    private parseStmt(): Stmt {
        // Annotations - dispatch based on keyword
        let publica = false;
        let futura = false;
        let externa = false;
        while (this.match('Punctuator', '@')) {
            const [pub, fut, ext] = this.parseAnnotatio();
            if (pub) publica = true;
            if (fut) futura = true;
            if (ext) externa = true;
        }

        // Section annotation: § keyword [args...]
        if (this.match('Punctuator', '§')) {
            return this.parseSectio();
        }

        const tok = this.peek();

        // Keywords
        if (tok.tag === 'Keyword') {
            switch (tok.valor) {
                case 'varia':
                case 'fixum':
                case 'figendum':
                case 'variandum':
                    return this.parseVaria(publica, externa);
                case 'ex':
                    return this.parseExStmt(publica);
                case 'functio':
                    return this.parseFunctio(publica, futura, externa);
                case 'abstractus': {
                    const locus = this.peek().locus;
                    this.advance(); // consume 'abstractus'
                    this.expect('Keyword', 'genus');
                    return this.parseGenusBody(locus, publica, true);
                }
                case 'genus':
                    return this.parseGenus(publica);
                case 'pactum':
                    return this.parsePactum(publica);
                case 'ordo':
                    return this.parseOrdo(publica);
                case 'discretio':
                    return this.parseDiscretio(publica);
                case 'si':
                    return this.parseSi();
                case 'dum':
                    return this.parseDum();
                case 'fac':
                    return this.parseFac();
                case 'elige':
                    return this.parseElige();
                case 'discerne':
                    return this.parseDiscerne();
                case 'custodi':
                    return this.parseCustodi();
                case 'tempta':
                    return this.parseTempta();
                case 'redde':
                    return this.parseRedde();
                case 'iace':
                case 'mori':
                    return this.parseIace();
                case 'scribe':
                case 'vide':
                case 'mone':
                    return this.parseScribe();
                case 'adfirma':
                    return this.parseAdfirma();
                case 'rumpe':
                    return this.parseRumpe();
                case 'perge':
                    return this.parsePerge();
                case 'incipit':
                case 'incipiet':
                    return this.parseIncipit();
                case 'probandum':
                    return this.parseProbandum();
                case 'proba':
                    return this.parseProba();
                case 'typus':
                    return this.parseTypusAlias(publica);
                case 'de':
                    return this.parseDeStmt();
            }
        }

        // Block
        if (this.check('Punctuator', '{')) {
            return this.parseMassa();
        }

        // Expression statement
        return this.parseExpressiaStmt();
    }

    // Dispatch § annotations based on keyword
    private parseSectio(): Stmt {
        const tok = this.peek();
        if (tok.tag !== 'Identifier' && tok.tag !== 'Keyword') {
            throw this.error('expected keyword after §');
        }
        const keyword = this.advance().valor;
        switch (keyword) {
            case 'importa':
                return this.parseSectioImporta();
            case 'sectio':
                return this.parseSectioSectio();
            case 'ex':
                return this.parseSectioExLegacy();
            default:
                throw this.error(`unknown § keyword: ${keyword}`);
        }
    }

    // New syntax: § importa ex "path" bindings
    private parseSectioImporta(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ex');
        const fons = this.expect('Textus').valor;

        // Check for wildcard import: * or * ut alias
        if (this.match('Operator', '*')) {
            let alias: string | null = null;
            if (this.match('Keyword', 'ut')) {
                alias = this.expect('Identifier').valor;
            }
            return { tag: 'Importa', locus, fons, specs: [], totum: true, alias };
        }

        const specs: ImportSpec[] = [];
        do {
            const loc = this.peek().locus;
            const imported = this.expect('Identifier').valor;
            let local = imported;
            if (this.match('Keyword', 'ut')) {
                local = this.expect('Identifier').valor;
            }
            specs.push({ locus: loc, imported, local });
        } while (this.match('Punctuator', ','));

        return { tag: 'Importa', locus, fons, specs, totum: false, alias: null };
    }

    // § sectio "name" - file section marker (ignored in nanus, but parsed)
    private parseSectioSectio(): Stmt {
        const locus = this.peek().locus;
        this.expect('Textus'); // section name, ignored
        return { tag: 'Expressia', locus, expr: { tag: 'Littera', locus, species: 'Nihil', valor: 'null' } };
    }

    // Legacy syntax: § ex "path" importa bindings
    private parseSectioExLegacy(): Stmt {
        const locus = this.peek().locus;
        const fons = this.expect('Textus').valor;
        this.expect('Keyword', 'importa');

        // Check for wildcard import: * or * ut alias
        if (this.match('Operator', '*')) {
            let alias: string | null = null;
            if (this.match('Keyword', 'ut')) {
                alias = this.expect('Identifier').valor;
            }
            return { tag: 'Importa', locus, fons, specs: [], totum: true, alias };
        }

        const specs: ImportSpec[] = [];
        do {
            const loc = this.peek().locus;
            const imported = this.expect('Identifier').valor;
            let local = imported;
            if (this.match('Keyword', 'ut')) {
                local = this.expect('Identifier').valor;
            }
            specs.push({ locus: loc, imported, local });
        } while (this.match('Punctuator', ','));

        return { tag: 'Importa', locus, fons, specs, totum: false, alias: null };
    }

    // Dispatch @ annotations based on keyword. Returns [publica, futura, externa].
    private parseAnnotatio(): [boolean, boolean, boolean] {
        const tok = this.peek();
        if (tok.tag !== 'Identifier' && tok.tag !== 'Keyword') {
            throw this.error('expected keyword after @');
        }
        const keyword = this.advance().valor;
        switch (keyword) {
            case 'publica':
            case 'publicum':
                return [true, false, false];
            case 'privata':
            case 'privatum':
                return [false, false, false];
            case 'futura':
                return [false, true, false];
            case 'externa':
                return [false, false, true];
            // Stdlib annotations - skip their arguments
            case 'innatum':
            case 'subsidia':
            case 'radix':
            case 'verte':
                this.skipAnnotatioArgs();
                return [false, false, false];
            // CLI annotations - skip their arguments
            case 'cli':
            case 'versio':
            case 'descriptio':
            case 'optio':
            case 'operandus':
            case 'imperium':
            case 'alias':
            case 'imperia':
            case 'nomen':
            // Formatter annotations - skip their arguments
            case 'indentum':
            case 'tabulae':
            case 'latitudo':
            case 'ordinatio':
            case 'separaGroups':
            case 'bracchiae':
            case 'methodiSeparatio':
                this.skipAnnotatioArgs();
                return [false, false, false];
            default:
                throw this.error(`unknown @ keyword: ${keyword}`);
        }
    }

    // Skip annotation arguments until next @ or § or declaration keyword
    private skipAnnotatioArgs(): void {
        while (!this.check('EOF') && !this.check('Punctuator', '@') && !this.check('Punctuator', '§') && !this.isDeclarationKeyword()) {
            this.advance();
        }
    }

    private parseVaria(publica: boolean, externa: boolean = false): Stmt {
        const locus = this.peek().locus;
        const kw = this.advance().valor;
        const species: VariaSpecies = kw === 'varia' ? 'Varia' : kw === 'figendum' ? 'Figendum' : kw === 'variandum' ? 'Variandum' : 'Fixum';

        // Handle type-first syntax: varia <type> <name> = value
        // vs name-first syntax: varia <name> = value
        let typus: Typus | null = null;
        let nomen: string;

        // Check for nullable prefix: varia si textus name
        const nullable = this.match('Keyword', 'si') !== null;

        const first = this.expectName().valor;

        // Check for generic type: Type<...>
        if (this.check('Operator', '<')) {
            // Type-first with generics: varia lista<textus> items = []
            const args: Typus[] = [];
            this.advance(); // consume <
            do {
                args.push(this.parseTypus());
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
            typus = { tag: 'Genericus', nomen: first, args };

            if (nullable) {
                typus = { tag: 'Nullabilis', inner: typus };
            }

            nomen = this.expectName().valor;
        } else if (this.checkName()) {
            // Type-first: varia numerus count = 0
            typus = { tag: 'Nomen', nomen: first };
            if (nullable) {
                typus = { tag: 'Nullabilis', inner: typus };
            }
            nomen = this.expectName().valor;
        } else {
            // Just a name: varia count = 0
            nomen = first;
        }

        let valor: Expr | null = null;
        if (this.match('Operator', '=')) {
            valor = this.parseExpr();
        }
        return { tag: 'Varia', locus, species, nomen, typus, valor, publica, externa };
    }

    private parseExStmt(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ex');

        // ex items fixum item { ... } — iteration
        // Check if this is iteration: ex <expr> fixum/varia <ident> { ... }
        const expr = this.parseExpr();

        if (this.check('Keyword', 'fixum') || this.check('Keyword', 'varia')) {
            const species: 'Ex' | 'De' = 'Ex';
            const asynca = false; // TODO: handle cede
            this.advance(); // fixum/varia
            const binding = this.expect('Identifier').valor;
            const corpus = this.parseMassa();
            return { tag: 'Iteratio', locus, species, binding, iter: expr, corpus, asynca };
        }

        // ex obj fixum a, b — destructuring (not implemented in nanus)
        throw this.error('destructuring not supported in nanus');
    }

    private parseFunctio(publica: boolean, futura: boolean = false, externa: boolean = false): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'functio');
        const asynca = futura;

        const nomen = this.expectName().valor;

        // Optional generics
        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '(');
        const params = this.parseParams();
        this.expect('Punctuator', ')');

        let typusReditus: Typus | null = null;
        if (this.match('Operator', '->')) {
            typusReditus = this.parseTypus();
        }

        let corpus: Stmt | null = null;
        if (this.check('Punctuator', '{')) {
            corpus = this.parseMassa();
        } else {
        }

        return { tag: 'Functio', locus, nomen, params, typusReditus, corpus, asynca, publica, generics, externa };
    }

    private parseParams(): Param[] {
        const params: Param[] = [];
        if (this.check('Punctuator', ')')) return params;

        do {
            const locus = this.peek().locus;
            let rest = false;
            if (this.match('Keyword', 'ceteri')) rest = true;

            // Check for optional param: si Type name
            let optional = false;
            if (this.match('Keyword', 'si')) {
                optional = true;
            }

            // Check for ownership preposition: ex/de/in
            let ownership: string | undefined;
            if (this.match('Keyword', 'ex')) {
                ownership = 'ex';
            } else if (this.match('Keyword', 'de')) {
                ownership = 'de';
            } else if (this.match('Keyword', 'in')) {
                ownership = 'in';
            }

            let typus: Typus | null = null;
            let nomen: string;

            if (this.checkName()) {
                const first = this.expectName().valor;

                // Check for generic type: Type<T>
                if (this.match('Operator', '<')) {
                    const args: Typus[] = [];
                    do {
                        args.push(this.parseTypus());
                    } while (this.match('Punctuator', ','));
                    this.expect('Operator', '>');
                    typus = { tag: 'Genericus', nomen: first, args };

                    nomen = this.expectName().valor;
                } else if (this.checkName()) {
                    // "Type name" pattern
                    typus = { tag: 'Nomen', nomen: first };
                    nomen = this.expectName().valor;
                } else {
                    // Just a name
                    nomen = first;
                }
            } else {
                throw this.error('expected parameter name');
            }

            // If optional (si prefix before type), wrap type in Nullabilis
            if (optional && typus) {
                typus = { tag: 'Nullabilis', inner: typus };
            }

            let default_: Expr | null = null;
            if (this.match('Operator', '=')) {
                default_ = this.parseExpr();
            }

            params.push({ locus, nomen, typus, default_, rest, optional: optional || undefined, ownership });
        } while (this.match('Punctuator', ','));

        return params;
    }

    private parseGenus(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'genus');
        return this.parseGenusBody(locus, publica, false);
    }

    private parseGenusBody(locus: Locus, publica: boolean, abstractus: boolean): Stmt {
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        const implet: string[] = [];
        if (this.match('Keyword', 'implet')) {
            do {
                implet.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
        }

        this.expect('Punctuator', '{');

        const campi: CampusDecl[] = [];
        const methodi: Stmt[] = [];

        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            // Handle annotations in class body
            let fieldPublica = false;
            let fieldFutura = false;
            let fieldExterna = false;
            while (this.match('Punctuator', '@')) {
                const [pub, fut, ext] = this.parseAnnotatio();
                if (pub) fieldPublica = true;
                if (fut) fieldFutura = true;
                if (ext) fieldExterna = true;
            }

            // Check for visibility keyword
            let visibilitas: 'Publica' | 'Privata' | 'Protecta' = fieldPublica || publica ? 'Publica' : 'Privata';
            if (this.match('Keyword', 'privata') || this.match('Keyword', 'privatus')) {
                visibilitas = 'Privata';
            } else if (this.match('Keyword', 'protecta') || this.match('Keyword', 'protectus')) {
                visibilitas = 'Protecta';
            }

            if (this.check('Keyword', 'functio')) {
                methodi.push(this.parseFunctio(fieldPublica, fieldFutura, fieldExterna));
            } else {
                // Field: si? Typus nomen or si? Typus<T> nomen
                const loc = this.peek().locus;

                // Check for nullable prefix: si Typus nomen
                const nullable = this.match('Keyword', 'si') !== null;

                const first = this.expectName().valor;
                let fieldTypus: Typus;
                let fieldNomen: string;

                // Check for generic: Typus<...>
                if (this.match('Operator', '<')) {
                    const args: Typus[] = [];
                    do {
                        args.push(this.parseTypus());
                    } while (this.match('Punctuator', ','));
                    this.expect('Operator', '>');
                    fieldTypus = { tag: 'Genericus', nomen: first, args };

                    if (nullable) {
                        fieldTypus = { tag: 'Nullabilis', inner: fieldTypus };
                    }

                    fieldNomen = this.expectName().valor;
                } else {
                    if (this.checkName()) {
                        // "Typus nomen" pattern
                        fieldTypus = { tag: 'Nomen', nomen: first };
                        if (nullable) {
                            fieldTypus = { tag: 'Nullabilis', inner: fieldTypus };
                        }
                        fieldNomen = this.expectName().valor;
                    } else {
                        throw this.error('expected field type or name');
                    }
                }

                let valor: Expr | null = null;
                if (this.match('Operator', '=')) {
                    valor = this.parseExpr();
                }

                campi.push({ locus: loc, nomen: fieldNomen, typus: fieldTypus, valor, visibilitas });
            }
        }

        this.expect('Punctuator', '}');
        return { tag: 'Genus', locus, nomen, campi, methodi, implet, generics, publica, abstractus };
    }

    private parsePactum(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'pactum');
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '{');

        const methodi: PactumMethodus[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            this.expect('Keyword', 'functio');
            let asynca = false;
            if (this.match('Keyword', 'asynca')) asynca = true;
            const name = this.expect('Identifier').valor;
            this.expect('Punctuator', '(');
            const params = this.parseParams();
            this.expect('Punctuator', ')');
            let typusReditus: Typus | null = null;
            if (this.match('Operator', '->')) {
                typusReditus = this.parseTypus();
            }
            methodi.push({ locus: loc, nomen: name, params, typusReditus, asynca });
        }

        this.expect('Punctuator', '}');
        return { tag: 'Pactum', locus, nomen, methodi, generics, publica };
    }

    private parseOrdo(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'ordo');
        const nomen = this.expect('Identifier').valor;
        this.expect('Punctuator', '{');

        const membra: OrdoMembrum[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            const name = this.expect('Identifier').valor;
            let valor: string | null = null;
            if (this.match('Operator', '=')) {
                const tok = this.peek();
                // Quote string values so they emit correctly in TypeScript
                valor = tok.tag === 'Textus' ? JSON.stringify(tok.valor) : tok.valor;
                this.advance(); // number or string
            }
            membra.push({ locus: loc, nomen: name, valor });
            this.match('Punctuator', ','); // optional trailing comma
        }

        this.expect('Punctuator', '}');
        return { tag: 'Ordo', locus, nomen, membra, publica };
    }

    private parseDiscretio(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'discretio');
        const nomen = this.expect('Identifier').valor;

        const generics: string[] = [];
        if (this.match('Operator', '<')) {
            do {
                generics.push(this.expect('Identifier').valor);
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
        }

        this.expect('Punctuator', '{');

        const variantes: VariansDecl[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            const name = this.expect('Identifier').valor;
            const campi: { nomen: string; typus: Typus }[] = [];

            if (this.match('Punctuator', '{')) {
                while (!this.check('Punctuator', '}') && !this.check('EOF')) {
                    // si? Typus nomen or si? Typus<T> nomen patterns
                    const nullable = this.match('Keyword', 'si') !== null;
                    const typNomen = this.expectName().valor;
                    let fieldTypus: Typus;

                    if (this.match('Operator', '<')) {
                        const args: Typus[] = [];
                        do {
                            args.push(this.parseTypus());
                        } while (this.match('Punctuator', ','));
                        this.expect('Operator', '>');
                        fieldTypus = { tag: 'Genericus', nomen: typNomen, args };
                    } else {
                        fieldTypus = { tag: 'Nomen', nomen: typNomen };
                    }

                    if (nullable) {
                        fieldTypus = { tag: 'Nullabilis', inner: fieldTypus };
                    }

                    const fieldNomen = this.expectName().valor;
                    campi.push({ nomen: fieldNomen, typus: fieldTypus });
                }
                this.expect('Punctuator', '}');
            }

            variantes.push({ locus: loc, nomen: name, campi });
        }

        this.expect('Punctuator', '}');
        return { tag: 'Discretio', locus, nomen, variantes, generics, publica };
    }

    private parseMassa(): Stmt {
        const locus = this.peek().locus;
        this.expect('Punctuator', '{');
        const corpus: Stmt[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            corpus.push(this.parseStmt());
        }
        this.expect('Punctuator', '}');
        return { tag: 'Massa', locus, corpus };
    }

    // Parse body: block, ergo stmt, reddit/iacit/moritor expr, or tacet
    private parseBody(): Stmt {
        const locus = this.peek().locus;

        // Block form: { ... }
        if (this.check('Punctuator', '{')) {
            return this.parseMassa();
        }

        // One-liner with statement: ergo stmt
        if (this.match('Keyword', 'ergo')) {
            const stmt = this.parseStmt();
            return { tag: 'Massa', locus, corpus: [stmt] };
        }

        // Inline return: reddit expr
        if (this.match('Keyword', 'reddit')) {
            const valor = this.parseExpr();
            return { tag: 'Massa', locus, corpus: [{ tag: 'Redde', locus, valor }] };
        }

        // Inline throw: iacit expr
        if (this.match('Keyword', 'iacit')) {
            const arg = this.parseExpr();
            return { tag: 'Massa', locus, corpus: [{ tag: 'Iace', locus, arg, fatale: false }] };
        }

        // Inline panic: moritor expr
        if (this.match('Keyword', 'moritor')) {
            const arg = this.parseExpr();
            return { tag: 'Massa', locus, corpus: [{ tag: 'Iace', locus, arg, fatale: true }] };
        }

        // No-op: tacet
        if (this.match('Keyword', 'tacet')) {
            return { tag: 'Massa', locus, corpus: [] };
        }

        // If none matched, require block
        return this.parseMassa();
    }

    private parseSi(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'si');
        return this.parseSiBody(locus);
    }

    private parseSiBody(locus: Locus): Stmt {
        const cond = this.parseExpr();
        const cons = this.parseBody();
        let alt: Stmt | null = null;
        if (this.match('Keyword', 'sin')) {
            // sin = else-if shorthand - parse body directly without 'si'
            const sinLocus = this.peek().locus;
            alt = this.parseSiBody(sinLocus);
        } else if (this.match('Keyword', 'secus')) {
            if (this.check('Keyword', 'si')) {
                alt = this.parseSi();
            } else {
                alt = this.parseBody();
            }
        }
        return { tag: 'Si', locus, cond, cons, alt };
    }

    private parseDum(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'dum');
        const cond = this.parseExpr();
        const corpus = this.parseBody();
        return { tag: 'Dum', locus, cond, corpus };
    }

    private parseFac(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'fac');
        const corpus = this.parseMassa();
        this.expect('Keyword', 'dum');
        const cond = this.parseExpr();
        return { tag: 'FacDum', locus, corpus, cond };
    }

    private parseElige(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'elige');
        const discrim = this.parseExpr();
        this.expect('Punctuator', '{');

        const casus: EligeCasus[] = [];
        let default_: Stmt | null = null;

        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            if (this.match('Keyword', 'ceterum')) {
                default_ = this.parseBody();
            } else {
                this.expect('Keyword', 'casu');
                const loc = this.peek().locus;
                const cond = this.parseExpr();
                const corpus = this.parseBody();
                casus.push({ locus: loc, cond, corpus });
            }
        }

        this.expect('Punctuator', '}');
        return { tag: 'Elige', locus, discrim, casus, default_ };
    }

    private parseDiscerne(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'discerne');
        const discrim: Expr[] = [this.parseExpr()];
        while (this.match('Punctuator', ',')) {
            discrim.push(this.parseExpr());
        }
        this.expect('Punctuator', '{');

        const casus: DiscerneCasus[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;

            // ceterum is wildcard/default case
            if (this.match('Keyword', 'ceterum')) {
                const patterns: VariansPattern[] = [
                    {
                        locus: loc,
                        variant: '_',
                        bindings: [],
                        alias: null,
                        wildcard: true,
                    },
                ];
                const corpus = this.parseBody();
                casus.push({ locus: loc, patterns, corpus });
                continue;
            }

            this.expect('Keyword', 'casu');
            const patterns: VariansPattern[] = [];

            // Parse pattern(s)
            do {
                const pLoc = this.peek().locus;
                const variant = this.expect('Identifier').valor;
                let alias: string | null = null;
                const bindings: string[] = [];
                const wildcard = variant === '_';

                if (this.match('Keyword', 'ut')) {
                    alias = this.expectName().valor;
                } else if (this.match('Keyword', 'pro') || this.match('Keyword', 'fixum')) {
                    do {
                        bindings.push(this.expectName().valor);
                    } while (this.match('Punctuator', ','));
                }

                patterns.push({ locus: pLoc, variant, bindings, alias, wildcard });
            } while (this.match('Punctuator', ','));

            const corpus = this.parseBody();
            casus.push({ locus: loc, patterns, corpus });
        }

        this.expect('Punctuator', '}');
        return { tag: 'Discerne', locus, discrim, casus };
    }

    private parseCustodi(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'custodi');
        this.expect('Punctuator', '{');

        const clausulae: CustodiClausula[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            const loc = this.peek().locus;
            this.expect('Keyword', 'si');
            const cond = this.parseExpr();
            const corpus = this.parseMassa();
            clausulae.push({ locus: loc, cond, corpus });
        }

        this.expect('Punctuator', '}');
        return { tag: 'Custodi', locus, clausulae };
    }

    private parseTempta(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'tempta');
        const corpus = this.parseMassa();

        let cape: CapeClausula | null = null;
        if (this.match('Keyword', 'cape')) {
            const loc = this.peek().locus;
            const param = this.expect('Identifier').valor;
            const body = this.parseMassa();
            cape = { locus: loc, param, corpus: body };
        }

        let demum: Stmt | null = null;
        if (this.match('Keyword', 'demum')) {
            demum = this.parseMassa();
        }

        return { tag: 'Tempta', locus, corpus, cape, demum };
    }

    private parseRedde(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'redde');
        let valor: Expr | null = null;
        // Parse expression if next token can start one (not } or EOF or statement keyword)
        if (!this.check('EOF') && !this.check('Punctuator', '}') && !this.isStatementKeyword()) {
            valor = this.parseExpr();
        }
        return { tag: 'Redde', locus, valor };
    }

    // Check if current token is a keyword that starts a statement (not an expression)
    private isStatementKeyword(): boolean {
        if (!this.check('Keyword')) return false;
        const kw = this.peek().valor;
        const STMT_KEYWORDS = new Set([
            'si',
            'sin',
            'secus',
            'dum',
            'fac',
            'ex',
            'de',
            'elige',
            'discerne',
            'custodi',
            'tempta',
            'cape',
            'demum',
            'redde',
            'rumpe',
            'perge',
            'iace',
            'mori',
            'scribe',
            'vide',
            'mone',
            'adfirma',
            'functio',
            'genus',
            'pactum',
            'ordo',
            'discretio',
            'varia',
            'fixum',
            'figendum',
            'incipit',
            'probandum',
            'proba',
            'casu',
            'ceterum',
            'reddit',
            'ergo',
            'tacet',
            'iacit',
            'moritor',
        ]);
        return STMT_KEYWORDS.has(kw);
    }

    // Check if current token is a declaration keyword (can follow annotations)
    private isDeclarationKeyword(): boolean {
        if (!this.check('Keyword')) return false;
        const kw = this.peek().valor;
        const DECL_KEYWORDS = new Set(['functio', 'genus', 'pactum', 'ordo', 'discretio', 'varia', 'fixum', 'figendum', 'incipit', 'probandum']);
        return DECL_KEYWORDS.has(kw);
    }

    private parseIace(): Stmt {
        const locus = this.peek().locus;
        const fatale = this.advance().valor === 'mori';
        const arg = this.parseExpr();
        return { tag: 'Iace', locus, arg, fatale };
    }

    private parseScribe(): Stmt {
        const locus = this.peek().locus;
        const kw = this.advance().valor;
        const gradus = kw === 'vide' ? 'Vide' : kw === 'mone' ? 'Mone' : 'Scribe';
        const args: Expr[] = [];
        // Parse args if next token can start an expression
        if (!this.check('EOF') && !this.check('Punctuator', '}') && !this.isStatementKeyword()) {
            do {
                args.push(this.parseExpr());
            } while (this.match('Punctuator', ','));
        }
        return { tag: 'Scribe', locus, gradus, args };
    }

    private parseAdfirma(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'adfirma');
        const cond = this.parseExpr();
        let msg: Expr | null = null;
        if (this.match('Punctuator', ',')) {
            msg = this.parseExpr();
        }
        return { tag: 'Adfirma', locus, cond, msg };
    }

    private parseRumpe(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'rumpe');
        return { tag: 'Rumpe', locus };
    }

    private parsePerge(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'perge');
        return { tag: 'Perge', locus };
    }

    private parseIncipit(): Stmt {
        const locus = this.peek().locus;
        const kw = this.advance().valor; // consume incipit or incipiet
        const asynca = kw === 'incipiet';
        const corpus = this.parseMassa();
        return { tag: 'Incipit', locus, corpus, asynca };
    }

    private parseProbandum(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'probandum');
        const nomen = this.expect('Textus').valor;
        this.expect('Punctuator', '{');

        const corpus: Stmt[] = [];
        while (!this.check('Punctuator', '}') && !this.check('EOF')) {
            corpus.push(this.parseStmt());
        }

        this.expect('Punctuator', '}');
        return { tag: 'Probandum', locus, nomen, corpus };
    }

    private parseProba(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'proba');
        const nomen = this.expect('Textus').valor;
        const corpus = this.parseMassa();
        return { tag: 'Proba', locus, nomen, corpus };
    }

    private parseTypusAlias(publica: boolean): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'typus');
        const nomen = this.expect('Identifier').valor;
        this.expect('Operator', '=');
        const typus = this.parseTypus();
        return { tag: 'TypusAlias', locus, nomen, typus, publica };
    }

    private parseDeStmt(): Stmt {
        const locus = this.peek().locus;
        this.expect('Keyword', 'de');
        const iter = this.parseExpr();
        // Expect 'fixum' or 'varia' before binding
        if (!this.check('Keyword', 'fixum') && !this.check('Keyword', 'varia')) {
            throw this.error("expected 'fixum' or 'varia' after 'de' expression");
        }
        this.advance(); // consume fixum/varia
        const binding = this.expect('Identifier').valor;
        const corpus = this.parseMassa();
        return { tag: 'Iteratio', locus, species: 'De', binding, iter, corpus, asynca: false };
    }

    private parseExpressiaStmt(): Stmt {
        const locus = this.peek().locus;
        const expr = this.parseExpr();
        return { tag: 'Expressia', locus, expr };
    }

    // Types
    private parseTypus(): Typus {
        // Check for nullable prefix: si Type
        const nullable = this.match('Keyword', 'si') !== null;

        let typus = this.parseTypusPrimary();

        // Wrap in Nullabilis if si prefix was present
        if (nullable) {
            typus = { tag: 'Nullabilis', inner: typus };
        }

        // Union: T | U
        if (this.match('Operator', '|')) {
            const members: Typus[] = [typus];
            do {
                members.push(this.parseTypusPrimary());
            } while (this.match('Operator', '|'));
            typus = { tag: 'Unio', members };
        }

        return typus;
    }

    private parseTypusPrimary(): Typus {
        const nomen = this.expect('Identifier').valor;

        // Generics: T<U, V>
        if (this.match('Operator', '<')) {
            const args: Typus[] = [];
            do {
                args.push(this.parseTypus());
            } while (this.match('Punctuator', ','));
            this.expect('Operator', '>');
            return { tag: 'Genericus', nomen, args };
        }

        return { tag: 'Nomen', nomen };
    }

    // Expressions - Pratt parser
    private parseExpr(minPrec = 0): Expr {
        let left = this.parseUnary();

        while (true) {
            const tok = this.peek();
            const op = tok.valor;
            const prec = PRECEDENCE[op];

            if (prec === undefined || prec < minPrec) break;

            this.advance();

            // Handle qua/innatum/novum specially (postfix type operators)
            if (op === 'qua') {
                const typus = this.parseTypus();
                left = { tag: 'Qua', locus: tok.locus, expr: left, typus };
                continue;
            }
            if (op === 'innatum') {
                const typus = this.parseTypus();
                left = { tag: 'Innatum', locus: tok.locus, expr: left, typus };
                continue;
            }
            if (op === 'novum') {
                const typus = this.parseTypus();
                left = { tag: 'PostfixNovum', locus: tok.locus, expr: left, typus };
                continue;
            }

            // Handle conversion operators: expr numeratum vel fallback
            if (op === 'numeratum' || op === 'fractatum' || op === 'textatum' || op === 'bivalentum') {
                let fallback: Expr | null = null;
                if (this.match('Keyword', 'vel')) {
                    fallback = this.parseExpr(prec + 1);
                }
                left = { tag: 'Conversio', locus: tok.locus, expr: left, species: op, fallback };
                continue;
            }

            const right = this.parseExpr(prec + 1);

            if (ASSIGN_OPS.has(op)) {
                left = { tag: 'Assignatio', locus: tok.locus, signum: op, sin: left, dex: right };
            } else {
                left = { tag: 'Binaria', locus: tok.locus, signum: op, sin: left, dex: right };
            }
        }

        // Ternary: cond sic cons secus alt
        // Only parse at top level (minPrec=0) so ternary has lowest precedence
        if (minPrec === 0 && this.match('Keyword', 'sic')) {
            const cons = this.parseExpr();
            this.expect('Keyword', 'secus');
            const alt = this.parseExpr();
            left = { tag: 'Condicio', locus: left.locus, cond: left, cons, alt };
        }

        return left;
    }

    private parseUnary(): Expr {
        const tok = this.peek();

        // Unary operators - but 'nihil' alone is a literal, not unary
        // Must be Operator or Keyword tag, not a Textus with value '-'
        if ((tok.tag === 'Operator' || tok.tag === 'Keyword') && UNARY_OPS.has(tok.valor)) {
            // Check if followed by expression (identifier, number, paren, keyword-as-identifier, etc.)
            // Exclude keywords that shouldn't start expressions
            const NON_EXPR_KEYWORDS = new Set([
                'qua',
                'innatum',
                'et',
                'aut',
                'vel',
                'sic',
                'secus',
                'inter',
                'intra', // operators
                'perge',
                'rumpe',
                'redde',
                'reddit',
                'iace',
                'mori', // control flow statements
                'si',
                'secussi',
                'dum',
                'ex',
                'de',
                'elige',
                'discerne',
                'custodi',
                'tempta', // block statements
                'functio',
                'genus',
                'pactum',
                'ordo',
                'discretio', // declarations
                'casu',
                'ceterum',
                'importa',
                'incipit',
                'incipiet',
                'probandum',
                'proba', // more
                // Note: cape, demum can be used as variable names, so not excluded
            ]);
            const next = this.tokens[this.pos + 1];
            const canBeUnary =
                next &&
                (next.tag === 'Identifier' ||
                    (next.tag === 'Keyword' && !NON_EXPR_KEYWORDS.has(next.valor)) ||
                    next.tag === 'Numerus' ||
                    next.tag === 'Textus' ||
                    next.valor === '(' ||
                    next.valor === '[' ||
                    next.valor === '{' ||
                    UNARY_OPS.has(next.valor));

            if (canBeUnary) {
                this.advance();
                const arg = this.parseUnary();
                return { tag: 'Unaria', locus: tok.locus, signum: tok.valor, arg };
            }
        }

        if (this.match('Keyword', 'cede')) {
            const arg = this.parseUnary();
            return { tag: 'Cede', locus: tok.locus, arg };
        }

        return this.parsePostfix();
    }

    private parsePostfix(): Expr {
        let expr = this.parsePrimary();

        while (true) {
            const tok = this.peek();

            // Call: expr(args)
            if (this.match('Punctuator', '(')) {
                const args = this.parseArgs();
                this.expect('Punctuator', ')');
                expr = { tag: 'Vocatio', locus: tok.locus, callee: expr, args };
                continue;
            }

            // Member: expr.prop
            if (this.match('Punctuator', '.')) {
                const prop: Expr = { tag: 'Littera', locus: this.peek().locus, species: 'Textus', valor: this.expectName().valor };
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: false, nonNull: false };
                continue;
            }

            // Non-null member: expr!.prop
            if (this.match('Operator', '!.') || (tok.valor === '!' && this.tokens[this.pos + 1]?.valor === '.')) {
                if (tok.valor === '!') {
                    this.advance(); // !
                    this.advance(); // .
                }
                const prop: Expr = { tag: 'Littera', locus: this.peek().locus, species: 'Textus', valor: this.expectName().valor };
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: false, nonNull: true };
                continue;
            }

            // Non-null computed member: expr![index]
            if (tok.valor === '!' && this.tokens[this.pos + 1]?.valor === '[') {
                this.advance(); // !
                this.advance(); // [
                const prop = this.parseExpr();
                this.expect('Punctuator', ']');
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: true, nonNull: true };
                continue;
            }

            // Computed member: expr[index]
            if (this.match('Punctuator', '[')) {
                const prop = this.parseExpr();
                this.expect('Punctuator', ']');
                expr = { tag: 'Membrum', locus: tok.locus, obj: expr, prop, computed: true, nonNull: false };
                continue;
            }

            break;
        }

        return expr;
    }

    private parsePrimary(): Expr {
        const tok = this.peek();

        // Parenthesized expression
        if (this.match('Punctuator', '(')) {
            const expr = this.parseExpr();
            this.expect('Punctuator', ')');
            return expr;
        }

        // Array literal
        if (this.match('Punctuator', '[')) {
            const elementa: Expr[] = [];
            if (!this.check('Punctuator', ']')) {
                do {
                    elementa.push(this.parseExpr());
                } while (this.match('Punctuator', ','));
            }
            this.expect('Punctuator', ']');
            return { tag: 'Series', locus: tok.locus, elementa };
        }

        // Object literal
        if (this.match('Punctuator', '{')) {
            const props: ObiectumProp[] = [];
            if (!this.check('Punctuator', '}')) {
                do {
                    const loc = this.peek().locus;
                    let key: Expr;
                    let computed = false;

                    if (this.match('Punctuator', '[')) {
                        key = this.parseExpr();
                        this.expect('Punctuator', ']');
                        computed = true;
                    } else if (this.check('Textus')) {
                        // String key: "name": value
                        const strKey = this.advance().valor;
                        key = { tag: 'Littera', locus: loc, species: 'Textus', valor: strKey };
                    } else {
                        const name = this.expectName().valor;
                        key = { tag: 'Littera', locus: loc, species: 'Textus', valor: name };
                    }

                    let valor: Expr;
                    let shorthand = false;

                    if (this.match('Punctuator', ':')) {
                        valor = this.parseExpr();
                    } else {
                        // Shorthand: { name } means { name: name }
                        shorthand = true;
                        valor = { tag: 'Nomen', locus: loc, valor: (key as { valor: string }).valor };
                    }

                    props.push({ locus: loc, key, valor, shorthand, computed });
                } while (this.match('Punctuator', ','));
            }
            this.expect('Punctuator', '}');
            return { tag: 'Obiectum', locus: tok.locus, props };
        }

        // Keywords
        if (tok.tag === 'Keyword') {
            switch (tok.valor) {
                case 'verum':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Verum', valor: 'true' };
                case 'falsum':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Falsum', valor: 'false' };
                case 'nihil':
                    this.advance();
                    return { tag: 'Littera', locus: tok.locus, species: 'Nihil', valor: 'null' };
                case 'ego':
                    this.advance();
                    return { tag: 'Ego', locus: tok.locus };
                case 'novum':
                    return this.parseNovum();
                case 'finge':
                    return this.parseFinge();
                case 'clausura':
                    return this.parseClausura();
                case 'scriptum':
                    return this.parseScriptum();
                default:
                    // Keywords used as identifiers (e.g., 'cape' as variable name)
                    this.advance();
                    return { tag: 'Nomen', locus: tok.locus, valor: tok.valor };
            }
        }

        // Number
        if (tok.tag === 'Numerus') {
            this.advance();
            const species: LitteraSpecies = tok.valor.includes('.') ? 'Fractus' : 'Numerus';
            return { tag: 'Littera', locus: tok.locus, species, valor: tok.valor };
        }

        // String
        if (tok.tag === 'Textus') {
            this.advance();
            return { tag: 'Littera', locus: tok.locus, species: 'Textus', valor: tok.valor };
        }

        // Identifier
        if (tok.tag === 'Identifier') {
            this.advance();
            return { tag: 'Nomen', locus: tok.locus, valor: tok.valor };
        }

        throw this.error(`unexpected token '${tok.valor}'`);
    }

    private parseArgs(): Expr[] {
        const args: Expr[] = [];
        if (this.check('Punctuator', ')')) return args;

        do {
            args.push(this.parseExpr());
        } while (this.match('Punctuator', ','));

        return args;
    }

    private parseNovum(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'novum');
        const callee = this.parsePrimary();
        let args: Expr[] = [];
        if (this.match('Punctuator', '(')) {
            args = this.parseArgs();
            this.expect('Punctuator', ')');
        }
        let init: Expr | null = null;
        if (this.check('Punctuator', '{')) {
            init = this.parsePrimary(); // object literal
        }
        return { tag: 'Novum', locus, callee, args, init };
    }

    private parseFinge(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'finge');
        const variant = this.expect('Identifier').valor;
        this.expect('Punctuator', '{');

        const campi: ObiectumProp[] = [];
        if (!this.check('Punctuator', '}')) {
            do {
                const loc = this.peek().locus;
                const name = this.expectName().valor;
                const key: Expr = { tag: 'Littera', locus: loc, species: 'Textus', valor: name };
                this.expect('Punctuator', ':');
                const valor = this.parseExpr();
                campi.push({ locus: loc, key, valor, shorthand: false, computed: false });
            } while (this.match('Punctuator', ','));
        }
        this.expect('Punctuator', '}');

        let typus: Typus | null = null;
        if (this.match('Keyword', 'qua')) {
            typus = this.parseTypus();
        }

        return { tag: 'Finge', locus, variant, campi, typus };
    }

    private parseClausura(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'clausura');

        const params: Param[] = [];
        if (this.check('Identifier')) {
            do {
                const loc = this.peek().locus;
                const nomen = this.expect('Identifier').valor;
                let typus: Typus | null = null;
                if (this.match('Punctuator', ':')) {
                    typus = this.parseTypus();
                }
                params.push({ locus: loc, nomen, typus, default_: null, rest: false });
            } while (this.match('Punctuator', ','));
        }

        let corpus: Expr | Stmt;
        if (this.check('Punctuator', '{')) {
            corpus = this.parseMassa();
        } else {
            this.expect('Punctuator', ':');
            corpus = this.parseExpr();
        }

        return { tag: 'Clausura', locus, params, corpus };
    }

    private parseScriptum(): Expr {
        const locus = this.peek().locus;
        this.expect('Keyword', 'scriptum');
        this.expect('Punctuator', '(');
        const template = this.expect('Textus').valor;
        const args: Expr[] = [];
        while (this.match('Punctuator', ',')) {
            args.push(this.parseExpr());
        }
        this.expect('Punctuator', ')');
        return { tag: 'Scriptum', locus, template, args };
    }
}

export function parse(tokens: Token[], filename = '<stdin>'): Modulus {
    return new Parser(tokens, filename).parse();
}
