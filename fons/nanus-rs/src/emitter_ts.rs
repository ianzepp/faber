use std::collections::HashMap;
use subsidia_rs::{
    ClausuraCorpus, Expr, LitteraSpecies, Modulus, Param, Stmt, Typus, VariaSpecies,
};

/// Emit a module to TypeScript source code.
pub fn emit_ts(module: &Modulus) -> String {
    let mut lines: Vec<String> = Vec::new();
    for stmt in &module.corpus {
        lines.push(emit_stmt(stmt, ""));
    }
    lines.join("\n")
}

fn emit_stmt(stmt: &Stmt, indent: &str) -> String {
    match stmt {
        Stmt::Massa { corpus, .. } => emit_massa(corpus, indent),
        Stmt::Expressia { expr, .. } => format!("{}{};", indent, emit_expr(expr)),
        Stmt::Varia { .. } => emit_varia(stmt, indent),
        Stmt::Functio { .. } => emit_functio(stmt, indent),
        Stmt::Genus { .. } => emit_genus(stmt, indent),
        Stmt::Pactum { .. } => emit_pactum(stmt, indent),
        Stmt::Ordo { .. } => emit_ordo(stmt, indent),
        Stmt::Discretio { .. } => emit_discretio(stmt, indent),
        Stmt::Importa { .. } => emit_importa(stmt, indent),
        Stmt::Si { cond, cons, alt, .. } => {
            let mut result = format!("{}if ({}) {}", indent, emit_expr(cond), emit_stmt(cons, indent));
            if let Some(a) = alt {
                result.push_str(" else ");
                result.push_str(&emit_stmt(a, indent));
            }
            result
        }
        Stmt::Dum { cond, corpus, .. } => {
            format!("{}while ({}) {}", indent, emit_expr(cond), emit_stmt(corpus, indent))
        }
        Stmt::FacDum { corpus, cond, .. } => {
            format!("{}do {} while ({});", indent, emit_stmt(corpus, indent), emit_expr(cond))
        }
        Stmt::Iteratio { binding, iter, corpus, asynca, species, .. } => {
            let kw = if species == "De" { "in" } else { "of" };
            let async_kw = if *asynca { "await " } else { "" };
            format!(
                "{}for {}(const {} {} {}) {}",
                indent, async_kw, binding, kw, emit_expr(iter), emit_stmt(corpus, indent)
            )
        }
        Stmt::Elige { discrim, casus, default, .. } => {
            let discrim_str = emit_expr(discrim);
            let mut lines: Vec<String> = Vec::new();
            for (i, c) in casus.iter().enumerate() {
                let kw = if i == 0 { "if" } else { "else if" };
                lines.push(format!(
                    "{}{} ({} === {}) {}",
                    indent, kw, discrim_str, emit_expr(&c.cond), emit_stmt(&c.corpus, indent)
                ));
            }
            if let Some(d) = default {
                lines.push(format!("{}else {}", indent, emit_stmt(d, indent)));
            }
            lines.join("\n")
        }
        Stmt::Discerne { discrim, casus, .. } => emit_discerne(discrim, casus, indent),
        Stmt::Custodi { clausulae, .. } => {
            let lines: Vec<String> = clausulae
                .iter()
                .map(|c| format!("{}if ({}) {}", indent, emit_expr(&c.cond), emit_stmt(&c.corpus, indent)))
                .collect();
            lines.join("\n")
        }
        Stmt::Tempta { corpus, cape, demum, .. } => {
            let mut result = format!("{}try {}", indent, emit_stmt(corpus, indent));
            if let Some(c) = cape {
                result.push_str(&format!(" catch ({}) {}", c.param, emit_stmt(&c.corpus, indent)));
            }
            if let Some(d) = demum {
                result.push_str(&format!(" finally {}", emit_stmt(d, indent)));
            }
            result
        }
        Stmt::Redde { valor, .. } => {
            if let Some(v) = valor {
                format!("{}return {};", indent, emit_expr(v))
            } else {
                format!("{}return;", indent)
            }
        }
        Stmt::Iace { arg, fatale, .. } => {
            if *fatale {
                format!("{}throw new Error({});", indent, emit_expr(arg))
            } else {
                format!("{}throw {};", indent, emit_expr(arg))
            }
        }
        Stmt::Rumpe { .. } => format!("{}break;", indent),
        Stmt::Perge { .. } => format!("{}continue;", indent),
        Stmt::Scribe { gradus, args, .. } => {
            let method = match gradus.as_str() {
                "Vide" => "debug",
                "Mone" => "warn",
                _ => "log",
            };
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}console.{}({});", indent, method, args_str.join(", "))
        }
        Stmt::Adfirma { cond, msg, .. } => {
            let mut result = format!("{}console.assert({}",  indent, emit_expr(cond));
            if let Some(m) = msg {
                result.push_str(&format!(", {}", emit_expr(m)));
            }
            result.push_str(");");
            result
        }
        Stmt::Incipit { corpus, asynca, .. } => {
            if *asynca {
                // Async: wrap in async IIFE
                format!("{}(async () => {})();", indent, emit_stmt(corpus, indent))
            } else {
                // Sync: emit body content directly (no wrapper)
                if let Stmt::Massa { corpus: stmts, .. } = corpus.as_ref() {
                    let lines: Vec<String> = stmts.iter().map(|s| emit_stmt(s, indent)).collect();
                    lines.join("\n")
                } else {
                    emit_stmt(corpus, indent)
                }
            }
        }
        Stmt::Probandum { nomen, corpus, .. } => {
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!("{}describe({}, () => {{", indent, quote_string(nomen)));
            for s in corpus {
                lines.push(emit_stmt(s, &format!("{}  ", indent)));
            }
            lines.push(format!("{}}});", indent));
            lines.join("\n")
        }
        Stmt::Proba { nomen, corpus, .. } => {
            format!("{}it({}, () => {});", indent, quote_string(nomen), emit_stmt(corpus, indent))
        }
    }
}

fn emit_massa(corpus: &[Stmt], indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("{".to_string());
    for stmt in corpus {
        lines.push(emit_stmt(stmt, &format!("{}  ", indent)));
    }
    lines.push(format!("{}}}", indent));
    lines.join("\n")
}

fn emit_varia(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Varia {
        species,
        nomen,
        typus,
        valor,
        publica,
        externa,
        ..
    } = stmt
    {
        let mut parts: Vec<&str> = Vec::new();
        if *publica {
            parts.push("export");
        }
        if *externa {
            parts.push("declare");
        }
        let kw = match species {
            VariaSpecies::Fixum | VariaSpecies::Figendum => "const",
            _ => "let",
        };
        parts.push(kw);

        let mut result = format!("{}{} {}", indent, parts.join(" "), nomen);
        if let Some(t) = typus {
            if *externa && matches!(t, Typus::Nomen { nomen } if nomen == "ignotum") {
                result.push_str(": any");
            } else {
                result.push_str(&format!(": {}", emit_typus(t)));
            }
        }
        if let Some(v) = valor {
            if !*externa {
                result.push_str(&format!(" = {}", emit_expr(v)));
            }
        }
        result.push(';');
        result
    } else {
        String::new()
    }
}

fn emit_functio(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Functio {
        nomen,
        params,
        typus_reditus,
        corpus,
        asynca,
        publica,
        externa,
        generics,
        ..
    } = stmt
    {
        let mut parts: Vec<&str> = Vec::new();
        if *publica {
            parts.push("export");
        }
        if *externa {
            parts.push("declare");
        }
        if *asynca {
            parts.push("async");
        }
        parts.push("function");

        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let params_str: Vec<String> = params.iter().map(emit_param).collect();
        let ret = if let Some(t) = typus_reditus {
            format!(": {}", emit_typus(t))
        } else {
            String::new()
        };

        let body = if let Some(c) = corpus {
            if !*externa {
                format!(" {}", emit_stmt(c, indent))
            } else {
                ";".to_string()
            }
        } else {
            ";".to_string()
        };

        format!(
            "{}{} {}{}({}){}{}",
            indent,
            parts.join(" "),
            nomen,
            generics_str,
            params_str.join(", "),
            ret,
            body
        )
    } else {
        String::new()
    }
}

fn emit_genus(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Genus {
        nomen,
        campi,
        methodi,
        implet,
        generics,
        publica,
        ..
    } = stmt
    {
        let exp = if *publica { "export " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };
        let impl_str = if implet.is_empty() {
            String::new()
        } else {
            format!(" implements {}", implet.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}{}class {}{}{} {{", indent, exp, nomen, generics_str, impl_str));

        // Fields
        for c in campi {
            let vis = if c.visibilitas == "Protecta" { "protected" } else { "private" };
            let typ = if let Some(t) = &c.typus {
                format!(": {}", emit_typus(t))
            } else {
                String::new()
            };
            let val = if let Some(v) = &c.valor {
                format!(" = {}", emit_expr(v))
            } else {
                String::new()
            };
            lines.push(format!("{}  {} {}{}{};", indent, vis, c.nomen, typ, val));
        }

        // Auto-generate constructor if there are fields
        if !campi.is_empty() {
            lines.push(String::new());
            let override_fields: Vec<String> = campi
                .iter()
                .map(|c| {
                    let typ = if let Some(t) = &c.typus {
                        emit_typus(t)
                    } else {
                        "any".to_string()
                    };
                    format!("{}?: {}", c.nomen, typ)
                })
                .collect();
            lines.push(format!("{}  constructor(overrides: {{ {} }} = {{}}) {{", indent, override_fields.join(", ")));
            for c in campi {
                lines.push(format!(
                    "{}    if (overrides.{} !== undefined) {{ this.{} = overrides.{}; }}",
                    indent, c.nomen, c.nomen, c.nomen
                ));
            }
            lines.push(format!("{}  }}", indent));
        }

        // Methods
        for m in methodi {
            if let Stmt::Functio {
                nomen: method_name,
                params,
                typus_reditus,
                corpus,
                asynca,
                publica: method_pub,
                ..
            } = m
            {
                lines.push(String::new());
                let vis = if *method_pub { "" } else { "private " };
                let async_kw = if *asynca { "async " } else { "" };
                let params_str: Vec<String> = params.iter().map(emit_param).collect();
                let ret = if let Some(t) = typus_reditus {
                    format!(": {}", emit_typus(t))
                } else {
                    String::new()
                };
                let body = if let Some(c) = corpus {
                    format!(" {}", emit_stmt(c, &format!("{}  ", indent)))
                } else {
                    ";".to_string()
                };
                lines.push(format!(
                    "{}  {}{}{}({}){}{}",
                    indent, vis, async_kw, method_name, params_str.join(", "), ret, body
                ));
            }
        }

        lines.push(format!("{}}}", indent));
        lines.join("\n")
    } else {
        String::new()
    }
}

fn emit_pactum(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Pactum {
        nomen,
        methodi,
        generics,
        publica,
        ..
    } = stmt
    {
        let exp = if *publica { "export " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}{}interface {}{} {{", indent, exp, nomen, generics_str));

        for m in methodi {
            let params_str: Vec<String> = m.params.iter().map(emit_param).collect();
            let ret = if let Some(t) = &m.typus_reditus {
                format!(": {}", emit_typus(t))
            } else {
                String::new()
            };
            lines.push(format!("{}  {}({}){};", indent, m.nomen, params_str.join(", "), ret));
        }

        lines.push(format!("{}}}", indent));
        lines.join("\n")
    } else {
        String::new()
    }
}

fn emit_ordo(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Ordo {
        nomen,
        membra,
        publica,
        ..
    } = stmt
    {
        let exp = if *publica { "export " } else { "" };
        let members: Vec<String> = membra
            .iter()
            .map(|m| {
                if let Some(v) = &m.valor {
                    format!("{} = {}", m.nomen, v)
                } else {
                    m.nomen.clone()
                }
            })
            .collect();
        format!("{}{}enum {} {{ {} }}", indent, exp, nomen, members.join(", "))
    } else {
        String::new()
    }
}

fn emit_discretio(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Discretio {
        nomen,
        variantes,
        generics,
        publica,
        ..
    } = stmt
    {
        let exp = if *publica { "export " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();
        let mut variant_names: Vec<String> = Vec::new();

        for v in variantes {
            variant_names.push(v.nomen.clone());
            if v.campi.is_empty() {
                lines.push(format!("{}{}type {} = {{ tag: '{}' }};", indent, exp, v.nomen, v.nomen));
            } else {
                let fields: Vec<String> = v
                    .campi
                    .iter()
                    .map(|f| format!("{}: {}", f.nomen, emit_typus(&f.typus)))
                    .collect();
                lines.push(format!(
                    "{}{}type {} = {{ tag: '{}'; {} }};",
                    indent, exp, v.nomen, v.nomen, fields.join("; ")
                ));
            }
        }

        lines.push(format!(
            "{}{}type {}{} = {};",
            indent, exp, nomen, generics_str, variant_names.join(" | ")
        ));
        lines.join("\n")
    } else {
        String::new()
    }
}

fn emit_importa(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Importa {
        fons,
        specs,
        totum,
        alias,
        ..
    } = stmt
    {
        if *totum {
            if let Some(a) = alias {
                format!("{}import * as {} from \"{}\";", indent, a, fons)
            } else {
                format!("{}import * from \"{}\";", indent, fons)
            }
        } else {
            let specs_str: Vec<String> = specs
                .iter()
                .map(|s| {
                    if !s.local.is_empty() && s.local != s.imported {
                        format!("{} as {}", s.imported, s.local)
                    } else {
                        s.imported.clone()
                    }
                })
                .collect();
            format!("{}import {{ {} }} from \"{}\";", indent, specs_str.join(", "), fons)
        }
    } else {
        String::new()
    }
}

fn emit_discerne(discrim: &[Expr], casus: &[subsidia_rs::DiscerneCasus], indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let num_discrim = discrim.len();

    // For single discriminant, use expression directly; for multi, create temp vars
    let discrim_vars: Vec<String> = if num_discrim == 1 {
        vec![emit_expr(&discrim[0])]
    } else {
        discrim
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let var_name = format!("discriminant_{}", i);
                lines.push(format!("{}const {} = {};", indent, var_name, emit_expr(d)));
                var_name
            })
            .collect()
    };

    for (ci, c) in casus.iter().enumerate() {
        let first_pattern = &c.patterns[0];
        let kw = if ci == 0 { "if" } else { "else if" };

        if first_pattern.wildcard {
            lines.push(format!("{}else {{", indent));
        } else {
            lines.push(format!(
                "{}{} ({}.tag === '{}') {{",
                indent, kw, discrim_vars[0], first_pattern.variant
            ));
        }

        // Extract bindings from patterns
        for (i, pattern) in c.patterns.iter().enumerate() {
            if i >= num_discrim {
                break;
            }
            let discrim_var = &discrim_vars[i];

            if let Some(alias) = &pattern.alias {
                lines.push(format!("{}  const {} = {};", indent, alias, discrim_var));
            }
            for b in &pattern.bindings {
                lines.push(format!("{}  const {} = {}.{};", indent, b, discrim_var, b));
            }
        }

        // Emit body contents (unwrap Massa if present)
        if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
            for s in stmts {
                lines.push(emit_stmt(s, &format!("{}  ", indent)));
            }
        } else {
            lines.push(emit_stmt(&c.corpus, &format!("{}  ", indent)));
        }
        lines.push(format!("{}}}", indent));
    }

    lines.join("\n")
}

fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Nomen { valor, .. } => valor.clone(),
        Expr::Ego { .. } => "this".to_string(),
        Expr::Littera { species, valor, .. } => match species {
            LitteraSpecies::Textus => quote_string(valor),
            LitteraSpecies::Verum => "true".to_string(),
            LitteraSpecies::Falsum => "false".to_string(),
            LitteraSpecies::Nihil => "null".to_string(),
            _ => valor.clone(),
        },
        Expr::Binaria { signum, sin, dex, .. } => {
            let op = map_binary_op(signum);
            format!("({} {} {})", emit_expr(sin), op, emit_expr(dex))
        }
        Expr::Unaria { signum, arg, .. } => {
            let op = map_unary_op(signum);
            format!("({}{})", op, emit_expr(arg))
        }
        Expr::Assignatio { signum, sin, dex, .. } => {
            format!("{} {} {}", emit_expr(sin), signum, emit_bare_expr(dex))
        }
        Expr::Condicio { cond, cons, alt, .. } => {
            format!("({} ? {} : {})", emit_expr(cond), emit_expr(cons), emit_expr(alt))
        }
        Expr::Vocatio { callee, args, .. } => {
            // Check for method name translation
            if let Expr::Membrum { obj, prop, computed, non_null, .. } = callee.as_ref() {
                if !computed {
                    if let Expr::Littera { valor: prop_name, .. } = prop.as_ref() {
                        // Check if it's a property-only access (longitudo)
                        if is_property_only(prop_name) {
                            return emit_expr(callee);
                        }
                        // Check for method translation
                        if let Some(translated) = map_method_name(prop_name) {
                            let obj_str = emit_expr(obj);
                            let access = if *non_null { "!." } else { "." };
                            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
                            return format!("{}{}{}({})", obj_str, access, translated, args_str.join(", "));
                        }
                    }
                }
            }
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}({})", emit_expr(callee), args_str.join(", "))
        }
        Expr::Membrum { obj, prop, computed, non_null, .. } => {
            let obj_str = emit_expr(obj);
            if *computed {
                return format!("{}[{}]", obj_str, emit_expr(prop));
            }
            let prop_str = if let Expr::Littera { valor, .. } = prop.as_ref() {
                valor.clone()
            } else {
                emit_expr(prop)
            };
            // Special property translations
            if prop_str == "primus" {
                return format!("{}[0]", obj_str);
            }
            if prop_str == "ultimus" {
                return format!("{}.at(-1)", obj_str);
            }
            let translated = if is_property_only(&prop_str) {
                map_method_name(&prop_str).unwrap_or(&prop_str).to_string()
            } else {
                prop_str
            };
            let access = if *non_null { "!." } else { "." };
            format!("{}{}{}", obj_str, access, translated)
        }
        Expr::Series { elementa, .. } => {
            let items: Vec<String> = elementa.iter().map(emit_expr).collect();
            format!("[{}]", items.join(", "))
        }
        Expr::Obiectum { props, .. } => {
            let pairs: Vec<String> = props
                .iter()
                .map(|p| {
                    if p.shorthand {
                        if let Expr::Littera { valor, .. } = &p.key {
                            valor.clone()
                        } else {
                            emit_expr(&p.key)
                        }
                    } else {
                        let key = if p.computed {
                            format!("[{}]", emit_expr(&p.key))
                        } else if let Expr::Littera { valor, .. } = &p.key {
                            valor.clone()
                        } else {
                            emit_expr(&p.key)
                        };
                        format!("{}: {}", key, emit_expr(&p.valor))
                    }
                })
                .collect();
            format!("{{ {} }}", pairs.join(", "))
        }
        Expr::Clausura { params, corpus, .. } => {
            let params_str: Vec<String> = params
                .iter()
                .map(|p| {
                    if let Some(t) = &p.typus {
                        format!("{}: {}", p.nomen, emit_typus(t))
                    } else {
                        p.nomen.clone()
                    }
                })
                .collect();
            let body = match corpus {
                ClausuraCorpus::Stmt(s) => emit_stmt(s, ""),
                ClausuraCorpus::Expr(e) => emit_expr(e),
            };
            format!("({}) => {}", params_str.join(", "), body)
        }
        Expr::Novum { callee, args, init, .. } => {
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            let mut result = format!("new {}({})", emit_expr(callee), args_str.join(", "));
            if let Some(i) = init {
                result = format!("Object.assign({}, {})", result, emit_expr(i));
            }
            result
        }
        Expr::Cede { arg, .. } => format!("await {}", emit_expr(arg)),
        Expr::Qua { expr, typus, .. } => {
            format!("({} as {})", emit_expr(expr), emit_typus(typus))
        }
        Expr::Innatum { expr, typus, .. } => {
            format!("({} as {})", emit_expr(expr), emit_typus(typus))
        }
        Expr::PostfixNovum { expr, typus, .. } => {
            format!("new {}({})", emit_typus(typus), emit_expr(expr))
        }
        Expr::Finge { variant, campi, typus, .. } => {
            let fields: Vec<String> = campi
                .iter()
                .map(|p| {
                    let key = if let Expr::Littera { valor, .. } = &p.key {
                        valor.clone()
                    } else {
                        emit_expr(&p.key)
                    };
                    format!("{}: {}", key, emit_expr(&p.valor))
                })
                .collect();
            let cast = if let Some(t) = typus {
                format!(" as {}", emit_typus(t))
            } else {
                String::new()
            };
            format!("{{ tag: '{}', {} }}{}", variant, fields.join(", "), cast)
        }
        Expr::Scriptum { template, args, .. } => {
            let parts: Vec<&str> = template.split('§').collect();
            if parts.len() == 1 {
                return quote_string(template);
            }
            let mut result = String::from("`");
            for (i, part) in parts.iter().enumerate() {
                result.push_str(&part.replace('`', "\\`"));
                if i < args.len() {
                    result.push_str(&format!("${{{}}}", emit_expr(&args[i])));
                }
            }
            result.push('`');
            result
        }
        Expr::Ambitus { start, end, inclusive, .. } => {
            let start_str = emit_expr(start);
            let end_str = emit_expr(end);
            if *inclusive {
                format!(
                    "Array.from({{ length: {} - {} + 1 }}, (_, i) => {} + i)",
                    end_str, start_str, start_str
                )
            } else {
                format!(
                    "Array.from({{ length: {} - {} }}, (_, i) => {} + i)",
                    end_str, start_str, start_str
                )
            }
        }
    }
}

/// Emit expression without outer parens on binary expressions.
fn emit_bare_expr(expr: &Expr) -> String {
    if let Expr::Binaria { signum, sin, dex, .. } = expr {
        let op = map_binary_op(signum);
        format!("{} {} {}", emit_bare_expr(sin), op, emit_bare_expr(dex))
    } else {
        emit_expr(expr)
    }
}

fn emit_typus(typus: &Typus) -> String {
    match typus {
        Typus::Nomen { nomen } => map_type_name(nomen),
        Typus::Nullabilis { inner } => format!("{} | null", emit_typus(inner)),
        Typus::Genericus { nomen, args } => {
            let args_str: Vec<String> = args.iter().map(emit_typus).collect();
            format!("{}<{}>", map_type_name(nomen), args_str.join(", "))
        }
        Typus::Functio { params, returns } => {
            let params_str: Vec<String> = params
                .iter()
                .enumerate()
                .map(|(i, p)| format!("arg{}: {}", i, emit_typus(p)))
                .collect();
            let ret = if let Some(r) = returns {
                emit_typus(r)
            } else {
                "void".to_string()
            };
            format!("({}) => {}", params_str.join(", "), ret)
        }
        Typus::Unio { members } => {
            let parts: Vec<String> = members.iter().map(emit_typus).collect();
            parts.join(" | ")
        }
        Typus::Litteralis { valor } => valor.clone(),
    }
}

fn emit_param(p: &Param) -> String {
    let rest = if p.rest { "..." } else { "" };
    let typ = if let Some(t) = &p.typus {
        format!(": {}", emit_typus(t))
    } else {
        String::new()
    };
    let def = if let Some(d) = &p.default {
        format!(" = {}", emit_expr(d))
    } else if let Some(Typus::Nullabilis { .. }) = &p.typus {
        " = null".to_string()
    } else {
        String::new()
    };
    format!("{}{}{}{}", rest, p.nomen, typ, def)
}

fn map_type_name(name: &str) -> String {
    match name {
        "textus" => "string",
        "numerus" => "number",
        "fractus" => "number",
        "decimus" => "number",
        "magnus" => "bigint",
        "bivalens" => "boolean",
        "nihil" => "null",
        "vacuum" | "vacuus" => "void",
        "ignotum" => "unknown",
        "quodlibet" | "quidlibet" => "any",
        "lista" => "Array",
        "tabula" => "Map",
        "collectio" | "copia" => "Set",
        _ => name,
    }
    .to_string()
}

fn map_binary_op(op: &str) -> &'static str {
    match op {
        "et" => "&&",
        "aut" => "||",
        "vel" => "??",
        "inter" => "in",
        "intra" => "instanceof",
        _ => {
            // Return the original op for standard operators (+, -, *, /, ==, etc.)
            // This is a bit hacky but works for our purposes
            match op {
                "+" => "+",
                "-" => "-",
                "*" => "*",
                "/" => "/",
                "%" => "%",
                "==" => "==",
                "!=" => "!=",
                "<" => "<",
                ">" => ">",
                "<=" => "<=",
                ">=" => ">=",
                _ => "/* unknown op */",
            }
        }
    }
}

fn map_unary_op(op: &str) -> &'static str {
    match op {
        "non" => "!",
        "nihil" => "!",
        "nonnihil" => "!!",
        "positivum" => "+",
        "-" => "-",
        "!" => "!",
        _ => "/* unknown op */",
    }
}

fn map_method_name(name: &str) -> Option<&'static str> {
    let map: HashMap<&str, &str> = [
        ("adde", "push"),
        ("praepone", "unshift"),
        ("remove", "pop"),
        ("decapita", "shift"),
        ("coniunge", "join"),
        ("continet", "includes"),
        ("indiceDe", "indexOf"),
        ("inveni", "find"),
        ("inveniIndicem", "findIndex"),
        ("omnes", "every"),
        ("aliquis", "some"),
        ("filtrata", "filter"),
        ("mappata", "map"),
        ("explanata", "flatMap"),
        ("plana", "flat"),
        ("sectio", "slice"),
        ("reducta", "reduce"),
        ("perambula", "forEach"),
        ("inverte", "reverse"),
        ("ordina", "sort"),
        ("pone", "set"),
        ("accipe", "get"),
        ("habet", "has"),
        ("dele", "delete"),
        ("purga", "clear"),
        ("claves", "keys"),
        ("valores", "values"),
        ("paria", "entries"),
        ("initium", "startsWith"),
        ("finis", "endsWith"),
        ("maiuscula", "toUpperCase"),
        ("minuscula", "toLowerCase"),
        ("recide", "trim"),
        ("divide", "split"),
        ("muta", "replaceAll"),
        ("longitudo", "length"),
    ]
    .iter()
    .cloned()
    .collect();
    map.get(name).copied()
}

fn is_property_only(name: &str) -> bool {
    matches!(name, "longitudo" | "primus" | "ultimus")
}

fn quote_string(s: &str) -> String {
    let mut result = String::from("\"");
    for c in s.chars() {
        match c {
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}
