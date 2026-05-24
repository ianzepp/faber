//! Declaration collection for the HIR typechecker.
//!
//! Typecheck needs a complete table of callable and aggregate contracts before
//! it checks expression bodies. This file extracts those contracts from resolved
//! HIR without validating bodies, so declaration order does not affect calls,
//! method lookup, enum variant constructors, or interface checks.
//!
//! INVARIANTS
//! ==========
//! - Collected `TypeId`s are already lowered into the shared semantic type
//!   table by earlier passes.
//! - Function and method signatures preserve parameter modes, optional slots,
//!   async/generator flags, and alternate-exit types for later call checking.
//! - Struct fields record requiredness from `sponte` and initializers; struct
//!   literal validation depends on that distinction.

use super::*;

impl<'a> TypeChecker<'a> {
    /// Build the callable contract for a lowered function item or method.
    ///
    /// Missing return annotations are treated as `vacuum` at the signature
    /// boundary. Body checking may still synthesize return flow internally, but
    /// call sites need a concrete contract before any body is visited.
    pub(super) fn function_signature(&mut self, func: &HirFunction) -> FuncSig {
        let params = func
            .params
            .iter()
            .map(|param| ParamType { ty: param.ty, mode: param_mode_from_hir(param.mode), optional: param.optional })
            .collect();
        let mut ret = func.ret_ty.unwrap_or_else(|| self.vacuum_type());
        if func.is_generator {
            ret = self.types.array(ret);
        }
        FuncSig { params, ret, err: func.err_ty, is_async: func.is_async, is_generator: func.is_generator }
    }

    /// Collect field and method contracts for a struct definition.
    ///
    /// Methods are stored as signatures rather than checked here so recursive
    /// and forward calls see the same declaration table as ordinary functions.
    pub(super) fn collect_struct(&mut self, def_id: DefId, struct_item: &HirStruct) {
        let mut fields = FxHashMap::default();
        let mut methods = FxHashMap::default();

        for field in &struct_item.fields {
            fields.insert(
                field.name,
                StructFieldInfo {
                    ty: field.ty,
                    optional: field.sponte,
                    required: !field.sponte && field.init.is_none(),
                    span: field.span,
                },
            );
        }

        for method in &struct_item.methods {
            let sig = self.function_signature(&method.func);
            methods.insert(method.func.name, sig);
        }

        self.structs.insert(def_id, StructInfo { fields, methods });
    }

    /// Populate all top-level typecheck lookup tables before checking bodies.
    ///
    /// This pass is intentionally shallow. It records declarations that later
    /// expression checks need, but leaves initializer, method body, and function
    /// body validation to the main traversal so errors are reported with the
    /// correct local scope and return/error context.
    pub(super) fn collect_items(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    let sig = self.function_signature(func);
                    self.functions.insert(item.def_id, sig);
                }
                HirItemKind::Struct(struct_item) => self.collect_struct(item.def_id, struct_item),
                HirItemKind::Interface(interface_item) => {
                    let mut methods = FxHashMap::default();
                    for method in &interface_item.methods {
                        let params = method
                            .params
                            .iter()
                            .map(|param| ParamType {
                                ty: param.ty,
                                mode: param_mode_from_hir(param.mode),
                                optional: param.optional,
                            })
                            .collect();
                        let ret = method.ret_ty.unwrap_or_else(|| self.vacuum_type());
                        methods.insert(
                            method.name,
                            FuncSig { params, ret, err: method.err_ty, is_async: false, is_generator: false },
                        );
                    }
                    self.interfaces.insert(item.def_id, methods);
                }
                HirItemKind::Enum(enum_item) => {
                    for variant in &enum_item.variants {
                        let fields = variant.fields.iter().map(|f| f.ty).collect();
                        let field_names = variant.fields.iter().map(|f| f.name).collect();
                        self.variant_fields.insert(variant.def_id, fields);
                        self.variant_field_names.insert(variant.def_id, field_names);
                        self.variant_parent.insert(variant.def_id, item.def_id);
                    }
                }
                HirItemKind::Const(const_item) => {
                    if let Some(ty) = const_item.ty {
                        self.consts.insert(item.def_id, ty);
                    }
                }
                _ => {}
            }
        }
    }
}
