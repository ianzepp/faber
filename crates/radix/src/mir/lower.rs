use crate::driver::AnalyzedUnit;
use crate::hir::{HirBlock, HirFunction, HirItem, HirItemKind};
use crate::lexer::Span;
use crate::mir::{
    dump_program, MirBlock, MirBlockId, MirFunction, MirFunctionId, MirLocalId, MirParam, MirProgram, MirTerminator,
    MirTerminatorKind, MirType,
};
use crate::semantic::Primitive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirError {
    pub message: String,
    pub span: Span,
}

impl MirError {
    fn unsupported(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("unsupported MIR lowering in phase 2: {}", what.into()), span }
    }

    fn missing_type(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("missing type information for MIR lowering: {}", what.into()), span }
    }
}

pub fn lower_analyzed_unit(unit: &AnalyzedUnit) -> Result<MirProgram, Vec<MirError>> {
    let mut lowerer = MirLowerer { unit, errors: Vec::new(), functions: Vec::new() };
    lowerer.lower();

    if lowerer.errors.is_empty() {
        Ok(MirProgram { functions: lowerer.functions })
    } else {
        Err(lowerer.errors)
    }
}

pub fn dump_analyzed_unit(unit: &AnalyzedUnit) -> Result<String, Vec<MirError>> {
    lower_analyzed_unit(unit).map(|program| dump_program(&program))
}

struct MirLowerer<'a> {
    unit: &'a AnalyzedUnit,
    errors: Vec<MirError>,
    functions: Vec<MirFunction>,
}

impl MirLowerer<'_> {
    fn lower(&mut self) {
        if self.unit.cli_program.is_some() {
            self.errors.push(MirError::unsupported(
                self.unit
                    .hir
                    .entry
                    .as_ref()
                    .map_or_else(Span::default, |entry| entry.span),
                "CLI program-specific MIR lowering",
            ));
            return;
        }

        for item in &self.unit.hir.items {
            self.lower_item(item);
        }

        if let Some(entry) = &self.unit.hir.entry {
            self.lower_entry(entry);
        }
    }

    fn lower_item(&mut self, item: &HirItem) {
        match &item.kind {
            HirItemKind::Function(function) => self.lower_function(item, function),
            other => self.errors.push(MirError::unsupported(
                item.span,
                format!("top-level {}", hir_item_kind_name(other)),
            )),
        }
    }

    fn lower_function(&mut self, item: &HirItem, function: &HirFunction) {
        let Some(return_ty) = function.ret_ty else {
            self.errors
                .push(MirError::missing_type(item.span, "function return type"));
            return;
        };

        let mut params = Vec::new();
        for (index, param) in function.params.iter().enumerate() {
            params.push(MirParam {
                local: MirLocalId(index as u32),
                name: Some(param.name),
                ty: MirType::semantic(param.ty),
                span: param.span,
            });
        }

        let blocks = match &function.body {
            Some(body) if block_is_empty(body) => vec![empty_return_block(body.span)],
            Some(body) => {
                self.errors.push(MirError::unsupported(
                    body.span,
                    "non-empty function bodies before primitive expression lowering",
                ));
                Vec::new()
            }
            None => Vec::new(),
        };

        self.functions.push(MirFunction {
            id: MirFunctionId(self.functions.len() as u32),
            source: Some(item.def_id),
            name: Some(function.name),
            params,
            locals: Vec::new(),
            temps: Vec::new(),
            blocks,
            return_ty: MirType::semantic(return_ty),
            span: item.span,
        });
    }

    fn lower_entry(&mut self, entry: &HirBlock) {
        if !block_is_empty(entry) {
            self.errors.push(MirError::unsupported(
                entry.span,
                "non-empty entry blocks before primitive expression lowering",
            ));
            return;
        }

        let vacuum = self.unit.types.primitive(Primitive::Vacuum);
        self.functions.push(MirFunction {
            id: MirFunctionId(self.functions.len() as u32),
            source: None,
            name: None,
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: vec![empty_return_block(entry.span)],
            return_ty: MirType::semantic(vacuum),
            span: entry.span,
        });
    }
}

fn block_is_empty(block: &HirBlock) -> bool {
    block.stmts.is_empty() && block.expr.is_none()
}

fn empty_return_block(span: Span) -> MirBlock {
    MirBlock {
        id: MirBlockId(0),
        statements: Vec::new(),
        terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span },
        span,
    }
}

fn hir_item_kind_name(kind: &HirItemKind) -> &'static str {
    match kind {
        HirItemKind::Function(_) => "function",
        HirItemKind::Struct(_) => "struct",
        HirItemKind::Enum(_) => "enum",
        HirItemKind::Interface(_) => "interface",
        HirItemKind::TypeAlias(_) => "type alias",
        HirItemKind::Const(_) => "const",
        HirItemKind::Import(_) => "import",
    }
}

#[cfg(test)]
#[path = "lower_test.rs"]
mod tests;
