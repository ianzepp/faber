use super::*;
use crate::hir::visit::HirVisitor;

pub(super) struct ItemLoweringPass<'lowerer, 'unit, 'maps> {
    lowerer: &'lowerer mut MirLowerer<'unit>,
    context_maps: &'maps LoweringContextMaps<'unit>,
    struct_fields: &'maps FxHashMap<DefId, Vec<&'unit HirField>>,
}

impl<'lowerer, 'unit, 'maps> ItemLoweringPass<'lowerer, 'unit, 'maps> {
    pub(super) fn new(
        lowerer: &'lowerer mut MirLowerer<'unit>,
        context_maps: &'maps LoweringContextMaps<'unit>,
        struct_fields: &'maps FxHashMap<DefId, Vec<&'unit HirField>>,
    ) -> Self {
        Self { lowerer, context_maps, struct_fields }
    }

    pub(super) fn lower_items(&mut self) {
        let items = &self.lowerer.unit.hir.items;
        for item in items {
            self.visit_item(item);
        }
    }
}

impl HirVisitor for ItemLoweringPass<'_, '_, '_> {
    fn visit_item(&mut self, item: &HirItem) {
        match &item.kind {
            HirItemKind::Function(function) => {
                self.lowerer
                    .lower_function(item, function, self.context_maps, self.struct_fields);
            }
            HirItemKind::Struct(_)
            | HirItemKind::Enum(_)
            | HirItemKind::Interface(_)
            | HirItemKind::TypeAlias(_)
            | HirItemKind::Import(_) => {}
            other => self.lowerer.errors.push(MirError::unsupported(
                item.span,
                format!("top-level {}", hir_item_kind_name(other)),
            )),
        }
    }
}
