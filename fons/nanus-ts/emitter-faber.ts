/**
 * NANUS - Minimal Faber Compiler
 *
 * Emitter: AST -> Faber source (canonical form)
 * Used for formatting/pretty-printing Faber code.
 */

import type {
    Typus,
    Expr,
    Stmt,
    Param,
    ObiectumProp,
    Modulus,
    CampusDecl,
    OrdoMembrum,
    VariansDecl,
    EligeCasus,
    DiscerneCasus,
    CustodiClausula,
    PactumMethodus,
} from './ast';

export function emitFaber(mod: Modulus): string {
    const lines: string[] = [];
    for (let i = 0; i < mod.corpus.length; i++) {
        if (i > 0) lines.push('');
        lines.push(emitStmt(mod.corpus[i], ''));
    }
    return lines.join('\n');
}

function emitStmt(stmt: Stmt, indent: string): string {
    switch (stmt.tag) {
        case 'Massa':
            return emitMassa(stmt.corpus, indent);

        case 'Expressia':
            return `${indent}${emitExpr(stmt.expr)}`;

        case 'Varia':
            return emitVaria(stmt, indent);

        case 'Functio':
            return emitFunctio(stmt, indent);

        case 'Genus':
            return emitGenus(stmt, indent);

        case 'Pactum':
            return emitPactum(stmt, indent);

        case 'Ordo':
            return emitOrdo(stmt, indent);

        case 'Discretio':
            return emitDiscretio(stmt, indent);

        case 'TypusAlias': {
            const pub = stmt.publica ? '@ publica\n' + indent : '';
            return `${pub}typus ${stmt.nomen} = ${emitTypus(stmt.typus)}`;
        }

        case 'In':
            return `${indent}in ${emitExpr(stmt.expr)} ${emitStmt(stmt.corpus, indent)}`;

        case 'Importa':
            return emitImporta(stmt, indent);

        case 'Redde':
            return stmt.valor ? `${indent}redde ${emitExpr(stmt.valor)}` : `${indent}redde`;

        case 'Si':
            return emitSi(stmt, indent);

        case 'Dum':
            return `${indent}dum ${emitExpr(stmt.cond)} ${emitStmt(stmt.corpus, indent)}`;

        case 'FacDum':
            return `${indent}fac ${emitStmt(stmt.corpus, indent)} dum ${emitExpr(stmt.cond)}`;

        case 'Iteratio': {
            const asyncPrefix = stmt.asynca ? 'cede ' : '';
            const kw = stmt.species === 'Ex' ? 'ex' : 'de';
            return `${indent}${asyncPrefix}itera ${kw} ${emitExpr(stmt.iter)} fixum ${stmt.binding} ${emitStmt(stmt.corpus, indent)}`;
        }

        case 'Elige':
            return emitElige(stmt, indent);

        case 'Discerne':
            return emitDiscerne(stmt, indent);

        case 'Custodi':
            return emitCustodi(stmt.clausulae, indent);

        case 'Tempta':
            return emitTempta(stmt, indent);

        case 'Iace': {
            const kw = stmt.fatale ? 'mori' : 'iace';
            return `${indent}${kw} ${emitExpr(stmt.arg)}`;
        }

        case 'Rumpe':
            return `${indent}rumpe`;

        case 'Perge':
            return `${indent}perge`;

        case 'Scribe': {
            const kw = stmt.gradus === 'Vide' ? 'vide' : stmt.gradus === 'Mone' ? 'mone' : 'scribe';
            const args = stmt.args.map(emitExpr).join(', ');
            return `${indent}${kw} ${args}`;
        }

        case 'Adfirma': {
            const msg = stmt.msg ? `, ${emitExpr(stmt.msg)}` : '';
            return `${indent}adfirma ${emitExpr(stmt.cond)}${msg}`;
        }

        case 'Incipit': {
            const kw = stmt.asynca ? 'incipiet' : 'incipit';
            return `${indent}${kw} ${emitStmt(stmt.corpus, indent)}`;
        }

        case 'Probandum': {
            const lines: string[] = [];
            lines.push(`${indent}probandum "${stmt.nomen}" {`);
            for (const s of stmt.corpus) {
                lines.push(emitStmt(s, indent + '\t'));
            }
            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Proba':
            return `${indent}proba "${stmt.nomen}" ${emitStmt(stmt.corpus, indent)}`;

        default:
            return `${indent}# unknown statement`;
    }
}

function emitMassa(corpus: Stmt[], indent: string): string {
    const lines: string[] = ['{'];
    for (const stmt of corpus) {
        lines.push(emitStmt(stmt, indent + '\t'));
    }
    lines.push(indent + '}');
    return lines.join('\n');
}

function emitVaria(stmt: Stmt & { tag: 'Varia' }, indent: string): string {
    const pub = stmt.publica ? '@ publica\n' + indent : '';
    const ext = stmt.externa ? '@ externa\n' + indent : '';
    const kw = stmt.species === 'Fixum' ? 'fixum' : 'varia';
    const typ = stmt.typus ? `: ${emitTypus(stmt.typus)}` : '';
    const val = stmt.valor ? ` = ${emitExpr(stmt.valor)}` : '';
    return `${ext}${pub}${indent}${kw} ${stmt.nomen}${typ}${val}`;
}

function emitFunctio(stmt: Stmt & { tag: 'Functio' }, indent: string): string {
    const lines: string[] = [];
    if (stmt.externa) lines.push(`${indent}@ externa`);
    if (stmt.publica) lines.push(`${indent}@ publica`);

    let decl = indent;
    if (stmt.asynca) decl += 'asynca ';
    decl += 'functio ';
    decl += stmt.nomen;
    if (stmt.generics.length > 0) {
        decl += `<${stmt.generics.join(', ')}>`;
    }
    decl += '(';
    decl += stmt.params.map(emitParam).join(', ');
    decl += ')';
    if (stmt.typusReditus) {
        decl += ` -> ${emitTypus(stmt.typusReditus)}`;
    }
    if (stmt.corpus) {
        decl += ' ' + emitStmt(stmt.corpus, indent);
    }
    lines.push(decl);
    return lines.join('\n');
}

function emitGenus(stmt: Stmt & { tag: 'Genus' }, indent: string): string {
    const lines: string[] = [];
    if (stmt.publica) lines.push(`${indent}@ publica`);

    let decl = indent;
    if (stmt.abstractus) decl += 'abstractus ';
    decl += 'genus ';
    decl += stmt.nomen;
    if (stmt.generics.length > 0) {
        decl += `<${stmt.generics.join(', ')}>`;
    }
    if (stmt.implet.length > 0) {
        decl += ` implet ${stmt.implet.join(', ')}`;
    }
    decl += ' {';
    lines.push(decl);

    for (const campo of stmt.campi) {
        lines.push(emitCampus(campo, indent + '\t'));
    }
    for (const method of stmt.methodi) {
        lines.push(emitStmt(method, indent + '\t'));
    }

    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitCampus(campo: CampusDecl, indent: string): string {
    const vis = campo.visibilitas === 'Privata' ? '@ privata\n' + indent : campo.visibilitas === 'Protecta' ? '@ protecta\n' + indent : '';
    const typ = campo.typus ? `: ${emitTypus(campo.typus)}` : '';
    const val = campo.valor ? ` = ${emitExpr(campo.valor)}` : '';
    return `${vis}${indent}${campo.nomen}${typ}${val}`;
}

function emitPactum(stmt: Stmt & { tag: 'Pactum' }, indent: string): string {
    const lines: string[] = [];
    if (stmt.publica) lines.push(`${indent}@ publica`);

    let decl = indent + 'pactum ' + stmt.nomen;
    if (stmt.generics.length > 0) {
        decl += `<${stmt.generics.join(', ')}>`;
    }
    decl += ' {';
    lines.push(decl);

    for (const method of stmt.methodi) {
        lines.push(emitPactumMethodus(method, indent + '\t'));
    }

    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitPactumMethodus(method: PactumMethodus, indent: string): string {
    let decl = indent;
    if (method.asynca) decl += 'asynca ';
    decl += 'functio ';
    decl += method.nomen;
    decl += '(';
    decl += method.params.map(emitParam).join(', ');
    decl += ')';
    if (method.typusReditus) {
        decl += ` -> ${emitTypus(method.typusReditus)}`;
    }
    return decl;
}

function emitOrdo(stmt: Stmt & { tag: 'Ordo' }, indent: string): string {
    const lines: string[] = [];
    if (stmt.publica) lines.push(`${indent}@ publica`);

    lines.push(`${indent}ordo ${stmt.nomen} {`);
    for (const m of stmt.membra) {
        const val = m.valor ? ` = ${m.valor}` : '';
        lines.push(`${indent}\t${m.nomen}${val}`);
    }
    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitDiscretio(stmt: Stmt & { tag: 'Discretio' }, indent: string): string {
    const lines: string[] = [];
    if (stmt.publica) lines.push(`${indent}@ publica`);

    let decl = indent + 'discretio ' + stmt.nomen;
    if (stmt.generics.length > 0) {
        decl += `<${stmt.generics.join(', ')}>`;
    }
    decl += ' {';
    lines.push(decl);

    for (const v of stmt.variantes) {
        if (v.campi.length === 0) {
            lines.push(`${indent}\t${v.nomen}`);
        } else {
            lines.push(`${indent}\t${v.nomen} {`);
            for (const f of v.campi) {
                lines.push(`${indent}\t\t${emitTypus(f.typus)} ${f.nomen}`);
            }
            lines.push(`${indent}\t}`);
        }
    }

    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitImporta(stmt: Stmt & { tag: 'Importa' }, indent: string): string {
    const visibility = stmt.publica ? 'publica' : 'privata';
    if (stmt.totum) {
        // Wildcard: importa ex "path" privata|publica * ut alias
        return `${indent}importa ex "${stmt.fons}" ${visibility} * ut ${stmt.local}`;
    }
    // Named: importa ex "path" privata|publica T [ut alias]
    const binding = stmt.imported === stmt.local ? stmt.imported : `${stmt.imported} ut ${stmt.local}`;
    return `${indent}importa ex "${stmt.fons}" ${visibility} ${binding}`;
}

function emitSi(stmt: Stmt & { tag: 'Si' }, indent: string): string {
    let code = `${indent}si ${emitExpr(stmt.cond)} ${emitStmt(stmt.cons, indent)}`;
    if (stmt.alt) {
        code += ` secus ${emitStmt(stmt.alt, indent)}`;
    }
    return code;
}

function emitElige(stmt: Stmt & { tag: 'Elige' }, indent: string): string {
    const lines: string[] = [];
    lines.push(`${indent}elige ${emitExpr(stmt.discrim)} {`);
    for (const c of stmt.casus) {
        lines.push(`${indent}\tcasu ${emitExpr(c.cond)} ${emitStmt(c.corpus, indent + '\t')}`);
    }
    if (stmt.default_) {
        lines.push(`${indent}\tceterum ${emitStmt(stmt.default_, indent + '\t')}`);
    }
    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitDiscerne(stmt: Stmt & { tag: 'Discerne' }, indent: string): string {
    const lines: string[] = [];
    const discrim = stmt.discrim.map(emitExpr).join(', ');
    lines.push(`${indent}discerne ${discrim} {`);
    for (const c of stmt.casus) {
        const patterns = c.patterns.map(p => {
            if (p.wildcard) return '_';
            let s = p.variant;
            if (p.bindings.length > 0) {
                s += `(${p.bindings.join(', ')})`;
            }
            if (p.alias) {
                s += ` ut ${p.alias}`;
            }
            return s;
        });
        lines.push(`${indent}\tcasu ${patterns.join(', ')} ${emitStmt(c.corpus, indent + '\t')}`);
    }
    lines.push(`${indent}}`);
    return lines.join('\n');
}

function emitCustodi(clausulae: CustodiClausula[], indent: string): string {
    const lines: string[] = [];
    for (const c of clausulae) {
        lines.push(`${indent}custodi ${emitExpr(c.cond)} ${emitStmt(c.corpus, indent)}`);
    }
    return lines.join('\n');
}

function emitTempta(stmt: Stmt & { tag: 'Tempta' }, indent: string): string {
    let code = `${indent}tempta ${emitStmt(stmt.corpus, indent)}`;
    if (stmt.cape) {
        code += ` cape ${stmt.cape.param} ${emitStmt(stmt.cape.corpus, indent)}`;
    }
    if (stmt.demum) {
        code += ` demum ${emitStmt(stmt.demum, indent)}`;
    }
    return code;
}

function emitExpr(expr: Expr): string {
    switch (expr.tag) {
        case 'Nomen':
            return expr.valor;

        case 'Ego':
            return 'ego';

        case 'Littera':
            switch (expr.species) {
                case 'Textus':
                    return `"${escapeString(expr.valor)}"`;
                case 'Verum':
                    return 'verum';
                case 'Falsum':
                    return 'falsum';
                case 'Nihil':
                    return 'nihil';
                default:
                    return expr.valor;
            }

        case 'Binaria':
            return `${emitExpr(expr.sin)} ${expr.signum} ${emitExpr(expr.dex)}`;

        case 'Unaria':
            if (expr.signum === 'nihil' || expr.signum === 'non' || expr.signum === 'nonnihil') {
                return `${expr.signum} ${emitExpr(expr.arg)}`;
            }
            return `${expr.signum}${emitExpr(expr.arg)}`;

        case 'Assignatio':
            return `${emitExpr(expr.sin)} ${expr.signum} ${emitExpr(expr.dex)}`;

        case 'Condicio':
            return `${emitExpr(expr.cond)} sic ${emitExpr(expr.cons)} secus ${emitExpr(expr.alt)}`;

        case 'Vocatio': {
            const args = expr.args.map(emitExpr).join(', ');
            return `${emitExpr(expr.callee)}(${args})`;
        }

        case 'Membrum': {
            const obj = emitExpr(expr.obj);
            if (expr.computed) {
                return `${obj}[${emitExpr(expr.prop)}]`;
            }
            const prop = expr.prop.tag === 'Littera' ? expr.prop.valor : emitExpr(expr.prop);
            const access = expr.nonNull ? '!.' : '.';
            return `${obj}${access}${prop}`;
        }

        case 'Series': {
            const elems = expr.elementa.map(emitExpr).join(', ');
            return `[${elems}]`;
        }

        case 'Obiectum': {
            const props = expr.props
                .map(p => {
                    if (p.shorthand) {
                        return p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key);
                    }
                    const key = p.computed ? `[${emitExpr(p.key)}]` : p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key);
                    return `${key}: ${emitExpr(p.valor)}`;
                })
                .join(', ');
            return `{ ${props} }`;
        }

        case 'Clausura': {
            const params = expr.params.map(emitParam).join(', ');
            const body = 'tag' in expr.corpus ? emitStmt(expr.corpus as Stmt, '') : emitExpr(expr.corpus as Expr);
            return `(${params}) => ${body}`;
        }

        case 'Novum': {
            const args = expr.args.map(emitExpr).join(', ');
            let code = `novum ${emitExpr(expr.callee)}(${args})`;
            if (expr.init) {
                code += ` ${emitExpr(expr.init)}`;
            }
            return code;
        }

        case 'Cede':
            return `cede ${emitExpr(expr.arg)}`;

        case 'Qua':
            return `${emitExpr(expr.expr)} qua ${emitTypus(expr.typus)}`;

        case 'Innatum':
            return `${emitExpr(expr.expr)} innatum ${emitTypus(expr.typus)}`;

        case 'PostfixNovum':
            return `${emitExpr(expr.expr)} novum ${emitTypus(expr.typus)}`;

        case 'Finge': {
            const fields = expr.campi
                .map(p => {
                    const key = p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key);
                    if (p.shorthand) return key;
                    return `${key}: ${emitExpr(p.valor)}`;
                })
                .join(', ');
            return `finge ${expr.variant} { ${fields} }`;
        }

        case 'Scriptum': {
            if (expr.args.length === 0) {
                return `scriptum("${escapeString(expr.template)}")`;
            }
            const args = expr.args.map(emitExpr).join(', ');
            return `scriptum("${escapeString(expr.template)}", ${args})`;
        }

        case 'Ambitus': {
            const op = expr.inclusive ? ' usque ' : ' ante ';
            return `${emitExpr(expr.start)}${op}${emitExpr(expr.end)}`;
        }

        case 'Conversio': {
            let code = `${emitExpr(expr.expr)} ${expr.species}`;
            if (expr.fallback) {
                code += ` vel ${emitExpr(expr.fallback)}`;
            }
            return code;
        }

        default:
            return '# unknown expr';
    }
}

function emitTypus(typus: Typus): string {
    switch (typus.tag) {
        case 'Nomen':
            return typus.nomen;

        case 'Nullabilis':
            return `si ${emitTypus(typus.inner)}`;

        case 'Genericus': {
            const args = typus.args.map(emitTypus).join(', ');
            return `${typus.nomen}<${args}>`;
        }

        case 'Functio': {
            const params = typus.params.map(emitTypus).join(', ');
            return `(${params}) -> ${emitTypus(typus.returns)}`;
        }

        case 'Unio': {
            return typus.members.map(emitTypus).join(' | ');
        }

        case 'Litteralis':
            return typus.valor;

        default:
            return '# unknown type';
    }
}

function emitParam(param: Param): string {
    let code = '';
    if (param.typus?.tag === 'Nullabilis') {
        code += 'si ';
    }
    if (param.ownership) {
        code += param.ownership + ' ';
    }
    if (param.rest) {
        code += 'ceteri ';
    }
    if (param.typus) {
        if (param.typus.tag === 'Nullabilis') {
            code += emitTypus(param.typus.inner) + ' ';
        } else {
            code += emitTypus(param.typus) + ' ';
        }
    }
    code += param.nomen;
    if (param.default_) {
        code += ` = ${emitExpr(param.default_)}`;
    }
    return code;
}

function escapeString(s: string): string {
    let result = '';
    for (const c of s) {
        switch (c) {
            case '\n':
                result += '\\n';
                break;
            case '\t':
                result += '\\t';
                break;
            case '\r':
                result += '\\r';
                break;
            case '\\':
                result += '\\\\';
                break;
            case '"':
                result += '\\"';
                break;
            default:
                result += c;
        }
    }
    return result;
}
