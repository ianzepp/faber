use crate::driver::AnalyzedUnit;
use crate::hir::{
    DefId, HirArrayElement, HirBinOp, HirBlock, HirCape, HirExpr, HirExprKind, HirField, HirFunction, HirItem,
    HirItemKind, HirLiteral, HirLocal, HirNonNullKind, HirObjectField, HirObjectKey, HirOptionalChainKind,
    HirScribeKind, HirStmt, HirStmtKind, HirUnOp,
};
use crate::lexer::{Interner, Span, Symbol};
use crate::mir::{
    dump_program, validate_program, MirAggregate, MirAggregateFields, MirAggregateItem, MirAggregateKind, MirBinOp,
    MirBlock, MirBlockId, MirCallee, MirCollectionOp, MirConstant, MirConversion, MirConversionFlavor,
    MirDiagnosticKind, MirFunction, MirFunctionId, MirFunctionSignature, MirIntrinsic, MirKeyValueOperand,
    MirLocal as MirLocalDecl, MirLocalId, MirNamedOperand, MirOperand, MirOptionChainLink, MirOptionOp,
    MirOptionUnwrapMode, MirParam, MirPlace, MirProgram, MirProjection, MirProvider, MirRuntimeCall, MirStmt,
    MirStmtKind, MirTemp, MirTempId, MirTerminator, MirTerminatorKind, MirType, MirUnOp, MirValidationContext,
    MirValue, MirValueId, MirValueKind,
};
use crate::semantic::{Primitive, Type, TypeId, TypeTable};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirError {
    pub message: String,
    pub span: Span,
}

impl MirError {
    fn unsupported(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("unsupported MIR lowering: {}", what.into()), span }
    }

    fn missing_type(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("missing type information for MIR lowering: {}", what.into()), span }
    }

    fn validation(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("invalid MIR: {}", what.into()), span }
    }
}

pub fn lower_analyzed_unit(unit: &AnalyzedUnit) -> Result<MirProgram, Vec<MirError>> {
    let mut lowerer = MirLowerer { unit, errors: Vec::new(), functions: Vec::new() };
    lowerer.lower();

    if !lowerer.errors.is_empty() {
        Err(lowerer.errors)
    } else {
        let validation = lowerer.validation_context();
        let program = MirProgram { functions: lowerer.functions.clone() };
        validate_program(&program, &validation).map_err(|errors| {
            errors
                .into_iter()
                .map(|error| MirError::validation(error.span, error.message))
                .collect::<Vec<_>>()
        })?;
        Ok(program)
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
            HirItemKind::Struct(_)
            | HirItemKind::Enum(_)
            | HirItemKind::Interface(_)
            | HirItemKind::TypeAlias(_)
            | HirItemKind::Import(_) => {}
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

        let error_ty = function.err_ty.map(MirType::semantic);
        let context = FunctionBuilderContext {
            interner: Some(&self.unit.interner),
            function_errors: self.function_error_map(),
            structs: self.struct_field_map(),
            variant_parents: self.variant_parent_map(),
            variant_fields: self.variant_field_map(),
            provider_imports: self.provider_import_map(),
        };
        let (params, locals, temps, blocks, errors) = {
            let mut builder = FunctionBuilder::for_function(&self.unit.types, error_ty, context);
            for param in &function.params {
                builder.add_param(param.def_id, param.name, param.ty, param.span);
            }

            let blocks = match &function.body {
                Some(body) => builder.lower_body(body),
                None => Vec::new(),
            };
            (builder.params, builder.locals, builder.temps, blocks, builder.errors)
        };
        self.errors.extend(errors);

        self.functions.push(MirFunction {
            id: MirFunctionId(self.functions.len() as u32),
            source: Some(item.def_id),
            name: Some(function.name),
            params,
            locals,
            temps,
            blocks,
            return_ty: MirType::semantic(return_ty),
            error_ty,
            span: item.span,
        });
    }

    fn function_error_map(&self) -> FxHashMap<DefId, MirType> {
        let mut errors = FxHashMap::default();
        for item in &self.unit.hir.items {
            let HirItemKind::Function(function) = &item.kind else {
                continue;
            };
            if let Some(err_ty) = function.err_ty {
                errors.insert(item.def_id, MirType::semantic(err_ty));
            }
        }
        errors
    }

    fn struct_field_map(&self) -> FxHashMap<DefId, Vec<&HirField>> {
        let mut structs = FxHashMap::default();
        for item in &self.unit.hir.items {
            let HirItemKind::Struct(strukt) = &item.kind else {
                continue;
            };
            structs.insert(item.def_id, strukt.fields.iter().collect());
        }
        structs
    }

    fn variant_parent_map(&self) -> FxHashMap<DefId, DefId> {
        let mut parents = FxHashMap::default();
        for item in &self.unit.hir.items {
            let HirItemKind::Enum(enum_item) = &item.kind else {
                continue;
            };
            for variant in &enum_item.variants {
                parents.insert(variant.def_id, item.def_id);
            }
        }
        parents
    }

    fn variant_field_map(&self) -> FxHashMap<DefId, Vec<crate::lexer::Symbol>> {
        let mut fields = FxHashMap::default();
        for item in &self.unit.hir.items {
            let HirItemKind::Enum(enum_item) = &item.kind else {
                continue;
            };
            for variant in &enum_item.variants {
                fields.insert(variant.def_id, variant.fields.iter().map(|field| field.name).collect());
            }
        }
        fields
    }

    fn provider_import_map(&self) -> FxHashMap<DefId, ProviderImport> {
        let mut providers = FxHashMap::default();
        for item in &self.unit.hir.items {
            let HirItemKind::Import(import) = &item.kind else {
                continue;
            };
            for import_item in &import.items {
                providers.insert(
                    import_item.def_id,
                    ProviderImport { module: vec![import.path], item: import_item.name },
                );
            }
        }
        providers
    }

    fn validation_context(&self) -> MirValidationContext<'_> {
        let mut context = MirValidationContext::new(&self.unit.types);
        for item in &self.unit.hir.items {
            match &item.kind {
                HirItemKind::Function(function) => {
                    if let Some(return_ty) = function.ret_ty {
                        context.functions.insert(
                            item.def_id,
                            MirFunctionSignature {
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
                    context.struct_fields.insert(item.def_id, fields);
                }
                HirItemKind::Enum(enum_item) => {
                    for variant in &enum_item.variants {
                        let mut fields = FxHashMap::default();
                        for field in &variant.fields {
                            fields.insert(field.name, MirType::semantic(field.ty));
                        }
                        context.variant_fields.insert(variant.def_id, fields);
                    }
                }
                _ => {}
            }
        }
        context
    }

    fn lower_entry(&mut self, entry: &HirBlock) {
        if !entry_is_empty(entry) {
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
            error_ty: None,
            span: entry.span,
        });
    }
}

#[derive(Debug, Clone, Copy)]
struct LocalBinding {
    local: MirLocalId,
    ty: MirType,
}

struct OpenBlock {
    id: MirBlockId,
    statements: Vec<MirStmt>,
    terminator: Option<MirTerminator>,
    span: Span,
}

#[derive(Debug, Clone, Copy)]
struct LoopContext {
    perge_target: MirBlockId,
    rumpe_target: MirBlockId,
}

#[derive(Debug, Clone)]
struct HandlerContext {
    error_place: MirPlace,
    error_block: MirBlockId,
}

#[derive(Debug, Clone)]
struct ProviderImport {
    module: Vec<Symbol>,
    item: Symbol,
}

struct FunctionBuilderContext<'a> {
    interner: Option<&'a Interner>,
    function_errors: FxHashMap<DefId, MirType>,
    structs: FxHashMap<DefId, Vec<&'a HirField>>,
    variant_parents: FxHashMap<DefId, DefId>,
    variant_fields: FxHashMap<DefId, Vec<crate::lexer::Symbol>>,
    provider_imports: FxHashMap<DefId, ProviderImport>,
}

impl FunctionBuilderContext<'_> {
    #[cfg(test)]
    fn empty() -> Self {
        Self {
            interner: None,
            function_errors: FxHashMap::default(),
            structs: FxHashMap::default(),
            variant_parents: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            provider_imports: FxHashMap::default(),
        }
    }
}

struct FunctionBuilder<'a> {
    types: &'a TypeTable,
    error_ty: Option<MirType>,
    context: FunctionBuilderContext<'a>,
    bindings: FxHashMap<DefId, LocalBinding>,
    params: Vec<MirParam>,
    locals: Vec<MirLocalDecl>,
    temps: Vec<MirTemp>,
    blocks: Vec<OpenBlock>,
    current: Option<MirBlockId>,
    loops: Vec<LoopContext>,
    handlers: Vec<HandlerContext>,
    errors: Vec<MirError>,
    next_value: u32,
}

impl<'a> FunctionBuilder<'a> {
    #[cfg(test)]
    fn new(types: &'a TypeTable) -> Self {
        Self::for_function(types, None, FunctionBuilderContext::empty())
    }

    fn for_function(types: &'a TypeTable, error_ty: Option<MirType>, context: FunctionBuilderContext<'a>) -> Self {
        Self {
            types,
            error_ty,
            context,
            bindings: FxHashMap::default(),
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: Vec::new(),
            current: None,
            loops: Vec::new(),
            handlers: Vec::new(),
            errors: Vec::new(),
            next_value: 0,
        }
    }

    fn add_param(&mut self, def_id: DefId, name: crate::lexer::Symbol, ty: TypeId, span: Span) {
        let local = self.next_local_id();
        let mir_ty = MirType::semantic(ty);
        self.params
            .push(MirParam { local, name: Some(name), ty: mir_ty, span });
        self.locals
            .push(MirLocalDecl { id: local, name: Some(name), ty: mir_ty, mutable: false, span });
        self.bindings
            .insert(def_id, LocalBinding { local, ty: mir_ty });
    }

    fn lower_body(&mut self, body: &HirBlock) -> Vec<MirBlock> {
        let entry = self.fresh_block(body.span);
        self.switch_to(entry);
        self.lower_block_statement(body);
        self.terminate_open_current(MirTerminatorKind::Return(None), body.span);
        self.finish_blocks()
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) {
        if !self.current_is_open() {
            self.errors
                .push(MirError::unsupported(stmt.span, "statement after a MIR terminator"));
            return;
        }

        match &stmt.kind {
            HirStmtKind::Local(local) => self.lower_local(local, stmt.span),
            HirStmtKind::Expr(expr) => {
                if matches!(expr.kind, HirExprKind::Assign(_, _)) {
                    self.lower_assignment_expr(expr);
                } else {
                    let _ = self.lower_expr(expr);
                }
            }
            HirStmtKind::Redde(Some(expr)) => {
                if let Some(value) = self.lower_return_expr(expr) {
                    self.terminate_current(MirTerminatorKind::Return(Some(value)), stmt.span);
                }
            }
            HirStmtKind::Redde(None) => {
                self.terminate_current(MirTerminatorKind::Return(None), stmt.span);
            }
            HirStmtKind::Ad(_) => self.errors.push(MirError::unsupported(
                stmt.span,
                "ad provider blocks before effectful MIR lowering",
            )),
            HirStmtKind::Rumpe => self.lower_rumpe(stmt.span),
            HirStmtKind::Perge => self.lower_perge(stmt.span),
            HirStmtKind::Tacet => self
                .errors
                .push(MirError::unsupported(stmt.span, "tacet before statement-level MIR lowering")),
        }
    }

    fn lower_local(&mut self, local: &HirLocal, span: Span) {
        let Some(ty) = local.ty else {
            self.errors
                .push(MirError::missing_type(span, "local declaration"));
            return;
        };

        let mir_ty = MirType::semantic(ty);
        let id = self.next_local_id();
        self.locals
            .push(MirLocalDecl { id, name: Some(local.name), ty: mir_ty, mutable: local.mutable, span });
        self.bindings
            .insert(local.def_id, LocalBinding { local: id, ty: mir_ty });

        let Some(init) = &local.init else {
            self.errors.push(MirError::unsupported(
                span,
                "uninitialized locals before definite-assignment MIR lowering",
            ));
            return;
        };

        self.lower_expr_to_destination(init, MirPlace::local(id), mir_ty);
    }

    fn lower_assignment_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        let HirExprKind::Assign(lhs, rhs) = &expr.kind else {
            return self.lower_expr(expr);
        };

        let fallback_ty = self.expr_ty(rhs)?;
        let (place, ty) = self.lower_assignment_place_with_fallback(lhs, fallback_ty)?;
        self.lower_expr_to_destination(rhs, place.clone(), ty)?;
        Some(MirOperand::Place(place))
    }

    fn lower_assignment_place_with_fallback(
        &mut self,
        expr: &HirExpr,
        fallback_ty: MirType,
    ) -> Option<(MirPlace, MirType)> {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                let Some(binding) = self.bindings.get(def_id).copied() else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "assignment target that does not resolve to a local",
                    ));
                    return None;
                };

                Some((MirPlace::local(binding.local), binding.ty))
            }
            HirExprKind::Field(object, name) => {
                let (mut place, _) = self.lower_assignment_place_with_fallback(object, fallback_ty)?;
                place.projections.push(MirProjection::Field(*name));
                let ty = expr.ty.map(MirType::semantic).unwrap_or(fallback_ty);
                Some((place, ty))
            }
            HirExprKind::Index(object, index) => {
                let (mut place, _) = self.lower_assignment_place_with_fallback(object, fallback_ty)?;
                let index = self.lower_expr(index)?;
                place.projections.push(MirProjection::Index(index));
                let ty = expr.ty.map(MirType::semantic).unwrap_or(fallback_ty);
                Some((place, ty))
            }
            _ => {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "assignment target that is not an addressable place",
                ));
                None
            }
        }
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        match &expr.kind {
            HirExprKind::Path(def_id) => self.lower_path(*def_id, expr.span),
            HirExprKind::Literal(literal) => self.lower_literal(literal, expr.span),
            HirExprKind::Unary(op, operand) => self.lower_unary(*op, operand, expr),
            HirExprKind::Binary(HirBinOp::Coalesce, lhs, rhs) => self.lower_coalesce(lhs, rhs, expr),
            HirExprKind::Binary(op, lhs, rhs) => self.lower_binary(*op, lhs, rhs, expr),
            HirExprKind::Call(callee, args) => self.lower_call(callee, args, expr),
            HirExprKind::MethodCall(receiver, method, args) => self.lower_method_call(receiver, *method, args, expr),
            HirExprKind::Field(object, name) => self.lower_field(object, *name, expr),
            HirExprKind::Index(object, index) => self.lower_index(object, index, expr),
            HirExprKind::OptionalChain(object, chain) => self.lower_optional_chain(object, chain, expr),
            HirExprKind::NonNull(object, chain) => self.lower_non_null(object, chain, expr),
            HirExprKind::Array(elements) => self.lower_array(elements, expr, MirAggregateKind::Array),
            HirExprKind::Struct(def_id, fields) => self.lower_struct_literal(*def_id, fields, expr),
            HirExprKind::Tuple(items) => self.lower_tuple(items, expr),
            HirExprKind::Verte { source, target, entries } => self.lower_verte(source, *target, entries.as_ref(), expr),
            HirExprKind::Scribe(kind, args) => self.lower_scribe(*kind, args, expr),
            HirExprKind::Scriptum(template, args) => self.lower_scriptum(*template, args, expr),
            HirExprKind::Conversio { source, target, params, fallback } => {
                self.lower_conversio(source, *target, params, fallback.as_deref(), expr)
            }
            HirExprKind::Block(block) => self.lower_block_expr(block, expr),
            HirExprKind::Si { cond, then_block, then_catch, else_block } => {
                self.lower_si_expr(cond, then_block, then_catch.as_deref(), else_block, expr)
            }
            HirExprKind::Dum(cond, block) => self.lower_dum_expr(cond, block, expr),
            HirExprKind::Handled { body, catch } => self.lower_handled_expr(body, catch, expr),
            HirExprKind::Assign(_, _) => self.lower_assignment_expr(expr),
            HirExprKind::Throw(value) => self.lower_iace(value, expr.span),
            HirExprKind::Panic(value) => self.lower_mori(value, expr.span),
            HirExprKind::AssignOp(_, _, _) => {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "compound assignment before assignment-op MIR lowering",
                ));
                None
            }
            other => {
                self.errors
                    .push(MirError::unsupported(expr.span, unsupported_expr_kind_name(other)));
                None
            }
        }
    }

    fn lower_transfer_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        let operand = self.lower_expr(expr)?;
        match operand {
            MirOperand::Constant(_) | MirOperand::Value(_) => {
                let ty = self.expr_ty(expr)?;
                Some(self.assign_temp(MirValueKind::Operand(operand), ty, expr.span))
            }
            MirOperand::Place(_) | MirOperand::Temp(_) => Some(operand),
        }
    }

    fn lower_return_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        self.lower_transfer_expr(expr)
    }

    fn lower_iace(&mut self, value: &HirExpr, span: Span) -> Option<MirOperand> {
        if let Some(handler) = self.handlers.last().cloned() {
            let error_ty = self.expr_ty(value)?;
            let value = self.lower_transfer_expr(value)?;
            let place = handler.error_place;
            self.assign(place, value, error_ty, span);
            self.terminate_current(MirTerminatorKind::Goto(handler.error_block), span);
            return None;
        }

        if self.error_ty.is_none() {
            self.errors
                .push(MirError::unsupported(span, "iace without a declared alternate-exit type"));
            return None;
        }

        let value = self.lower_transfer_expr(value)?;
        self.terminate_current(MirTerminatorKind::ReturnError(value), span);
        None
    }

    fn lower_mori(&mut self, value: &HirExpr, span: Span) -> Option<MirOperand> {
        let value = self.lower_expr(value)?;
        let numquam = MirType::semantic(self.types.primitive(Primitive::Numquam));
        self.append_stmt(MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: None,
                call: crate::mir::MirRuntimeCall {
                    intrinsic: crate::mir::MirIntrinsic::Panic,
                    args: vec![value],
                    return_ty: numquam,
                },
            },
            span,
        });
        self.terminate_current(MirTerminatorKind::Unreachable, span);
        None
    }

    fn lower_scribe(&mut self, kind: HirScribeKind, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expr(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Diagnostic(mir_diagnostic_kind(kind)), lowered_args, ty, expr.span))
    }

    fn lower_scriptum(&mut self, template: Symbol, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            lowered_args.push(self.lower_expr(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::FormatString { template }, lowered_args, ty, expr.span))
    }

    fn lower_conversio(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        params: &[Symbol],
        fallback: Option<&HirExpr>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let source = self.lower_expr(source)?;
        let fallback = match fallback {
            Some(fallback) => Some(self.lower_expr(fallback)?),
            None => None,
        };
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(
            MirIntrinsic::Convert(MirConversion {
                flavor: MirConversionFlavor::Runtime,
                target_ty: MirType::semantic(target),
                params: params.to_vec(),
                fallback,
            }),
            vec![source],
            ty,
            expr.span,
        ))
    }

    fn lower_handled_expr(&mut self, body: &HirBlock, catch: &HirCape, expr: &HirExpr) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors.push(MirError::unsupported(
                expr.span,
                "expression-valued cape handler before value-join MIR lowering",
            ));
            return None;
        }

        self.lower_handled_block(body, catch, expr.span);
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    fn lower_handled_block(&mut self, body: &HirBlock, catch: &HirCape, span: Span) {
        let Some(error_ty) = catch.binding_ty.map(MirType::semantic) else {
            self.errors
                .push(MirError::missing_type(catch.span, "cape handler binding"));
            return;
        };

        let handler_id = self.fresh_block(catch.body.span);
        let after_id = self.fresh_block(span);
        let handler_local = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: handler_local,
            name: Some(catch.binding_name),
            ty: error_ty,
            mutable: false,
            span: catch.span,
        });
        let handler_binding = LocalBinding { local: handler_local, ty: error_ty };
        self.bindings.insert(catch.binding_def_id, handler_binding);

        self.handlers
            .push(HandlerContext { error_place: MirPlace::local(handler_local), error_block: handler_id });
        self.lower_block_statement(body);
        self.handlers.pop();
        let body_reaches = self.terminate_open_current(MirTerminatorKind::Goto(after_id), span);

        self.switch_to(handler_id);
        self.lower_block_statement(&catch.body);
        let handler_reaches = self.terminate_open_current(MirTerminatorKind::Goto(after_id), catch.span);

        if body_reaches || handler_reaches {
            self.switch_to(after_id);
        } else {
            self.seal_unreachable(after_id, span);
        }
    }

    fn lower_expr_to_destination(&mut self, expr: &HirExpr, destination: MirPlace, ty: MirType) -> Option<()> {
        match &expr.kind {
            HirExprKind::Block(block) => self.lower_block_to_destination(block, destination, ty, expr.span),
            HirExprKind::Si { cond, then_block, then_catch, else_block } => {
                if then_catch.is_some() {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "expression-valued si with cape before handler value MIR lowering",
                    ));
                    return None;
                }
                self.lower_si_to_destination(cond, then_block, else_block.as_ref(), destination, ty, expr.span)
            }
            _ => {
                let value = self.lower_expr(expr)?;
                self.assign(destination, value, ty, expr.span);
                Some(())
            }
        }
    }

    fn lower_block_expr(&mut self, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if self.is_vacuum(ty) {
            self.lower_block_statement(block);
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        let temp = self.push_temp(ty, expr.span);
        self.lower_block_to_destination(block, MirPlace::temp(temp), ty, expr.span)?;
        Some(MirOperand::Temp(temp))
    }

    fn lower_si_expr(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: &Option<HirBlock>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if self.is_vacuum(ty) {
            self.lower_si_statement(cond, then_block, then_catch, else_block.as_ref(), expr.span);
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        if then_catch.is_some() {
            self.errors.push(MirError::unsupported(
                expr.span,
                "expression-valued si with cape before handler value MIR lowering",
            ));
            return None;
        }

        let temp = self.push_temp(ty, expr.span);
        self.lower_si_to_destination(cond, then_block, else_block.as_ref(), MirPlace::temp(temp), ty, expr.span)?;
        Some(MirOperand::Temp(temp))
    }

    fn lower_dum_expr(&mut self, cond: &HirExpr, block: &HirBlock, expr: &HirExpr) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if !self.is_vacuum(ty) {
            self.errors
                .push(MirError::unsupported(expr.span, "dum expression with non-vacuum result"));
            return None;
        }

        self.lower_dum(cond, block, expr.span);
        Some(MirOperand::Constant(MirConstant::Unit))
    }

    fn lower_path(&mut self, def_id: DefId, span: Span) -> Option<MirOperand> {
        let Some(binding) = self.bindings.get(&def_id).copied() else {
            self.errors
                .push(MirError::unsupported(span, "path that does not resolve to a local value"));
            return None;
        };

        Some(MirOperand::Place(MirPlace::local(binding.local)))
    }

    fn lower_literal(&mut self, literal: &HirLiteral, span: Span) -> Option<MirOperand> {
        let constant = match literal {
            HirLiteral::Int(value) => MirConstant::Int(*value),
            HirLiteral::Float(value) => MirConstant::Float(*value),
            HirLiteral::String(value) => MirConstant::String(*value),
            HirLiteral::Bool(value) => MirConstant::Bool(*value),
            HirLiteral::Nil => MirConstant::Nil,
            HirLiteral::Regex(_, _) => {
                self.errors
                    .push(MirError::unsupported(span, "regex literals before runtime regex MIR lowering"));
                return None;
            }
        };

        Some(MirOperand::Constant(constant))
    }

    fn lower_unary(&mut self, op: HirUnOp, operand: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        let Some(op) = mir_un_op(op) else {
            self.errors
                .push(MirError::unsupported(expr.span, "unary operator without a MIR primitive"));
            return None;
        };
        let operand = self.lower_expr(operand)?;
        let ty = self.expr_ty(expr)?;

        Some(self.assign_temp(MirValueKind::Unary { op, operand }, ty, expr.span))
    }

    fn lower_binary(&mut self, op: HirBinOp, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        let Some(op) = mir_bin_op(op) else {
            self.errors
                .push(MirError::unsupported(expr.span, "binary operator without a MIR primitive"));
            return None;
        };
        let lhs = self.lower_expr(lhs)?;
        let rhs = self.lower_expr(rhs)?;
        let ty = self.expr_ty(expr)?;

        Some(self.assign_temp(MirValueKind::Binary { op, lhs, rhs }, ty, expr.span))
    }

    fn lower_coalesce(&mut self, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        let value = self.lower_expr(lhs)?;
        let fallback = self.lower_expr(rhs)?;
        let ty = self.expr_ty(expr)?;
        Some(self.assign_temp(MirValueKind::Option(MirOptionOp::Coalesce { value, fallback }), ty, expr.span))
    }

    fn lower_tuple(&mut self, items: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let mut fields = Vec::with_capacity(items.len());
        for item in items {
            fields.push(MirAggregateItem::Operand(self.lower_expr(item)?));
        }
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(MirAggregateKind::Tuple, MirAggregateFields::Ordered(fields), ty, expr.span))
    }

    fn lower_array(
        &mut self,
        elements: &[HirArrayElement],
        expr: &HirExpr,
        kind: MirAggregateKind,
    ) -> Option<MirOperand> {
        let fields = self.lower_array_items(elements)?;
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(kind, MirAggregateFields::Ordered(fields), ty, expr.span))
    }

    fn lower_array_items(&mut self, elements: &[HirArrayElement]) -> Option<Vec<MirAggregateItem>> {
        let mut fields = Vec::with_capacity(elements.len());
        for element in elements {
            match element {
                HirArrayElement::Expr(expr) => {
                    fields.push(MirAggregateItem::Operand(self.lower_expr(expr)?));
                }
                HirArrayElement::Spread(expr) => {
                    fields.push(MirAggregateItem::Spread(self.lower_expr(expr)?));
                }
            }
        }
        Some(fields)
    }

    fn lower_struct_literal(
        &mut self,
        def_id: DefId,
        fields: &[(crate::lexer::Symbol, HirExpr)],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let mut named = Vec::with_capacity(fields.len());
        for (name, value) in fields {
            named.push(MirNamedOperand { name: *name, value: self.lower_expr(value)? });
        }
        let ty = self.expr_ty(expr)?;
        Some(self.construct_temp(
            MirAggregateKind::Struct(def_id),
            MirAggregateFields::Named(named),
            ty,
            expr.span,
        ))
    }

    fn lower_verte(
        &mut self,
        source: &HirExpr,
        target: TypeId,
        entries: Option<&Vec<HirObjectField>>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        match self.types.get(target) {
            Type::Struct(def_id) => {
                let Some(entries) = entries else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "struct construction without object entries before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_struct_object_fields(*def_id, entries, expr.span)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(
                    MirAggregateKind::Struct(*def_id),
                    MirAggregateFields::Named(fields),
                    ty,
                    expr.span,
                ))
            }
            Type::Map(_, _) => {
                let Some(entries) = entries else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "map construction without object entries before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_map_object_fields(entries, expr.span)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Map, MirAggregateFields::Keyed(fields), ty, expr.span))
            }
            Type::Array(_) => {
                let HirExprKind::Array(elements) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "array construction from non-array source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_array_items(elements)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Array, MirAggregateFields::Ordered(fields), ty, expr.span))
            }
            Type::Set(_) => {
                let HirExprKind::Array(elements) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "set construction from non-array source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let fields = self.lower_array_items(elements)?;
                let ty = MirType::semantic(target);
                Some(self.construct_temp(MirAggregateKind::Set, MirAggregateFields::Ordered(fields), ty, expr.span))
            }
            Type::Enum(_) => {
                let HirExprKind::Call(callee, args) = &source.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from non-variant source before aggregate MIR lowering",
                    ));
                    return None;
                };
                let HirExprKind::Path(def_id) = &callee.kind else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from indirect variant before aggregate MIR lowering",
                    ));
                    return None;
                };
                if !self.context.variant_parents.contains_key(def_id) {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "enum construction from non-variant call before aggregate MIR lowering",
                    ));
                    return None;
                }
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr(arg)?);
                }
                let fields = self.variant_payload(*def_id, lowered_args);
                Some(self.construct_temp(
                    MirAggregateKind::EnumVariant(*def_id),
                    fields,
                    MirType::semantic(target),
                    expr.span,
                ))
            }
            _ => {
                self.errors
                    .push(MirError::unsupported(expr.span, "verte cast before aggregate MIR lowering"));
                None
            }
        }
    }

    fn lower_struct_object_fields(
        &mut self,
        def_id: DefId,
        entries: &[HirObjectField],
        span: Span,
    ) -> Option<Vec<MirNamedOperand>> {
        let mut supplied = FxHashSet::default();
        let mut fields = Vec::new();
        for entry in entries {
            let name = match &entry.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => *name,
                HirObjectKey::Computed(expr) => {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "computed struct keys before aggregate MIR lowering",
                    ));
                    return None;
                }
                HirObjectKey::Spread(expr) => {
                    self.errors
                        .push(MirError::unsupported(expr.span, "struct spread before aggregate MIR lowering"));
                    return None;
                }
            };
            let Some(value) = &entry.value else {
                self.errors.push(MirError::unsupported(
                    span,
                    "struct field without value before aggregate MIR lowering",
                ));
                return None;
            };
            supplied.insert(name);
            fields.push(MirNamedOperand { name, value: self.lower_expr(value)? });
        }

        if let Some(defaults) = self.context.structs.get(&def_id).cloned() {
            for field in defaults {
                if supplied.contains(&field.name) {
                    continue;
                }
                if let Some(init) = &field.init {
                    fields.push(MirNamedOperand { name: field.name, value: self.lower_expr(init)? });
                }
            }
        }

        Some(fields)
    }

    fn lower_map_object_fields(&mut self, entries: &[HirObjectField], span: Span) -> Option<Vec<MirKeyValueOperand>> {
        let mut fields = Vec::with_capacity(entries.len());
        for entry in entries {
            let key = match &entry.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    MirOperand::Constant(MirConstant::String(*name))
                }
                HirObjectKey::Computed(expr) => self.lower_expr(expr)?,
                HirObjectKey::Spread(expr) => {
                    self.errors
                        .push(MirError::unsupported(expr.span, "map spread before aggregate MIR lowering"));
                    return None;
                }
            };
            let Some(value) = &entry.value else {
                self.errors.push(MirError::unsupported(
                    span,
                    "map field without value before aggregate MIR lowering",
                ));
                return None;
            };
            fields.push(MirKeyValueOperand { key, value: self.lower_expr(value)? });
        }
        Some(fields)
    }

    fn lower_field(&mut self, object: &HirExpr, name: crate::lexer::Symbol, _expr: &HirExpr) -> Option<MirOperand> {
        let mut place = self.lower_projectable_place(object)?;
        place.projections.push(MirProjection::Field(name));
        Some(MirOperand::Place(place))
    }

    fn lower_index(&mut self, object: &HirExpr, index: &HirExpr, _expr: &HirExpr) -> Option<MirOperand> {
        let mut place = self.lower_projectable_place(object)?;
        let index = self.lower_expr(index)?;
        place.projections.push(MirProjection::Index(index));
        Some(MirOperand::Place(place))
    }

    fn lower_projectable_place(&mut self, expr: &HirExpr) -> Option<MirPlace> {
        match &expr.kind {
            HirExprKind::Path(def_id) => {
                let Some(binding) = self.bindings.get(def_id).copied() else {
                    self.errors.push(MirError::unsupported(
                        expr.span,
                        "projection base that does not resolve to a local value",
                    ));
                    return None;
                };
                Some(MirPlace::local(binding.local))
            }
            HirExprKind::Field(object, name) => {
                let mut place = self.lower_projectable_place(object)?;
                place.projections.push(MirProjection::Field(*name));
                Some(place)
            }
            HirExprKind::Index(object, index) => {
                let mut place = self.lower_projectable_place(object)?;
                let index = self.lower_expr(index)?;
                place.projections.push(MirProjection::Index(index));
                Some(place)
            }
            _ => {
                let ty = self.expr_ty(expr)?;
                let operand = self.lower_expr(expr)?;
                match operand {
                    MirOperand::Place(place) => Some(place),
                    MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
                    MirOperand::Constant(_) | MirOperand::Value(_) => {
                        let temp = self.assign_temp(MirValueKind::Operand(operand), ty, expr.span);
                        match temp {
                            MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
                            _ => unreachable!("assign_temp always returns a temp operand"),
                        }
                    }
                }
            }
        }
    }

    fn lower_optional_chain(
        &mut self,
        object: &HirExpr,
        chain: &HirOptionalChainKind,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let base = self.lower_expr(object)?;
        let link = self.lower_optional_chain_link(base.clone(), chain)?;
        let ty = self.expr_ty(expr)?;
        Some(self.assign_temp(MirValueKind::Option(MirOptionOp::Chain { base, link }), ty, expr.span))
    }

    fn lower_optional_chain_link(
        &mut self,
        base: MirOperand,
        chain: &HirOptionalChainKind,
    ) -> Option<MirOptionChainLink> {
        match chain {
            HirOptionalChainKind::Member(name) => Some(MirOptionChainLink::Field(*name)),
            HirOptionalChainKind::Index(index) => Some(MirOptionChainLink::Index(self.lower_expr(index)?)),
            HirOptionalChainKind::Call(args) => {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr(arg)?);
                }
                Some(MirOptionChainLink::Call { callee: MirCallee::Value(base), args: lowered_args })
            }
        }
    }

    fn lower_non_null(&mut self, object: &HirExpr, chain: &HirNonNullKind, expr: &HirExpr) -> Option<MirOperand> {
        let mut place = self.lower_non_null_base(object)?;
        match chain {
            HirNonNullKind::Member(name) => {
                place.projections.push(MirProjection::Field(*name));
                Some(MirOperand::Place(place))
            }
            HirNonNullKind::Index(index) => {
                let index = self.lower_expr(index)?;
                place.projections.push(MirProjection::Index(index));
                Some(MirOperand::Place(place))
            }
            HirNonNullKind::Call(_) => {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "non-null calls before callable-value MIR lowering",
                ));
                None
            }
        }
    }

    fn lower_non_null_base(&mut self, object: &HirExpr) -> Option<MirPlace> {
        let value = self.lower_expr(object)?;
        let inner_ty = self
            .option_inner_ty(object)
            .unwrap_or(self.expr_ty(object)?);
        let temp = self.assign_temp(
            MirValueKind::Option(MirOptionOp::Unwrap { value, mode: MirOptionUnwrapMode::Assert }),
            inner_ty,
            object.span,
        );
        match temp {
            MirOperand::Temp(temp) => Some(MirPlace::temp(temp)),
            _ => unreachable!("assign_temp always returns a temp operand"),
        }
    }

    fn option_inner_ty(&mut self, expr: &HirExpr) -> Option<MirType> {
        let ty = expr.ty?;
        match self.types.get(ty) {
            Type::Option(inner) => Some(MirType::semantic(*inner)),
            _ => None,
        }
    }

    fn lower_method_call(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirExpr],
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        if let HirExprKind::Path(def_id) = receiver.kind {
            if let Some(import) = self.context.provider_imports.get(&def_id).cloned() {
                let mut lowered_args = Vec::with_capacity(args.len());
                for arg in args {
                    lowered_args.push(self.lower_expr(arg)?);
                }
                let ty = self.expr_ty(expr)?;
                let mut module = import.module;
                module.push(import.item);
                return Some(self.runtime_call_value(
                    MirIntrinsic::Provider(MirProvider { module, name: method }),
                    lowered_args,
                    ty,
                    expr.span,
                ));
            }
        }

        let Some(op) = self.collection_method_op(receiver, method, args, expr.span) else {
            self.errors.push(MirError::unsupported(
                expr.span,
                "method call before runtime/provider MIR lowering",
            ));
            return None;
        };

        let mut lowered_args = Vec::with_capacity(args.len() + 1);
        lowered_args.push(self.lower_expr(receiver)?);
        for arg in args {
            lowered_args.push(self.lower_expr(arg)?);
        }
        let ty = self.expr_ty(expr)?;
        Some(self.runtime_call_value(MirIntrinsic::Collection(op), lowered_args, ty, expr.span))
    }

    fn lower_call(&mut self, callee: &HirExpr, args: &[HirExpr], expr: &HirExpr) -> Option<MirOperand> {
        let HirExprKind::Path(def_id) = &callee.kind else {
            self.errors.push(MirError::unsupported(
                callee.span,
                "indirect calls before callable-value MIR lowering",
            ));
            return None;
        };

        let mut lowered_args = Vec::with_capacity(args.len());
        for arg in args {
            let arg = self.lower_expr(arg)?;
            lowered_args.push(arg);
        }

        let ty = self.expr_ty(expr)?;

        if let Some(import) = self.context.provider_imports.get(def_id).cloned() {
            return Some(self.runtime_call_value(
                MirIntrinsic::Provider(MirProvider { module: import.module, name: import.item }),
                lowered_args,
                ty,
                expr.span,
            ));
        }

        if self.context.variant_parents.contains_key(def_id) {
            let fields = self.variant_payload(*def_id, lowered_args);
            return Some(self.construct_temp(MirAggregateKind::EnumVariant(*def_id), fields, ty, expr.span));
        }

        if let Some(_err_ty) = self.context.function_errors.get(def_id).copied() {
            let Some(handler) = self.handlers.last().cloned() else {
                self.errors.push(MirError::unsupported(
                    expr.span,
                    "failable call without an active local cape handler",
                ));
                return None;
            };

            let ok_block = self.fresh_block(expr.span);
            if self.is_vacuum(ty) {
                self.terminate_current(
                    MirTerminatorKind::TryCall {
                        destination: None,
                        callee: MirCallee::Definition(*def_id),
                        args: lowered_args,
                        ok_block,
                        error_place: handler.error_place,
                        error_block: handler.error_block,
                    },
                    expr.span,
                );
                self.switch_to(ok_block);
                return Some(MirOperand::Constant(MirConstant::Unit));
            }

            let destination = self.push_temp(ty, expr.span);
            self.terminate_current(
                MirTerminatorKind::TryCall {
                    destination: Some(MirPlace::temp(destination)),
                    callee: MirCallee::Definition(*def_id),
                    args: lowered_args,
                    ok_block,
                    error_place: handler.error_place,
                    error_block: handler.error_block,
                },
                expr.span,
            );
            self.switch_to(ok_block);
            return Some(MirOperand::Temp(destination));
        }

        if self.is_vacuum(ty) {
            self.append_stmt(MirStmt {
                kind: MirStmtKind::Call {
                    destination: None,
                    callee: MirCallee::Definition(*def_id),
                    args: lowered_args,
                },
                span: expr.span,
            });
            return Some(MirOperand::Constant(MirConstant::Unit));
        }

        let destination = self.push_temp(ty, expr.span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::Call {
                destination: Some(MirPlace::temp(destination)),
                callee: MirCallee::Definition(*def_id),
                args: lowered_args,
            },
            span: expr.span,
        });
        Some(MirOperand::Temp(destination))
    }

    fn lower_block_statement(&mut self, block: &HirBlock) {
        for stmt in &block.stmts {
            if !self.current_is_open() {
                break;
            }
            self.lower_stmt(stmt);
        }

        if let Some(expr) = &block.expr {
            if self.current_is_open() {
                let _ = self.lower_expr(expr);
            }
        }
    }

    fn lower_block_to_destination(
        &mut self,
        block: &HirBlock,
        destination: MirPlace,
        ty: MirType,
        span: Span,
    ) -> Option<()> {
        for stmt in &block.stmts {
            if !self.current_is_open() {
                return Some(());
            }
            self.lower_stmt(stmt);
        }

        if !self.current_is_open() {
            return Some(());
        }

        let Some(expr) = &block.expr else {
            if self.is_vacuum(ty) {
                self.assign(destination, MirOperand::Constant(MirConstant::Unit), ty, span);
                return Some(());
            }
            self.errors.push(MirError::unsupported(
                block.span,
                "expression-valued block without a tail expression",
            ));
            return None;
        };

        self.lower_expr_to_destination(expr, destination, ty)
    }

    fn lower_si_statement(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        then_catch: Option<&HirCape>,
        else_block: Option<&HirBlock>,
        span: Span,
    ) {
        if let Some(catch) = then_catch {
            self.lower_handled_si_statement(cond, then_block, catch, else_block, span);
            return;
        }

        let Some(condition) = self.lower_expr(cond) else {
            return;
        };

        let then_id = self.fresh_block(then_block.span);
        let (else_id, join_id) = match else_block {
            Some(block) => {
                let else_id = self.fresh_block(block.span);
                let join_id = self.fresh_block(span);
                (else_id, join_id)
            }
            None => {
                let join_id = self.fresh_block(span);
                (join_id, join_id)
            }
        };

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.lower_block_statement(then_block);
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        let else_reaches = if let Some(block) = else_block {
            self.switch_to(else_id);
            self.lower_block_statement(block);
            self.terminate_open_current(MirTerminatorKind::Goto(join_id), span)
        } else {
            true
        };

        if then_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }
    }

    fn lower_handled_si_statement(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        catch: &HirCape,
        else_block: Option<&HirBlock>,
        span: Span,
    ) {
        let Some(error_ty) = catch.binding_ty.map(MirType::semantic) else {
            self.errors
                .push(MirError::missing_type(catch.span, "cape handler binding"));
            return;
        };

        let then_id = self.fresh_block(then_block.span);
        let (else_id, join_id) = match else_block {
            Some(block) => {
                let else_id = self.fresh_block(block.span);
                let join_id = self.fresh_block(span);
                (else_id, join_id)
            }
            None => {
                let join_id = self.fresh_block(span);
                (join_id, join_id)
            }
        };
        let handler_id = self.fresh_block(catch.body.span);

        let handler_local = self.next_local_id();
        self.locals.push(MirLocalDecl {
            id: handler_local,
            name: Some(catch.binding_name),
            ty: error_ty,
            mutable: false,
            span: catch.span,
        });
        self.bindings
            .insert(catch.binding_def_id, LocalBinding { local: handler_local, ty: error_ty });

        self.handlers
            .push(HandlerContext { error_place: MirPlace::local(handler_local), error_block: handler_id });
        let Some(condition) = self.lower_expr(cond) else {
            self.handlers.pop();
            self.seal_unreachable(handler_id, catch.span);
            self.switch_to(join_id);
            return;
        };

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.lower_block_statement(then_block);
        self.handlers.pop();
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        self.switch_to(handler_id);
        self.lower_block_statement(&catch.body);
        let handler_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), catch.span);

        let else_reaches = if let Some(block) = else_block {
            self.switch_to(else_id);
            self.lower_block_statement(block);
            self.terminate_open_current(MirTerminatorKind::Goto(join_id), span)
        } else {
            true
        };

        if then_reaches || handler_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }
    }

    fn lower_si_to_destination(
        &mut self,
        cond: &HirExpr,
        then_block: &HirBlock,
        else_block: Option<&HirBlock>,
        destination: MirPlace,
        ty: MirType,
        span: Span,
    ) -> Option<()> {
        let Some(else_block) = else_block else {
            self.errors
                .push(MirError::unsupported(span, "expression-valued si without secus destination"));
            return None;
        };

        let condition = self.lower_expr(cond)?;
        let then_id = self.fresh_block(then_block.span);
        let else_id = self.fresh_block(else_block.span);
        let join_id = self.fresh_block(span);

        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: then_id, else_block: else_id },
            span,
        );

        self.switch_to(then_id);
        self.lower_block_to_destination(then_block, destination.clone(), ty, then_block.span)?;
        let then_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        self.switch_to(else_id);
        self.lower_block_to_destination(else_block, destination, ty, else_block.span)?;
        let else_reaches = self.terminate_open_current(MirTerminatorKind::Goto(join_id), span);

        if then_reaches || else_reaches {
            self.switch_to(join_id);
        } else {
            self.seal_unreachable(join_id, span);
        }

        Some(())
    }

    fn lower_dum(&mut self, cond: &HirExpr, body: &HirBlock, span: Span) {
        let cond_id = self.fresh_block(cond.span);
        let body_id = self.fresh_block(body.span);
        let after_id = self.fresh_block(span);

        self.terminate_current(MirTerminatorKind::Goto(cond_id), span);

        self.switch_to(cond_id);
        let Some(condition) = self.lower_expr(cond) else {
            self.seal_unreachable(cond_id, cond.span);
            self.switch_to(after_id);
            return;
        };
        self.terminate_current(
            MirTerminatorKind::Branch { condition, then_block: body_id, else_block: after_id },
            cond.span,
        );

        self.loops
            .push(LoopContext { perge_target: cond_id, rumpe_target: after_id });
        self.switch_to(body_id);
        self.lower_block_statement(body);
        self.loops.pop();
        self.terminate_open_current(MirTerminatorKind::Goto(cond_id), span);

        self.switch_to(after_id);
    }

    fn lower_rumpe(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "rumpe without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.rumpe_target), span);
    }

    fn lower_perge(&mut self, span: Span) {
        let Some(context) = self.loops.last().copied() else {
            self.errors
                .push(MirError::unsupported(span, "perge without an active dum loop"));
            return;
        };
        self.terminate_current(MirTerminatorKind::Goto(context.perge_target), span);
    }

    fn assign(&mut self, place: MirPlace, operand: MirOperand, ty: MirType, span: Span) {
        let value = self.new_value(MirValueKind::Operand(operand), ty, span);
        self.append_stmt(MirStmt { kind: MirStmtKind::Assign { place, value }, span });
    }

    fn assign_temp(&mut self, kind: MirValueKind, ty: MirType, span: Span) -> MirOperand {
        let temp = self.push_temp(ty, span);
        let value = self.new_value(kind, ty, span);
        self.append_stmt(MirStmt { kind: MirStmtKind::Assign { place: MirPlace::temp(temp), value }, span });
        MirOperand::Temp(temp)
    }

    fn construct_temp(
        &mut self,
        kind: MirAggregateKind,
        fields: MirAggregateFields,
        ty: MirType,
        span: Span,
    ) -> MirOperand {
        let temp = self.push_temp(ty, span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::Construct {
                destination: MirPlace::temp(temp),
                aggregate: MirAggregate { kind, ty, fields },
            },
            span,
        });
        MirOperand::Temp(temp)
    }

    fn runtime_call_value(
        &mut self,
        intrinsic: MirIntrinsic,
        args: Vec<MirOperand>,
        return_ty: MirType,
        span: Span,
    ) -> MirOperand {
        if self.is_vacuum(return_ty) {
            self.append_stmt(MirStmt {
                kind: MirStmtKind::RuntimeCall {
                    destination: None,
                    call: MirRuntimeCall { intrinsic, args, return_ty },
                },
                span,
            });
            return MirOperand::Constant(MirConstant::Unit);
        }

        let temp = self.push_temp(return_ty, span);
        self.append_stmt(MirStmt {
            kind: MirStmtKind::RuntimeCall {
                destination: Some(MirPlace::temp(temp)),
                call: MirRuntimeCall { intrinsic, args, return_ty },
            },
            span,
        });
        MirOperand::Temp(temp)
    }

    fn variant_payload(&self, variant: DefId, args: Vec<MirOperand>) -> MirAggregateFields {
        let Some(field_names) = self.context.variant_fields.get(&variant) else {
            return MirAggregateFields::Ordered(args.into_iter().map(MirAggregateItem::Operand).collect());
        };
        if field_names.len() != args.len() {
            return MirAggregateFields::Ordered(args.into_iter().map(MirAggregateItem::Operand).collect());
        }
        MirAggregateFields::Named(
            field_names
                .iter()
                .copied()
                .zip(args)
                .map(|(name, value)| MirNamedOperand { name, value })
                .collect(),
        )
    }

    fn new_value(&mut self, kind: MirValueKind, ty: MirType, span: Span) -> MirValue {
        let id = MirValueId(self.next_value);
        self.next_value += 1;
        MirValue { id, kind, ty, span }
    }

    fn push_temp(&mut self, ty: MirType, span: Span) -> MirTempId {
        let id = MirTempId(self.temps.len() as u32);
        self.temps.push(MirTemp { id, ty, span });
        id
    }

    fn fresh_block(&mut self, span: Span) -> MirBlockId {
        let id = MirBlockId(self.blocks.len() as u32);
        self.blocks
            .push(OpenBlock { id, statements: Vec::new(), terminator: None, span });
        id
    }

    fn switch_to(&mut self, block: MirBlockId) {
        if self.block(block).terminator.is_some() {
            self.current = None;
            return;
        }
        self.current = Some(block);
    }

    fn current_is_open(&self) -> bool {
        self.current
            .map(|id| self.block(id).terminator.is_none())
            .unwrap_or(false)
    }

    fn append_stmt(&mut self, stmt: MirStmt) {
        let Some(current) = self.current else {
            self.errors
                .push(MirError::unsupported(stmt.span, "statement after a MIR terminator"));
            return;
        };

        let block = self.block_mut(current);
        if block.terminator.is_some() {
            self.errors
                .push(MirError::unsupported(stmt.span, "statement after a MIR terminator"));
            self.current = None;
            return;
        }
        block.statements.push(stmt);
    }

    fn terminate_current(&mut self, kind: MirTerminatorKind, span: Span) -> bool {
        let Some(current) = self.current else {
            self.errors.push(MirError::unsupported(
                span,
                "terminator emitted after current MIR block was sealed",
            ));
            return false;
        };

        let block = self.block_mut(current);
        if block.terminator.is_some() {
            self.errors
                .push(MirError::unsupported(span, "duplicate MIR block terminator"));
            self.current = None;
            return false;
        }

        block.terminator = Some(MirTerminator { kind, span });
        self.current = None;
        true
    }

    fn terminate_open_current(&mut self, kind: MirTerminatorKind, span: Span) -> bool {
        if !self.current_is_open() {
            return false;
        }
        self.terminate_current(kind, span)
    }

    fn seal_unreachable(&mut self, block: MirBlockId, span: Span) {
        let open = self.block(block).terminator.is_none();
        if open {
            self.block_mut(block).terminator = Some(MirTerminator { kind: MirTerminatorKind::Unreachable, span });
        }
        if self.current == Some(block) {
            self.current = None;
        }
    }

    fn finish_blocks(&mut self) -> Vec<MirBlock> {
        for index in 0..self.blocks.len() {
            if self.blocks[index].terminator.is_none() {
                let span = self.blocks[index].span;
                self.blocks[index].terminator = Some(MirTerminator { kind: MirTerminatorKind::Unreachable, span });
            }
        }

        self.blocks
            .drain(..)
            .map(|block| MirBlock {
                id: block.id,
                statements: block.statements,
                terminator: block
                    .terminator
                    .expect("MIR block finalized with terminator"),
                span: block.span,
            })
            .collect()
    }

    fn block(&self, id: MirBlockId) -> &OpenBlock {
        &self.blocks[id.0 as usize]
    }

    fn block_mut(&mut self, id: MirBlockId) -> &mut OpenBlock {
        &mut self.blocks[id.0 as usize]
    }

    fn next_local_id(&self) -> MirLocalId {
        MirLocalId(self.locals.len() as u32)
    }

    fn collection_method_op(
        &mut self,
        receiver: &HirExpr,
        method: Symbol,
        args: &[HirExpr],
        span: Span,
    ) -> Option<MirCollectionOp> {
        let Some(receiver_ty) = receiver.ty else {
            self.errors
                .push(MirError::missing_type(span, "collection method receiver"));
            return None;
        };
        let method_name = self.resolve_method_name(method)?;
        let receiver_ty = self.normalized_type(receiver_ty);
        let is_array = matches!(receiver_ty, Type::Array(_));
        let is_map = matches!(receiver_ty, Type::Map(_, _));
        let is_set = matches!(receiver_ty, Type::Set(_));
        let is_text = matches!(receiver_ty, Type::Primitive(Primitive::Textus));

        match method_name {
            "appende" | "adde" if args.len() == 1 && is_array => Some(MirCollectionOp::Append),
            "addita" if args.len() == 1 && is_array => Some(MirCollectionOp::AppendImmutable),
            "accipe" if args.len() == 1 && (is_array || is_map || is_text) => Some(MirCollectionOp::Index),
            "longitudo" if args.is_empty() && (is_array || is_map || is_set || is_text) => {
                Some(MirCollectionOp::Length)
            }
            "continet" if args.len() == 1 && (is_array || is_set || is_text) => Some(MirCollectionOp::Contains),
            "habet" if args.len() == 1 && is_map => Some(MirCollectionOp::Contains),
            _ => None,
        }
    }

    fn resolve_method_name(&self, method: Symbol) -> Option<&str> {
        self.context
            .interner
            .map(|interner| interner.resolve(method))
    }

    fn expr_ty(&mut self, expr: &HirExpr) -> Option<MirType> {
        let Some(ty) = expr.ty else {
            self.errors
                .push(MirError::missing_type(expr.span, "expression"));
            return None;
        };
        Some(MirType::semantic(ty))
    }

    fn is_vacuum(&self, ty: MirType) -> bool {
        ty.semantic_id() == self.types.primitive(Primitive::Vacuum)
    }

    fn normalized_type(&self, mut ty: TypeId) -> &Type {
        loop {
            match self.types.get(ty) {
                Type::Alias(_, inner) => ty = *inner,
                other => return other,
            }
        }
    }
}

fn entry_is_empty(block: &HirBlock) -> bool {
    block.stmts.is_empty() && block.expr.is_none()
}

fn mir_un_op(op: HirUnOp) -> Option<MirUnOp> {
    match op {
        HirUnOp::Neg => Some(MirUnOp::Neg),
        HirUnOp::Not => Some(MirUnOp::Not),
        HirUnOp::BitNot => Some(MirUnOp::BitNot),
        HirUnOp::IsNull | HirUnOp::IsNil => Some(MirUnOp::IsNil),
        HirUnOp::IsNotNull | HirUnOp::IsNotNil => Some(MirUnOp::IsNonNil),
        HirUnOp::IsNeg | HirUnOp::IsPos | HirUnOp::IsTrue | HirUnOp::IsFalse => None,
    }
}

fn mir_bin_op(op: HirBinOp) -> Option<MirBinOp> {
    match op {
        HirBinOp::Add => Some(MirBinOp::Add),
        HirBinOp::Sub => Some(MirBinOp::Sub),
        HirBinOp::Mul => Some(MirBinOp::Mul),
        HirBinOp::Div => Some(MirBinOp::Div),
        HirBinOp::Mod => Some(MirBinOp::Mod),
        HirBinOp::Eq | HirBinOp::StrictEq => Some(MirBinOp::Eq),
        HirBinOp::NotEq | HirBinOp::StrictNotEq => Some(MirBinOp::NotEq),
        HirBinOp::Lt => Some(MirBinOp::Lt),
        HirBinOp::Gt => Some(MirBinOp::Gt),
        HirBinOp::LtEq => Some(MirBinOp::LtEq),
        HirBinOp::GtEq => Some(MirBinOp::GtEq),
        HirBinOp::And => Some(MirBinOp::And),
        HirBinOp::Or => Some(MirBinOp::Or),
        HirBinOp::Coalesce => Some(MirBinOp::Coalesce),
        HirBinOp::BitAnd => Some(MirBinOp::BitAnd),
        HirBinOp::BitOr => Some(MirBinOp::BitOr),
        HirBinOp::BitXor => Some(MirBinOp::BitXor),
        HirBinOp::Shl => Some(MirBinOp::Shl),
        HirBinOp::Shr => Some(MirBinOp::Shr),
        HirBinOp::Is | HirBinOp::IsNot | HirBinOp::InRange | HirBinOp::Between => None,
    }
}

fn unsupported_expr_kind_name(kind: &HirExprKind) -> &'static str {
    match kind {
        HirExprKind::MethodCall(_, _, _) => "method calls before receiver-aware MIR lowering",
        HirExprKind::Field(_, _) => "field access before aggregate place MIR lowering",
        HirExprKind::Index(_, _) => "index access before aggregate place MIR lowering",
        HirExprKind::OptionalChain(_, _) => "optional chains before nullable control-flow MIR lowering",
        HirExprKind::NonNull(_, _) => "non-null assertions before nullable control-flow MIR lowering",
        HirExprKind::Ab { .. } => "ab collection pipelines before collection MIR lowering",
        HirExprKind::Block(_) => "block expressions before nested-block MIR lowering",
        HirExprKind::Si { .. } => "si before control-flow MIR lowering",
        HirExprKind::Discerne(_, _) => "discerne before switch MIR lowering",
        HirExprKind::Loop(_) => "loop before control-flow MIR lowering",
        HirExprKind::Dum(_, _) => "dum before control-flow MIR lowering",
        HirExprKind::Itera(_, _, _, _, _) => "itera before iterator MIR lowering",
        HirExprKind::Intervallum { .. } => "range expressions before range MIR lowering",
        HirExprKind::Array(_) => "array literals before aggregate MIR lowering",
        HirExprKind::Struct(_, _) => "struct literals before aggregate MIR lowering",
        HirExprKind::Tuple(_) => "tuple literals before aggregate MIR lowering",
        HirExprKind::Scribe(kind, _) => scribe_kind_name(*kind),
        HirExprKind::Scriptum(_, _) => "scriptum templates before format intrinsic MIR lowering",
        HirExprKind::Adfirma(_, _) => "adfirma before assert intrinsic MIR lowering",
        HirExprKind::Panic(_) => "mori fatal flow",
        HirExprKind::Throw(_) => "iace error-flow",
        HirExprKind::Handled { .. } => "structured cape before local-handler MIR lowering",
        HirExprKind::Tempta { .. } => "tempta legacy local-handler surface deferred to Phase 5C",
        HirExprKind::Clausura(_, _, _) => "closures before callable-value MIR lowering",
        HirExprKind::Cede(_) => "cede before async MIR lowering",
        HirExprKind::Verte { .. } => "verte before conversion MIR lowering",
        HirExprKind::Conversio { .. } => "conversio before runtime conversion MIR lowering",
        HirExprKind::Ref(_, _) => "references before borrow-aware MIR lowering",
        HirExprKind::Deref(_) => "dereferences before borrow-aware MIR lowering",
        HirExprKind::Error => "error expressions",
        HirExprKind::Path(_)
        | HirExprKind::Literal(_)
        | HirExprKind::Binary(_, _, _)
        | HirExprKind::Unary(_, _)
        | HirExprKind::Call(_, _)
        | HirExprKind::Assign(_, _)
        | HirExprKind::AssignOp(_, _, _) => "primitive expression",
    }
}

fn scribe_kind_name(kind: HirScribeKind) -> &'static str {
    match kind {
        HirScribeKind::Nota => "nota before print/runtime intrinsic MIR lowering",
        HirScribeKind::Vide => "vide before print/runtime intrinsic MIR lowering",
        HirScribeKind::Mone => "mone before print/runtime intrinsic MIR lowering",
        HirScribeKind::Scribe => "scribe before print/runtime intrinsic MIR lowering",
    }
}

fn mir_diagnostic_kind(kind: HirScribeKind) -> MirDiagnosticKind {
    match kind {
        HirScribeKind::Nota => MirDiagnosticKind::Nota,
        HirScribeKind::Vide => MirDiagnosticKind::Vide,
        HirScribeKind::Mone => MirDiagnosticKind::Mone,
        HirScribeKind::Scribe => MirDiagnosticKind::Scribe,
    }
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
