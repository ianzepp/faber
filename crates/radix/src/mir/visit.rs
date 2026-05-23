//! Read-only MIR visitor trait plus CFG edge helpers.
//!
//! WHY: MIR is stored as functions containing ordered blocks, statements, and
//! terminators. Structural walks should follow that storage order; CFG walks
//! should use explicit successor helpers so loops and shared targets do not
//! cause accidental recursive revisits.

use super::nodes::*;

pub trait MirVisitor: Sized {
    fn visit_program(&mut self, program: &MirProgram) {
        walk_program(self, program);
    }

    fn visit_function(&mut self, function: &MirFunction) {
        walk_function(self, function);
    }

    fn visit_param(&mut self, _param: &MirParam) {}

    fn visit_local(&mut self, _local: &MirLocal) {}

    fn visit_temp(&mut self, _temp: &MirTemp) {}

    fn visit_block(&mut self, block: &MirBlock) {
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &MirStmt) {
        walk_stmt(self, stmt);
    }

    fn visit_terminator(&mut self, terminator: &MirTerminator) {
        walk_terminator(self, terminator);
    }

    fn visit_value(&mut self, value: &MirValue) {
        walk_value(self, value);
    }

    fn visit_operand(&mut self, operand: &MirOperand) {
        walk_operand(self, operand);
    }

    fn visit_place(&mut self, place: &MirPlace) {
        walk_place(self, place);
    }

    fn visit_callee(&mut self, callee: &MirCallee) {
        walk_callee(self, callee);
    }

    fn visit_runtime_call(&mut self, call: &MirRuntimeCall) {
        walk_runtime_call(self, call);
    }

    fn visit_aggregate(&mut self, aggregate: &MirAggregate) {
        walk_aggregate(self, aggregate);
    }

    fn visit_option_op(&mut self, op: &MirOptionOp) {
        walk_option_op(self, op);
    }

    fn visit_option_chain_link(&mut self, link: &MirOptionChainLink) {
        walk_option_chain_link(self, link);
    }
}

pub fn walk_program<V: MirVisitor>(visitor: &mut V, program: &MirProgram) {
    for function in &program.functions {
        visitor.visit_function(function);
    }
}

pub fn walk_function<V: MirVisitor>(visitor: &mut V, function: &MirFunction) {
    for param in &function.params {
        visitor.visit_param(param);
    }
    for local in &function.locals {
        visitor.visit_local(local);
    }
    for temp in &function.temps {
        visitor.visit_temp(temp);
    }
    for block in &function.blocks {
        visitor.visit_block(block);
    }
}

pub fn walk_block<V: MirVisitor>(visitor: &mut V, block: &MirBlock) {
    for stmt in &block.statements {
        visitor.visit_stmt(stmt);
    }
    visitor.visit_terminator(&block.terminator);
}

pub fn walk_stmt<V: MirVisitor>(visitor: &mut V, stmt: &MirStmt) {
    match &stmt.kind {
        MirStmtKind::Assign { place, value } => {
            visitor.visit_place(place);
            visitor.visit_value(value);
        }
        MirStmtKind::Call { destination, callee, args } => {
            if let Some(destination) = destination {
                visitor.visit_place(destination);
            }
            visitor.visit_callee(callee);
            for arg in args {
                visitor.visit_operand(arg);
            }
        }
        MirStmtKind::RuntimeCall { destination, call } => {
            if let Some(destination) = destination {
                visitor.visit_place(destination);
            }
            visitor.visit_runtime_call(call);
        }
        MirStmtKind::Construct { destination, aggregate } => {
            visitor.visit_place(destination);
            visitor.visit_aggregate(aggregate);
        }
    }
}

pub fn walk_terminator<V: MirVisitor>(visitor: &mut V, terminator: &MirTerminator) {
    match &terminator.kind {
        MirTerminatorKind::Return(Some(value)) | MirTerminatorKind::ReturnError(value) => {
            visitor.visit_operand(value);
        }
        MirTerminatorKind::TryCall { destination, callee, args, error_place, .. } => {
            if let Some(destination) = destination {
                visitor.visit_place(destination);
            }
            visitor.visit_callee(callee);
            for arg in args {
                visitor.visit_operand(arg);
            }
            visitor.visit_place(error_place);
        }
        MirTerminatorKind::Branch { condition, .. } => {
            visitor.visit_operand(condition);
        }
        MirTerminatorKind::Switch { value, .. } => {
            visitor.visit_operand(value);
        }
        MirTerminatorKind::Return(None) | MirTerminatorKind::Goto(_) | MirTerminatorKind::Unreachable => {}
    }
}

pub fn walk_value<V: MirVisitor>(visitor: &mut V, value: &MirValue) {
    match &value.kind {
        MirValueKind::Operand(operand) => visitor.visit_operand(operand),
        MirValueKind::Unary { operand, .. } => visitor.visit_operand(operand),
        MirValueKind::Binary { lhs, rhs, .. } => {
            visitor.visit_operand(lhs);
            visitor.visit_operand(rhs);
        }
        MirValueKind::Option(op) => visitor.visit_option_op(op),
    }
}

pub fn walk_operand<V: MirVisitor>(visitor: &mut V, operand: &MirOperand) {
    if let MirOperand::Place(place) = operand {
        visitor.visit_place(place);
    }
}

pub fn walk_place<V: MirVisitor>(visitor: &mut V, place: &MirPlace) {
    for projection in &place.projections {
        if let MirProjection::Index(index) = projection {
            visitor.visit_operand(index);
        }
    }
}

pub fn walk_callee<V: MirVisitor>(visitor: &mut V, callee: &MirCallee) {
    if let MirCallee::Value(value) = callee {
        visitor.visit_operand(value);
    }
}

pub fn walk_runtime_call<V: MirVisitor>(visitor: &mut V, call: &MirRuntimeCall) {
    for arg in &call.args {
        visitor.visit_operand(arg);
    }
    if let MirIntrinsic::Convert(conversion) = &call.intrinsic {
        if let Some(fallback) = &conversion.fallback {
            visitor.visit_operand(fallback);
        }
    }
}

pub fn walk_aggregate<V: MirVisitor>(visitor: &mut V, aggregate: &MirAggregate) {
    match &aggregate.fields {
        MirAggregateFields::Ordered(items) => {
            for item in items {
                match item {
                    MirAggregateItem::Operand(value) | MirAggregateItem::Spread(value) => {
                        visitor.visit_operand(value);
                    }
                }
            }
        }
        MirAggregateFields::Named(items) => {
            for item in items {
                visitor.visit_operand(&item.value);
            }
        }
        MirAggregateFields::Keyed(items) => {
            for item in items {
                visitor.visit_operand(&item.key);
                visitor.visit_operand(&item.value);
            }
        }
    }
}

pub fn walk_option_op<V: MirVisitor>(visitor: &mut V, op: &MirOptionOp) {
    match op {
        MirOptionOp::None => {}
        MirOptionOp::Some(value)
        | MirOptionOp::IsNil(value)
        | MirOptionOp::IsNonNil(value)
        | MirOptionOp::Unwrap { value, .. } => visitor.visit_operand(value),
        MirOptionOp::Coalesce { value, fallback } => {
            visitor.visit_operand(value);
            visitor.visit_operand(fallback);
        }
        MirOptionOp::Chain { base, link } => {
            visitor.visit_operand(base);
            visitor.visit_option_chain_link(link);
        }
    }
}

pub fn walk_option_chain_link<V: MirVisitor>(visitor: &mut V, link: &MirOptionChainLink) {
    match link {
        MirOptionChainLink::Field(_) | MirOptionChainLink::VariantField { .. } => {}
        MirOptionChainLink::Index(index) => visitor.visit_operand(index),
        MirOptionChainLink::Call { callee, args } => {
            visitor.visit_callee(callee);
            for arg in args {
                visitor.visit_operand(arg);
            }
        }
    }
}

pub fn terminator_successors(kind: &MirTerminatorKind) -> Vec<MirBlockId> {
    match kind {
        MirTerminatorKind::TryCall { ok_block, error_block, .. } => vec![*ok_block, *error_block],
        MirTerminatorKind::Goto(target) => vec![*target],
        MirTerminatorKind::Branch { then_block, else_block, .. } => vec![*then_block, *else_block],
        MirTerminatorKind::Switch { cases, default, .. } => {
            let mut successors = cases.iter().map(|case| case.target).collect::<Vec<_>>();
            successors.push(*default);
            successors
        }
        MirTerminatorKind::Return(_) | MirTerminatorKind::ReturnError(_) | MirTerminatorKind::Unreachable => Vec::new(),
    }
}

#[cfg(test)]
#[path = "visit_test.rs"]
mod tests;
