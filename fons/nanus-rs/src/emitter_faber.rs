use subsidia_rs::{
    ClausuraCorpus, Expr, LitteraSpecies, Modulus, Param, Stmt, Typus, VariaSpecies,
};

/// Emit a module back to Faber source code.
pub fn emit_faber(module: &Modulus) -> String {
    let mut result = String::new();
    for (i, stmt) in module.corpus.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        result.push_str(&emit_stmt(stmt, ""));
    }
    result
}

fn emit_stmt(stmt: &Stmt, indent: &str) -> String {
    match stmt {
        Stmt::Varia { .. } => emit_varia(stmt, indent),
        Stmt::Functio { .. } => emit_functio(stmt, indent),
        Stmt::Genus { .. } => emit_genus(stmt, indent),
        Stmt::Pactum { .. } => emit_pactum(stmt, indent),
        Stmt::Ordo { .. } => emit_ordo(stmt, indent),
        Stmt::Discretio { .. } => emit_discretio(stmt, indent),
        Stmt::TypusAlias { nomen, typus, publica, .. } => {
            let pub_prefix = if *publica { "publica " } else { "" };
            format!("{}{}typus {} = {}", indent, pub_prefix, nomen, emit_typus(typus))
        }
        Stmt::In { expr, corpus, .. } => {
            format!("{}in {} {}", indent, emit_expr(expr), emit_stmt(corpus, indent))
        }
        Stmt::Importa { .. } => emit_importa(stmt, indent),
        Stmt::Redde { valor, .. } => {
            if let Some(v) = valor {
                format!("{}redde {}", indent, emit_expr(v))
            } else {
                format!("{}redde", indent)
            }
        }
        Stmt::Si { .. } => emit_si(stmt, indent),
        Stmt::Dum { cond, corpus, .. } => {
            format!("{}dum {} {}", indent, emit_expr(cond), emit_stmt(corpus, indent))
        }
        Stmt::FacDum { corpus, cond, .. } => {
            format!("{}fac {} dum {}", indent, emit_stmt(corpus, indent), emit_expr(cond))
        }
        Stmt::Iteratio {
            binding,
            iter,
            corpus,
            asynca,
            ..
        } => {
            let prefix = if *asynca { "cede " } else { "" };
            format!(
                "{}{}ex {} fixum {} {}",
                indent,
                prefix,
                emit_expr(iter),
                binding,
                emit_stmt(corpus, indent)
            )
        }
        Stmt::Elige { .. } => emit_elige(stmt, indent),
        Stmt::Discerne { .. } => emit_discerne(stmt, indent),
        Stmt::Custodi { clausulae, .. } => {
            let mut result = String::new();
            for (i, c) in clausulae.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(&format!(
                    "{}custodi {} {}",
                    indent,
                    emit_expr(&c.cond),
                    emit_stmt(&c.corpus, indent)
                ));
            }
            result
        }
        Stmt::Tempta { corpus, cape, demum, .. } => {
            let mut result = format!("{}tempta {}", indent, emit_stmt(corpus, indent));
            if let Some(c) = cape {
                result.push_str(&format!(" cape {} {}", c.param, emit_stmt(&c.corpus, indent)));
            }
            if let Some(d) = demum {
                result.push_str(&format!(" demum {}", emit_stmt(d, indent)));
            }
            result
        }
        Stmt::Iace { arg, fatale, .. } => {
            let kw = if *fatale { "mori" } else { "iace" };
            format!("{}{} {}", indent, kw, emit_expr(arg))
        }
        Stmt::Rumpe { .. } => format!("{}rumpe", indent),
        Stmt::Perge { .. } => format!("{}perge", indent),
        Stmt::Scribe { gradus, args, .. } => {
            let keyword = match gradus.as_str() {
                "Vide" => "vide",
                "Mone" => "mone",
                _ => "scribe",
            };
            let args_str: Vec<String> = args.iter().map(|a| emit_expr(a)).collect();
            format!("{}{} {}", indent, keyword, args_str.join(", "))
        }
        Stmt::Adfirma { cond, msg, .. } => {
            let mut result = format!("{}adfirma {}", indent, emit_expr(cond));
            if let Some(m) = msg {
                result.push_str(&format!(", {}", emit_expr(m)));
            }
            result
        }
        Stmt::Expressia { expr, .. } => format!("{}{}", indent, emit_expr(expr)),
        Stmt::Massa { corpus, .. } => emit_massa(corpus, indent),
        Stmt::Incipit { corpus, asynca, .. } => {
            let kw = if *asynca { "incipiet" } else { "incipit" };
            format!("{}{} {}", indent, kw, emit_stmt(corpus, indent))
        }
        Stmt::Probandum { nomen, corpus, .. } => {
            let mut result = format!("{}probandum \"{}\" {{\n", indent, nomen);
            for s in corpus {
                result.push_str(&emit_stmt(s, &format!("{}    ", indent)));
                result.push('\n');
            }
            result.push_str(indent);
            result.push('}');
            result
        }
        Stmt::Proba { nomen, corpus, .. } => {
            format!("{}proba \"{}\" {}", indent, nomen, emit_stmt(corpus, indent))
        }
    }
}

fn emit_massa(corpus: &[Stmt], indent: &str) -> String {
    let mut result = String::from("{\n");
    for stmt in corpus {
        result.push_str(&emit_stmt(stmt, &format!("{}    ", indent)));
        result.push('\n');
    }
    result.push_str(indent);
    result.push('}');
    result
}

fn emit_varia(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Varia {
        species,
        nomen,
        typus,
        valor,
        publica,
        ..
    } = stmt
    {
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        let keyword = match species {
            VariaSpecies::Fixum => "fixum",
            VariaSpecies::Figendum => "figendum",
            _ => "varia",
        };
        result.push_str(&format!("{}{} {}", indent, keyword, nomen));
        if let Some(t) = typus {
            result.push_str(&format!(": {}", emit_typus(t)));
        }
        if let Some(v) = valor {
            result.push_str(&format!(" = {}", emit_expr(v)));
        }
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
        generics,
        ..
    } = stmt
    {
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        result.push_str(indent);
        if *asynca {
            result.push_str("asynca ");
        }
        result.push_str("functio ");
        result.push_str(nomen);
        if !generics.is_empty() {
            result.push('<');
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        result.push('(');
        let params_str: Vec<String> = params.iter().map(emit_param).collect();
        result.push_str(&params_str.join(", "));
        result.push(')');
        if let Some(t) = typus_reditus {
            result.push_str(" -> ");
            result.push_str(&emit_typus(t));
        }
        if let Some(c) = corpus {
            result.push(' ');
            result.push_str(&emit_stmt(c, indent));
        }
        result
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
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        result.push_str(indent);
        result.push_str("genus ");
        result.push_str(nomen);
        if !generics.is_empty() {
            result.push('<');
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        if !implet.is_empty() {
            result.push_str(" implet ");
            result.push_str(&implet.join(", "));
        }
        result.push_str(" {\n");
        for c in campi {
            result.push_str(&format!("{}    ", indent));
            result.push_str(&c.nomen);
            if let Some(t) = &c.typus {
                result.push_str(": ");
                result.push_str(&emit_typus(t));
            }
            if let Some(v) = &c.valor {
                result.push_str(" = ");
                result.push_str(&emit_expr(v));
            }
            result.push('\n');
        }
        for m in methodi {
            result.push_str(&emit_stmt(m, &format!("{}    ", indent)));
            result.push('\n');
        }
        result.push_str(indent);
        result.push('}');
        result
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
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        result.push_str(indent);
        result.push_str("pactum ");
        result.push_str(nomen);
        if !generics.is_empty() {
            result.push('<');
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        result.push_str(" {\n");
        for m in methodi {
            result.push_str(&format!("{}    ", indent));
            if m.asynca {
                result.push_str("asynca ");
            }
            result.push_str("functio ");
            result.push_str(&m.nomen);
            result.push('(');
            let params_str: Vec<String> = m.params.iter().map(emit_param).collect();
            result.push_str(&params_str.join(", "));
            result.push(')');
            if let Some(t) = &m.typus_reditus {
                result.push_str(" -> ");
                result.push_str(&emit_typus(t));
            }
            result.push('\n');
        }
        result.push_str(indent);
        result.push('}');
        result
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
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        result.push_str(indent);
        result.push_str("ordo ");
        result.push_str(nomen);
        result.push_str(" {\n");
        for m in membra {
            result.push_str(&format!("{}{}", indent, m.nomen));
            if let Some(v) = &m.valor {
                result.push_str(" = ");
                result.push_str(v);
            }
            result.push('\n');
        }
        result.push_str(indent);
        result.push('}');
        result
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
        let mut result = String::new();
        if *publica {
            result.push_str(indent);
            result.push_str("@ publica\n");
        }
        result.push_str(indent);
        result.push_str("discretio ");
        result.push_str(nomen);
        if !generics.is_empty() {
            result.push('<');
            result.push_str(&generics.join(", "));
            result.push('>');
        }
        result.push_str(" {\n");
        let inner = format!("{}    ", indent);
        let inner2 = format!("{}    ", inner);
        for v in variantes {
            result.push_str(&format!("{}{}", inner, v.nomen));
            if !v.campi.is_empty() {
                result.push_str(" {\n");
                for f in &v.campi {
                    result.push_str(&format!("{}{} {}\n", inner2, emit_typus(&f.typus), f.nomen));
                }
                result.push_str(&inner);
                result.push('}');
            }
            result.push('\n');
        }
        result.push_str(indent);
        result.push('}');
        result
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
        let mut result = format!("{}ex \"{}\" importa ", indent, fons);
        if *totum {
            if let Some(a) = alias {
                result.push_str(&format!("* ut {}", a));
            } else {
                result.push('*');
            }
        } else {
            let specs_str: Vec<String> = specs
                .iter()
                .map(|s| {
                    if !s.local.is_empty() && s.local != s.imported {
                        format!("{} ut {}", s.imported, s.local)
                    } else {
                        s.imported.clone()
                    }
                })
                .collect();
            result.push_str(&specs_str.join(", "));
        }
        result
    } else {
        String::new()
    }
}

fn emit_si(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Si { cond, cons, alt, .. } = stmt {
        let mut result = format!("{}si {} {}", indent, emit_expr(cond), emit_stmt(cons, indent));
        if let Some(a) = alt {
            result.push_str(" secus ");
            result.push_str(&emit_stmt(a, indent));
        }
        result
    } else {
        String::new()
    }
}

fn emit_elige(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Elige {
        discrim,
        casus,
        default,
        ..
    } = stmt
    {
        let mut result = format!("{}elige {} {{\n", indent, emit_expr(discrim));
        for c in casus {
            result.push_str(&format!(
                "{}casu {} {}\n",
                indent,
                emit_expr(&c.cond),
                emit_stmt(&c.corpus, &format!("{}    ", indent))
            ));
        }
        if let Some(d) = default {
            result.push_str(&format!(
                "{}ceterum {}\n",
                indent,
                emit_stmt(d, &format!("{}    ", indent))
            ));
        }
        result.push_str(indent);
        result.push('}');
        result
    } else {
        String::new()
    }
}

fn emit_discerne(stmt: &Stmt, indent: &str) -> String {
    if let Stmt::Discerne { discrim, casus, .. } = stmt {
        let discrim_str: Vec<String> = discrim.iter().map(|d| emit_expr(d)).collect();
        let mut result = format!("{}discerne {} {{\n", indent, discrim_str.join(", "));
        for c in casus {
            result.push_str(&format!("{}casu ", indent));
            let patterns: Vec<String> = c
                .patterns
                .iter()
                .map(|p| {
                    if p.wildcard {
                        "_".to_string()
                    } else {
                        let mut s = p.variant.clone();
                        if !p.bindings.is_empty() {
                            s.push('(');
                            s.push_str(&p.bindings.join(", "));
                            s.push(')');
                        }
                        if let Some(alias) = &p.alias {
                            s.push_str(&format!(" ut {}", alias));
                        }
                        s
                    }
                })
                .collect();
            result.push_str(&patterns.join(", "));
            result.push(' ');
            result.push_str(&emit_stmt(&c.corpus, &format!("{}    ", indent)));
            result.push('\n');
        }
        result.push_str(indent);
        result.push('}');
        result
    } else {
        String::new()
    }
}

fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Nomen { valor, .. } => valor.clone(),
        Expr::Ego { .. } => "ego".to_string(),
        Expr::Littera { species, valor, .. } => match species {
            LitteraSpecies::Textus => format!("\"{}\"", escape_string(valor)),
            LitteraSpecies::Verum => "verum".to_string(),
            LitteraSpecies::Falsum => "falsum".to_string(),
            LitteraSpecies::Nihil => "nihil".to_string(),
            _ => valor.clone(),
        },
        Expr::Binaria { signum, sin, dex, .. } => {
            format!("{} {} {}", emit_expr(sin), signum, emit_expr(dex))
        }
        Expr::Unaria { signum, arg, .. } => {
            if signum == "nihil" || signum == "non" || signum == "nonnihil" {
                format!("{} {}", signum, emit_expr(arg))
            } else {
                format!("{}{}", signum, emit_expr(arg))
            }
        }
        Expr::Assignatio { signum, sin, dex, .. } => {
            format!("{} {} {}", emit_expr(sin), signum, emit_expr(dex))
        }
        Expr::Vocatio { callee, args, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| emit_expr(a)).collect();
            format!("{}({})", emit_expr(callee), args_str.join(", "))
        }
        Expr::Membrum {
            obj,
            prop,
            computed,
            non_null,
            ..
        } => {
            let obj_str = emit_expr(obj);
            if *computed {
                format!("{}[{}]", obj_str, emit_expr(prop))
            } else {
                let prop_str = if let Expr::Littera { valor, .. } = prop.as_ref() {
                    valor.clone()
                } else {
                    emit_expr(prop)
                };
                if *non_null {
                    format!("{}!.{}", obj_str, prop_str)
                } else {
                    format!("{}.{}", obj_str, prop_str)
                }
            }
        }
        Expr::Condicio { cond, cons, alt, .. } => {
            format!(
                "{} sic {} secus {}",
                emit_expr(cond),
                emit_expr(cons),
                emit_expr(alt)
            )
        }
        Expr::Series { elementa, .. } => {
            let items: Vec<String> = elementa.iter().map(|e| emit_expr(e)).collect();
            format!("[{}]", items.join(", "))
        }
        Expr::Obiectum { props, .. } => {
            let pairs: Vec<String> = props
                .iter()
                .map(|p| {
                    if p.shorthand {
                        emit_expr(&p.key)
                    } else {
                        format!("{}: {}", emit_expr(&p.key), emit_expr(&p.valor))
                    }
                })
                .collect();
            format!("{{ {} }}", pairs.join(", "))
        }
        Expr::Clausura { params, corpus, .. } => {
            let params_str: Vec<String> = params.iter().map(emit_param).collect();
            let body = match corpus {
                ClausuraCorpus::Stmt(s) => emit_stmt(s, ""),
                ClausuraCorpus::Expr(e) => emit_expr(e),
            };
            format!("({}) => {}", params_str.join(", "), body)
        }
        Expr::Novum { callee, args, init, .. } => {
            let args_str: Vec<String> = args.iter().map(|a| emit_expr(a)).collect();
            let mut result = format!("novum {}({})", emit_expr(callee), args_str.join(", "));
            if let Some(i) = init {
                result.push(' ');
                result.push_str(&emit_expr(i));
            }
            result
        }
        Expr::Qua { expr, typus, .. } => {
            format!("{} qua {}", emit_expr(expr), emit_typus(typus))
        }
        Expr::Innatum { expr, typus, .. } => {
            format!("{} innatum {}", emit_expr(expr), emit_typus(typus))
        }
        Expr::Conversio { expr, species, fallback, .. } => {
            let base = format!("{} {}", emit_expr(expr), species);
            if let Some(fb) = fallback {
                format!("{} vel {}", base, emit_expr(fb))
            } else {
                base
            }
        }
        Expr::Cede { arg, .. } => {
            format!("cede {}", emit_expr(arg))
        }
        Expr::Finge { variant, campi, .. } => {
            let pairs: Vec<String> = campi
                .iter()
                .map(|p| {
                    let key_str = match &p.key {
                        Expr::Littera { valor, .. } => valor.clone(),
                        Expr::Nomen { valor, .. } => valor.clone(),
                        _ => emit_expr(&p.key),
                    };
                    if p.shorthand {
                        key_str
                    } else {
                        format!("{}: {}", key_str, emit_expr(&p.valor))
                    }
                })
                .collect();
            format!("finge {} {{ {} }}", variant, pairs.join(", "))
        }
        Expr::Scriptum { template, args, .. } => {
            if args.is_empty() {
                format!("scriptum(\"{}\")", escape_string(template))
            } else {
                let args_str: Vec<String> = args.iter().map(emit_expr).collect();
                format!("scriptum(\"{}\", {})", escape_string(template), args_str.join(", "))
            }
        }
        Expr::Ambitus {
            start,
            end,
            inclusive,
            ..
        } => {
            let op = if *inclusive { " usque " } else { " ante " };
            format!("{}{}{}", emit_expr(start), op, emit_expr(end))
        }
        Expr::PostfixNovum { expr, typus, .. } => {
            format!("{} novum {}", emit_expr(expr), emit_typus(typus))
        }
    }
}

fn emit_typus(typus: &Typus) -> String {
    match typus {
        Typus::Nomen { nomen } => nomen.clone(),
        Typus::Genericus { nomen, args } => {
            let args_str: Vec<String> = args.iter().map(emit_typus).collect();
            format!("{}<{}>", nomen, args_str.join(", "))
        }
        Typus::Functio { params, returns } => {
            let params_str: Vec<String> = params.iter().map(emit_typus).collect();
            let mut result = format!("({}) -> ", params_str.join(", "));
            if let Some(r) = returns {
                result.push_str(&emit_typus(r));
            }
            result
        }
        Typus::Nullabilis { inner } => {
            format!("{}?", emit_typus(inner))
        }
        Typus::Unio { members } => {
            let parts: Vec<String> = members.iter().map(emit_typus).collect();
            parts.join(" | ")
        }
        Typus::Litteralis { valor } => valor.clone(),
    }
}

fn emit_param(p: &Param) -> String {
    let mut result = String::new();
    // Order per EBNF: si, ownership (ex/de/in), ceteri, type, name
    if let Some(t) = &p.typus {
        // If param type is Nullabilis, emit "si" (optional param syntax)
        if matches!(t, Typus::Nullabilis { .. }) {
            result.push_str("si ");
        }
    }
    if let Some(own) = &p.ownership {
        result.push_str(own);
        result.push(' ');
    }
    if p.rest {
        result.push_str("ceteri ");
    }
    if let Some(t) = &p.typus {
        // Emit inner type for Nullabilis, otherwise the type itself
        if let Typus::Nullabilis { inner } = t {
            result.push_str(&emit_typus(inner));
        } else {
            result.push_str(&emit_typus(t));
        }
        result.push(' ');
    }
    result.push_str(&p.nomen);
    if let Some(d) = &p.default {
        result.push_str(" = ");
        result.push_str(&emit_expr(d));
    }
    result
}

fn escape_string(s: &str) -> String {
    let mut result = String::new();
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
    result
}
