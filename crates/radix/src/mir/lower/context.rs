//! Whole-unit lookup tables for HIR-to-MIR lowering.
//!
//! MIR builders lower one function body at a time, but body lowering frequently
//! needs facts that are declared elsewhere in the analyzed unit: callable
//! signatures for validation, alternate-exit types, enum variant ownership,
//! variant payload field names, struct fields for aggregate construction, and
//! provider-import identities. This module gathers those facts once from typed
//! HIR and packages them into the read-mostly context used by function lowering.
//!
//! DESIGN
//! ======
//! The maps are not a second semantic analysis pass. They preserve information
//! already established by collect, resolve, lower, and typecheck, then reshape it
//! into MIR-oriented lookup tables. Missing type information still fails later at
//! the lowering site; these maps should not synthesize types or invent fallback
//! signatures.
//!
//! VALIDATION HANDOFF
//! ==================
//! [`LoweringContextMaps`] also owns the [`MirValidationContext`] built from HIR
//! declarations. Validation receives this context after lowering so it can check
//! function calls, struct fields, and variant payloads against semantic source
//! truth rather than trusting the MIR producer.

use super::*;
use crate::hir::visit::HirVisitor;
use crate::mir::MirFunctionSignature;

/// Whole-unit maps shared by item and function lowering.
///
/// The lowering-only maps feed [`FunctionBuilderContext`]. The validation map is
/// carried beside them because it is built from the same HIR walk but consumed
/// only after a complete [`MirProgram`] exists.
pub(super) struct LoweringContextMaps<'a> {
    function_errors: FxHashMap<DefId, MirType>,
    variant_parents: FxHashMap<DefId, DefId>,
    variant_fields: FxHashMap<DefId, Vec<Symbol>>,
    provider_imports: FxHashMap<DefId, ProviderImport>,
    method_targets: FxHashMap<(DefId, Symbol), MethodTarget>,
    pub(super) validation: MirValidationContext<'a>,
}

impl<'a> LoweringContextMaps<'a> {
    /// Collect MIR lowering and validation maps from an analyzed unit.
    ///
    /// This walk intentionally follows HIR item structure instead of MIR output:
    /// declarations that never lower into executable MIR, such as structs,
    /// enums, and imports, still define facts required by function-body lowering
    /// and validation.
    pub(super) fn collect(unit: &'a AnalyzedUnit) -> Self {
        let mut maps = Self {
            function_errors: FxHashMap::default(),
            variant_parents: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            provider_imports: FxHashMap::default(),
            method_targets: FxHashMap::default(),
            validation: MirValidationContext::new(&unit.types),
        };
        maps.visit_program(&unit.hir);
        maps
    }

    /// Build the immutable context used by one [`FunctionBuilder`].
    ///
    /// Function builders own their CFG and local state, but they should all see
    /// the same whole-unit declaration facts. Cloning the small maps here keeps
    /// body lowering isolated from cross-function mutation.
    pub(super) fn builder_context(
        &self,
        interner: &'a Interner,
        structs: FxHashMap<DefId, Vec<&'a HirField>>,
    ) -> FunctionBuilderContext<'a> {
        FunctionBuilderContext {
            interner: Some(interner),
            function_errors: self.function_errors.clone(),
            structs,
            variant_parents: self.variant_parents.clone(),
            variant_fields: self.variant_fields.clone(),
            provider_imports: self.provider_imports.clone(),
            method_targets: self.method_targets.clone(),
        }
    }
}

/// Collect struct field nodes for aggregate lowering.
///
/// Unlike validation's type-only field table, aggregate lowering needs borrowed
/// HIR field nodes so default initializers and field ordering remain available
/// when a constructor is lowered.
pub(super) fn struct_field_map(unit: &AnalyzedUnit) -> FxHashMap<DefId, Vec<&HirField>> {
    // WHY: this map intentionally keeps borrowed field nodes for default
    // initializer lowering. `HirVisitor::visit_item` cannot store those
    // borrows with the unit lifetime, so this remains an explicit unit scan.
    let mut structs = FxHashMap::default();
    for item in &unit.hir.items {
        let HirItemKind::Struct(strukt) = &item.kind else {
            continue;
        };
        structs.insert(item.def_id, strukt.fields.iter().collect());
    }
    structs
}

impl<'a> HirVisitor for LoweringContextMaps<'a> {
    fn visit_item(&mut self, item: &HirItem) {
        match &item.kind {
            HirItemKind::Function(function) => {
                if let Some(err_ty) = function.err_ty {
                    self.function_errors
                        .insert(item.def_id, MirType::semantic(err_ty));
                }
                if let Some(return_ty) = function.ret_ty {
                    self.validation.functions.insert(
                        item.def_id,
                        MirFunctionSignature {
                            params: function
                                .params
                                .iter()
                                .map(|param| MirType::semantic(param.ty))
                                .collect(),
                            return_ty: MirType::semantic(return_ty),
                            error_ty: function.err_ty.map(MirType::semantic),
                        },
                    );
                }
            }
            HirItemKind::Struct(strukt) => {
                let receiver_ty = self.validation.types.find_struct(item.def_id);
                let mut fields = FxHashMap::default();
                let mut optional_fields = FxHashSet::default();
                for field in &strukt.fields {
                    if !field.is_static {
                        fields.insert(field.name, MirType::semantic(field.ty));
                        if field.sponte || field.init.is_some() {
                            optional_fields.insert(field.name);
                        }
                    }
                }
                self.validation.struct_fields.insert(item.def_id, fields);
                self.validation
                    .optional_struct_fields
                    .insert(item.def_id, optional_fields);

                if let Some(receiver_ty) = receiver_ty {
                    for method in &strukt.methods {
                        self.method_targets
                            .insert((item.def_id, method.func.name), MethodTarget { def_id: method.def_id });
                        if let Some(err_ty) = method.func.err_ty {
                            self.function_errors
                                .insert(method.def_id, MirType::semantic(err_ty));
                        }
                        if let Some(return_ty) = method.func.ret_ty {
                            let mut params = Vec::with_capacity(method.func.params.len() + 1);
                            params.push(MirType::semantic(receiver_ty));
                            params.extend(
                                method
                                    .func
                                    .params
                                    .iter()
                                    .map(|param| MirType::semantic(param.ty)),
                            );
                            self.validation.functions.insert(
                                method.def_id,
                                MirFunctionSignature {
                                    params,
                                    return_ty: MirType::semantic(return_ty),
                                    error_ty: method.func.err_ty.map(MirType::semantic),
                                },
                            );
                        }
                    }
                }
            }
            HirItemKind::Enum(enum_item) => {
                for variant in &enum_item.variants {
                    self.variant_parents.insert(variant.def_id, item.def_id);
                    self.variant_fields
                        .insert(variant.def_id, variant.fields.iter().map(|field| field.name).collect());
                    let mut fields = FxHashMap::default();
                    for field in &variant.fields {
                        fields.insert(field.name, MirType::semantic(field.ty));
                    }
                    self.validation
                        .variant_fields
                        .insert(variant.def_id, fields);
                }
            }
            HirItemKind::Import(import) => {
                for import_item in &import.items {
                    self.provider_imports.insert(
                        import_item.def_id,
                        ProviderImport { module: vec![import.path], item: import_item.name },
                    );
                }
            }
            HirItemKind::Interface(_) | HirItemKind::TypeAlias(_) | HirItemKind::Const(_) => {}
        }
    }
}
