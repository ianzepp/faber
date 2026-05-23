use super::*;
use crate::lexer::Span;
use crate::semantic::TypeId;

fn span() -> Span {
    Span::new(0, 0)
}

fn ty() -> MirType {
    MirType::semantic(TypeId(0))
}

fn sample_program() -> MirProgram {
    MirProgram {
        functions: vec![MirFunction {
            id: MirFunctionId(7),
            source: None,
            name: None,
            params: vec![MirParam { local: MirLocalId(0), name: None, ty: ty(), span: span() }],
            locals: vec![MirLocal { id: MirLocalId(1), name: None, ty: ty(), mutable: false, span: span() }],
            temps: vec![MirTemp { id: MirTempId(0), ty: ty(), span: span() }],
            blocks: vec![
                MirBlock {
                    id: MirBlockId(0),
                    statements: vec![MirStmt {
                        kind: MirStmtKind::Assign {
                            place: MirPlace::local(MirLocalId(1)),
                            value: MirValue {
                                id: MirValueId(0),
                                kind: MirValueKind::Operand(MirOperand::Place(MirPlace::local(MirLocalId(0)))),
                                ty: ty(),
                                span: span(),
                            },
                        },
                        span: span(),
                    }],
                    terminator: MirTerminator {
                        kind: MirTerminatorKind::Branch {
                            condition: MirOperand::Value(MirValueId(0)),
                            then_block: MirBlockId(1),
                            else_block: MirBlockId(2),
                        },
                        span: span(),
                    },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(1),
                    statements: Vec::new(),
                    terminator: MirTerminator { kind: MirTerminatorKind::Goto(MirBlockId(2)), span: span() },
                    span: span(),
                },
                MirBlock {
                    id: MirBlockId(2),
                    statements: Vec::new(),
                    terminator: MirTerminator { kind: MirTerminatorKind::Return(None), span: span() },
                    span: span(),
                },
            ],
            return_ty: ty(),
            error_ty: None,
            span: span(),
        }],
    }
}

#[derive(Default)]
struct CountingVisitor {
    functions: usize,
    params: usize,
    locals: usize,
    temps: usize,
    blocks: usize,
    statements: usize,
    terminators: usize,
    values: usize,
    places: usize,
}

impl MirVisitor for CountingVisitor {
    fn visit_function(&mut self, function: &MirFunction) {
        self.functions += 1;
        walk_function(self, function);
    }

    fn visit_param(&mut self, _param: &MirParam) {
        self.params += 1;
    }

    fn visit_local(&mut self, _local: &MirLocal) {
        self.locals += 1;
    }

    fn visit_temp(&mut self, _temp: &MirTemp) {
        self.temps += 1;
    }

    fn visit_block(&mut self, block: &MirBlock) {
        self.blocks += 1;
        walk_block(self, block);
    }

    fn visit_stmt(&mut self, stmt: &MirStmt) {
        self.statements += 1;
        walk_stmt(self, stmt);
    }

    fn visit_terminator(&mut self, terminator: &MirTerminator) {
        self.terminators += 1;
        walk_terminator(self, terminator);
    }

    fn visit_value(&mut self, value: &MirValue) {
        self.values += 1;
        walk_value(self, value);
    }

    fn visit_place(&mut self, place: &MirPlace) {
        self.places += 1;
        walk_place(self, place);
    }
}

#[test]
fn structural_visitor_walks_storage_order_without_following_cfg_edges() {
    let mut visitor = CountingVisitor::default();
    visitor.visit_program(&sample_program());

    assert_eq!(visitor.functions, 1);
    assert_eq!(visitor.params, 1);
    assert_eq!(visitor.locals, 1);
    assert_eq!(visitor.temps, 1);
    assert_eq!(visitor.blocks, 3);
    assert_eq!(visitor.statements, 1);
    assert_eq!(visitor.terminators, 3);
    assert_eq!(visitor.values, 1);
    assert_eq!(visitor.places, 2);
}

#[test]
fn terminator_successors_reports_cfg_edges_without_operands() {
    let branch = MirTerminatorKind::Branch {
        condition: MirOperand::Constant(MirConstant::Bool(true)),
        then_block: MirBlockId(3),
        else_block: MirBlockId(4),
    };
    assert_eq!(terminator_successors(&branch), vec![MirBlockId(3), MirBlockId(4)]);

    let switch = MirTerminatorKind::Switch {
        value: MirOperand::Constant(MirConstant::Int(1)),
        cases: vec![
            MirSwitchCase { value: MirConstant::Int(1), target: MirBlockId(8) },
            MirSwitchCase { value: MirConstant::Int(2), target: MirBlockId(9) },
        ],
        default: MirBlockId(10),
    };
    assert_eq!(
        terminator_successors(&switch),
        vec![MirBlockId(8), MirBlockId(9), MirBlockId(10)]
    );

    assert_eq!(terminator_successors(&MirTerminatorKind::Return(None)), Vec::new());
}
