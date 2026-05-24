//! HIR-to-MIR lowering orchestration for the compiler-developer MIR surface.
//!
//! This module is the boundary between a fully analyzed HIR unit and the
//! execution-shaped MIR model used by MIR dumps, validation, and temporary MIR
//! probes. It deliberately depends on [`AnalyzedUnit`] rather than raw HIR so
//! lowering can preserve semantic identity, resolved definitions, inferred
//! types, entry metadata, and the interner without rebuilding semantic context.
//!
//! ERROR STRATEGY
//! ==============
//! MIR lowering is fail-closed infrastructure. Unsupported HIR surfaces, missing
//! semantic types, sealed-block misuse, and post-lowering validation failures are
//! reported as [`MirError`] values; they are not guessed around in order to emit
//! partial MIR. This keeps developer-facing MIR inspection honest while the MIR
//! subset grows toward the full language.
//!
//! ORCHESTRATION
//! =============
//! [`MirLowerer`] owns whole-program order: reject unsupported CLI-package entry
//! shapes, collect context maps, lower supported top-level items, synthesize the
//! primitive entry function when allowed, and hand the complete program to
//! validation. [`FunctionBuilder`] owns one function body at a time, including
//! local bindings, temporaries, open basic blocks, loop/handler stacks, and
//! value numbering.
//!
//! INVARIANTS
//! ==========
//! - Every emitted block is finalized with exactly one terminator before the
//!   program reaches validation.
//! - MIR types are copied from semantic `TypeId`s; missing HIR type information
//!   is a lowering error, not a codegen fallback.
//! - Unsupported features remain explicit diagnostics so later MIR work can add
//!   support without inheriting silent behavior.

use crate::driver::AnalyzedUnit;
use crate::hir::visit::HirVisitor;
use crate::hir::{
    DefId, HirArrayElement, HirBinOp, HirBlock, HirCape, HirExpr, HirExprKind, HirField, HirFunction, HirItem,
    HirItemKind, HirLiteral, HirLocal, HirNonNullKind, HirObjectField, HirObjectKey, HirOptionalChainKind,
    HirScribeKind, HirStmt, HirStmtKind, HirUnOp,
};
use crate::lexer::{Interner, Span, Symbol};
use crate::mir::{
    dump_program, validate_program, MirAggregate, MirAggregateFields, MirAggregateItem, MirAggregateKind, MirBinOp,
    MirBlock, MirBlockId, MirCallee, MirCollectionOp, MirConstant, MirConversion, MirConversionFlavor,
    MirDiagnosticKind, MirFunction, MirFunctionId, MirIntrinsic, MirKeyValueOperand, MirLocal as MirLocalDecl,
    MirLocalId, MirNamedOperand, MirOperand, MirOptionChainLink, MirOptionOp, MirOptionUnwrapMode, MirParam, MirPlace,
    MirProgram, MirProjection, MirProvider, MirRuntimeCall, MirStmt, MirStmtKind, MirTemp, MirTempId, MirTerminator,
    MirTerminatorKind, MirType, MirUnOp, MirValidationContext, MirValue, MirValueId, MirValueKind,
};
use crate::semantic::{Primitive, Type, TypeId, TypeTable};
use rustc_hash::{FxHashMap, FxHashSet};

mod aggregate;
mod context;
mod control;
mod expr;
mod item;
mod place;
mod runtime;
mod stmt;

use context::{struct_field_map, LoweringContextMaps};
use expr::HirExprLoweringVisitor;
use item::ItemLoweringPass;

/// Developer-facing MIR lowering diagnostic.
///
/// MIR lowering errors describe gaps between the typed HIR surface and the MIR
/// subset currently implemented. They intentionally keep only a span and message
/// because the caller is a compiler inspection/probe surface, not the primary
/// user diagnostic renderer.
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

/// Lower a semantically analyzed HIR unit into a validated MIR program.
///
/// The returned program has already passed MIR validation. Lowering failures and
/// validation failures share the same error type so callers can treat the MIR
/// pipeline as one fail-closed developer command.
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

/// Lower and render a deterministic MIR dump for compiler inspection.
///
/// This helper exists so `radix mir` can expose one operation without letting
/// dump rendering bypass validation.
pub fn dump_analyzed_unit(unit: &AnalyzedUnit) -> Result<String, Vec<MirError>> {
    lower_analyzed_unit(unit).map(|program| dump_program(&program))
}

struct MirLowerer<'a> {
    unit: &'a AnalyzedUnit,
    errors: Vec<MirError>,
    functions: Vec<MirFunction>,
}

impl MirLowerer<'_> {
    /// Run the whole-unit lowering sequence before MIR validation.
    ///
    /// Whole-unit maps are collected up front because function bodies need
    /// signatures, enum variant metadata, provider imports, and validation
    /// tables that may be declared later in source order.
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

        let unit = self.unit;
        let context_maps = LoweringContextMaps::collect(unit);
        let struct_fields = struct_field_map(unit);
        ItemLoweringPass::new(self, &context_maps, &struct_fields).lower_items();

        if let Some(entry) = &self.unit.hir.entry {
            self.lower_entry(entry);
        }
    }

    /// Lower one HIR function item into one MIR function.
    ///
    /// Function lowering is intentionally type-strict: parameters, locals, the
    /// return type, and alternate-exit type must already be present on HIR. The
    /// builder may still emit local lowering errors, but this wrapper preserves
    /// source-order accounting for the function while the whole-unit pipeline
    /// decides whether validation is allowed to run.
    fn lower_function(
        &mut self,
        item: &HirItem,
        function: &HirFunction,
        context_maps: &LoweringContextMaps<'_>,
        struct_fields: &FxHashMap<DefId, Vec<&HirField>>,
    ) {
        let Some(return_ty) = function.ret_ty else {
            self.errors
                .push(MirError::missing_type(item.span, "function return type"));
            return;
        };

        let error_ty = function.err_ty.map(MirType::semantic);
        let context = context_maps.builder_context(&self.unit.interner, struct_fields.clone());
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

    /// Rebuild validation context from the analyzed unit for the final handoff.
    ///
    /// The validation maps are derived from immutable HIR/type data rather than
    /// from partially lowered functions, so validation checks the MIR against the
    /// same semantic source of truth as lowering.
    fn validation_context(&self) -> MirValidationContext<'_> {
        LoweringContextMaps::collect(self.unit).validation
    }

    /// Lower the synthetic top-level entry block accepted by the current MIR subset.
    ///
    /// Non-empty entry bodies remain unsupported here because top-level entry
    /// semantics still need a fuller statement and runtime policy before they
    /// can be represented faithfully in MIR.
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

/// Immutable whole-unit facts needed while lowering one function body.
///
/// This context is intentionally cloned into each [`FunctionBuilder`] instead of
/// letting builders mutate global lowering state. Per-function lowering can then
/// accumulate local errors and CFG state while still resolving function errors,
/// aggregate fields, enum payload names, and provider imports consistently.
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

/// Stateful builder for one MIR function.
///
/// The builder translates typed HIR body structure into MIR locals, temporaries,
/// values, and basic blocks. Its mutable state models the current CFG frontier:
/// `current` is `None` after a terminator, `loops` records `perge`/`rumpe`
/// targets, and `handlers` records active alternate-exit handlers. Any attempt
/// to keep emitting after a block is sealed becomes a lowering error.
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
        self.visit_block(body);
        self.terminate_open_current(MirTerminatorKind::Return(None), body.span);
        self.finish_blocks()
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
            return self.lower_expr_value(expr);
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
                let index = self.lower_expr_value(index)?;
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

    fn lower_transfer_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        let operand = self.lower_expr_value(expr)?;
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
        let operand = self.lower_expr_value(operand)?;
        let ty = self.expr_ty(expr)?;

        Some(self.assign_temp(MirValueKind::Unary { op, operand }, ty, expr.span))
    }

    fn lower_binary(&mut self, op: HirBinOp, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        let Some(op) = mir_bin_op(op) else {
            self.errors
                .push(MirError::unsupported(expr.span, "binary operator without a MIR primitive"));
            return None;
        };
        let lhs = self.lower_expr_value(lhs)?;
        let rhs = self.lower_expr_value(rhs)?;
        let ty = self.expr_ty(expr)?;

        Some(self.assign_temp(MirValueKind::Binary { op, lhs, rhs }, ty, expr.span))
    }

    fn lower_coalesce(&mut self, lhs: &HirExpr, rhs: &HirExpr, expr: &HirExpr) -> Option<MirOperand> {
        let value = self.lower_expr_value(lhs)?;
        let fallback = self.lower_expr_value(rhs)?;
        let ty = self.expr_ty(expr)?;
        Some(self.assign_temp(MirValueKind::Option(MirOptionOp::Coalesce { value, fallback }), ty, expr.span))
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
        | HirExprKind::Vacua
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
