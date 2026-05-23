use super::*;
use crate::hir::visit::HirVisitor;
use crate::mir::MirFunctionSignature;

pub(super) struct LoweringContextMaps<'a> {
    function_errors: FxHashMap<DefId, MirType>,
    variant_parents: FxHashMap<DefId, DefId>,
    variant_fields: FxHashMap<DefId, Vec<Symbol>>,
    provider_imports: FxHashMap<DefId, ProviderImport>,
    pub(super) validation: MirValidationContext<'a>,
}

impl<'a> LoweringContextMaps<'a> {
    pub(super) fn collect(unit: &'a AnalyzedUnit) -> Self {
        let mut maps = Self {
            function_errors: FxHashMap::default(),
            variant_parents: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            provider_imports: FxHashMap::default(),
            validation: MirValidationContext::new(&unit.types),
        };
        maps.visit_program(&unit.hir);
        maps
    }

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
        }
    }
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
                let mut fields = FxHashMap::default();
                for field in &strukt.fields {
                    if !field.is_static {
                        fields.insert(field.name, MirType::semantic(field.ty));
                    }
                }
                self.validation.struct_fields.insert(item.def_id, fields);
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
