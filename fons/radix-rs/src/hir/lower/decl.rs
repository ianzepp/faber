//! Declaration lowering
//!
//! Lowers AST declarations to HIR items.

use super::Lowerer;
use crate::hir::{
    HirConst, HirEnum, HirExpr, HirExprKind, HirField, HirFunction, HirImport, HirImportItem,
    HirInterface, HirInterfaceMethod, HirItem, HirItemKind, HirMethod, HirParam, HirParamMode,
    HirReceiver, HirStruct, HirTypeAlias, HirTypeParam, HirVariant, HirVariantField,
};
use crate::lexer::Span;
use crate::syntax::{
    ClassDecl, EnumDecl, FuncDecl, ImportDecl, InterfaceDecl, Stmt, TypeAliasDecl, UnionDecl,
    VarDecl,
};

fn error_expr(lowerer: &mut Lowerer, span: Span) -> HirExpr {
    HirExpr {
        id: lowerer.next_hir_id(),
        kind: HirExprKind::Error,
        ty: None,
        span,
    }
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
    /// Lower varia/ficum declaration
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
                self.error("top-level variable declaration requires initializer");
                error_expr(self, name_span)
            }
        };

        if decl.mutability == crate::syntax::Mutability::Mutable {
            self.error("mutable top-level variables are not lowered yet");
        }

        let item = HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Const(HirConst { name, ty, value }),
            span: stmt.span,
        };

        Some(item)
    }

    /// Lower functio declaration
    pub fn lower_functio(&mut self, stmt: &Stmt, decl: &FuncDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam {
                def_id: self.next_def_id(),
                name: param.name.name,
                span: param.span,
            })
            .collect();

        let params = decl
            .params
            .iter()
            .map(|param| HirParam {
                def_id: self.next_def_id(),
                name: param.name.name,
                ty: self.lower_type(&param.ty),
                mode: lower_param_mode(param.mode),
                span: param.span,
            })
            .collect();

        let ret_ty = decl.ret.as_ref().map(|ty| self.lower_type(ty));
        let body = decl.body.as_ref().map(|body| self.lower_block(body));

        let func = HirFunction {
            name: decl.name.name,
            type_params,
            params,
            ret_ty,
            body,
            is_async: false,
            is_generator: false,
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Function(func),
            span: stmt.span,
        })
    }

    /// Lower gens (class) declaration
    pub fn lower_gens(&mut self, stmt: &Stmt, decl: &ClassDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam {
                def_id: self.next_def_id(),
                name: param.name.name,
                span: param.span,
            })
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
                        init,
                        span: member.span,
                    });
                }
                crate::syntax::ClassMemberKind::Method(method) => {
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
                        params: method
                            .params
                            .iter()
                            .map(|param| HirParam {
                                def_id: self.next_def_id(),
                                name: param.name.name,
                                ty: self.lower_type(&param.ty),
                                mode: lower_param_mode(param.mode),
                                span: param.span,
                            })
                            .collect(),
                        ret_ty: method.ret.as_ref().map(|ty| self.lower_type(ty)),
                        body: method.body.as_ref().map(|body| self.lower_block(body)),
                        is_async: false,
                        is_generator: false,
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

        let struct_item = HirStruct {
            name: decl.name.name,
            type_params,
            fields,
            methods,
            extends,
            implements,
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Struct(struct_item),
            span: stmt.span,
        })
    }

    /// Lower ordo (enum) declaration
    pub fn lower_ordo(&mut self, stmt: &Stmt, decl: &EnumDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let variants = decl
            .members
            .iter()
            .map(|member| {
                if member.value.is_some() {
                    self.error("enum member values are not lowered yet");
                }

                HirVariant {
                    def_id: self.next_def_id(),
                    name: member.name.name,
                    fields: Vec::new(),
                    span: member.span,
                }
            })
            .collect();

        let enum_item = HirEnum {
            name: decl.name.name,
            type_params: Vec::new(),
            variants,
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Enum(enum_item),
            span: stmt.span,
        })
    }

    /// Lower discretio (union) declaration
    pub fn lower_discretio(&mut self, stmt: &Stmt, decl: &UnionDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam {
                def_id: self.next_def_id(),
                name: param.name.name,
                span: param.span,
            })
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
                    def_id: self.next_def_id(),
                    name: variant.name.name,
                    fields,
                    span: variant.span,
                }
            })
            .collect();

        let enum_item = HirEnum {
            name: decl.name.name,
            type_params,
            variants,
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Enum(enum_item),
            span: stmt.span,
        })
    }

    /// Lower pactum (interface) declaration
    pub fn lower_pactum(&mut self, stmt: &Stmt, decl: &InterfaceDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);

        let type_params = decl
            .type_params
            .iter()
            .map(|param| HirTypeParam {
                def_id: self.next_def_id(),
                name: param.name.name,
                span: param.span,
            })
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
                        span: param.span,
                    })
                    .collect(),
                ret_ty: method.ret.as_ref().map(|ty| self.lower_type(ty)),
                span: method.span,
            })
            .collect();

        let interface = HirInterface {
            name: decl.name.name,
            type_params,
            methods,
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Interface(interface),
            span: stmt.span,
        })
    }

    /// Lower typus (type alias) declaration
    pub fn lower_typus(&mut self, stmt: &Stmt, decl: &TypeAliasDecl) -> Option<HirItem> {
        let def_id = self.def_id_for(decl.name.name);
        let ty = self.lower_type(&decl.ty);

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::TypeAlias(HirTypeAlias {
                name: decl.name.name,
                ty,
            }),
            span: stmt.span,
        })
    }

    /// Lower importa (import) declaration
    pub fn lower_importa(&mut self, stmt: &Stmt, decl: &ImportDecl) -> Option<HirItem> {
        let def_id = self.next_def_id();

        let items = match &decl.kind {
            crate::syntax::ImportKind::Named { name, alias } => vec![HirImportItem {
                def_id: self.next_def_id(),
                name: name.name,
                alias: alias.as_ref().map(|ident| ident.name),
            }],
            crate::syntax::ImportKind::Wildcard { alias } => {
                self.error("wildcard imports are not lowered yet");
                vec![HirImportItem {
                    def_id: self.next_def_id(),
                    name: alias.name,
                    alias: None,
                }]
            }
        };

        Some(HirItem {
            id: self.next_hir_id(),
            def_id,
            kind: HirItemKind::Import(HirImport {
                path: decl.path,
                items,
            }),
            span: stmt.span,
        })
    }
}
