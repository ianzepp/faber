use crate::driver::AnalyzedUnit;
use crate::hir::{
    DefId, HirBinOp, HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirLocal,
    HirScribeKind, HirStmt, HirStmtKind, HirUnOp,
};
use crate::lexer::Span;
use crate::mir::{
    dump_program, MirBinOp, MirBlock, MirBlockId, MirCallee, MirConstant, MirFunction, MirFunctionId,
    MirLocal as MirLocalDecl, MirLocalId, MirOperand, MirParam, MirPlace, MirProgram, MirStmt, MirStmtKind, MirTemp,
    MirTempId, MirTerminator, MirTerminatorKind, MirType, MirUnOp, MirValue, MirValueId, MirValueKind,
};
use crate::semantic::{Primitive, TypeId, TypeTable};
use rustc_hash::FxHashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirError {
    pub message: String,
    pub span: Span,
}

impl MirError {
    fn unsupported(span: Span, what: impl Into<String>) -> Self {
        Self { message: format!("unsupported MIR lowering in phase 4: {}", what.into()), span }
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

        let mut builder = FunctionBuilder::new(&self.unit.types);
        for param in &function.params {
            builder.add_param(param.def_id, param.name, param.ty, param.span);
        }

        let blocks = match &function.body {
            Some(body) => builder.lower_body(body),
            None => Vec::new(),
        };
        self.errors.extend(builder.errors);

        self.functions.push(MirFunction {
            id: MirFunctionId(self.functions.len() as u32),
            source: Some(item.def_id),
            name: Some(function.name),
            params: builder.params,
            locals: builder.locals,
            temps: builder.temps,
            blocks,
            return_ty: MirType::semantic(return_ty),
            span: item.span,
        });
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

struct FunctionBuilder<'a> {
    types: &'a TypeTable,
    bindings: FxHashMap<DefId, LocalBinding>,
    params: Vec<MirParam>,
    locals: Vec<MirLocalDecl>,
    temps: Vec<MirTemp>,
    blocks: Vec<OpenBlock>,
    current: Option<MirBlockId>,
    loops: Vec<LoopContext>,
    errors: Vec<MirError>,
    next_value: u32,
}

impl<'a> FunctionBuilder<'a> {
    fn new(types: &'a TypeTable) -> Self {
        Self {
            types,
            bindings: FxHashMap::default(),
            params: Vec::new(),
            locals: Vec::new(),
            temps: Vec::new(),
            blocks: Vec::new(),
            current: None,
            loops: Vec::new(),
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

        let (place, ty) = self.lower_assignment_place(lhs)?;
        self.lower_expr_to_destination(rhs, place.clone(), ty)?;
        Some(MirOperand::Place(place))
    }

    fn lower_assignment_place(&mut self, expr: &HirExpr) -> Option<(MirPlace, MirType)> {
        let HirExprKind::Path(def_id) = &expr.kind else {
            self.errors
                .push(MirError::unsupported(expr.span, "assignment target that is not a local place"));
            return None;
        };

        let Some(binding) = self.bindings.get(def_id).copied() else {
            self.errors.push(MirError::unsupported(
                expr.span,
                "assignment target that does not resolve to a local",
            ));
            return None;
        };

        Some((MirPlace::local(binding.local), binding.ty))
    }

    fn lower_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        match &expr.kind {
            HirExprKind::Path(def_id) => self.lower_path(*def_id, expr.span),
            HirExprKind::Literal(literal) => self.lower_literal(literal, expr.span),
            HirExprKind::Unary(op, operand) => self.lower_unary(*op, operand, expr),
            HirExprKind::Binary(op, lhs, rhs) => self.lower_binary(*op, lhs, rhs, expr),
            HirExprKind::Call(callee, args) => self.lower_call(callee, args, expr),
            HirExprKind::Block(block) => self.lower_block_expr(block, expr),
            HirExprKind::Si(cond, then_block, else_block) => self.lower_si_expr(cond, then_block, else_block, expr),
            HirExprKind::Dum(cond, block) => self.lower_dum_expr(cond, block, expr),
            HirExprKind::Assign(_, _) => self.lower_assignment_expr(expr),
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

    fn lower_return_expr(&mut self, expr: &HirExpr) -> Option<MirOperand> {
        let operand = self.lower_expr(expr)?;
        match operand {
            MirOperand::Constant(_) | MirOperand::Value(_) => {
                let ty = self.expr_ty(expr)?;
                Some(self.assign_temp(MirValueKind::Operand(operand), ty, expr.span))
            }
            MirOperand::Place(_) | MirOperand::Temp(_) => Some(operand),
        }
    }

    fn lower_expr_to_destination(&mut self, expr: &HirExpr, destination: MirPlace, ty: MirType) -> Option<()> {
        match &expr.kind {
            HirExprKind::Block(block) => self.lower_block_to_destination(block, destination, ty, expr.span),
            HirExprKind::Si(cond, then_block, else_block) => {
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
        else_block: &Option<HirBlock>,
        expr: &HirExpr,
    ) -> Option<MirOperand> {
        let ty = self.expr_ty(expr)?;
        if self.is_vacuum(ty) {
            self.lower_si_statement(cond, then_block, else_block.as_ref(), expr.span);
            return Some(MirOperand::Constant(MirConstant::Unit));
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

    fn lower_si_statement(&mut self, cond: &HirExpr, then_block: &HirBlock, else_block: Option<&HirBlock>, span: Span) {
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
        HirExprKind::Si(_, _, _) => "si before control-flow MIR lowering",
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
        HirExprKind::Panic(_) => "panic before panic intrinsic MIR lowering",
        HirExprKind::Throw(_) => "iace before error-flow MIR lowering",
        HirExprKind::Tempta { .. } => "tempta before error-flow MIR lowering",
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
