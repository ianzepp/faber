/**
 * NANUS - Minimal Faber Compiler
 *
 * AST type definitions for the subset of Faber needed to compile rivus.
 * This is intentionally minimal: 84 features, not 139.
 *
 * Design: Discriminated unions with 'tag' field. No base classes, no visitors.
 * Extend by adding variants to the unions and cases to the switches.
 */

// Source location for error reporting
export interface Locus {
    linea: number;
    columna: number;
    index: number;
}

// Tokens produced by lexer
export interface Token {
    tag: TokenTag;
    valor: string;
    locus: Locus;
}

export type TokenTag =
    | 'EOF'
    | 'Newline'
    | 'Identifier'
    | 'Numerus'
    | 'Textus'
    | 'Operator'
    | 'Punctuator'
    | 'Keyword'
    | 'Comment';

// Type annotations
export type Typus =
    | { tag: 'Nomen'; nomen: string }
    | { tag: 'Nullabilis'; inner: Typus }
    | { tag: 'Genericus'; nomen: string; args: Typus[] }
    | { tag: 'Functio'; params: Typus[]; returns: Typus }
    | { tag: 'Unio'; members: Typus[] }
    | { tag: 'Litteralis'; valor: string };

// Expressions
export type Expr =
    | { tag: 'Nomen'; locus: Locus; valor: string }
    | { tag: 'Ego'; locus: Locus }
    | { tag: 'Littera'; locus: Locus; species: LitteraSpecies; valor: string }
    | { tag: 'Binaria'; locus: Locus; signum: string; sin: Expr; dex: Expr }
    | { tag: 'Unaria'; locus: Locus; signum: string; arg: Expr }
    | { tag: 'Assignatio'; locus: Locus; signum: string; sin: Expr; dex: Expr }
    | { tag: 'Condicio'; locus: Locus; cond: Expr; cons: Expr; alt: Expr }
    | { tag: 'Vocatio'; locus: Locus; callee: Expr; args: Expr[] }
    | { tag: 'Membrum'; locus: Locus; obj: Expr; prop: Expr; computed: boolean; nonNull: boolean }
    | { tag: 'Series'; locus: Locus; elementa: Expr[] }
    | { tag: 'Obiectum'; locus: Locus; props: ObiectumProp[] }
    | { tag: 'Clausura'; locus: Locus; params: Param[]; corpus: Expr | Stmt }
    | { tag: 'Novum'; locus: Locus; callee: Expr; args: Expr[]; init: Expr | null }
    | { tag: 'Cede'; locus: Locus; arg: Expr }
    | { tag: 'Qua'; locus: Locus; expr: Expr; typus: Typus }
    | { tag: 'Innatum'; locus: Locus; expr: Expr; typus: Typus }
    | { tag: 'Finge'; locus: Locus; variant: string; campi: ObiectumProp[]; typus: Typus | null }
    | { tag: 'Scriptum'; locus: Locus; template: string; args: Expr[] }
    | { tag: 'Ambitus'; locus: Locus; start: Expr; end: Expr; inclusive: boolean };

export type LitteraSpecies = 'Numerus' | 'Fractus' | 'Textus' | 'Verum' | 'Falsum' | 'Nihil';

export interface ObiectumProp {
    locus: Locus;
    key: Expr;
    valor: Expr;
    shorthand: boolean;
    computed: boolean;
}

// Statements
export type Stmt =
    | { tag: 'Massa'; locus: Locus; corpus: Stmt[] }
    | { tag: 'Expressia'; locus: Locus; expr: Expr }
    | { tag: 'Varia'; locus: Locus; species: VariaSpecies; nomen: string; typus: Typus | null; valor: Expr | null; publica: boolean; externa: boolean }
    | { tag: 'Functio'; locus: Locus; nomen: string; params: Param[]; typusReditus: Typus | null; corpus: Stmt | null; asynca: boolean; publica: boolean; generics: string[] }
    | { tag: 'Genus'; locus: Locus; nomen: string; campi: CampusDecl[]; methodi: Stmt[]; implet: string[]; generics: string[]; publica: boolean }
    | { tag: 'Pactum'; locus: Locus; nomen: string; methodi: PactumMethodus[]; generics: string[]; publica: boolean }
    | { tag: 'Ordo'; locus: Locus; nomen: string; membra: OrdoMembrum[]; publica: boolean }
    | { tag: 'Discretio'; locus: Locus; nomen: string; variantes: VariansDecl[]; generics: string[]; publica: boolean }
    | { tag: 'Importa'; locus: Locus; fons: string; specs: ImportSpec[]; totum: boolean; alias: string | null }
    | { tag: 'Si'; locus: Locus; cond: Expr; cons: Stmt; alt: Stmt | null }
    | { tag: 'Dum'; locus: Locus; cond: Expr; corpus: Stmt }
    | { tag: 'FacDum'; locus: Locus; corpus: Stmt; cond: Expr }
    | { tag: 'Iteratio'; locus: Locus; species: 'Ex' | 'De'; binding: string; iter: Expr; corpus: Stmt; asynca: boolean }
    | { tag: 'Elige'; locus: Locus; discrim: Expr; casus: EligeCasus[]; default_: Stmt | null }
    | { tag: 'Discerne'; locus: Locus; discrim: Expr[]; casus: DiscerneCasus[] }
    | { tag: 'Custodi'; locus: Locus; clausulae: CustodiClausula[] }
    | { tag: 'Tempta'; locus: Locus; corpus: Stmt; cape: CapeClausula | null; demum: Stmt | null }
    | { tag: 'Redde'; locus: Locus; valor: Expr | null }
    | { tag: 'Iace'; locus: Locus; arg: Expr; fatale: boolean }
    | { tag: 'Scribe'; locus: Locus; gradus: 'Scribe' | 'Vide' | 'Mone'; args: Expr[] }
    | { tag: 'Adfirma'; locus: Locus; cond: Expr; msg: Expr | null }
    | { tag: 'Rumpe'; locus: Locus }
    | { tag: 'Perge'; locus: Locus }
    | { tag: 'Incipit'; locus: Locus; corpus: Stmt; asynca: boolean }
    | { tag: 'Probandum'; locus: Locus; nomen: string; corpus: Stmt[] }
    | { tag: 'Proba'; locus: Locus; nomen: string; corpus: Stmt };

export type VariaSpecies = 'Varia' | 'Fixum' | 'Figendum';

export interface Param {
    locus: Locus;
    nomen: string;
    typus: Typus | null;
    default_: Expr | null;
    rest: boolean;
}

export interface CampusDecl {
    locus: Locus;
    nomen: string;
    typus: Typus;
    valor: Expr | null;
    visibilitas: 'Publica' | 'Privata' | 'Protecta';
}

export interface PactumMethodus {
    locus: Locus;
    nomen: string;
    params: Param[];
    typusReditus: Typus | null;
    asynca: boolean;
}

export interface OrdoMembrum {
    locus: Locus;
    nomen: string;
    valor: string | null;
}

export interface VariansDecl {
    locus: Locus;
    nomen: string;
    campi: { nomen: string; typus: Typus }[];
}

export interface ImportSpec {
    locus: Locus;
    imported: string;
    local: string;
}

export interface EligeCasus {
    locus: Locus;
    cond: Expr;
    corpus: Stmt;
}

export interface DiscerneCasus {
    locus: Locus;
    patterns: VariansPattern[];
    corpus: Stmt;
}

export interface VariansPattern {
    locus: Locus;
    variant: string;
    bindings: string[];
    alias: string | null;
    wildcard: boolean;
}

export interface CustodiClausula {
    locus: Locus;
    cond: Expr;
    corpus: Stmt;
}

export interface CapeClausula {
    locus: Locus;
    param: string;
    corpus: Stmt;
}

// Top-level compilation unit
export interface Modulus {
    locus: Locus;
    corpus: Stmt[];
}
