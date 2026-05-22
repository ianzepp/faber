use super::*;

impl<'a> TypeChecker<'a> {
    pub(super) fn function_signature(&mut self, func: &HirFunction) -> FuncSig {
        let params = func
            .params
            .iter()
            .map(|param| ParamType { ty: param.ty, mode: param_mode_from_hir(param.mode), optional: param.optional })
            .collect();
        let ret = func.ret_ty.unwrap_or_else(|| self.vacuum_type());
        FuncSig { params, ret, err: func.err_ty, is_async: func.is_async, is_generator: func.is_generator }
    }
    pub(super) fn collect_struct(&mut self, def_id: DefId, struct_item: &HirStruct) {
        let mut fields = FxHashMap::default();
        let mut methods = FxHashMap::default();

        for field in &struct_item.fields {
            fields.insert(
                field.name,
                StructFieldInfo { ty: field.ty, required: !field.sponte && field.init.is_none(), span: field.span },
            );
        }

        for method in &struct_item.methods {
            let sig = self.function_signature(&method.func);
            methods.insert(method.func.name, sig);
        }

        self.structs.insert(def_id, StructInfo { fields, methods });
    }
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
                        self.variant_fields.insert(variant.def_id, fields);
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
