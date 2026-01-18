/**
 * NANUS - Minimal Faber Compiler
 *
 * Emitter: AST → TypeScript source
 * Direct code generation, no intermediate representation.
 *
 * Supports the subset of Faber needed to compile rivus.
 */

import type {
    Typus, Expr, Stmt, Param, ObiectumProp, Modulus,
    CampusDecl, OrdoMembrum, VariansDecl, ImportSpec,
    EligeCasus, DiscerneCasus, CustodiClausula,
} from './ast';

// Operator translation: Faber → TypeScript
const BINARY_OPS: Record<string, string> = {
    'et': '&&',
    'aut': '||',
    'vel': '??',
    'inter': 'in',
    'intra': 'instanceof',
};

const UNARY_OPS: Record<string, string> = {
    'non': '!',        // non x → !x (logical not)
    'nihil': '!',      // nihil x → !x (null check)
    'nonnihil': '!!',  // nonnihil x → !!x (non-null assertion as boolean)
    'positivum': '+',  // positivum x → +x (to number)
};

// Method/property name translations from norma (Faber → TypeScript)
const METHOD_MAP: Record<string, string> = {
    // lista (Array)
    'adde': 'push',
    'praepone': 'unshift',
    'remove': 'pop',
    'decapita': 'shift',
    'coniunge': 'join',
    'continet': 'includes',
    'indiceDe': 'indexOf',
    'inveni': 'find',
    'inveniIndicem': 'findIndex',
    'omnes': 'every',
    'aliquis': 'some',
    'filtrata': 'filter',
    'mappata': 'map',
    'explanata': 'flatMap',
    'plana': 'flat',
    'sectio': 'slice',
    'reducta': 'reduce',
    'perambula': 'forEach',
    'inverte': 'reverse',
    'ordina': 'sort',
    // tabula (Map)
    'pone': 'set',
    'accipe': 'get',
    'habet': 'has',
    'dele': 'delete',
    'purga': 'clear',
    'claves': 'keys',
    'valores': 'values',
    'paria': 'entries',
    // textus (string)
    'initium': 'startsWith',
    'finis': 'endsWith',
    'maiuscula': 'toUpperCase',
    'minuscula': 'toLowerCase',
    'recide': 'trim',
    'divide': 'split',
    'muta': 'replaceAll',
    // copia (Set)  - adde→add for Set, but push for Array; we use push as default
    // Properties (not methods, but accessed the same way)
    'longitudo': 'length',
};

// Properties that should NOT be called as methods (emit without parentheses)
const PROPERTY_ONLY: Set<string> = new Set([
    'longitudo',  // .length (arrays, strings)
    'primus',     // [0] (first element)
    'ultimus',    // .at(-1) (last element)
]);

export function emit(mod: Modulus): string {
    const lines: string[] = [];

    for (const stmt of mod.corpus) {
        lines.push(emitStmt(stmt));
    }

    return lines.join('\n');
}

function emitStmt(stmt: Stmt, indent = ''): string {
    switch (stmt.tag) {
        case 'Massa':
            return `{\n${stmt.corpus.map(s => emitStmt(s, indent + '    ')).join('\n')}\n${indent}}`;

        case 'Expressia':
            return `${indent}${emitExpr(stmt.expr)};`;

        case 'Varia': {
            const decl = stmt.externa ? 'declare ' : '';
            const kw = stmt.species === 'Varia' ? 'let' : 'const';
            // For externa with ignotum type, use 'any' for usability (allows property access)
            let typ = '';
            if (stmt.typus) {
                if (stmt.externa && stmt.typus.tag === 'Nomen' && stmt.typus.nomen === 'ignotum') {
                    typ = ': any';
                } else {
                    typ = `: ${emitTypus(stmt.typus)}`;
                }
            }
            const val = stmt.valor && !stmt.externa ? ` = ${emitExpr(stmt.valor)}` : '';
            const exp = stmt.publica ? 'export ' : '';
            return `${indent}${exp}${decl}${kw} ${stmt.nomen}${typ}${val};`;
        }

        case 'Functio': {
            const exp = stmt.publica ? 'export ' : '';
            const async = stmt.asynca ? 'async ' : '';
            const generics = stmt.generics.length > 0 ? `<${stmt.generics.join(', ')}>` : '';
            const params = stmt.params.map(emitParam).join(', ');
            const ret = stmt.typusReditus ? `: ${emitTypus(stmt.typusReditus)}` : '';
            const body = stmt.corpus ? ` ${emitStmt(stmt.corpus)}` : ';';
            return `${indent}${exp}${async}function ${stmt.nomen}${generics}(${params})${ret}${body}`;
        }

        case 'Genus': {
            const exp = stmt.publica ? 'export ' : '';
            const generics = stmt.generics.length > 0 ? `<${stmt.generics.join(', ')}>` : '';
            const impl = stmt.implet.length > 0 ? ` implements ${stmt.implet.join(', ')}` : '';
            const lines: string[] = [];
            lines.push(`${indent}${exp}class ${stmt.nomen}${generics}${impl} {`);

            for (const campo of stmt.campi) {
                const vis = campo.visibilitas === 'Privata' ? 'private ' : campo.visibilitas === 'Protecta' ? 'protected ' : '';
                const val = campo.valor ? ` = ${emitExpr(campo.valor)}` : '';
                lines.push(`${indent}    ${vis}${campo.nomen}: ${emitTypus(campo.typus)}${val};`);
            }

            for (const method of stmt.methodi) {
                if (method.tag === 'Functio') {
                    const async = method.asynca ? 'async ' : '';
                    const params = method.params.map(emitParam).join(', ');
                    const ret = method.typusReditus ? `: ${emitTypus(method.typusReditus)}` : '';
                    const body = method.corpus ? ` ${emitStmt(method.corpus, indent + '    ')}` : ';';
                    lines.push(`${indent}    ${async}${method.nomen}(${params})${ret}${body}`);
                }
            }

            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Pactum': {
            const exp = stmt.publica ? 'export ' : '';
            const generics = stmt.generics.length > 0 ? `<${stmt.generics.join(', ')}>` : '';
            const lines: string[] = [];
            lines.push(`${indent}${exp}interface ${stmt.nomen}${generics} {`);

            for (const method of stmt.methodi) {
                const async = method.asynca ? 'async ' : '';
                const params = method.params.map(emitParam).join(', ');
                const ret = method.typusReditus ? `: ${emitTypus(method.typusReditus)}` : '';
                lines.push(`${indent}    ${method.nomen}(${params})${ret};`);
            }

            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Ordo': {
            const exp = stmt.publica ? 'export ' : '';
            const lines: string[] = [];
            lines.push(`${indent}${exp}enum ${stmt.nomen} {`);

            for (const m of stmt.membra) {
                const val = m.valor ? ` = ${m.valor}` : '';
                lines.push(`${indent}    ${m.nomen}${val},`);
            }

            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Discretio': {
            const exp = stmt.publica ? 'export ' : '';
            const generics = stmt.generics.length > 0 ? `<${stmt.generics.join(', ')}>` : '';
            const lines: string[] = [];

            // First, export each variant as a separate type alias
            const variantNames: string[] = [];
            for (const v of stmt.variantes) {
                variantNames.push(v.nomen);
                if (v.campi.length === 0) {
                    lines.push(`${indent}${exp}type ${v.nomen} = { tag: '${v.nomen}' };`);
                } else {
                    const fields = v.campi.map(f => `${f.nomen}: ${emitTypus(f.typus)}`).join('; ');
                    lines.push(`${indent}${exp}type ${v.nomen} = { tag: '${v.nomen}'; ${fields} };`);
                }
            }

            // Then generate the union type referencing the variant types
            lines.push(`${indent}${exp}type ${stmt.nomen}${generics} = ${variantNames.join(' | ')};`);

            return lines.join('\n');
        }

        case 'Importa': {
            const specs = stmt.specs.map(s => s.imported === s.local ? s.imported : `${s.imported} as ${s.local}`);
            return `${indent}import { ${specs.join(', ')} } from '${stmt.fons}';`;
        }

        case 'Si': {
            let code = `${indent}if (${emitExpr(stmt.cond)}) ${emitStmt(stmt.cons, indent)}`;
            if (stmt.alt) {
                if (stmt.alt.tag === 'Si') {
                    code += ` else ${emitStmt(stmt.alt, indent)}`;
                } else {
                    code += ` else ${emitStmt(stmt.alt, indent)}`;
                }
            }
            return code;
        }

        case 'Dum':
            return `${indent}while (${emitExpr(stmt.cond)}) ${emitStmt(stmt.corpus, indent)}`;

        case 'FacDum':
            return `${indent}do ${emitStmt(stmt.corpus, indent)} while (${emitExpr(stmt.cond)});`;

        case 'Iteratio': {
            const kw = stmt.species === 'Ex' ? 'of' : 'in';
            const async = stmt.asynca ? 'await ' : '';
            return `${indent}for ${async}(const ${stmt.binding} ${kw} ${emitExpr(stmt.iter)}) ${emitStmt(stmt.corpus, indent)}`;
        }

        case 'Elige': {
            const lines: string[] = [];
            lines.push(`${indent}switch (${emitExpr(stmt.discrim)}) {`);
            for (const c of stmt.casus) {
                lines.push(`${indent}    case ${emitExpr(c.cond)}:`);
                lines.push(emitStmt(c.corpus, indent + '        '));
                lines.push(`${indent}        break;`);
            }
            if (stmt.default_) {
                lines.push(`${indent}    default:`);
                lines.push(emitStmt(stmt.default_, indent + '        '));
            }
            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Discerne': {
            // Pattern matching → switch on tag
            const lines: string[] = [];
            const numDiscrim = stmt.discrim.length;

            // For single discriminant, use expression directly; for multi, create temp vars
            const discrimVars: string[] = [];
            if (numDiscrim === 1) {
                discrimVars.push(emitExpr(stmt.discrim[0]));
            } else {
                for (let i = 0; i < numDiscrim; i++) {
                    const varName = `discriminant_${i}`;
                    discrimVars.push(varName);
                    lines.push(`${indent}const ${varName} = ${emitExpr(stmt.discrim[i])};`);
                }
            }

            // Switch on first discriminant's tag
            lines.push(`${indent}switch (${discrimVars[0]}.tag) {`);

            for (const c of stmt.casus) {
                const firstPattern = c.patterns[0];
                if (firstPattern.wildcard) {
                    lines.push(`${indent}    default: {`);
                } else {
                    lines.push(`${indent}    case '${firstPattern.variant}': {`);
                }

                // Extract bindings from ALL patterns
                for (let i = 0; i < c.patterns.length && i < numDiscrim; i++) {
                    const pattern = c.patterns[i];
                    const discrimVar = discrimVars[i];

                    if (pattern.alias) {
                        lines.push(`${indent}        const ${pattern.alias} = ${discrimVar};`);
                    }
                    for (const b of pattern.bindings) {
                        lines.push(`${indent}        const ${b} = ${discrimVar}.${b};`);
                    }
                }

                lines.push(emitStmt(c.corpus, indent + '        '));
                lines.push(`${indent}        break;`);
                lines.push(`${indent}    }`);
            }

            lines.push(`${indent}}`);
            return lines.join('\n');
        }

        case 'Custodi': {
            const lines: string[] = [];
            for (const c of stmt.clausulae) {
                lines.push(`${indent}if (${emitExpr(c.cond)}) ${emitStmt(c.corpus, indent)}`);
            }
            return lines.join('\n');
        }

        case 'Tempta': {
            let code = `${indent}try ${emitStmt(stmt.corpus, indent)}`;
            if (stmt.cape) {
                code += ` catch (${stmt.cape.param}) ${emitStmt(stmt.cape.corpus, indent)}`;
            }
            if (stmt.demum) {
                code += ` finally ${emitStmt(stmt.demum, indent)}`;
            }
            return code;
        }

        case 'Redde':
            return stmt.valor
                ? `${indent}return ${emitExpr(stmt.valor)};`
                : `${indent}return;`;

        case 'Iace': {
            const kw = stmt.fatale ? 'throw new Error' : 'throw';
            return stmt.fatale
                ? `${indent}${kw}(${emitExpr(stmt.arg)});`
                : `${indent}${kw} ${emitExpr(stmt.arg)};`;
        }

        case 'Scribe': {
            const method = stmt.gradus === 'Vide' ? 'debug' : stmt.gradus === 'Mone' ? 'warn' : 'log';
            const args = stmt.args.map(emitExpr).join(', ');
            return `${indent}console.${method}(${args});`;
        }

        case 'Adfirma': {
            const msg = stmt.msg ? `, ${emitExpr(stmt.msg)}` : '';
            return `${indent}console.assert(${emitExpr(stmt.cond)}${msg});`;
        }

        case 'Rumpe':
            return `${indent}break;`;

        case 'Perge':
            return `${indent}continue;`;

        case 'Incipit': {
            // Entry point → IIFE (sync or async)
            const async = stmt.asynca ? 'async ' : '';
            return `${indent}(${async}function() ${emitStmt(stmt.corpus, indent)})();`;
        }

        case 'Probandum': {
            const lines: string[] = [];
            lines.push(`${indent}describe(${JSON.stringify(stmt.nomen)}, () => {`);
            for (const s of stmt.corpus) {
                lines.push(emitStmt(s, indent + '    '));
            }
            lines.push(`${indent}});`);
            return lines.join('\n');
        }

        case 'Proba':
            return `${indent}it(${JSON.stringify(stmt.nomen)}, () => ${emitStmt(stmt.corpus, indent)});`;

        default:
            return `${indent}/* unhandled: ${(stmt as Stmt).tag} */`;
    }
}

function emitExpr(expr: Expr): string {
    switch (expr.tag) {
        case 'Nomen':
            return expr.valor;

        case 'Ego':
            return 'this';

        case 'Littera':
            switch (expr.species) {
                case 'Textus':
                    return JSON.stringify(expr.valor);
                case 'Verum':
                    return 'true';
                case 'Falsum':
                    return 'false';
                case 'Nihil':
                    return 'null';
                default:
                    return expr.valor;
            }

        case 'Binaria': {
            const op = BINARY_OPS[expr.signum] ?? expr.signum;
            return `(${emitExpr(expr.sin)} ${op} ${emitExpr(expr.dex)})`;
        }

        case 'Unaria': {
            const op = UNARY_OPS[expr.signum] ?? expr.signum;
            return `(${op}${emitExpr(expr.arg)})`;
        }

        case 'Assignatio':
            return `(${emitExpr(expr.sin)} ${expr.signum} ${emitExpr(expr.dex)})`;

        case 'Condicio':
            return `(${emitExpr(expr.cond)} ? ${emitExpr(expr.cons)} : ${emitExpr(expr.alt)})`;

        case 'Vocatio': {
            // Check if callee is a method call that needs translation
            if (expr.callee.tag === 'Membrum' && !expr.callee.computed) {
                const propName = expr.callee.prop.tag === 'Littera' ? expr.callee.prop.valor : null;
                if (propName) {
                    // Property-only access (e.g., .longitudo() → .length)
                    if (PROPERTY_ONLY.has(propName)) {
                        return emitExpr(expr.callee);
                    }
                    // Method name translation (only for method calls, not field access)
                    const translated = METHOD_MAP[propName];
                    if (translated) {
                        const obj = emitExpr(expr.callee.obj);
                        const access = expr.callee.nonNull ? '!.' : '.';
                        const args = expr.args.map(emitExpr).join(', ');
                        return `${obj}${access}${translated}(${args})`;
                    }
                }
            }
            const args = expr.args.map(emitExpr).join(', ');
            return `${emitExpr(expr.callee)}(${args})`;
        }

        case 'Membrum': {
            const obj = emitExpr(expr.obj);
            if (expr.computed) {
                return `${obj}[${emitExpr(expr.prop)}]`;
            }
            let prop = expr.prop.tag === 'Littera' ? expr.prop.valor : emitExpr(expr.prop);
            // Special property translations (templates)
            if (prop === 'primus') {
                return `${obj}[0]`;
            }
            if (prop === 'ultimus') {
                return `${obj}.at(-1)`;
            }
            // Only translate property-like names (longitudo), not method names
            // Method name translations are handled in Vocatio case
            if (PROPERTY_ONLY.has(prop)) {
                prop = METHOD_MAP[prop] ?? prop;
            }
            const access = expr.nonNull ? '!.' : '.';
            return `${obj}${access}${prop}`;
        }

        case 'Series': {
            const elems = expr.elementa.map(emitExpr).join(', ');
            return `[${elems}]`;
        }

        case 'Obiectum': {
            const props = expr.props.map(p => {
                if (p.shorthand) {
                    return p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key);
                }
                const key = p.computed
                    ? `[${emitExpr(p.key)}]`
                    : (p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key));
                return `${key}: ${emitExpr(p.valor)}`;
            }).join(', ');
            return `{ ${props} }`;
        }

        case 'Clausura': {
            const params = expr.params.map(p => p.typus ? `${p.nomen}: ${emitTypus(p.typus)}` : p.nomen).join(', ');
            if ('tag' in expr.corpus && expr.corpus.tag === 'Massa') {
                return `(${params}) => ${emitStmt(expr.corpus as Stmt)}`;
            }
            return `(${params}) => ${emitExpr(expr.corpus as Expr)}`;
        }

        case 'Novum': {
            const args = expr.args.map(emitExpr).join(', ');
            let code = `new ${emitExpr(expr.callee)}(${args})`;
            if (expr.init) {
                // Object.assign pattern for 'de' initializer
                code = `Object.assign(${code}, ${emitExpr(expr.init)})`;
            }
            return code;
        }

        case 'Cede':
            return `await ${emitExpr(expr.arg)}`;

        case 'Qua':
            return `(${emitExpr(expr.expr)} as ${emitTypus(expr.typus)})`;

        case 'Innatum':
            return `(${emitExpr(expr.expr)} as ${emitTypus(expr.typus)})`;

        case 'Finge': {
            const fields = expr.campi.map(p => {
                const key = p.key.tag === 'Littera' ? p.key.valor : emitExpr(p.key);
                return `${key}: ${emitExpr(p.valor)}`;
            }).join(', ');
            const cast = expr.typus ? ` as ${emitTypus(expr.typus)}` : '';
            return `{ tag: '${expr.variant}', ${fields} }${cast}`;
        }

        case 'Scriptum': {
            // scriptum("Hello, §!", name) → `Hello, ${name}!`
            const parts = expr.template.split('§');
            if (parts.length === 1) {
                return JSON.stringify(expr.template);
            }
            let result = '`';
            for (let i = 0; i < parts.length; i++) {
                result += parts[i].replace(/`/g, '\\`');
                if (i < expr.args.length) {
                    result += '${' + emitExpr(expr.args[i]) + '}';
                }
            }
            result += '`';
            return result;
        }

        case 'Ambitus': {
            // Range → array generation (simplified)
            const start = emitExpr(expr.start);
            const end = emitExpr(expr.end);
            if (expr.inclusive) {
                return `Array.from({ length: ${end} - ${start} + 1 }, (_, i) => ${start} + i)`;
            }
            return `Array.from({ length: ${end} - ${start} }, (_, i) => ${start} + i)`;
        }

        default:
            return `/* unhandled: ${(expr as Expr).tag} */`;
    }
}

function emitTypus(typus: Typus): string {
    switch (typus.tag) {
        case 'Nomen':
            return mapTypeName(typus.nomen);

        case 'Nullabilis':
            return `${emitTypus(typus.inner)} | null`;

        case 'Genericus':
            return `${mapTypeName(typus.nomen)}<${typus.args.map(emitTypus).join(', ')}>`;

        case 'Functio':
            return `(${typus.params.map((p, i) => `arg${i}: ${emitTypus(p)}`).join(', ')}) => ${emitTypus(typus.returns)}`;

        case 'Unio':
            return typus.members.map(emitTypus).join(' | ');

        case 'Litteralis':
            return typus.valor;

        default:
            return 'unknown';
    }
}

function mapTypeName(name: string): string {
    const MAP: Record<string, string> = {
        'textus': 'string',
        'numerus': 'number',
        'fractus': 'number',
        'bivalens': 'boolean',
        'nihil': 'null',
        'vacuum': 'void',
        'vacuus': 'void',
        'ignotum': 'unknown',
        'quodlibet': 'any',
        'quidlibet': 'any',
        'lista': 'Array',
        'tabula': 'Map',
        'collectio': 'Set',
        'copia': 'Set',
    };
    return MAP[name] ?? name;
}

function emitParam(param: Param): string {
    const rest = param.rest ? '...' : '';
    const typ = param.typus ? `: ${emitTypus(param.typus)}` : '';
    // For Nullabilis params without explicit default, use = null
    let def = '';
    if (param.default_) {
        def = ` = ${emitExpr(param.default_)}`;
    } else if (param.typus?.tag === 'Nullabilis') {
        def = ' = null';
    }
    return `${rest}${param.nomen}${typ}${def}`;
}
