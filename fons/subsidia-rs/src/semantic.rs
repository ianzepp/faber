use crate::{
    Expr, Locus, Modulus, Param, SemanticContext, SemanticTypus, ScopusSpecies, Stmt, Symbolum,
    SymbolSpecies, Typus,
};
use crate::types::*;
use std::collections::HashMap;

/// Perform semantic analysis on a parsed module.
pub fn analyze(module: &Modulus) -> SemanticContext {
    let mut ctx = SemanticContext::new();

    // Pass 1: Collect all type declarations
    for stmt in &module.corpus {
        collect_declaration(&mut ctx, stmt);
    }

    // Pass 2: Resolve types and analyze bodies
    for stmt in &module.corpus {
        analyze_statement(&mut ctx, stmt);
    }

    ctx
}

fn collect_declaration(ctx: &mut SemanticContext, stmt: &Stmt) {
    match stmt {
        Stmt::Genus {
            locus,
            nomen,
            campi,
            methodi,
            ..
        } => {
            let mut agri = HashMap::new();
            for campo in campi {
                let field_type = campo
                    .typus
                    .as_ref()
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(ignotum);
                agri.insert(campo.nomen.clone(), field_type);
            }

            let mut methodi_map = HashMap::new();
            for method in methodi {
                if let Stmt::Functio {
                    nomen: fn_name,
                    params,
                    typus_reditus,
                    ..
                } = method
                {
                    let func_type = resolve_functio_typus(ctx, params, typus_reditus.as_ref());
                    methodi_map.insert(fn_name.clone(), Box::new(func_type));
                }
            }

            let genus_type = SemanticTypus::Genus {
                nomen: nomen.clone(),
                agri,
                methodi: methodi_map,
                nullabilis: false,
            };

            ctx.genus_registry.insert(nomen.clone(), genus_type.clone());
            ctx.register_typus(nomen.clone(), genus_type.clone());
            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: genus_type,
                species: SymbolSpecies::Genus,
                mutabilis: false,
                locus: *locus,
            });
        }
        Stmt::Ordo {
            locus, nomen, membra, ..
        } => {
            let mut membra_map = HashMap::new();
            for (i, m) in membra.iter().enumerate() {
                membra_map.insert(m.nomen.clone(), i as i64);
            }

            let ordo_type = SemanticTypus::Ordo {
                nomen: nomen.clone(),
                membra: membra_map,
            };

            ctx.ordo_registry.insert(nomen.clone(), ordo_type.clone());
            ctx.register_typus(nomen.clone(), ordo_type.clone());
            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: ordo_type,
                species: SymbolSpecies::Ordo,
                mutabilis: false,
                locus: *locus,
            });
        }
        Stmt::Discretio {
            locus,
            nomen,
            variantes,
            ..
        } => {
            let mut variantes_map = HashMap::new();
            for v in variantes {
                let mut agri = HashMap::new();
                for f in &v.campi {
                    agri.insert(f.nomen.clone(), resolve_typus_annotatio(ctx, &f.typus));
                }
                let variant_type = SemanticTypus::Genus {
                    nomen: v.nomen.clone(),
                    agri,
                    methodi: HashMap::new(),
                    nullabilis: false,
                };
                variantes_map.insert(v.nomen.clone(), Box::new(variant_type.clone()));

                ctx.define(Symbolum {
                    nomen: v.nomen.clone(),
                    typus: variant_type,
                    species: SymbolSpecies::Varians,
                    mutabilis: false,
                    locus: v.locus,
                });
            }

            let disc_type = SemanticTypus::Discretio {
                nomen: nomen.clone(),
                variantes: variantes_map,
            };

            ctx.disc_registry.insert(nomen.clone(), disc_type.clone());
            ctx.register_typus(nomen.clone(), disc_type.clone());
            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: disc_type,
                species: SymbolSpecies::Discretio,
                mutabilis: false,
                locus: *locus,
            });
        }
        Stmt::Pactum {
            locus, nomen, methodi, ..
        } => {
            let mut methodi_map = HashMap::new();
            for m in methodi {
                let func_type = resolve_functio_typus(ctx, &m.params, m.typus_reditus.as_ref());
                methodi_map.insert(m.nomen.clone(), Box::new(func_type));
            }

            let pactum_type = SemanticTypus::Pactum {
                nomen: nomen.clone(),
                methodi: methodi_map,
            };

            ctx.register_typus(nomen.clone(), pactum_type.clone());
            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: pactum_type,
                species: SymbolSpecies::Pactum,
                mutabilis: false,
                locus: *locus,
            });
        }
        Stmt::Functio {
            locus,
            nomen,
            params,
            typus_reditus,
            externa,
            ..
        } => {
            if *externa {
                return;
            }
            let func_type = resolve_functio_typus(ctx, params, typus_reditus.as_ref());
            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: func_type,
                species: SymbolSpecies::Functio,
                mutabilis: false,
                locus: *locus,
            });
        }
        _ => {}
    }
}

fn resolve_functio_typus(
    ctx: &SemanticContext,
    params: &[Param],
    typus_reditus: Option<&Typus>,
) -> SemanticTypus {
    let param_types: Vec<SemanticTypus> = params
        .iter()
        .map(|p| {
            p.typus
                .as_ref()
                .map(|t| resolve_typus_annotatio(ctx, t))
                .unwrap_or_else(ignotum)
        })
        .collect();

    let reditus = typus_reditus.map(|t| Box::new(resolve_typus_annotatio(ctx, t)));

    SemanticTypus::Functio {
        params: param_types,
        reditus,
        nullabilis: false,
    }
}

fn resolve_typus_annotatio(ctx: &SemanticContext, typus: &Typus) -> SemanticTypus {
    match typus {
        Typus::Nomen { nomen } => ctx.resolve_typus_nomen(nomen),
        Typus::Nullabilis { inner } => {
            let inner_type = resolve_typus_annotatio(ctx, inner);
            nullabilis(inner_type)
        }
        Typus::Genericus { nomen, args } => match nomen.as_str() {
            "lista" => {
                let elem = args
                    .first()
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(ignotum);
                SemanticTypus::Lista {
                    elementum: Box::new(elem),
                    nullabilis: false,
                }
            }
            "tabula" => {
                let clavis = args
                    .first()
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(textus);
                let valor = args
                    .get(1)
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(ignotum);
                SemanticTypus::Tabula {
                    clavis: Box::new(clavis),
                    valor: Box::new(valor),
                    nullabilis: false,
                }
            }
            "copia" | "collectio" => {
                let elem = args
                    .first()
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(ignotum);
                SemanticTypus::Copia {
                    elementum: Box::new(elem),
                    nullabilis: false,
                }
            }
            _ => SemanticTypus::Usitatum {
                nomen: nomen.clone(),
                nullabilis: false,
            },
        },
        Typus::Functio { params, returns } => {
            let param_types: Vec<SemanticTypus> =
                params.iter().map(|p| resolve_typus_annotatio(ctx, p)).collect();
            let reditus = returns.as_ref().map(|t| Box::new(resolve_typus_annotatio(ctx, t)));
            SemanticTypus::Functio {
                params: param_types,
                reditus,
                nullabilis: false,
            }
        }
        Typus::Unio { members } => {
            let member_types: Vec<SemanticTypus> =
                members.iter().map(|m| resolve_typus_annotatio(ctx, m)).collect();
            SemanticTypus::Unio {
                membra: member_types,
                nullabilis: false,
            }
        }
        Typus::Litteralis { .. } => textus(),
    }
}

fn analyze_statement(ctx: &mut SemanticContext, stmt: &Stmt) {
    match stmt {
        Stmt::Massa { corpus, .. } => {
            ctx.enter_scope(ScopusSpecies::Massa, "");
            for inner in corpus {
                analyze_statement(ctx, inner);
            }
            ctx.exit_scope();
        }
        Stmt::Varia {
            locus,
            nomen,
            typus,
            valor,
            species,
            externa,
            ..
        } => {
            if *externa {
                return;
            }
            let mut var_type = typus
                .as_ref()
                .map(|t| resolve_typus_annotatio(ctx, t))
                .unwrap_or_else(ignotum);

            if let Some(init) = valor {
                let init_type = analyze_expression(ctx, init);
                if typus.is_none() {
                    var_type = init_type;
                }
            }

            ctx.define(Symbolum {
                nomen: nomen.clone(),
                typus: var_type,
                species: SymbolSpecies::Variabilis,
                mutabilis: *species == crate::VariaSpecies::Varia,
                locus: *locus,
            });
        }
        Stmt::Functio {
            nomen,
            params,
            corpus,
            externa,
            ..
        } => {
            if *externa || corpus.is_none() {
                return;
            }
            ctx.enter_scope(ScopusSpecies::Functio, nomen);
            for p in params {
                let param_type = p
                    .typus
                    .as_ref()
                    .map(|t| resolve_typus_annotatio(ctx, t))
                    .unwrap_or_else(ignotum);
                ctx.define(Symbolum {
                    nomen: p.nomen.clone(),
                    typus: param_type,
                    species: SymbolSpecies::Parametrum,
                    mutabilis: false,
                    locus: p.locus,
                });
            }
            if let Some(body) = corpus {
                analyze_statement(ctx, body);
            }
            ctx.exit_scope();
        }
        Stmt::Genus {
            nomen, methodi, ..
        } => {
            for method in methodi {
                if let Stmt::Functio { .. } = method {
                    ctx.enter_scope(ScopusSpecies::Genus, nomen);
                    if let Some(genus) = ctx.genus_registry.get(nomen) {
                        ctx.define(Symbolum {
                            nomen: "ego".to_string(),
                            typus: genus.clone(),
                            species: SymbolSpecies::Variabilis,
                            mutabilis: false,
                            locus: Locus::default(),
                        });
                    }
                    analyze_statement(ctx, method);
                    ctx.exit_scope();
                }
            }
        }
        Stmt::Si { cond, cons, alt, .. } => {
            analyze_expression(ctx, cond);
            analyze_statement(ctx, cons);
            if let Some(alt_stmt) = alt {
                analyze_statement(ctx, alt_stmt);
            }
        }
        Stmt::Dum { cond, corpus, .. } => {
            analyze_expression(ctx, cond);
            analyze_statement(ctx, corpus);
        }
        Stmt::FacDum { corpus, cond, .. } => {
            analyze_statement(ctx, corpus);
            analyze_expression(ctx, cond);
        }
        Stmt::Iteratio {
            binding,
            iter,
            corpus,
            ..
        } => {
            analyze_expression(ctx, iter);
            ctx.enter_scope(ScopusSpecies::Massa, "");
            ctx.define(Symbolum {
                nomen: binding.clone(),
                typus: ignotum(),
                species: SymbolSpecies::Variabilis,
                mutabilis: false,
                locus: Locus::default(),
            });
            analyze_statement(ctx, corpus);
            ctx.exit_scope();
        }
        Stmt::Elige {
            discrim,
            casus,
            default,
            ..
        } => {
            analyze_expression(ctx, discrim);
            for c in casus {
                analyze_expression(ctx, &c.cond);
                analyze_statement(ctx, &c.corpus);
            }
            if let Some(def) = default {
                analyze_statement(ctx, def);
            }
        }
        Stmt::Discerne { discrim, casus, .. } => {
            for d in discrim {
                analyze_expression(ctx, d);
            }
            for c in casus {
                ctx.enter_scope(ScopusSpecies::Massa, "");
                for p in &c.patterns {
                    for b in &p.bindings {
                        ctx.define(Symbolum {
                            nomen: b.clone(),
                            typus: ignotum(),
                            species: SymbolSpecies::Variabilis,
                            mutabilis: false,
                            locus: p.locus,
                        });
                    }
                    if let Some(alias) = &p.alias {
                        ctx.define(Symbolum {
                            nomen: alias.clone(),
                            typus: ignotum(),
                            species: SymbolSpecies::Variabilis,
                            mutabilis: false,
                            locus: p.locus,
                        });
                    }
                }
                analyze_statement(ctx, &c.corpus);
                ctx.exit_scope();
            }
        }
        Stmt::Redde { valor, .. } => {
            if let Some(v) = valor {
                analyze_expression(ctx, v);
            }
        }
        Stmt::Expressia { expr, .. } => {
            analyze_expression(ctx, expr);
        }
        Stmt::Scribe { args, .. } => {
            for arg in args {
                analyze_expression(ctx, arg);
            }
        }
        Stmt::Adfirma { cond, msg, .. } => {
            analyze_expression(ctx, cond);
            if let Some(m) = msg {
                analyze_expression(ctx, m);
            }
        }
        Stmt::Iace { arg, .. } => {
            analyze_expression(ctx, arg);
        }
        Stmt::Custodi { clausulae, .. } => {
            for c in clausulae {
                analyze_expression(ctx, &c.cond);
                analyze_statement(ctx, &c.corpus);
            }
        }
        Stmt::Incipit { corpus, .. } => {
            analyze_statement(ctx, corpus);
        }
        Stmt::Tempta { corpus, cape, demum, .. } => {
            analyze_statement(ctx, corpus);
            if let Some(c) = cape {
                analyze_statement(ctx, &c.corpus);
            }
            if let Some(d) = demum {
                analyze_statement(ctx, d);
            }
        }
        _ => {}
    }
}

fn analyze_expression(ctx: &mut SemanticContext, expr: &Expr) -> SemanticTypus {
    let result = match expr {
        Expr::Littera { species, .. } => match species {
            crate::LitteraSpecies::Textus => textus(),
            crate::LitteraSpecies::Numerus => numerus(),
            crate::LitteraSpecies::Fractus => fractus(),
            crate::LitteraSpecies::Verum | crate::LitteraSpecies::Falsum => bivalens(),
            crate::LitteraSpecies::Nihil => nihil(),
        },
        Expr::Nomen { valor, locus, .. } => {
            if let Some(sym) = ctx.lookup(valor) {
                sym.typus.clone()
            } else {
                ctx.error(format!("undefined identifier: {}", valor), *locus);
                ignotum()
            }
        }
        Expr::Ego { locus, .. } => {
            if let Some(sym) = ctx.lookup("ego") {
                sym.typus.clone()
            } else {
                ctx.error("'ego' used outside of class context", *locus);
                ignotum()
            }
        }
        Expr::Binaria { signum, sin, dex, .. } => {
            let left_type = analyze_expression(ctx, sin);
            let right_type = analyze_expression(ctx, dex);

            match signum.as_str() {
                "+" | "-" | "*" | "/" | "%" => {
                    if left_type.is_numeric() && right_type.is_numeric() {
                        if left_type.is_fractus() || right_type.is_fractus() {
                            fractus()
                        } else {
                            numerus()
                        }
                    } else if signum == "+" && left_type.is_textus() {
                        textus()
                    } else {
                        ignotum()
                    }
                }
                "==" | "!=" | "<" | ">" | "<=" | ">=" | "et" | "aut" | "&&" | "||" => bivalens(),
                "vel" => left_type,
                _ => ignotum(),
            }
        }
        Expr::Unaria { signum, arg, .. } => {
            let arg_type = analyze_expression(ctx, arg);
            match signum.as_str() {
                "non" | "!" | "nihil" | "nonnihil" => bivalens(),
                "-" | "+" | "positivum" | "negativum" => arg_type,
                _ => arg_type,
            }
        }
        Expr::Assignatio { sin, dex, .. } => {
            analyze_expression(ctx, sin);
            analyze_expression(ctx, dex)
        }
        Expr::Condicio { cond, cons, alt, .. } => {
            analyze_expression(ctx, cond);
            let cons_type = analyze_expression(ctx, cons);
            analyze_expression(ctx, alt);
            cons_type
        }
        Expr::Vocatio { callee, args, .. } => {
            for arg in args {
                analyze_expression(ctx, arg);
            }
            let callee_type = analyze_expression(ctx, callee);
            if let SemanticTypus::Functio { reditus, .. } = callee_type {
                reditus.map(|r| *r).unwrap_or_else(vacuum)
            } else {
                ignotum()
            }
        }
        Expr::Membrum { obj, prop, computed, .. } => {
            let obj_type = analyze_expression(ctx, obj);
            if *computed {
                analyze_expression(ctx, prop);
            }
            match obj_type {
                SemanticTypus::Lista { elementum, .. } if *computed => *elementum,
                SemanticTypus::Tabula { valor, .. } if *computed => *valor,
                SemanticTypus::Genus { agri, methodi, .. } => {
                    if let Expr::Littera { valor: prop_name, .. } = prop.as_ref() {
                        if let Some(field_type) = agri.get(prop_name) {
                            field_type.clone()
                        } else if let Some(method_type) = methodi.get(prop_name) {
                            *method_type.clone()
                        } else {
                            ignotum()
                        }
                    } else {
                        ignotum()
                    }
                }
                SemanticTypus::Ordo { .. } => obj_type,
                _ => ignotum(),
            }
        }
        Expr::Series { elementa, .. } => {
            let elem_type = elementa
                .first()
                .map(|e| analyze_expression(ctx, e))
                .unwrap_or_else(ignotum);
            for e in elementa.iter().skip(1) {
                analyze_expression(ctx, e);
            }
            SemanticTypus::Lista {
                elementum: Box::new(elem_type),
                nullabilis: false,
            }
        }
        Expr::Obiectum { props, .. } => {
            let mut fields = HashMap::new();
            for p in props {
                let value_type = analyze_expression(ctx, &p.valor);
                if let Expr::Littera { valor: key, .. } = &p.key {
                    fields.insert(key.clone(), value_type);
                }
            }
            SemanticTypus::Genus {
                nomen: String::new(),
                agri: fields,
                methodi: HashMap::new(),
                nullabilis: false,
            }
        }
        Expr::Clausura { params, corpus, .. } => {
            ctx.enter_scope(ScopusSpecies::Functio, "");
            let param_types: Vec<SemanticTypus> = params
                .iter()
                .map(|p| {
                    let t = p
                        .typus
                        .as_ref()
                        .map(|typ| resolve_typus_annotatio(ctx, typ))
                        .unwrap_or_else(ignotum);
                    ctx.define(Symbolum {
                        nomen: p.nomen.clone(),
                        typus: t.clone(),
                        species: SymbolSpecies::Parametrum,
                        mutabilis: false,
                        locus: p.locus,
                    });
                    t
                })
                .collect();

            let reditus = match corpus {
                crate::ClausuraCorpus::Stmt(s) => {
                    analyze_statement(ctx, s);
                    None
                }
                crate::ClausuraCorpus::Expr(e) => Some(Box::new(analyze_expression(ctx, e))),
            };

            ctx.exit_scope();

            SemanticTypus::Functio {
                params: param_types,
                reditus,
                nullabilis: false,
            }
        }
        Expr::Novum { callee, args, init, .. } => {
            for arg in args {
                analyze_expression(ctx, arg);
            }
            if let Some(i) = init {
                analyze_expression(ctx, i);
            }
            if let Expr::Nomen { valor, .. } = callee.as_ref() {
                if let Some(genus) = ctx.genus_registry.get(valor) {
                    genus.clone()
                } else {
                    SemanticTypus::Usitatum {
                        nomen: valor.clone(),
                        nullabilis: false,
                    }
                }
            } else {
                ignotum()
            }
        }
        Expr::Cede { arg, .. } => analyze_expression(ctx, arg),
        Expr::Qua { typus, .. } => resolve_typus_annotatio(ctx, typus),
        Expr::Innatum { typus, .. } => resolve_typus_annotatio(ctx, typus),
        Expr::Conversio { expr, species, fallback, .. } => {
            analyze_expression(ctx, expr);
            if let Some(fb) = fallback {
                analyze_expression(ctx, fb);
            }
            match species.as_str() {
                "numeratum" => numerus(),
                "fractatum" => fractus(),
                "textatum" => textus(),
                "bivalentum" => bivalens(),
                _ => ignotum(),
            }
        }
        Expr::PostfixNovum { typus, .. } => resolve_typus_annotatio(ctx, typus),
        Expr::Finge { variant, campi, .. } => {
            for p in campi {
                analyze_expression(ctx, &p.valor);
            }
            if let Some(sym) = ctx.lookup(variant) {
                sym.typus.clone()
            } else {
                SemanticTypus::Usitatum {
                    nomen: variant.clone(),
                    nullabilis: false,
                }
            }
        }
        Expr::Scriptum { args, .. } => {
            for arg in args {
                analyze_expression(ctx, arg);
            }
            textus()
        }
        Expr::Ambitus { start, end, .. } => {
            analyze_expression(ctx, start);
            analyze_expression(ctx, end);
            SemanticTypus::Lista {
                elementum: Box::new(numerus()),
                nullabilis: false,
            }
        }
    };

    result
}
