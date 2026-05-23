//! Declaration lowering for item-shaped HIR.
//!
//! This file owns the item boundary of HIR lowering: top-level Faber
//! declarations become `HirItem`s with stable names, resolver-backed or
//! synthetic definition ids, lowered type annotations, and bodies normalized
//! enough for type checking. Statement and expression lowering remain in their
//! sibling modules; declarations coordinate those pieces because item shape,
//! local parameter scope, and member ownership have to be established together.
//!
//! INVARIANTS
//! ==========
//! - Top-level item names prefer resolver `DefId`s; missing resolver entries use
//!   synthetic ids so lowering can keep producing diagnostics.
//! - Function and method parameters are bound before bodies lower, because body
//!   expressions resolve locals through the lowerer scope stack.
//! - Type syntax is lowered into semantic type ids, but type compatibility and
//!   interface conformance are deferred to later semantic/typecheck passes.
//! - Declaration spans stay attached to the resulting HIR item, while fields,
//!   variants, parameters, and synthesized locals keep their narrower spans.
//!
//! ERROR STRATEGY
//! ==============
//! This pass reports declaration-shape errors, such as unsupported top-level
//! bindings or missing base/interface names, then builds the best HIR it can.
//! Extern top-level variables without source initializers deliberately receive a
//! nil placeholder; ordinary top-level variables without initializers receive an
//! error expression so downstream passes can continue from the declaration.
//!
//! MEMBER POLICY
//! =============
//! `gens` members are lowered in one pass so fields, methods, current `ego`
//! context, modifier locals, and inherited/implemented type references stay on
//! the same HIR struct. `ordo` and `discretio` both lower to enum-shaped HIR:
//! plain `ordo` variants have no payload fields, while `discretio` preserves
//! payload type information for later pattern and type analysis.

use super::Lowerer;
use crate::hir::{
    HirConst, HirEnum, HirExpr, HirExprKind, HirField, HirFunction, HirImport, HirImportItem, HirInterface,
    HirInterfaceMethod, HirItem, HirItemKind, HirLiteral, HirMethod, HirParam, HirParamMode, HirReceiver, HirStruct,
    HirTypeAlias, HirTypeParam, HirVariant, HirVariantField,
};
use crate::lexer::Span;
use crate::syntax::{
    ClassDecl, EnumDecl, FuncDecl, ImportDecl, InterfaceDecl, Stmt, TypeAliasDecl, UnionDecl, VarDecl,
};

fn error_expr(lowerer: &mut Lowerer, span: Span) -> HirExpr {
    HirExpr { id: lowerer.next_hir_id(), kind: HirExprKind::Error, ty: None, span }
}

fn lower_param_mode(mode: crate::syntax::ParamMode) -> HirParamMode {
    match mode {
        crate::syntax::ParamMode::Owned => HirParamMode::Owned,
        crate::syntax::ParamMode::Ref => HirParamMode::Ref,
        crate::syntax::ParamMode::MutRef => HirParamMode::MutRef,
        crate::syntax::ParamMode::Move => HirParamMode::Move,
    }
}

impl<'a> Lowerer<'a> {
    fn has_externa_annotation(&self, stmt: &Stmt) -> bool {
        stmt.annotations
            .iter()
            .any(|annotation| match &annotation.kind {
                crate::syntax::AnnotationKind::Externa => true,
                crate::syntax::AnnotationKind::Statement(stmt) => self.interner.resolve(stmt.name.name) == "externa",
                _ => false,
            })
    }

    /// Lower a top-level `varia`/`fixum` declaration into a const item.
    ///
    /// HIR currently represents top-level variables as item constants. This
    /// method enforces the item-level identifier binding policy and preserves an
    /// explicit error node when an initializer is required but missing.
    pub fn lower_varia(&mut self, stmt: &Stmt, decl: &VarDecl) -> Option<HirItem> {
        let (name, name_span) = match &decl.binding {
            crate::syntax::BindingPattern::Ident(ident) => (ident.name, ident.span),
            _ => {
                self.error("top-level variable declaration requires identifier binding");
                return None;
            }
        };

        let def_id = self.def_id_for(name);
        let ty = decl.ty.as_ref().map(|ty| self.lower_type(ty));
        let value = match &decl.init {
            Some(init) => self.lower_expr(init),
            None => {
                if self.has_externa_annotation(stmt) {
                    // Extern declarations promise a target-provided value; the
                    // placeholder keeps HIR total without pretending source had
                    // an initializer.
                    HirExpr {
                        id: self.next_hir_id(),
                        kind: HirExprKind::Literal(HirLiteral::Nil),
                        ty: None,
                        span: name_span,
                    }
                } else {
                    self.error("top-level variable declaration requires initializer");
                    error_expr(self, name_span)
                }
            }
        };

        let item = HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Const(HirConst { name, ty, value }),
            span: stmt.span,
        };

        Some(item)
    }

    /// Lower a `functio` declaration into a function item.
    ///
    /// Parameter and modifier bindings are registered before the body lowers so
    /// body expressions can resolve them as locals. CLI command argument
    /// metadata, when present, is represented as a synthetic parameter rather
    /// than a separate HIR concept.
    pub fn lower_functio(&mut self, stmt: &Stmt, decl: &FuncDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam { def_id: self.next_def_id(), name: param.name.name, span: param.span })
            .collect();

        let mut params = Vec::new();
        for param in &decl.params {
            let param_def_id = self.next_def_id();
            let binding_name = param.alias.as_ref().unwrap_or(&param.name);
            let is_optional = param.sponte || param.default.is_some();
            params.push(HirParam {
                def_id: param_def_id,
                name: binding_name.name,
                ty: self.lower_type(&param.ty),
                mode: lower_param_mode(param.mode),
                optional: is_optional,
                sponte: param.sponte,
                fixus: param.fixus,
                span: param.span,
            });
        }
        let cli_args = command_argument_binding(&decl.modifiers).map(|ident| {
            let def_id = self.next_def_id();
            HirParam {
                def_id,
                name: ident.name,
                ty: self.command_args_type(decl.name.name),
                mode: HirParamMode::Owned,
                optional: false,
                sponte: false,
                fixus: false,
                span: ident.span,
            }
        });

        let ret_ty = decl.ret.as_ref().map(|ty| self.lower_type(ty));
        self.push_scope();
        for param in &params {
            self.bind_local(param.name, param.def_id);
        }
        if let Some(param) = &cli_args {
            self.bind_local(param.name, param.def_id);
        }
        let mut modifier_locals = Vec::new();
        for ident in modifier_bindings(&decl.modifiers) {
            let def_id = self.next_def_id();
            self.bind_local(ident.name, def_id);
            modifier_locals.push((def_id, ident.name, ident.span));
        }
        let body = decl.body.as_ref().map(|body| self.lower_block(body));
        self.pop_scope();
        let body = body.map(|mut body| {
            // Function modifiers can bind values supplied by the runtime or
            // generated CLI adapter. HIR models them as locals so later passes
            // use the same name/type path as ordinary source bindings.
            for (def_id, name, span) in modifier_locals.into_iter().rev() {
                body.stmts.insert(
                    0,
                    crate::hir::HirStmt {
                        id: self.next_hir_id(),
                        kind: crate::hir::HirStmtKind::Local(crate::hir::HirLocal {
                            def_id,
                            name,
                            ty: Some(self.types.primitive(crate::semantic::Primitive::Ignotum)),
                            init: None,
                            mutable: true,
                        }),
                        span,
                    },
                );
            }
            body
        });

        let func = HirFunction {
            name: decl.name.name,
            type_params,
            params,
            cli_args,
            ret_ty,
            err_ty: decl.err.as_ref().map(|ty| self.lower_type(ty)),
            body,
            is_async: false,
            is_generator: false,
            test: None,
        };

        Some(HirItem { id: self.next_hir_id(), def_id, kind: HirItemKind::Function(func), span: stmt.span })
    }

    /// Lower a `gens` declaration into struct-shaped HIR.
    ///
    /// `gens` lowering owns member scope setup because method bodies need both
    /// ordinary parameter locals and the current `ego` struct context. Inherited
    /// and implemented names are looked up here, but their semantic validity is
    /// checked later.
    pub fn lower_gens(&mut self, stmt: &Stmt, decl: &ClassDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam { def_id: self.next_def_id(), name: param.name.name, span: param.span })
            .collect();

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        for member in &decl.members {
            match &member.kind {
                crate::syntax::ClassMemberKind::Field(field) => {
                    let init = field.init.as_ref().map(|expr| self.lower_expr(expr));
                    fields.push(HirField {
                        def_id: self.next_def_id(),
                        name: field.name.name,
                        ty: self.lower_type(&field.ty),
                        is_static: field.is_static,
                        sponte: field.sponte,
                        fixus: field.fixus,
                        init,
                        span: member.span,
                    });
                }
                crate::syntax::ClassMemberKind::Method(method) => {
                    let mut params = Vec::new();
                    for param in &method.params {
                        let param_def_id = self.next_def_id();
                        let binding_name = param.alias.as_ref().unwrap_or(&param.name);
                        let is_optional = param.sponte || param.default.is_some();
                        params.push(HirParam {
                            def_id: param_def_id,
                            name: binding_name.name,
                            ty: self.lower_type(&param.ty),
                            mode: lower_param_mode(param.mode),
                            optional: is_optional,
                            sponte: param.sponte,
                            fixus: param.fixus,
                            span: param.span,
                        });
                    }

                    self.push_scope();
                    for param in &params {
                        self.bind_local(param.name, param.def_id);
                    }
                    let prev_ego = self.current_ego_struct;
                    self.current_ego_struct = Some(def_id);
                    let mut modifier_locals = Vec::new();
                    for ident in modifier_bindings(&method.modifiers) {
                        let def_id = self.next_def_id();
                        self.bind_local(ident.name, def_id);
                        modifier_locals.push((def_id, ident.name, ident.span));
                    }
                    let body = method.body.as_ref().map(|block| self.lower_block(block));
                    self.pop_scope();
                    self.current_ego_struct = prev_ego;
                    let body = body.map(|mut body| {
                        // Method modifiers follow the same synthetic-local
                        // contract as function modifiers, but lower while the
                        // current `ego` struct is still known.
                        for (def_id, name, span) in modifier_locals.into_iter().rev() {
                            body.stmts.insert(
                                0,
                                crate::hir::HirStmt {
                                    id: self.next_hir_id(),
                                    kind: crate::hir::HirStmtKind::Local(crate::hir::HirLocal {
                                        def_id,
                                        name,
                                        ty: Some(self.types.primitive(crate::semantic::Primitive::Ignotum)),
                                        init: None,
                                        mutable: true,
                                    }),
                                    span,
                                },
                            );
                        }
                        body
                    });

                    let func = HirFunction {
                        name: method.name.name,
                        type_params: method
                            .type_params
                            .iter()
                            .map(|param| HirTypeParam {
                                def_id: self.next_def_id(),
                                name: param.name.name,
                                span: param.span,
                            })
                            .collect(),
                        params,
                        cli_args: None,
                        ret_ty: method.ret.as_ref().map(|ty| self.lower_type(ty)),
                        err_ty: method.err.as_ref().map(|ty| self.lower_type(ty)),
                        body,
                        is_async: false,
                        is_generator: false,
                        test: None,
                    };

                    methods.push(HirMethod {
                        def_id: self.next_def_id(),
                        func,
                        receiver: HirReceiver::None,
                        span: member.span,
                    });
                }
            }
        }

        let extends = decl
            .extends
            .as_ref()
            .and_then(|ident| self.resolver.lookup(ident.name));

        if decl.extends.is_some() && extends.is_none() {
            self.error("unknown base class in gens declaration");
        }

        let implements = decl
            .implements
            .iter()
            .filter_map(|ident| {
                let def_id = self.resolver.lookup(ident.name);
                if def_id.is_none() {
                    self.error("unknown interface in gens declaration");
                }
                def_id
            })
            .collect();

        let struct_item = HirStruct { name: decl.name.name, type_params, fields, methods, extends, implements };

        Some(HirItem { id: self.next_hir_id(), def_id, kind: HirItemKind::Struct(struct_item), span: stmt.span })
    }

    /// Lower an `ordo` declaration into enum-shaped HIR.
    ///
    /// Plain `ordo` variants carry names and `DefId`s only. Payload-bearing
    /// variant fields are reserved for `discretio`.
    pub fn lower_ordo(&mut self, stmt: &Stmt, decl: &EnumDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let variants = decl
            .members
            .iter()
            .map(|member| HirVariant {
                def_id: self.def_id_for(member.name.name),
                name: member.name.name,
                fields: Vec::new(),
                span: member.span,
            })
            .collect();

        let enum_item = HirEnum { name: decl.name.name, type_params: Vec::new(), variants };

        Some(HirItem { id: self.next_hir_id(), def_id, kind: HirItemKind::Enum(enum_item), span: stmt.span })
    }

    /// Lower a `discretio` declaration into enum-shaped HIR with payload fields.
    ///
    /// This shares the HIR enum representation with `ordo` so pattern matching
    /// and codegen can operate on one variant model while still preserving
    /// payload type annotations for semantic analysis.
    pub fn lower_discretio(&mut self, stmt: &Stmt, decl: &UnionDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam { def_id: self.next_def_id(), name: param.name.name, span: param.span })
            .collect();

        let variants = decl
            .variants
            .iter()
            .map(|variant| {
                let fields = variant
                    .fields
                    .iter()
                    .map(|field| HirVariantField {
                        name: field.name.name,
                        ty: self.lower_type(&field.ty),
                        span: field.span,
                    })
                    .collect();

                HirVariant {
                    def_id: self.def_id_for(variant.name.name),
                    name: variant.name.name,
                    fields,
                    span: variant.span,
                }
            })
            .collect();

        let enum_item = HirEnum { name: decl.name.name, type_params, variants };

        Some(HirItem { id: self.next_hir_id(), def_id, kind: HirItemKind::Enum(enum_item), span: stmt.span })
    }

    /// Lower a `pactum` declaration into interface HIR.
    ///
    /// Method signatures are lowered without bodies. Compatibility between an
    /// implementing `gens` and these signatures is intentionally left to later
    /// passes that can compare fully lowered type information.
    pub fn lower_pactum(&mut self, stmt: &Stmt, decl: &InterfaceDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam { def_id: self.next_def_id(), name: param.name.name, span: param.span })
            .collect();

        let methods = decl
            .methods
            .iter()
            .map(|method| HirInterfaceMethod {
                name: method.name.name,
                params: method
                    .params
                    .iter()
                    .map(|param| HirParam {
                        def_id: self.next_def_id(),
                        name: param.name.name,
                        ty: self.lower_type(&param.ty),
                        mode: lower_param_mode(param.mode),
                        optional: param.sponte || param.default.is_some(),
                        sponte: param.sponte,
                        fixus: param.fixus,
                        span: param.span,
                    })
                    .collect(),
                ret_ty: method.ret.as_ref().map(|ty| self.lower_type(ty)),
                err_ty: method.err.as_ref().map(|ty| self.lower_type(ty)),
                span: method.span,
            })
            .collect();

        let interface = HirInterface { name: decl.name.name, type_params, methods };

        Some(HirItem { id: self.next_hir_id(), def_id, kind: HirItemKind::Interface(interface), span: stmt.span })
    }

    /// Lower a `typus` declaration into a type-alias item.
    pub fn lower_typus(&mut self, stmt: &Stmt, decl: &TypeAliasDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);
        let ty = self.lower_type(&decl.ty);

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::TypeAlias(HirTypeAlias { name: decl.name.name, ty }),
            span: stmt.span,
        })
    }

    /// Lower an `importa` declaration into import HIR.
    ///
    /// Earlier collection/resolution records the names an import makes visible.
    /// Lowering preserves that import contract and attaches HIR item ids without
    /// deciding package or provider loading policy here.
    pub fn lower_importa(&mut self, stmt: &Stmt, decl: &ImportDecl) -> Option<HirItem> {
        let def_id = self.next_def_id();

        let items = match &decl.kind {
            crate::syntax::ImportKind::Named { name, alias } => {
                let bound_name = alias.as_ref().map(|ident| ident.name).unwrap_or(name.name);
                let item_def_id = self
                    .lookup_name(bound_name)
                    .unwrap_or_else(|| self.next_def_id());
                vec![HirImportItem {
                    def_id: item_def_id,
                    name: name.name,
                    alias: alias.as_ref().map(|ident| ident.name),
                }]
            }
            crate::syntax::ImportKind::Wildcard { alias } => {
                let item_def_id = self
                    .lookup_name(alias.name)
                    .unwrap_or_else(|| self.next_def_id());
                vec![HirImportItem { def_id: item_def_id, name: alias.name, alias: Some(alias.name) }]
            }
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Import(HirImport { path: decl.path, visibility: decl.visibility, items }),
            span: stmt.span,
        })
    }
}

fn modifier_bindings(modifiers: &[crate::syntax::FuncModifier]) -> Vec<&crate::syntax::Ident> {
    let mut out = Vec::new();
    for modifier in modifiers {
        match modifier {
            crate::syntax::FuncModifier::Curata { required, alias } => {
                out.push(alias.as_ref().unwrap_or(required));
            }
            crate::syntax::FuncModifier::Errata(ident) | crate::syntax::FuncModifier::Optiones(ident) => {
                out.push(ident);
            }
            crate::syntax::FuncModifier::Exitus(crate::syntax::ExitusValue::Name(ident)) => out.push(ident),
            _ => {}
        }
    }
    out
}

fn command_argument_binding(modifiers: &[crate::syntax::FuncModifier]) -> Option<&crate::syntax::Ident> {
    modifiers.iter().find_map(|modifier| match modifier {
        crate::syntax::FuncModifier::Argumenta(ident) => Some(ident),
        _ => None,
    })
}
