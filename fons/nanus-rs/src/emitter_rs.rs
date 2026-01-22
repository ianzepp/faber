use std::collections::HashMap;
use subsidia_rs::{
    analyze, ClausuraCorpus, Expr, LitteraSpecies, Modulus, Param, SemanticContext, SemanticTypus,
    Stmt, Typus, VariaSpecies,
};

/// Emitter state with semantic context
struct RsEmitter<'a> {
    ctx: &'a SemanticContext,
}

impl<'a> RsEmitter<'a> {
    fn is_known_enum(&self, name: &str) -> bool {
        match name {
            "SymbolumGenus" | "VerbumId" | "Visibilitas" | "VariaGenus" | 
            "LitteraGenus" | "ScribeGradus" | "IteratioGenus" | "CuratorGenus" | 
            "AdVerbumVinculandi" | "PraeparaTempus" | "ProbaModificator" |
            "LexorErrorCodice" | "ParserErrorCodice" | "SemanticErrorCodice" => true,
            _ => false,
        }
    }

    fn new(ctx: &'a SemanticContext) -> Self {
        Self { ctx }
    }

    /// Find which discretio contains a given variant name
    fn find_discretio_for_variant(&self, variant_name: &str) -> Option<String> {
        // Local check
        for (disc_name, disc_type) in &self.ctx.disc_registry {
            if let SemanticTypus::Discretio { variantes, .. } = disc_type {
                if variantes.contains_key(variant_name) {
                    return Some(disc_name.clone());
                }
            }
        }

        // Heuristic fallback for rivus AST
        match variant_name {
            "MassaSententia" | "VariaSententia" | "ImportaSententia" | "DestructuraSententia" |
            "SeriesDestructuraSententia" | "FunctioDeclaratio" | "GenusDeclaratio" | "PactumDeclaratio" |
            "TypusAliasDeclaratio" | "OrdoDeclaratio" | "DiscretioDeclaratio" | "SiSententia" |
            "DumSententia" | "ExSententia" | "DeSententia" | "EligeSententia" | "DiscerneSententia" |
            "CustodiSententia" | "ReddeSententia" | "RumpeSententia" | "PergeSententia" |
            "IaceSententia" | "ScribeSententia" | "IncipitSententia" | "IncipietSententia" |
            "CuraSententia" | "TemptaSententia" | "FacSententia" | "AdfirmaSententia" |
            "ProbandumSententia" | "ProbaSententia" | "ExpressiaSententia" => Some("Sententia".to_string()),

            "Nomen" | "Ego" | "Littera" | "Binaria" | "Unaria" | "Assignatio" |
            "Condicio" | "Vocatio" | "Membrum" | "Series" | "Obiectum" | "Clausura" |
            "Novum" | "Cede" | "Qua" | "Innatum" | "Conversio" | "PostfixNovum" |
            "Finge" | "Scriptum" | "Ambitus" => Some("Expressia".to_string()),

            "Nullabilis" | "Genericus" | "Unio" | "Litteralis" => Some("Typus".to_string()),

            "CurataModificator" | "ErrataModificator" | "ExitusModificator" |
            "ImmutataModificator" | "IacitModificator" | "OptionesModificator" => Some("FunctioModificator".to_string()),

            _ => None,
        }
    }

    /// Get the enum name for a discerne statement by looking up variant names
    fn get_discrim_type_from_patterns(&self, casus: &[subsidia_rs::DiscerneCasus]) -> String {
        // Find the first non-wildcard pattern's variant name
        for c in casus {
            for p in &c.patterns {
                if !p.wildcard && !p.variant.is_empty() {
                    if let Some(disc_name) = self.find_discretio_for_variant(&p.variant) {
                        return disc_name;
                    }
                }
            }
        }
        "Unknown".to_string()
    }

    fn emit_stmt(&self, stmt: &Stmt, indent: &str) -> String {
        match stmt {
            Stmt::Massa { corpus, .. } => self.emit_massa(corpus, indent),
            Stmt::Expressia { expr, .. } => format!("{}{};", indent, self.emit_expr(expr)),
            Stmt::Varia { .. } => self.emit_varia(stmt, indent),
            Stmt::Functio { .. } => self.emit_functio(stmt, indent),
            Stmt::Genus { .. } => self.emit_genus(stmt, indent),
            Stmt::Pactum { .. } => self.emit_pactum(stmt, indent),
            Stmt::Ordo { .. } => self.emit_ordo(stmt, indent),
            Stmt::Discretio { .. } => self.emit_discretio_decl(stmt, indent),
            Stmt::TypusAlias {
                nomen,
                typus,
                publica,
                ..
            } => {
                let vis = if *publica { "pub " } else { "" };
                format!(
                    "{}{}type {} = {};",
                    indent,
                    vis,
                    nomen,
                    self.emit_typus(typus)
                )
            }
            Stmt::In { expr, corpus, .. } => {
                format!(
                    "{}{{\n{}    let __target = {};\n{}\n{}}}",
                    indent,
                    indent,
                    self.emit_expr(expr),
                    self.emit_stmt(corpus, &format!("{}    ", indent)),
                    indent
                )
            }
            Stmt::Importa { .. } => self.emit_importa(stmt, indent),
            Stmt::Si {
                cond, cons, alt, ..
            } => {
                let mut result = format!(
                    "{}if {} {}",
                    indent,
                    self.emit_expr(cond),
                    self.emit_stmt(cons, indent)
                );
                if let Some(a) = alt {
                    result.push_str(" else ");
                    result.push_str(&self.emit_stmt(a, indent));
                }
                result
            }
            Stmt::Dum { cond, corpus, .. } => {
                format!(
                    "{}while {} {}",
                    indent,
                    self.peek_and_emit_expr(cond),
                    self.emit_stmt(corpus, indent)
                )
            }
            Stmt::FacDum { corpus, cond, .. } => {
                let body = self.emit_stmt(corpus, &format!("{}    ", indent));
                format!(
                    "{}loop {{\n{}\n{}    if !({}) {{ break; }}\n{}}}",
                    indent,
                    body,
                    indent,
                    self.emit_expr(cond),
                    indent
                )
            }
            Stmt::Iteratio {
                binding,
                iter,
                corpus,
                species,
                ..
            } => {
                let iter_method = if species == "De" {
                    ".iter()"
                } else {
                    ".into_iter()"
                };
                format!(
                    "{}for {} in {}{} {}",
                    indent,
                    binding,
                    self.emit_expr(iter),
                    iter_method,
                    self.emit_stmt(corpus, indent)
                )
            }
            Stmt::Elige {
                discrim,
                casus,
                default,
                ..
            } => {
                let discrim_str = self.emit_expr(discrim);
                let mut lines: Vec<String> = Vec::new();
                for (i, c) in casus.iter().enumerate() {
                    let kw = if i == 0 { "if" } else { "else if" };
                    lines.push(format!(
                        "{}{} {} == {} {}",
                        indent,
                        kw,
                        discrim_str,
                        self.emit_expr(&c.cond),
                        self.emit_stmt(&c.corpus, indent)
                    ));
                }
                if let Some(d) = default {
                    lines.push(format!("{}else {}", indent, self.emit_stmt(d, indent)));
                }
                lines.join("\n")
            }
            Stmt::Discerne { discrim, casus, .. } => self.emit_discerne(discrim, casus, indent),
            Stmt::Custodi { clausulae, .. } => {
                let lines: Vec<String> = clausulae
                    .iter()
                    .map(|c| {
                        format!(
                            "{}if {} {}",
                            indent,
                            self.emit_expr(&c.cond),
                            self.emit_stmt(&c.corpus, indent)
                        )
                    })
                    .collect();
                lines.join("\n")
            }
            Stmt::Tempta {
                corpus,
                cape,
                demum,
                ..
            } => {
                let mut lines: Vec<String> = Vec::new();
                lines.push(format!("{}// try", indent));
                lines.push(self.emit_stmt(corpus, indent));
                if let Some(c) = cape {
                    lines.push(format!("{}// catch({})", indent, c.param));
                    lines.push(self.emit_stmt(&c.corpus, indent));
                }
                if let Some(d) = demum {
                    lines.push(format!("{}// finally", indent));
                    lines.push(self.emit_stmt(d, indent));
                }
                lines.join("\n")
            }
            Stmt::Redde { valor, .. } => {
                if let Some(v) = valor {
                    format!("{}return {};", indent, self.emit_expr(v))
                } else {
                    format!("{}return;", indent)
                }
            }
            Stmt::Iace { arg, fatale, .. } => {
                if *fatale {
                    format!("{}panic!(\"{{}}\", {});", indent, self.emit_expr(arg))
                } else {
                    format!("{}return Err({});", indent, self.emit_expr(arg))
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
                    let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                    format!(
                        "{}{}(\"{}\", {});",
                        indent,
                        macro_name,
                        format_str,
                        args_str.join(", ")
                    )
                }
            }
            Stmt::Adfirma { cond, msg, .. } => {
                if let Some(m) = msg {
                    format!(
                        "{}assert!({}, {});",
                        indent,
                        self.emit_expr(cond),
                        self.emit_expr(m)
                    )
                } else {
                    format!("{}assert!({});", indent, self.emit_expr(cond))
                }
            }
            Stmt::Incipit { corpus, asynca, .. } => {
                if *asynca {
                    format!(
                        "{}#[tokio::main]\n{}async fn main() {}",
                        indent,
                        indent,
                        self.emit_stmt(corpus, indent)
                    )
                } else {
                    format!("{}fn main() {}", indent, self.emit_stmt(corpus, indent))
                }
            }
            Stmt::Probandum { nomen, corpus, .. } => {
                let mut lines: Vec<String> = Vec::new();
                lines.push(format!("{}#[cfg(test)]", indent));
                lines.push(format!("{}mod {} {{", indent, sanitize_ident(nomen)));
                lines.push(format!("{}    use super::*;", indent));
                for s in corpus {
                    lines.push(self.emit_stmt(s, &format!("{}    ", indent)));
                }
                lines.push(format!("{}}}", indent));
                lines.join("\n")
            }
            Stmt::Proba { nomen, corpus, .. } => {
                format!(
                    "{}#[test]\n{}fn {}() {}",
                    indent,
                    indent,
                    sanitize_ident(nomen),
                    self.emit_stmt(corpus, indent)
                )
            }
        }
    }

    fn peek_and_emit_expr(&self, expr: &Expr) -> String {
        self.emit_expr(expr)
    }

    fn emit_massa(&self, corpus: &[Stmt], indent: &str) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push("{".to_string());
        for stmt in corpus {
            lines.push(self.emit_stmt(stmt, &format!("{}    ", indent)));
        }
        lines.push(format!("{}}}", indent));
        lines.join("\n")
    }

    fn emit_varia(&self, stmt: &Stmt, indent: &str) -> String {
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
            let is_module_level = indent.is_empty();
            let kw = match (species, is_module_level) {
                (VariaSpecies::Fixum | VariaSpecies::Figendum, true) => "static",
                (VariaSpecies::Fixum | VariaSpecies::Figendum, false) => "let",
                (_, true) => "static mut",
                (_, false) => "let mut",
            };

            let mut result = format!("{}{}{} {}", indent, vis, kw, nomen);
            if let Some(t) = typus {
                result.push_str(&format!(": {}", self.emit_typus(t)));
            }
            if let Some(v) = valor {
                result.push_str(&format!(" = {}", self.emit_expr(v)));
            }
            result.push(';');
            result
        } else {
            String::new()
        }
    }

    fn emit_functio(&self, stmt: &Stmt, indent: &str) -> String {
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

            let params_str: Vec<String> = params.iter().map(|p| self.emit_param(p)).collect();
            let ret = if let Some(t) = typus_reditus {
                format!(" -> {}", self.emit_typus(t))
            } else {
                String::new()
            };

            let body = if let Some(c) = corpus {
                format!(" {}", self.emit_stmt(c, indent))
            } else {
                ";".to_string()
            };

            format!(
                "{}{}{}fn {}{}({}){}{}",
                indent,
                vis,
                async_kw,
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

    fn emit_genus(&self, stmt: &Stmt, indent: &str) -> String {
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
            lines.push(format!("{}#[derive(Debug, Clone)]", indent));
            lines.push(format!(
                "{}{}struct {}{} {{",
                indent, vis, nomen, generics_str
            ));

            for c in campi {
                let field_vis = match c.visibilitas.as_str() {
                    "Publica" => "pub ",
                    _ => "",
                };
                let typ = if let Some(t) = &c.typus {
                    self.emit_typus(t)
                } else {
                    "()".to_string()
                };
                lines.push(format!("{}    {}{}: {},", indent, field_vis, sanitize_rs_ident(&c.nomen), typ));
            }
            lines.push(format!("{}}}", indent));

            if !methodi.is_empty() {
                lines.push(String::new());
                lines.push(format!(
                    "{}impl{} {}{} {{",
                    indent, generics_str, nomen, generics_str
                ));

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

                        let mut all_params: Vec<String> = vec!["&self".to_string()];
                        all_params.extend(params.iter().map(|p| self.emit_param(p)));

                        let ret = if let Some(t) = typus_reditus {
                            format!(" -> {}", self.emit_typus(t))
                        } else {
                            String::new()
                        };
                        let body = if let Some(c) = corpus {
                            format!(" {}", self.emit_stmt(c, &format!("{}    ", indent)))
                        } else {
                            " {}".to_string()
                        };
                        lines.push(format!(
                            "{}    {}{}fn {}({}){}{}",
                            indent,
                            method_vis,
                            async_kw,
                            method_name,
                            all_params.join(", "),
                            ret,
                            body
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

    fn emit_pactum(&self, stmt: &Stmt, indent: &str) -> String {
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
            lines.push(format!(
                "{}{}trait {}{} {{",
                indent, vis, nomen, generics_str
            ));

            for m in methodi {
                let params_str: Vec<String> = m.params.iter().map(|p| self.emit_param(p)).collect();
                let mut all_params: Vec<String> = vec!["&self".to_string()];
                all_params.extend(params_str);

                let ret = if let Some(t) = &m.typus_reditus {
                    format!(" -> {}", self.emit_typus(t))
                } else {
                    String::new()
                };
                lines.push(format!(
                    "{}    fn {}({}){};",
                    indent,
                    m.nomen,
                    all_params.join(", "),
                    ret
                ));
            }

            lines.push(format!("{}}}", indent));
            lines.join("\n")
        } else {
            String::new()
        }
    }

    fn emit_ordo(&self, stmt: &Stmt, indent: &str) -> String {
        if let Stmt::Ordo {
            nomen,
            membra,
            publica,
            ..
        } = stmt
        {
            let vis = if *publica { "pub " } else { "" };
            let mut lines: Vec<String> = Vec::new();

            lines.push(format!(
                "{}#[derive(Debug, Clone, Copy, PartialEq, Eq)]",
                indent
            ));
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

    fn emit_discretio_decl(&self, stmt: &Stmt, indent: &str) -> String {
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
            lines.push(format!(
                "{}{}enum {}{} {{",
                indent, vis, nomen, generics_str
            ));

            for v in variantes {
                if v.campi.is_empty() {
                    lines.push(format!("{}    {},", indent, v.nomen));
                } else {
                    let fields: Vec<String> = v
                        .campi
                        .iter()
                        .map(|f| format!("{}: {}", f.nomen, self.emit_typus(&f.typus)))
                        .collect();
                    lines.push(format!(
                        "{}    {} {{ {} }},",
                        indent,
                        v.nomen,
                        fields.join(", ")
                    ));
                }
            }

            lines.push(format!("{}}}", indent));
            lines.join("\n")
        } else {
            String::new()
        }
    }

    fn emit_importa(&self, stmt: &Stmt, indent: &str) -> String {
        if let Stmt::Importa {
            fons,
            specs,
            totum,
            alias,
            ..
        } = stmt
        {
            let path = if fons.starts_with("./") {
                format!("crate::{}", fons[2..].replace(".fab", "").replace("/", "::").replace(".", "_"))
            } else if fons.starts_with("../") {
                let mut super_count = 0;
                let mut current_fons: &str = fons;
                while current_fons.starts_with("../") {
                    super_count += 1;
                    current_fons = &current_fons[3..];
                }
                let supers = vec!["super"; super_count].join("::");
                format!("{}::{}", supers, current_fons.replace(".fab", "").replace("/", "::").replace(".", "_"))
            } else {
                fons.replace(".fab", "").replace("/", "::").replace(".", "_")
            };

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

    fn emit_discerne(
        &self,
        discrim: &[Expr],
        casus: &[subsidia_rs::DiscerneCasus],
        indent: &str,
    ) -> String {
        let mut lines: Vec<String> = Vec::new();

        if discrim.len() == 1 {
            let enum_name = self.get_discrim_type_from_patterns(casus);
            lines.push(format!(
                "{}match {} {{",
                indent,
                self.emit_expr(&discrim[0])
            ));

            for c in casus {
                let pattern = &c.patterns[0];

                if pattern.wildcard {
                    lines.push(format!("{}    _ => {{", indent));
                } else if pattern.bindings.is_empty() && pattern.alias.is_none() {
                    lines.push(format!(
                        "{}    {}::{} => {{",
                        indent, enum_name, pattern.variant
                    ));
                } else {
                    let binding_parts: Vec<String> = pattern.bindings.iter().cloned().collect();
                    if let Some(alias) = &pattern.alias {
                        if binding_parts.is_empty() {
                            lines.push(format!(
                                "{}    {} @ {}::{} {{ .. }} => {{",
                                indent, alias, enum_name, pattern.variant
                            ));
                            if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                                for s in stmts {
                                    lines.push(self.emit_stmt(s, &format!("{}        ", indent)));
                                }
                            } else {
                                lines.push(
                                    self.emit_stmt(&c.corpus, &format!("{}        ", indent)),
                                );
                            }
                            lines.push(format!("{}    }}", indent));
                            continue;
                        }
                    }
                    let binding_str = format!("{{ {} }}", binding_parts.join(", "));
                    lines.push(format!(
                        "{}    {}::{} {} => {{",
                        indent, enum_name, pattern.variant, binding_str
                    ));
                }

                if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                    for s in stmts {
                        lines.push(self.emit_stmt(s, &format!("{}        ", indent)));
                    }
                } else {
                    lines.push(self.emit_stmt(&c.corpus, &format!("{}        ", indent)));
                }
                lines.push(format!("{}    }}", indent));
            }

            lines.push(format!("{}}}", indent));
        } else {
            let discrim_tuple: Vec<String> = discrim.iter().map(|d| self.emit_expr(d)).collect();
            lines.push(format!("{}match ({}) {{", indent, discrim_tuple.join(", ")));

            for c in casus {
                let patterns: Vec<String> = c
                    .patterns
                    .iter()
                    .map(|p| {
                        if p.wildcard {
                            "_".to_string()
                        } else {
                            let enum_name = self
                                .find_discretio_for_variant(&p.variant)
                                .unwrap_or_else(|| "Unknown".to_string());
                            format!("{}::{}", enum_name, p.variant)
                        }
                    })
                    .collect();

                lines.push(format!("{}    ({}) => {{", indent, patterns.join(", ")));

                if let Stmt::Massa { corpus: stmts, .. } = c.corpus.as_ref() {
                    for s in stmts {
                        lines.push(self.emit_stmt(s, &format!("{}        ", indent)));
                    }
                } else {
                    lines.push(self.emit_stmt(&c.corpus, &format!("{}        ", indent)));
                }
                lines.push(format!("{}    }}", indent));
            }

            lines.push(format!("{}}}", indent));
        }

        lines.join("\n")
    }

    fn emit_expr(&self, expr: &Expr) -> String {
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
            Expr::Binaria {
                signum, sin, dex, ..
            } => {
                if signum == "inter" {
                    return format!("{}.contains(&{})", self.emit_expr(dex), self.emit_expr(sin));
                }
                if signum == "intra" {
                    let variant_name = self.emit_expr(dex);
                    if let Some(enum_name) = self.find_discretio_for_variant(&variant_name) {
                        return format!("matches!({}, {}::{} {{ .. }})", self.emit_expr(sin), enum_name, variant_name);
                    }
                    return format!("matches!({}, {} {{ .. }})", self.emit_expr(sin), variant_name);
                }
                let op = map_binary_op(signum);
                format!("({} {} {})", self.emit_expr(sin), op, self.emit_expr(dex))
            }
            Expr::Unaria { signum, arg, .. } => {
                let op = map_unary_op(signum);
                format!("({}{})", op, self.emit_expr(arg))
            }
            Expr::Assignatio {
                signum, sin, dex, ..
            } => {
                format!("{} {} {}", self.emit_expr(sin), signum, self.emit_expr(dex))
            }
            Expr::Condicio {
                cond, cons, alt, ..
            } => {
                format!(
                    "if {} {{ {} }} else {{ {} }}",
                    self.emit_expr(cond),
                    self.emit_expr(cons),
                    self.emit_expr(alt)
                )
            }
            Expr::Vocatio { callee, args, .. } => {
                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();

                if let Expr::Membrum { obj, prop, .. } = callee.as_ref() {
                    if let Expr::Littera {
                        valor: prop_name, ..
                    } = prop.as_ref()
                    {
                        if prop_name == "longitudo" {
                            return format!("{}.len()", self.emit_expr(obj));
                        }
                        if let Some(translated) = map_method_name(prop_name) {
                            return format!(
                                "{}.{}({})",
                                self.emit_expr(obj),
                                translated,
                                args_str.join(", ")
                            );
                        }
                    }
                }

                format!("{}({})", self.emit_expr(callee), args_str.join(", "))
            }
            Expr::Membrum {
                obj,
                prop,
                computed,
                ..
            } => {
                let obj_str = self.emit_expr(obj);
                if *computed {
                    return format!("{}[{}]", obj_str, self.emit_expr(prop));
                }

                let sep = if self.is_known_enum(&obj_str) { "::" } else { "." };

                let prop_str = if let Expr::Littera { valor, .. } = prop.as_ref() {
                    if valor == "longitudo" {
                        return format!("{}.len()", obj_str);
                    }
                    if valor == "primus" {
                        return format!("{}[0]", obj_str);
                    }
                    if valor == "ultimus" {
                        return format!("{}.last().unwrap()", obj_str);
                    }
                    sanitize_rs_ident(valor)
                } else {
                    self.emit_expr(prop)
                };
                format!("{}{}{}", obj_str, sep, prop_str)
            }
            Expr::Series { elementa, .. } => {
                let items: Vec<String> = elementa.iter().map(|e| self.emit_expr(e)).collect();
                format!("vec![{}]", items.join(", "))
            }
            Expr::Obiectum { props, .. } => {
                let pairs: Vec<String> = props
                    .iter()
                    .map(|p| {
                        let key = if let Expr::Littera { valor, .. } = &p.key {
                            format!("\"{}\"", valor)
                        } else {
                            self.emit_expr(&p.key)
                        };
                        format!("({}, {})", key, self.emit_expr(&p.valor))
                    })
                    .collect();
                format!("HashMap::from([{}])", pairs.join(", "))
            }
            Expr::Clausura { params, corpus, .. } => {
                let params_str: Vec<String> = params
                    .iter()
                    .map(|p| {
                        if let Some(t) = &p.typus {
                            format!("{}: {}", p.nomen, self.emit_typus(t))
                        } else {
                            p.nomen.clone()
                        }
                    })
                    .collect();
                let body = match corpus {
                    ClausuraCorpus::Stmt(s) => self.emit_stmt(s, ""),
                    ClausuraCorpus::Expr(e) => self.emit_expr(e),
                };
                format!("|{}| {}", params_str.join(", "), body)
            }
            Expr::Novum { callee, args, .. } => {
                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                format!("{}::new({})", self.emit_expr(callee), args_str.join(", "))
            }
            Expr::Cede { arg, .. } => format!("{}.await", self.emit_expr(arg)),
            Expr::Qua { expr, typus, .. } => {
                format!("({} as {})", self.emit_expr(expr), self.emit_typus(typus))
            }
            Expr::Innatum { expr, typus, .. } => {
                format!("({} as {})", self.emit_expr(expr), self.emit_typus(typus))
            }
            Expr::Conversio {
                expr,
                species,
                fallback,
                ..
            } => {
                let conversion = match species.as_str() {
                    "numeratum" => format!("{}.parse::<i64>()", self.emit_expr(expr)),
                    "fractatum" => format!("{}.parse::<f64>()", self.emit_expr(expr)),
                    "textatum" => format!("{}.to_string()", self.emit_expr(expr)),
                    "bivalentum" => format!("({} != 0)", self.emit_expr(expr)),
                    _ => format!(
                        "/* unknown conversion {} */ {}",
                        species,
                        self.emit_expr(expr)
                    ),
                };
                if let Some(fb) = fallback {
                    format!("{}.unwrap_or({})", conversion, self.emit_expr(fb))
                } else if species == "numeratum" || species == "fractatum" {
                    format!("{}.unwrap()", conversion)
                } else {
                    conversion
                }
            }
            Expr::PostfixNovum { expr, typus, .. } => {
                format!("{}::from({})", self.emit_typus(typus), self.emit_expr(expr))
            }
            Expr::Finge { variant, campi, .. } => {
                let fields: Vec<String> = campi
                    .iter()
                    .map(|p| {
                        let key = if let Expr::Littera { valor, .. } = &p.key {
                            valor.clone()
                        } else {
                            self.emit_expr(&p.key)
                        };
                        format!("{}: {}", sanitize_rs_ident(&key), self.emit_expr(&p.valor))
                    })
                    .collect();
                format!("{} {{ {} }}", variant, fields.join(", "))
            }
            Expr::Scriptum { template, args, .. } => {
                let parts: Vec<&str> = template.split('§').collect();
                if parts.len() == 1 {
                    return format!("{}.to_string()", quote_string(template));
                }
                let format_str = parts.iter().map(|p| p.replace("{", "{{").replace("}", "}}")).collect::<Vec<_>>().join("{}");
                let args_str: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                format!("format!(\"{}\", {})", format_str, args_str.join(", "))
            }
            Expr::Ambitus {
                start,
                end,
                inclusive,
                ..
            } => {
                if *inclusive {
                    format!("({}..={})", self.emit_expr(start), self.emit_expr(end))
                } else {
                    format!("({}..{})", self.emit_expr(start), self.emit_expr(end))
                }
            }
        }
    }

    fn emit_typus(&self, typus: &Typus) -> String {
        match typus {
            Typus::Nomen { nomen } => map_type_name(nomen),
            Typus::Nullabilis { inner } => format!("Option<{}>", self.emit_typus(inner)),
            Typus::Genericus { nomen, args } => {
                let args_str: Vec<String> = args.iter().map(|t| self.emit_typus(t)).collect();
                format!("{}<{}>", map_type_name(nomen), args_str.join(", "))
            }
            Typus::Functio { params, returns } => {
                let params_str: Vec<String> = params.iter().map(|t| self.emit_typus(t)).collect();
                let ret = if let Some(r) = returns {
                    self.emit_typus(r)
                } else {
                    "()".to_string()
                };
                format!("fn({}) -> {}", params_str.join(", "), ret)
            }
            Typus::Unio { members } => {
                let parts: Vec<String> = members.iter().map(|t| self.emit_typus(t)).collect();
                format!("/* {} */", parts.join(" | "))
            }
            Typus::Litteralis { valor } => valor.clone(),
        }
    }

    fn emit_param(&self, p: &Param) -> String {
        let ownership = if let Some(o) = &p.ownership {
            match o.as_str() {
                "de" => "&",
                "in" => "&mut ",
                _ => "",
            }
        } else {
            ""
        };

        let typ = if let Some(t) = &p.typus {
            format!(": {}{}", ownership, self.emit_typus(t))
        } else {
            String::new()
        };

        format!("{}{}", sanitize_rs_ident(&p.nomen), typ)
    }
}

/// Emit a module to Rust source code.
pub fn emit_rs(module: &Modulus) -> String {
    let ctx = analyze(module);
    let emitter = RsEmitter::new(&ctx);

    let mut lines: Vec<String> = Vec::new();
    lines.push("use std::collections::HashMap;".to_string());
    lines.push(String::new());

    for stmt in &module.corpus {
        lines.push(emitter.emit_stmt(stmt, ""));
    }
    lines.join("\n")
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
        "et" | "&&" => "&&",
        "aut" | "||" => "||",
        "vel" => ".unwrap_or",
        "inter" => "/* handled in emit_expr */",
        "intra" => "/* handled in emit_expr */",
        "+" => "+",
        "-" => "-",
        "*" => "*",
        "/" => "/",
        "%" => "%",
        "==" | "===" => "==",
        "!=" | "!==" => "!=",
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
        "nonnihil" => "",
        "positivum" => "",
        "-" => "-",
        "!" => "!",
        _ => "/* unknown op */",
    }
}

fn map_method_name(name: &str) -> Option<&'static str> {
    let map: HashMap<&str, &str> = [
        ("adde", "push"),
        ("praepone", "insert"),
        ("remove", "pop"),
        ("decapita", "remove"),
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

fn sanitize_rs_ident(s: &str) -> String {
    match s {
        "as" | "async" | "await" | "break" | "const" | "continue" | "crate" | "dyn" | "else" | "enum" |
        "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let" | "loop" | "match" | "mod" |
        "move" | "mut" | "pub" | "ref" | "return" | "self" | "Self" | "static" | "struct" | "super" |
        "trait" | "true" | "type" | "unsafe" | "use" | "where" | "while" => {
            format!("r#{}", s)
        }
        _ => s.to_string(),
    }
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
