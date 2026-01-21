use std::collections::HashMap;
use subsidia_rs::{
    ClausuraCorpus, Expr, LitteraSpecies, Modulus, Param, Stmt, Typus, VariaSpecies,
};

/// Emit a module to Rust source code.
pub fn emit_rs(module: &Modulus) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Add common imports
    lines.push("use std::collections::HashMap;".to_string());
    lines.push(String::new());

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
            let mut result = format!("{}if {} {}", indent, emit_expr(cond), emit_stmt(cons, indent));
            if let Some(a) = alt {
                result.push_str(" else ");
                result.push_str(&emit_stmt(a, indent));
            }
            result
        }
        Stmt::Dum { cond, corpus, .. } => {
            format!("{}while {} {}", indent, emit_expr(cond), emit_stmt(corpus, indent))
        }
        Stmt::FacDum { corpus, cond, .. } => {
            // Rust doesn't have do-while, emulate with loop + break
            let body = emit_stmt(corpus, &format!("{}    ", indent));
            format!("{}loop {{\n{}\n{}    if !({}) {{ break; }}\n{}}}", indent, body, indent, emit_expr(cond), indent)
        }
        Stmt::Iteratio { binding, iter, corpus, species, .. } => {
            let iter_method = if species == "De" { ".iter()" } else { ".into_iter()" };
            format!(
                "{}for {} in {}{} {}",
                indent, binding, emit_expr(iter), iter_method, emit_stmt(corpus, indent)
            )
        }
        Stmt::Elige { discrim, casus, default, .. } => {
            let discrim_str = emit_expr(discrim);
            let mut lines: Vec<String> = Vec::new();
            for (i, c) in casus.iter().enumerate() {
                let kw = if i == 0 { "if" } else { "else if" };
                lines.push(format!(
                    "{}{} {} == {} {}",
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
                .map(|c| format!("{}if {} {}", indent, emit_expr(&c.cond), emit_stmt(&c.corpus, indent)))
                .collect();
            lines.join("\n")
        }
        Stmt::Tempta { corpus, cape, demum, .. } => {
            // Rust doesn't have try-catch, emit as comment + body
            let mut lines: Vec<String> = Vec::new();
            lines.push(format!("{}// try", indent));
            lines.push(emit_stmt(corpus, indent));
            if let Some(c) = cape {
                lines.push(format!("{}// catch({})", indent, c.param));
                lines.push(emit_stmt(&c.corpus, indent));
            }
            if let Some(d) = demum {
                lines.push(format!("{}// finally", indent));
                lines.push(emit_stmt(d, indent));
            }
            lines.join("\n")
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
                format!("{}panic!(\"{{}}\", {});", indent, emit_expr(arg))
            } else {
                format!("{}return Err({});", indent, emit_expr(arg))
            }
        }
        Stmt::Rumpe { .. } => format!("{}break;", indent),
        Stmt::Perge { .. } => format!("{}continue;", indent),
        Stmt::Scribe { gradus, args, .. } => {
            let macro_name = match gradus.as_str() {
                "Mone" => "eprintln!",
                _ => "println!",
            };
            if args.is_empty() {
                format!("{}{}();", indent, macro_name)
            } else {
                let format_str = args.iter().map(|_| "{}").collect::<Vec<_>>().join(" ");
                let args_str: Vec<String> = args.iter().map(emit_expr).collect();
                format!("{}{}(\"{}\", {});", indent, macro_name, format_str, args_str.join(", "))
            }
        }
        Stmt::Adfirma { cond, msg, .. } => {
            if let Some(m) = msg {
                format!("{}assert!({}, {});", indent, emit_expr(cond), emit_expr(m))
            } else {
                format!("{}assert!({});", indent, emit_expr(cond))
            }
        }
        Stmt::Incipit { corpus, asynca, .. } => {
            if *asynca {
                format!("{}// async entry point\n{}tokio::runtime::Runtime::new().unwrap().block_on(async {});", indent, indent, emit_stmt(corpus, indent))
            } else {
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
            lines.push(format!("{}#[cfg(test)]", indent));
            lines.push(format!("{}mod {} {{", indent, sanitize_ident(nomen)));
            lines.push(format!("{}    use super::*;", indent));
            for s in corpus {
                lines.push(emit_stmt(s, &format!("{}    ", indent)));
            }
            lines.push(format!("{}}}", indent));
            lines.join("\n")
        }
        Stmt::Proba { nomen, corpus, .. } => {
            format!("{}#[test]\n{}fn {}() {}", indent, indent, sanitize_ident(nomen), emit_stmt(corpus, indent))
        }
    }
}

fn emit_massa(corpus: &[Stmt], indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("{".to_string());
    for stmt in corpus {
        lines.push(emit_stmt(stmt, &format!("{}    ", indent)));
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
        if *externa {
            return format!("{}// extern: {}", indent, nomen);
        }

        let vis = if *publica { "pub " } else { "" };
        // At module level (empty indent), use static/const; inside functions, use let
        let is_module_level = indent.is_empty();
        let kw = match (species, is_module_level) {
            (VariaSpecies::Fixum | VariaSpecies::Figendum, true) => "static",
            (VariaSpecies::Fixum | VariaSpecies::Figendum, false) => "let",
            (_, true) => "static mut",
            (_, false) => "let mut",
        };

        let mut result = format!("{}{}{} {}", indent, vis, kw, nomen);
        if let Some(t) = typus {
            result.push_str(&format!(": {}", emit_typus(t)));
        }
        if let Some(v) = valor {
            result.push_str(&format!(" = {}", emit_expr(v)));
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
        if *externa {
            return format!("{}// extern: {}", indent, nomen);
        }

        let vis = if *publica { "pub " } else { "" };
        let async_kw = if *asynca { "async " } else { "" };

        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let params_str: Vec<String> = params.iter().map(emit_param).collect();
        let ret = if let Some(t) = typus_reditus {
            format!(" -> {}", emit_typus(t))
        } else {
            String::new()
        };

        let body = if let Some(c) = corpus {
            format!(" {}", emit_stmt(c, indent))
        } else {
            ";".to_string()
        };

        format!(
            "{}{}{}fn {}{}({}){}{}",
            indent, vis, async_kw, nomen, generics_str,
            params_str.join(", "), ret, body
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
        generics,
        publica,
        ..
    } = stmt
    {
        let vis = if *publica { "pub " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();

        // Derive common traits
        lines.push(format!("{}#[derive(Debug, Clone)]", indent));
        lines.push(format!("{}{}struct {}{} {{", indent, vis, nomen, generics_str));

        // Fields
        for c in campi {
            let field_vis = match c.visibilitas.as_str() {
                "Publica" => "pub ",
                _ => "",
            };
            let typ = if let Some(t) = &c.typus {
                emit_typus(t)
            } else {
                "()".to_string()
            };
            lines.push(format!("{}    {}{}: {},", indent, field_vis, c.nomen, typ));
        }
        lines.push(format!("{}}}", indent));

        // Impl block for methods
        if !methodi.is_empty() {
            lines.push(String::new());
            lines.push(format!("{}impl{} {}{} {{", indent, generics_str, nomen, generics_str));

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
                    let method_vis = if *method_pub { "pub " } else { "" };
                    let async_kw = if *asynca { "async " } else { "" };

                    // Add &self as first param for methods
                    let mut all_params: Vec<String> = vec!["&self".to_string()];
                    all_params.extend(params.iter().map(emit_param));

                    let ret = if let Some(t) = typus_reditus {
                        format!(" -> {}", emit_typus(t))
                    } else {
                        String::new()
                    };
                    let body = if let Some(c) = corpus {
                        format!(" {}", emit_stmt(c, &format!("{}    ", indent)))
                    } else {
                        " {}".to_string()
                    };
                    lines.push(format!(
                        "{}    {}{}fn {}({}){}{}",
                        indent, method_vis, async_kw, method_name, all_params.join(", "), ret, body
                    ));
                }
            }
            lines.push(format!("{}}}", indent));
        }

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
        let vis = if *publica { "pub " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}{}trait {}{} {{", indent, vis, nomen, generics_str));

        for m in methodi {
            let params_str: Vec<String> = m.params.iter().map(emit_param).collect();
            let mut all_params: Vec<String> = vec!["&self".to_string()];
            all_params.extend(params_str);

            let ret = if let Some(t) = &m.typus_reditus {
                format!(" -> {}", emit_typus(t))
            } else {
                String::new()
            };
            lines.push(format!("{}    fn {}({}){};", indent, m.nomen, all_params.join(", "), ret));
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
        let vis = if *publica { "pub " } else { "" };
        let mut lines: Vec<String> = Vec::new();

        lines.push(format!("{}#[derive(Debug, Clone, Copy, PartialEq, Eq)]", indent));
        lines.push(format!("{}{}enum {} {{", indent, vis, nomen));

        for m in membra {
            if let Some(v) = &m.valor {
                lines.push(format!("{}    {} = {},", indent, m.nomen, v));
            } else {
                lines.push(format!("{}    {},", indent, m.nomen));
            }
        }

        lines.push(format!("{}}}", indent));
        lines.join("\n")
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
        let vis = if *publica { "pub " } else { "" };
        let generics_str = if generics.is_empty() {
            String::new()
        } else {
            format!("<{}>", generics.join(", "))
        };

        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("{}#[derive(Debug, Clone)]", indent));
        lines.push(format!("{}{}enum {}{} {{", indent, vis, nomen, generics_str));

        for v in variantes {
            if v.campi.is_empty() {
                lines.push(format!("{}    {},", indent, v.nomen));
            } else {
                let fields: Vec<String> = v
                    .campi
                    .iter()
                    .map(|f| format!("{}: {}", f.nomen, emit_typus(&f.typus)))
                    .collect();
                lines.push(format!("{}    {} {{ {} }},", indent, v.nomen, fields.join(", ")));
            }
        }

        lines.push(format!("{}}}", indent));
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
        // Convert module path to Rust use statement
        let path = fons.replace("/", "::").replace(".fab", "");

        if *totum {
            if let Some(a) = alias {
                format!("{}use {} as {};", indent, path, a)
            } else {
                format!("{}use {}::*;", indent, path)
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
            format!("{}use {}::{{{}}};", indent, path, specs_str.join(", "))
        }
    } else {
        String::new()
    }
}

fn emit_discerne(discrim: &[Expr], casus: &[subsidia_rs::DiscerneCasus], indent: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Single discriminant - use match directly
    if discrim.len() == 1 {
        lines.push(format!("{}match {} {{", indent, emit_expr(&discrim[0])));

        for c in casus {
            let pattern = &c.patterns[0];

            if pattern.wildcard {
                lines.push(format!("{}    _ => {{", indent));
            } else if pattern.bindings.is_empty() && pattern.alias.is_none() {
                lines.push(format!("{}    {}::{} => {{", indent, get_enum_name(&discrim[0]), pattern.variant));
            } else {
                // Build pattern with bindings/alias
                let mut binding_parts: Vec<String> = pattern.bindings.iter().cloned().collect();
                if let Some(alias) = &pattern.alias {
                    // In Rust, we bind the whole variant with @ pattern
                    if binding_parts.is_empty() {
                        lines.push(format!("{}    {} @ {}::{} {{ .. }} => {{", indent, alias, get_enum_name(&discrim[0]), pattern.variant));
                        // Skip the normal line push below
                        if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                            for s in stmts {
                                lines.push(emit_stmt(s, &format!("{}        ", indent)));
                            }
                        } else {
                            lines.push(emit_stmt(&c.corpus, &format!("{}        ", indent)));
                        }
                        lines.push(format!("{}    }}", indent));
                        continue;
                    }
                }
                let binding_str = format!("{{ {} }}", binding_parts.join(", "));
                lines.push(format!("{}    {}::{} {} => {{", indent, get_enum_name(&discrim[0]), pattern.variant, binding_str));
            }

            // Emit body contents
            if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                for s in stmts {
                    lines.push(emit_stmt(s, &format!("{}        ", indent)));
                }
            } else {
                lines.push(emit_stmt(&c.corpus, &format!("{}        ", indent)));
            }
            lines.push(format!("{}    }}", indent));
        }

        lines.push(format!("{}}}", indent));
    } else {
        // Multiple discriminants - use tuple match
        let discrim_tuple: Vec<String> = discrim.iter().map(emit_expr).collect();
        lines.push(format!("{}match ({}) {{", indent, discrim_tuple.join(", ")));

        for c in casus {
            let patterns: Vec<String> = c.patterns.iter().map(|p| {
                if p.wildcard {
                    "_".to_string()
                } else {
                    format!("{}::{}", get_enum_name(&discrim[0]), p.variant)
                }
            }).collect();

            lines.push(format!("{}    ({}) => {{", indent, patterns.join(", ")));

            if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                for s in stmts {
                    lines.push(emit_stmt(s, &format!("{}        ", indent)));
                }
            } else {
                lines.push(emit_stmt(&c.corpus, &format!("{}        ", indent)));
            }
            lines.push(format!("{}    }}", indent));
        }

        lines.push(format!("{}}}", indent));
    }

    lines.join("\n")
}

fn get_enum_name(expr: &Expr) -> String {
    // Try to extract type name from expression for match patterns
    if let Expr::Nomen { valor, .. } = expr {
        // Capitalize first letter for enum name convention
        let mut chars = valor.chars();
        match chars.next() {
            None => String::new(),
            Some(c) => c.to_uppercase().chain(chars).collect(),
        }
    } else {
        "Unknown".to_string()
    }
}

fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Nomen { valor, .. } => valor.clone(),
        Expr::Ego { .. } => "self".to_string(),
        Expr::Littera { species, valor, .. } => match species {
            LitteraSpecies::Textus => format!("{}.to_string()", quote_string(valor)),
            LitteraSpecies::Verum => "true".to_string(),
            LitteraSpecies::Falsum => "false".to_string(),
            LitteraSpecies::Nihil => "None".to_string(),
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
            format!("{} {} {}", emit_expr(sin), signum, emit_expr(dex))
        }
        Expr::Condicio { cond, cons, alt, .. } => {
            format!("if {} {{ {} }} else {{ {} }}", emit_expr(cond), emit_expr(cons), emit_expr(alt))
        }
        Expr::Vocatio { callee, args, .. } => {
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();

            // Check for method calls that need translation
            if let Expr::Membrum { obj, prop, .. } = callee.as_ref() {
                if let Expr::Littera { valor: prop_name, .. } = prop.as_ref() {
                    // Properties that become methods in Rust (already include parens)
                    if prop_name == "longitudo" {
                        return format!("{}.len()", emit_expr(obj));
                    }
                    if let Some(translated) = map_method_name(prop_name) {
                        return format!("{}.{}({})", emit_expr(obj), translated, args_str.join(", "));
                    }
                }
            }

            format!("{}({})", emit_expr(callee), args_str.join(", "))
        }
        Expr::Membrum { obj, prop, computed, .. } => {
            let obj_str = emit_expr(obj);
            if *computed {
                return format!("{}[{}]", obj_str, emit_expr(prop));
            }
            let prop_str = if let Expr::Littera { valor, .. } = prop.as_ref() {
                // Translate property names
                if valor == "longitudo" {
                    return format!("{}.len()", obj_str);
                }
                if valor == "primus" {
                    return format!("{}[0]", obj_str);
                }
                if valor == "ultimus" {
                    return format!("{}.last().unwrap()", obj_str);
                }
                valor.clone()
            } else {
                emit_expr(prop)
            };
            format!("{}.{}", obj_str, prop_str)
        }
        Expr::Series { elementa, .. } => {
            let items: Vec<String> = elementa.iter().map(emit_expr).collect();
            format!("vec![{}]", items.join(", "))
        }
        Expr::Obiectum { props, .. } => {
            // Rust doesn't have object literals, use HashMap or struct init
            let pairs: Vec<String> = props
                .iter()
                .map(|p| {
                    let key = if let Expr::Littera { valor, .. } = &p.key {
                        format!("\"{}\"", valor)
                    } else {
                        emit_expr(&p.key)
                    };
                    format!("({}, {})", key, emit_expr(&p.valor))
                })
                .collect();
            format!("HashMap::from([{}])", pairs.join(", "))
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
            format!("|{}| {}", params_str.join(", "), body)
        }
        Expr::Novum { callee, args, .. } => {
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("{}::new({})", emit_expr(callee), args_str.join(", "))
        }
        Expr::Cede { arg, .. } => format!("{}.await", emit_expr(arg)),
        Expr::Qua { expr, typus, .. } => {
            format!("({} as {})", emit_expr(expr), emit_typus(typus))
        }
        Expr::Innatum { expr, typus, .. } => {
            format!("({} as {})", emit_expr(expr), emit_typus(typus))
        }
        Expr::PostfixNovum { expr, typus, .. } => {
            format!("{}::from({})", emit_typus(typus), emit_expr(expr))
        }
        Expr::Finge { variant, campi, .. } => {
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
            format!("{} {{ {} }}", variant, fields.join(", "))
        }
        Expr::Scriptum { template, args, .. } => {
            let parts: Vec<&str> = template.split('§').collect();
            if parts.len() == 1 {
                return format!("{}.to_string()", quote_string(template));
            }
            let format_str = parts.join("{}");
            let args_str: Vec<String> = args.iter().map(emit_expr).collect();
            format!("format!(\"{}\", {})", format_str, args_str.join(", "))
        }
        Expr::Ambitus { start, end, inclusive, .. } => {
            if *inclusive {
                format!("({}..={})", emit_expr(start), emit_expr(end))
            } else {
                format!("({}..{})", emit_expr(start), emit_expr(end))
            }
        }
    }
}

fn emit_typus(typus: &Typus) -> String {
    match typus {
        Typus::Nomen { nomen } => map_type_name(nomen),
        Typus::Nullabilis { inner } => format!("Option<{}>", emit_typus(inner)),
        Typus::Genericus { nomen, args } => {
            let args_str: Vec<String> = args.iter().map(emit_typus).collect();
            format!("{}<{}>", map_type_name(nomen), args_str.join(", "))
        }
        Typus::Functio { params, returns } => {
            let params_str: Vec<String> = params.iter().map(emit_typus).collect();
            let ret = if let Some(r) = returns {
                emit_typus(r)
            } else {
                "()".to_string()
            };
            format!("fn({}) -> {}", params_str.join(", "), ret)
        }
        Typus::Unio { members } => {
            // Rust doesn't have union types like TS, emit as comment
            let parts: Vec<String> = members.iter().map(emit_typus).collect();
            format!("/* {} */", parts.join(" | "))
        }
        Typus::Litteralis { valor } => valor.clone(),
    }
}

fn emit_param(p: &Param) -> String {
    let ownership = if let Some(o) = &p.ownership {
        match o.as_str() {
            "de" => "&",      // immutable borrow
            "in" => "&mut ",  // mutable borrow
            _ => "",          // ex = owned (no annotation)
        }
    } else {
        ""
    };

    let typ = if let Some(t) = &p.typus {
        format!(": {}{}", ownership, emit_typus(t))
    } else {
        String::new()
    };

    format!("{}{}", p.nomen, typ)
}

fn map_type_name(name: &str) -> String {
    match name {
        "textus" => "String",
        "numerus" => "i64",
        "fractus" => "f64",
        "decimus" => "f64",
        "magnus" => "i128",
        "bivalens" => "bool",
        "nihil" => "()",
        "vacuum" | "vacuus" => "()",
        "ignotum" => "Box<dyn std::any::Any>",
        "quodlibet" | "quidlibet" => "Box<dyn std::any::Any>",
        "lista" => "Vec",
        "tabula" => "HashMap",
        "collectio" | "copia" => "HashSet",
        "octeti" => "Vec<u8>",
        _ => name,
    }
    .to_string()
}

fn map_binary_op(op: &str) -> &'static str {
    match op {
        "et" => "&&",
        "aut" => "||",
        "vel" => ".unwrap_or",  // Not a direct translation
        "inter" => "/* in */",  // No direct equivalent
        "intra" => "/* instanceof */",
        "+" => "+",
        "-" => "-",
        "*" => "*",
        "/" => "/",
        "%" => "%",
        "==" => "==",
        "!=" => "!=",
        "===" => "==",
        "!==" => "!=",
        "<" => "<",
        ">" => ">",
        "<=" => "<=",
        ">=" => ">=",
        _ => "/* unknown op */",
    }
}

fn map_unary_op(op: &str) -> &'static str {
    match op {
        "non" => "!",
        "nihil" => "!",
        "nonnihil" => "",  // Rust doesn't need this
        "positivum" => "+",
        "-" => "-",
        "!" => "!",
        _ => "/* unknown op */",
    }
}

fn map_method_name(name: &str) -> Option<&'static str> {
    let map: HashMap<&str, &str> = [
        ("adde", "push"),
        ("praepone", "insert"),  // insert(0, x)
        ("remove", "pop"),
        ("decapita", "remove"),  // remove(0)
        ("coniunge", "join"),
        ("continet", "contains"),
        ("indiceDe", "position"),
        ("inveni", "find"),
        ("inveniIndicem", "position"),
        ("omnes", "all"),
        ("aliquis", "any"),
        ("filtrata", "filter"),
        ("mappata", "map"),
        ("explanata", "flat_map"),
        ("plana", "flatten"),
        ("sectio", "get"),
        ("reducta", "fold"),
        ("perambula", "for_each"),
        ("inverte", "reverse"),
        ("ordina", "sort"),
        ("pone", "insert"),
        ("accipe", "get"),
        ("habet", "contains_key"),
        ("dele", "remove"),
        ("purga", "clear"),
        ("claves", "keys"),
        ("valores", "values"),
        ("paria", "iter"),
        ("initium", "starts_with"),
        ("finis", "ends_with"),
        ("maiuscula", "to_uppercase"),
        ("minuscula", "to_lowercase"),
        ("recide", "trim"),
        ("divide", "split"),
        ("muta", "replace"),
    ]
    .iter()
    .cloned()
    .collect();
    map.get(name).copied()
}

fn sanitize_ident(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
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
