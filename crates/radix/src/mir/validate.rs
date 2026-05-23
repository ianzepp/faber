use crate::hir::DefId;
use crate::lexer::{Span, Symbol};
use crate::mir::visit::{terminator_successors, MirVisitor};
use crate::mir::*;
use crate::semantic::{Primitive, Type, TypeTable};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirValidationError {
    pub message: String,
    pub span: Span,
}

impl MirValidationError {
    fn new(span: Span, message: impl Into<String>) -> Self {
        Self { message: message.into(), span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunctionSignature {
    pub params: Vec<MirType>,
    pub return_ty: MirType,
    pub error_ty: Option<MirType>,
}

pub struct MirValidationContext<'a> {
    pub types: &'a TypeTable,
    pub functions: FxHashMap<DefId, MirFunctionSignature>,
    pub struct_fields: FxHashMap<DefId, FxHashMap<Symbol, MirType>>,
    pub variant_fields: FxHashMap<DefId, FxHashMap<Symbol, MirType>>,
}

impl<'a> MirValidationContext<'a> {
    pub fn new(types: &'a TypeTable) -> Self {
        Self {
            types,
            functions: FxHashMap::default(),
            struct_fields: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
        }
    }
}

pub fn validate_program(
    program: &MirProgram,
    context: &MirValidationContext<'_>,
) -> Result<(), Vec<MirValidationError>> {
    let mut validator = Validator { program, context, errors: Vec::new() };
    validator.validate_program();
    if validator.errors.is_empty() {
        Ok(())
    } else {
        Err(validator.errors)
    }
}

struct Validator<'a, 'ctx> {
    program: &'a MirProgram,
    context: &'ctx MirValidationContext<'ctx>,
    errors: Vec<MirValidationError>,
}

struct FunctionScope<'a> {
    function: &'a MirFunction,
    functions_by_id: FxHashMap<MirFunctionId, &'a MirFunction>,
    locals: FxHashMap<MirLocalId, MirType>,
    temps: FxHashMap<MirTempId, MirType>,
    blocks: FxHashSet<MirBlockId>,
    values: FxHashMap<MirValueId, MirType>,
}

struct DuplicateFunctionFinder {
    ids: FxHashSet<MirFunctionId>,
    duplicates: Vec<(MirFunctionId, Span)>,
}

impl DuplicateFunctionFinder {
    fn find(program: &MirProgram) -> Vec<(MirFunctionId, Span)> {
        let mut finder = Self { ids: FxHashSet::default(), duplicates: Vec::new() };
        finder.visit_program(program);
        finder.duplicates
    }
}

impl MirVisitor for DuplicateFunctionFinder {
    fn visit_function(&mut self, function: &MirFunction) {
        if !self.ids.insert(function.id) {
            self.duplicates.push((function.id, function.span));
        }
    }
}

struct FunctionShape {
    locals: FxHashMap<MirLocalId, MirType>,
    temps: FxHashMap<MirTempId, MirType>,
    blocks: FxHashSet<MirBlockId>,
    duplicate_param_locals: Vec<(MirLocalId, Span)>,
    conflicting_locals: Vec<(MirLocalId, Span)>,
    duplicate_temps: Vec<(MirTempId, Span)>,
    duplicate_blocks: Vec<(MirBlockId, Span)>,
}

impl FunctionShape {
    fn collect(function: &MirFunction) -> Self {
        let mut shape = Self {
            locals: FxHashMap::default(),
            temps: FxHashMap::default(),
            blocks: FxHashSet::default(),
            duplicate_param_locals: Vec::new(),
            conflicting_locals: Vec::new(),
            duplicate_temps: Vec::new(),
            duplicate_blocks: Vec::new(),
        };
        shape.visit_function(function);
        shape
    }
}

impl MirVisitor for FunctionShape {
    fn visit_param(&mut self, param: &MirParam) {
        if self.locals.insert(param.local, param.ty).is_some() {
            self.duplicate_param_locals.push((param.local, param.span));
        }
    }

    fn visit_local(&mut self, local: &MirLocal) {
        match self.locals.insert(local.id, local.ty) {
            Some(existing) if existing.semantic_id() != local.ty.semantic_id() => {
                self.conflicting_locals.push((local.id, local.span));
            }
            _ => {}
        }
    }

    fn visit_temp(&mut self, temp: &MirTemp) {
        if self.temps.insert(temp.id, temp.ty).is_some() {
            self.duplicate_temps.push((temp.id, temp.span));
        }
    }

    fn visit_block(&mut self, block: &MirBlock) {
        if !self.blocks.insert(block.id) {
            self.duplicate_blocks.push((block.id, block.span));
        }
    }
}

fn signature_from_function(function: &MirFunction) -> MirFunctionSignature {
    MirFunctionSignature {
        params: function.params.iter().map(|param| param.ty).collect(),
        return_ty: function.return_ty,
        error_ty: function.error_ty,
    }
}

impl Validator<'_, '_> {
    fn validate_program(&mut self) {
        for (id, span) in DuplicateFunctionFinder::find(self.program) {
            self.error(span, format!("duplicate MIR function id f{}", id.0));
        }
        let functions_by_id = self
            .program
            .functions
            .iter()
            .map(|function| (function.id, function))
            .collect::<FxHashMap<_, _>>();

        for function in &self.program.functions {
            self.validate_function(function, functions_by_id.clone());
        }
    }

    fn validate_function(&mut self, function: &MirFunction, functions_by_id: FxHashMap<MirFunctionId, &MirFunction>) {
        self.validate_mir_type(function.return_ty, function.span);
        if let Some(error_ty) = function.error_ty {
            self.validate_mir_type(error_ty, function.span);
        }
        for param in &function.params {
            self.validate_mir_type(param.ty, param.span);
        }
        for local in &function.locals {
            self.validate_mir_type(local.ty, local.span);
        }
        for temp in &function.temps {
            self.validate_mir_type(temp.ty, temp.span);
        }
        let shape = FunctionShape::collect(function);
        for (id, span) in &shape.duplicate_param_locals {
            self.error(*span, format!("duplicate MIR local id _{}", id.0));
        }
        for (id, span) in &shape.conflicting_locals {
            self.error(*span, format!("duplicate MIR local id _{} has conflicting type", id.0));
        }
        for (id, span) in &shape.duplicate_temps {
            self.error(*span, format!("duplicate MIR temp id %{}", id.0));
        }
        for (id, span) in &shape.duplicate_blocks {
            self.error(*span, format!("duplicate MIR block id bb{}", id.0));
        }
        let mut scope = FunctionScope {
            function,
            functions_by_id,
            locals: shape.locals,
            temps: shape.temps,
            blocks: shape.blocks,
            values: FxHashMap::default(),
        };

        for block in &function.blocks {
            self.validate_block(&mut scope, block);
        }
    }

    fn validate_block(&mut self, scope: &mut FunctionScope<'_>, block: &MirBlock) {
        scope.values.clear();
        for stmt in &block.statements {
            self.validate_stmt(scope, stmt);
        }
        self.validate_terminator(scope, &block.terminator);
    }

    fn validate_stmt(&mut self, scope: &mut FunctionScope<'_>, stmt: &MirStmt) {
        match &stmt.kind {
            MirStmtKind::Assign { place, value } => {
                let place_ty = self.validate_place(scope, place, stmt.span);
                self.validate_value(scope, value);
                if let Some(place_ty) = place_ty {
                    self.require_assignable(
                        value.ty,
                        place_ty,
                        value.span,
                        "assigned value type does not match destination",
                    );
                }
                scope.values.insert(value.id, value.ty);
            }
            MirStmtKind::Call { destination, callee, args } => {
                let sig = self.validate_callee(scope, callee, stmt.span);
                if let Some(sig) = &sig {
                    if sig.error_ty.is_some() {
                        self.error(stmt.span, "ordinary call callee is failable; use try_call");
                    }
                }
                self.validate_call_args(scope, sig.as_ref(), args, stmt.span);
                self.validate_optional_destination(
                    scope,
                    destination.as_ref(),
                    sig.as_ref().map(|sig| sig.return_ty),
                    stmt.span,
                );
            }
            MirStmtKind::RuntimeCall { destination, call } => {
                self.validate_runtime_call(scope, destination.as_ref(), call, stmt.span);
            }
            MirStmtKind::Construct { destination, aggregate } => {
                let destination_ty = self.validate_place(scope, destination, stmt.span);
                self.validate_aggregate(scope, aggregate, stmt.span);
                if let Some(destination_ty) = destination_ty {
                    self.require_assignable(
                        aggregate.ty,
                        destination_ty,
                        stmt.span,
                        "construct destination type does not match aggregate type",
                    );
                }
            }
        }
    }

    fn validate_terminator(&mut self, scope: &mut FunctionScope<'_>, terminator: &MirTerminator) {
        for target in terminator_successors(&terminator.kind) {
            self.require_block(scope, target, terminator.span);
        }

        match &terminator.kind {
            MirTerminatorKind::Return(None) => {
                if !self.is_vacuum(scope.function.return_ty) {
                    self.error(terminator.span, "no-value return in non-vacuum function");
                }
            }
            MirTerminatorKind::Return(Some(value)) => {
                if let Some(value_ty) = self.validate_operand(scope, value, terminator.span) {
                    self.require_assignable(
                        value_ty,
                        scope.function.return_ty,
                        terminator.span,
                        "return type mismatch",
                    );
                }
            }
            MirTerminatorKind::ReturnError(value) => {
                let Some(error_ty) = scope.function.error_ty else {
                    self.error(terminator.span, "return_error in function without alternate-exit type");
                    self.validate_operand(scope, value, terminator.span);
                    return;
                };
                if let Some(value_ty) = self.validate_operand(scope, value, terminator.span) {
                    self.require_assignable(value_ty, error_ty, terminator.span, "return_error type mismatch");
                }
            }
            MirTerminatorKind::TryCall { destination, callee, args, error_place, .. } => {
                let sig = self.validate_callee(scope, callee, terminator.span);
                self.validate_call_args(scope, sig.as_ref(), args, terminator.span);
                self.validate_optional_destination(
                    scope,
                    destination.as_ref(),
                    sig.as_ref().map(|sig| sig.return_ty),
                    terminator.span,
                );
                let error_place_ty = self.validate_place(scope, error_place, terminator.span);
                if sig.is_none() {
                    self.error(terminator.span, "try_call callee does not have a known failable signature");
                }
                match (sig.as_ref().and_then(|sig| sig.error_ty), error_place_ty) {
                    (Some(error_ty), Some(place_ty)) => {
                        self.require_assignable(
                            error_ty,
                            place_ty,
                            terminator.span,
                            "try_call error place type mismatch",
                        );
                    }
                    (None, _) if sig.is_some() => {
                        self.error(terminator.span, "try_call callee is not known failable");
                    }
                    _ => {}
                }
            }
            MirTerminatorKind::Goto(_) => {}
            MirTerminatorKind::Branch { condition, .. } => {
                if let Some(condition_ty) = self.validate_operand(scope, condition, terminator.span) {
                    self.require_exact(
                        condition_ty,
                        self.primitive(Primitive::Bivalens),
                        terminator.span,
                        "branch condition is not bivalens",
                    );
                }
            }
            MirTerminatorKind::Switch { value, .. } => {
                self.validate_operand(scope, value, terminator.span);
            }
            MirTerminatorKind::Unreachable => {}
        }
    }

    fn validate_value(&mut self, scope: &FunctionScope<'_>, value: &MirValue) {
        self.validate_mir_type(value.ty, value.span);
        match &value.kind {
            MirValueKind::Operand(operand) => {
                self.validate_operand(scope, operand, value.span);
            }
            MirValueKind::Unary { op, operand } => {
                let operand_ty = self.validate_operand(scope, operand, value.span);
                match op {
                    MirUnOp::Not => {
                        if let Some(operand_ty) = operand_ty {
                            self.require_exact(
                                operand_ty,
                                self.primitive(Primitive::Bivalens),
                                value.span,
                                "not operand is not bivalens",
                            );
                        }
                        self.require_exact(
                            value.ty,
                            self.primitive(Primitive::Bivalens),
                            value.span,
                            "not result is not bivalens",
                        );
                    }
                    MirUnOp::IsNil | MirUnOp::IsNonNil => {
                        self.require_exact(
                            value.ty,
                            self.primitive(Primitive::Bivalens),
                            value.span,
                            "nil-test result is not bivalens",
                        );
                    }
                    MirUnOp::Neg | MirUnOp::BitNot => {}
                }
            }
            MirValueKind::Binary { lhs, rhs, .. } => {
                self.validate_operand(scope, lhs, value.span);
                self.validate_operand(scope, rhs, value.span);
            }
            MirValueKind::Option(op) => self.validate_option_op(scope, op, value.ty, value.span),
        }
    }

    fn validate_option_op(&mut self, scope: &FunctionScope<'_>, op: &MirOptionOp, result_ty: MirType, span: Span) {
        match op {
            MirOptionOp::None => {
                if !matches!(self.type_kind(result_ty), Type::Option(_)) {
                    self.error(span, "option none result is not nullable");
                }
            }
            MirOptionOp::Some(value) => {
                let value_ty = self.validate_operand(scope, value, span);
                match self.type_kind(result_ty) {
                    Type::Option(inner) => {
                        if let Some(value_ty) = value_ty {
                            self.require_assignable(
                                value_ty,
                                MirType::semantic(*inner),
                                span,
                                "option some payload type mismatch",
                            );
                        }
                    }
                    _ => self.error(span, "option some result is not nullable"),
                }
            }
            MirOptionOp::IsNil(value) | MirOptionOp::IsNonNil(value) => {
                self.validate_operand(scope, value, span);
                self.require_exact(
                    result_ty,
                    self.primitive(Primitive::Bivalens),
                    span,
                    "option test result is not bivalens",
                );
            }
            MirOptionOp::Unwrap { value, .. } => {
                if let Some(value_ty) = self.validate_operand(scope, value, span) {
                    match self.type_kind(value_ty) {
                        Type::Option(inner) => {
                            self.require_assignable(
                                MirType::semantic(*inner),
                                result_ty,
                                span,
                                "option unwrap result type mismatch",
                            );
                        }
                        _ => self.error(span, "option unwrap operand is not nullable"),
                    }
                }
            }
            MirOptionOp::Coalesce { value, fallback } => {
                let value_ty = self.validate_operand(scope, value, span);
                let fallback_ty = self.validate_operand(scope, fallback, span);
                if let Some(value_ty) = value_ty {
                    match self.type_kind(value_ty) {
                        Type::Option(inner) => {
                            self.require_assignable(
                                MirType::semantic(*inner),
                                result_ty,
                                span,
                                "option coalesce value type mismatch",
                            );
                        }
                        _ => self.error(span, "option coalesce value is not nullable"),
                    }
                }
                if let Some(fallback_ty) = fallback_ty {
                    self.require_assignable(fallback_ty, result_ty, span, "option coalesce fallback type mismatch");
                }
            }
            MirOptionOp::Chain { base, link } => {
                if let Some(base_ty) = self.validate_operand(scope, base, span) {
                    if !matches!(self.type_kind(base_ty), Type::Option(_)) {
                        self.error(span, "optional chain base is not nullable");
                    }
                }
                self.validate_option_chain_link(scope, link, span);
                if !matches!(self.type_kind(result_ty), Type::Option(_)) {
                    self.error(span, "optional chain result is not nullable");
                }
            }
        }
    }

    fn validate_option_chain_link(&mut self, scope: &FunctionScope<'_>, link: &MirOptionChainLink, span: Span) {
        match link {
            MirOptionChainLink::Field(_) | MirOptionChainLink::VariantField { .. } => {}
            MirOptionChainLink::Index(index) => {
                self.validate_operand(scope, index, span);
            }
            MirOptionChainLink::Call { callee, args } => {
                let sig = self.validate_callee(scope, callee, span);
                self.validate_call_args(scope, sig.as_ref(), args, span);
            }
        }
    }

    fn validate_runtime_call(
        &mut self,
        scope: &FunctionScope<'_>,
        destination: Option<&MirPlace>,
        call: &MirRuntimeCall,
        span: Span,
    ) {
        self.validate_mir_type(call.return_ty, span);
        for arg in &call.args {
            self.validate_operand(scope, arg, span);
        }
        self.validate_intrinsic(scope, call, span);
        self.validate_optional_destination(scope, destination, Some(call.return_ty), span);
    }

    fn validate_intrinsic(&mut self, scope: &FunctionScope<'_>, call: &MirRuntimeCall, span: Span) {
        match &call.intrinsic {
            MirIntrinsic::Diagnostic(_) => {
                self.require_arg_count(call.args.len(), 1, span, "diagnostic runtime call");
                self.require_exact(
                    call.return_ty,
                    self.primitive(Primitive::Vacuum),
                    span,
                    "diagnostic runtime call does not return vacuum",
                );
            }
            MirIntrinsic::FormatString { .. } => {
                self.require_exact(
                    call.return_ty,
                    self.primitive(Primitive::Textus),
                    span,
                    "format_string runtime call does not return textus",
                );
            }
            MirIntrinsic::Convert(conversion) => {
                self.require_arg_count(call.args.len(), 1, span, "conversion runtime call");
                self.validate_mir_type(conversion.target_ty, span);
                self.require_assignable(conversion.target_ty, call.return_ty, span, "conversion return type mismatch");
                if let Some(fallback) = &conversion.fallback {
                    if let Some(fallback_ty) = self.validate_operand(scope, fallback, span) {
                        self.require_assignable(
                            fallback_ty,
                            conversion.target_ty,
                            span,
                            "conversion fallback type mismatch",
                        );
                    }
                }
            }
            MirIntrinsic::Collection(op) => self.validate_collection_call(scope, *op, call, span),
            MirIntrinsic::Panic => {
                self.require_arg_count(call.args.len(), 1, span, "panic runtime call");
                self.require_exact(
                    call.return_ty,
                    self.primitive(Primitive::Numquam),
                    span,
                    "panic runtime call does not return numquam",
                );
            }
            MirIntrinsic::Provider(provider) => {
                if provider.module.is_empty() {
                    self.error(span, "provider runtime call has empty provider module identity");
                }
            }
        }
    }

    fn validate_collection_call(
        &mut self,
        scope: &FunctionScope<'_>,
        op: MirCollectionOp,
        call: &MirRuntimeCall,
        span: Span,
    ) {
        let expected_args = match op {
            MirCollectionOp::Length => 1,
            MirCollectionOp::Append
            | MirCollectionOp::AppendImmutable
            | MirCollectionOp::Index
            | MirCollectionOp::Contains => 2,
        };
        if call.args.len() != expected_args {
            self.error(span, format!("collection {:?} expects {expected_args} MIR arguments", op));
        }
        let first_arg_ty = call
            .args
            .first()
            .and_then(|arg| self.validate_operand(scope, arg, span));
        match op {
            MirCollectionOp::Length => {
                self.require_exact(
                    call.return_ty,
                    self.primitive(Primitive::Numerus),
                    span,
                    "collection length result is not numerus",
                );
            }
            MirCollectionOp::Contains => {
                self.require_exact(
                    call.return_ty,
                    self.primitive(Primitive::Bivalens),
                    span,
                    "collection contains result is not bivalens",
                );
            }
            MirCollectionOp::Append | MirCollectionOp::AppendImmutable => {
                if let (Some(collection_ty), Some(value)) = (first_arg_ty, call.args.get(1)) {
                    let value_ty = self.validate_operand(scope, value, span);
                    if let (Some(element_ty), Some(value_ty)) = (self.collection_element_ty(collection_ty), value_ty) {
                        self.require_assignable(value_ty, element_ty, span, "collection append value type mismatch");
                    }
                }
            }
            MirCollectionOp::Index => {
                if let (Some(collection_ty), Some(index)) = (first_arg_ty, call.args.get(1)) {
                    let index_ty = self.validate_operand(scope, index, span);
                    let result_ty = self.project_index(collection_ty, index_ty, span);
                    if let Some(result_ty) = result_ty {
                        self.require_assignable(
                            result_ty,
                            call.return_ty,
                            span,
                            "collection index result type mismatch",
                        );
                    }
                }
            }
        }
    }

    fn require_arg_count(&mut self, actual: usize, expected: usize, span: Span, label: &'static str) {
        if actual != expected {
            self.error(span, format!("{label} expects {expected} MIR arguments"));
        }
    }

    fn collection_element_ty(&mut self, collection_ty: MirType) -> Option<MirType> {
        match self.type_kind(collection_ty) {
            Type::Array(inner) | Type::Set(inner) => Some(MirType::semantic(*inner)),
            _ => None,
        }
    }

    fn validate_aggregate(&mut self, scope: &FunctionScope<'_>, aggregate: &MirAggregate, span: Span) {
        self.validate_mir_type(aggregate.ty, span);
        match (&aggregate.kind, &aggregate.fields) {
            (MirAggregateKind::Tuple, MirAggregateFields::Ordered(items)) => {
                for item in items {
                    match item {
                        MirAggregateItem::Operand(value) | MirAggregateItem::Spread(value) => {
                            self.validate_operand(scope, value, span);
                        }
                    }
                }
            }
            (MirAggregateKind::Array | MirAggregateKind::Set, MirAggregateFields::Ordered(items)) => {
                let element_ty = match self.type_kind(aggregate.ty) {
                    Type::Array(inner) | Type::Set(inner) => Some(MirType::semantic(*inner)),
                    _ => {
                        self.error(span, "ordered collection aggregate type is not array or set");
                        None
                    }
                };
                for item in items {
                    match item {
                        MirAggregateItem::Operand(value) => {
                            let value_ty = self.validate_operand(scope, value, span);
                            if let (Some(value_ty), Some(element_ty)) = (value_ty, element_ty) {
                                self.require_assignable(value_ty, element_ty, span, "aggregate element type mismatch");
                            }
                        }
                        MirAggregateItem::Spread(value) => {
                            let value_ty = self.validate_operand(scope, value, span);
                            if let Some(value_ty) = value_ty {
                                self.require_assignable(value_ty, aggregate.ty, span, "aggregate spread type mismatch");
                            }
                        }
                    }
                }
            }
            (MirAggregateKind::Map, MirAggregateFields::Keyed(items)) => {
                let (key_ty, value_ty) = match self.type_kind(aggregate.ty) {
                    Type::Map(key, value) => (Some(MirType::semantic(*key)), Some(MirType::semantic(*value))),
                    _ => {
                        self.error(span, "map aggregate type is not map");
                        (None, None)
                    }
                };
                for item in items {
                    let actual_key_ty = self.validate_operand(scope, &item.key, span);
                    let actual_value_ty = self.validate_operand(scope, &item.value, span);
                    if let (Some(actual_key_ty), Some(key_ty)) = (actual_key_ty, key_ty) {
                        self.require_assignable(actual_key_ty, key_ty, span, "map aggregate key type mismatch");
                    }
                    if let (Some(actual_value_ty), Some(value_ty)) = (actual_value_ty, value_ty) {
                        self.require_assignable(actual_value_ty, value_ty, span, "map aggregate value type mismatch");
                    }
                }
            }
            (MirAggregateKind::Struct(def_id), MirAggregateFields::Named(items)) => {
                match self.type_kind(aggregate.ty) {
                    Type::Struct(type_def) if type_def == def_id => {}
                    Type::Struct(_) => self.error(span, "struct aggregate DefId does not match aggregate type"),
                    _ => self.error(span, "struct aggregate type is not struct"),
                }
                self.validate_named_aggregate_fields(scope, *def_id, items, true, span);
            }
            (MirAggregateKind::EnumVariant(def_id), MirAggregateFields::Named(items)) => {
                if !matches!(self.type_kind(aggregate.ty), Type::Enum(_)) {
                    self.error(span, "enum variant aggregate type is not enum");
                }
                self.validate_named_aggregate_fields(scope, *def_id, items, false, span);
            }
            (MirAggregateKind::EnumVariant(_), MirAggregateFields::Ordered(items)) => {
                for item in items {
                    match item {
                        MirAggregateItem::Operand(value) | MirAggregateItem::Spread(value) => {
                            self.validate_operand(scope, value, span);
                        }
                    }
                }
            }
            _ => self.error(span, "aggregate payload shape does not match aggregate kind"),
        }
    }

    fn validate_named_aggregate_fields(
        &mut self,
        scope: &FunctionScope<'_>,
        def_id: DefId,
        items: &[MirNamedOperand],
        is_struct: bool,
        span: Span,
    ) {
        let fields = if is_struct {
            self.context.struct_fields.get(&def_id)
        } else {
            self.context.variant_fields.get(&def_id)
        };

        let mut seen = FxHashSet::default();
        for item in items {
            let value_ty = self.validate_operand(scope, &item.value, span);
            if !seen.insert(item.name) {
                self.error(span, "named aggregate field is duplicated");
            }

            let Some(fields) = fields else {
                continue;
            };
            let Some(expected_ty) = fields.get(&item.name).copied() else {
                self.error(span, "named aggregate references unknown field");
                continue;
            };
            if let Some(value_ty) = value_ty {
                self.require_assignable(value_ty, expected_ty, span, "named aggregate field type mismatch");
            }
        }

        let Some(fields) = fields else {
            self.error(span, "named aggregate is missing field metadata");
            return;
        };
        for field in fields.keys() {
            if !seen.contains(field) {
                self.error(span, "named aggregate is missing required field");
            }
        }
    }

    fn validate_optional_destination(
        &mut self,
        scope: &FunctionScope<'_>,
        destination: Option<&MirPlace>,
        expected_ty: Option<MirType>,
        span: Span,
    ) {
        match (destination, expected_ty) {
            (Some(destination), Some(expected_ty)) => {
                if let Some(destination_ty) = self.validate_place(scope, destination, span) {
                    self.require_assignable(expected_ty, destination_ty, span, "destination type mismatch");
                }
            }
            (Some(destination), None) => {
                self.validate_place(scope, destination, span);
            }
            (None, Some(expected_ty)) => {
                if !self.is_vacuum(expected_ty) && !self.is_numquam(expected_ty) {
                    self.error(span, "non-vacuum result has no destination");
                }
            }
            (None, None) => {}
        }
    }

    fn validate_callee(
        &mut self,
        scope: &FunctionScope<'_>,
        callee: &MirCallee,
        span: Span,
    ) -> Option<MirFunctionSignature> {
        match callee {
            MirCallee::Function(id) => {
                let Some(function) = scope.functions_by_id.get(id).copied() else {
                    self.error(span, format!("callee function f{} does not exist", id.0));
                    return None;
                };
                Some(signature_from_function(function))
            }
            MirCallee::Definition(def_id) => self
                .context
                .functions
                .get(def_id)
                .cloned()
                .or_else(|| self.function_signature_from_program_def(scope, *def_id)),
            MirCallee::Value(value) => {
                self.validate_operand(scope, value, span);
                None
            }
        }
    }

    fn function_signature_from_program_def(
        &self,
        scope: &FunctionScope<'_>,
        def_id: DefId,
    ) -> Option<MirFunctionSignature> {
        scope
            .functions_by_id
            .values()
            .find(|function| function.source == Some(def_id))
            .map(|function| signature_from_function(function))
    }

    fn validate_call_args(
        &mut self,
        scope: &FunctionScope<'_>,
        sig: Option<&MirFunctionSignature>,
        args: &[MirOperand],
        span: Span,
    ) {
        let Some(sig) = sig else {
            for arg in args {
                self.validate_operand(scope, arg, span);
            }
            return;
        };

        if args.len() != sig.params.len() {
            self.error(
                span,
                format!(
                    "call argument count mismatch: expected {}, got {}",
                    sig.params.len(),
                    args.len()
                ),
            );
        }

        for (index, arg) in args.iter().enumerate() {
            let arg_ty = self.validate_operand(scope, arg, span);
            let Some(expected_ty) = sig.params.get(index).copied() else {
                continue;
            };
            if let Some(arg_ty) = arg_ty {
                self.require_assignable(arg_ty, expected_ty, span, "call argument type mismatch");
            }
        }
    }

    fn validate_operand(&mut self, scope: &FunctionScope<'_>, operand: &MirOperand, span: Span) -> Option<MirType> {
        match operand {
            MirOperand::Place(place) => self.validate_place(scope, place, span),
            MirOperand::Temp(id) => match scope.temps.get(id).copied() {
                Some(ty) => Some(ty),
                None => {
                    self.error(span, format!("temp %{} does not exist", id.0));
                    None
                }
            },
            MirOperand::Value(id) => match scope.values.get(id).copied() {
                Some(ty) => Some(ty),
                None => {
                    self.error(span, format!("value v{} is not defined earlier in MIR", id.0));
                    None
                }
            },
            MirOperand::Constant(value) => Some(self.constant_ty(value)),
        }
    }

    fn validate_place(&mut self, scope: &FunctionScope<'_>, place: &MirPlace, span: Span) -> Option<MirType> {
        let mut ty = match place.base {
            MirPlaceBase::Local(id) => match scope.locals.get(&id).copied() {
                Some(ty) => ty,
                None => {
                    self.error(span, format!("local _{} does not exist", id.0));
                    return None;
                }
            },
            MirPlaceBase::Temp(id) => match scope.temps.get(&id).copied() {
                Some(ty) => ty,
                None => {
                    self.error(span, format!("temp %{} does not exist", id.0));
                    return None;
                }
            },
        };

        for projection in &place.projections {
            ty = match projection {
                MirProjection::Field(field) => self.project_field(ty, *field, span)?,
                MirProjection::VariantField { variant, field } => self.project_variant_field(*variant, *field, span)?,
                MirProjection::Index(index) => {
                    let index_ty = self.validate_operand(scope, index, span);
                    self.project_index(ty, index_ty, span)?
                }
            };
        }

        Some(ty)
    }

    fn project_field(&mut self, base_ty: MirType, field: Symbol, span: Span) -> Option<MirType> {
        match self.type_kind(base_ty) {
            Type::Struct(def_id) => self
                .context
                .struct_fields
                .get(def_id)
                .and_then(|fields| fields.get(&field))
                .copied()
                .or_else(|| {
                    self.error(span, "struct field projection is missing field metadata");
                    None
                }),
            Type::Record(fields) => fields
                .get(&field)
                .copied()
                .map(MirType::semantic)
                .or_else(|| {
                    self.error(span, "record field projection references unknown field");
                    None
                }),
            _ => {
                self.error(span, "field projection base is not a struct or record");
                None
            }
        }
    }

    fn project_variant_field(&mut self, variant: DefId, field: Symbol, span: Span) -> Option<MirType> {
        self.context
            .variant_fields
            .get(&variant)
            .and_then(|fields| fields.get(&field))
            .copied()
            .or_else(|| {
                self.error(span, "variant field projection is missing field metadata");
                None
            })
    }

    fn project_index(&mut self, base_ty: MirType, index_ty: Option<MirType>, span: Span) -> Option<MirType> {
        match self.type_kind(base_ty).clone() {
            Type::Array(inner) => {
                if let Some(index_ty) = index_ty {
                    self.require_assignable(
                        index_ty,
                        self.primitive(Primitive::Numerus),
                        span,
                        "array index is not numerus",
                    );
                }
                Some(MirType::semantic(inner))
            }
            Type::Map(key, value) => {
                if let Some(index_ty) = index_ty {
                    self.require_assignable(index_ty, MirType::semantic(key), span, "map index key type mismatch");
                }
                Some(MirType::semantic(value))
            }
            Type::Primitive(Primitive::Textus) => {
                if let Some(index_ty) = index_ty {
                    self.require_assignable(
                        index_ty,
                        self.primitive(Primitive::Numerus),
                        span,
                        "textus index is not numerus",
                    );
                }
                Some(self.primitive(Primitive::Textus))
            }
            _ => {
                self.error(span, "index projection base is not indexable");
                None
            }
        }
    }

    fn require_block(&mut self, scope: &FunctionScope<'_>, block: MirBlockId, span: Span) {
        if !scope.blocks.contains(&block) {
            self.error(span, format!("block bb{} does not exist", block.0));
        }
    }

    fn validate_mir_type(&mut self, ty: MirType, span: Span) {
        let mut seen = FxHashSet::default();
        self.validate_type_id(ty.semantic_id(), span, &mut seen);
    }

    fn validate_type_id(
        &mut self,
        ty: crate::semantic::TypeId,
        span: Span,
        seen: &mut FxHashSet<crate::semantic::TypeId>,
    ) {
        if !seen.insert(ty) {
            return;
        }

        match self.context.types.get(ty).clone() {
            Type::Infer(_) => self.error(span, "MIR type is unresolved inference variable"),
            Type::Error => self.error(span, "MIR type is semantic error recovery type"),
            Type::Array(inner)
            | Type::Set(inner)
            | Type::Option(inner)
            | Type::Ref(_, inner)
            | Type::Alias(_, inner) => self.validate_type_id(inner, span, seen),
            Type::Map(key, value) => {
                self.validate_type_id(key, span, seen);
                self.validate_type_id(value, span, seen);
            }
            Type::Record(fields) => {
                for field_ty in fields.values() {
                    self.validate_type_id(*field_ty, span, seen);
                }
            }
            Type::Func(sig) => {
                for param in &sig.params {
                    self.validate_type_id(param.ty, span, seen);
                }
                self.validate_type_id(sig.ret, span, seen);
                if let Some(error_ty) = sig.err {
                    self.validate_type_id(error_ty, span, seen);
                }
            }
            Type::Applied(base, args) => {
                self.validate_type_id(base, span, seen);
                for arg in args {
                    self.validate_type_id(arg, span, seen);
                }
            }
            Type::Union(members) => {
                for member in members {
                    self.validate_type_id(member, span, seen);
                }
            }
            Type::Primitive(_) | Type::Struct(_) | Type::Enum(_) | Type::Interface(_) | Type::Param(_) => {}
        }
    }

    fn require_assignable(&mut self, from: MirType, to: MirType, span: Span, message: &'static str) {
        if !self
            .context
            .types
            .assignable(from.semantic_id(), to.semantic_id())
        {
            self.error(span, message);
        }
    }

    fn require_exact(&mut self, found: MirType, expected: MirType, span: Span, message: &'static str) {
        if found.semantic_id() != expected.semantic_id() {
            self.error(span, message);
        }
    }

    fn constant_ty(&self, value: &MirConstant) -> MirType {
        match value {
            MirConstant::Int(_) => self.primitive(Primitive::Numerus),
            MirConstant::Float(_) => self.primitive(Primitive::Fractus),
            MirConstant::String(_) => self.primitive(Primitive::Textus),
            MirConstant::Bool(_) => self.primitive(Primitive::Bivalens),
            MirConstant::Nil => self.primitive(Primitive::Nihil),
            MirConstant::Unit => self.primitive(Primitive::Vacuum),
        }
    }

    fn primitive(&self, primitive: Primitive) -> MirType {
        MirType::semantic(self.context.types.primitive(primitive))
    }

    fn is_vacuum(&self, ty: MirType) -> bool {
        ty.semantic_id() == self.context.types.primitive(Primitive::Vacuum)
    }

    fn is_numquam(&self, ty: MirType) -> bool {
        ty.semantic_id() == self.context.types.primitive(Primitive::Numquam)
    }

    fn type_kind(&self, ty: MirType) -> &Type {
        self.context.types.get(ty.semantic_id())
    }

    fn error(&mut self, span: Span, message: impl Into<String>) {
        self.errors.push(MirValidationError::new(span, message));
    }
}

#[cfg(test)]
#[path = "validate_test.rs"]
mod tests;
