use crate::hir::DefId;
use crate::lexer::{Span, Symbol};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MirFunctionSignature {
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

impl Validator<'_, '_> {
    fn validate_program(&mut self) {
        let mut function_ids = FxHashSet::default();
        let mut functions_by_id = FxHashMap::default();
        for function in &self.program.functions {
            if !function_ids.insert(function.id) {
                self.error(function.span, format!("duplicate MIR function id f{}", function.id.0));
            }
            functions_by_id.insert(function.id, function);
        }

        for function in &self.program.functions {
            self.validate_function(function, functions_by_id.clone());
        }
    }

    fn validate_function(&mut self, function: &MirFunction, functions_by_id: FxHashMap<MirFunctionId, &MirFunction>) {
        self.validate_mir_type(function.return_ty, function.span);
        if let Some(error_ty) = function.error_ty {
            self.validate_mir_type(error_ty, function.span);
        }
        let locals = self.collect_locals(function);
        let temps = self.collect_temps(function);
        let blocks = self.collect_blocks(function);
        let mut scope =
            FunctionScope { function, functions_by_id, locals, temps, blocks, values: FxHashMap::default() };

        for block in &function.blocks {
            self.validate_block(&mut scope, block);
        }
    }

    fn collect_locals(&mut self, function: &MirFunction) -> FxHashMap<MirLocalId, MirType> {
        let mut locals = FxHashMap::default();
        for param in &function.params {
            self.validate_mir_type(param.ty, param.span);
            if locals.insert(param.local, param.ty).is_some() {
                self.error(param.span, format!("duplicate MIR local id _{}", param.local.0));
            }
        }
        for local in &function.locals {
            self.validate_mir_type(local.ty, local.span);
            match locals.insert(local.id, local.ty) {
                Some(existing) if existing.semantic_id() != local.ty.semantic_id() => {
                    self.error(
                        local.span,
                        format!("duplicate MIR local id _{} has conflicting type", local.id.0),
                    );
                }
                _ => {}
            }
        }
        locals
    }

    fn collect_temps(&mut self, function: &MirFunction) -> FxHashMap<MirTempId, MirType> {
        let mut temps = FxHashMap::default();
        for temp in &function.temps {
            self.validate_mir_type(temp.ty, temp.span);
            if temps.insert(temp.id, temp.ty).is_some() {
                self.error(temp.span, format!("duplicate MIR temp id %{}", temp.id.0));
            }
        }
        temps
    }

    fn collect_blocks(&mut self, function: &MirFunction) -> FxHashSet<MirBlockId> {
        let mut blocks = FxHashSet::default();
        for block in &function.blocks {
            if !blocks.insert(block.id) {
                self.error(block.span, format!("duplicate MIR block id bb{}", block.id.0));
            }
        }
        blocks
    }

    fn validate_block(&mut self, scope: &mut FunctionScope<'_>, block: &MirBlock) {
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
                for arg in args {
                    self.validate_operand(scope, arg, stmt.span);
                }
                self.validate_optional_destination(
                    scope,
                    destination.as_ref(),
                    sig.map(|sig| sig.return_ty),
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
            MirTerminatorKind::TryCall { destination, callee, args, ok_block, error_place, error_block } => {
                self.require_block(scope, *ok_block, terminator.span);
                self.require_block(scope, *error_block, terminator.span);
                let sig = self.validate_callee(scope, callee, terminator.span);
                for arg in args {
                    self.validate_operand(scope, arg, terminator.span);
                }
                self.validate_optional_destination(
                    scope,
                    destination.as_ref(),
                    sig.map(|sig| sig.return_ty),
                    terminator.span,
                );
                let error_place_ty = self.validate_place(scope, error_place, terminator.span);
                match (sig.and_then(|sig| sig.error_ty), error_place_ty) {
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
            MirTerminatorKind::Goto(target) => self.require_block(scope, *target, terminator.span),
            MirTerminatorKind::Branch { condition, then_block, else_block } => {
                self.require_block(scope, *then_block, terminator.span);
                self.require_block(scope, *else_block, terminator.span);
                if let Some(condition_ty) = self.validate_operand(scope, condition, terminator.span) {
                    self.require_exact(
                        condition_ty,
                        self.primitive(Primitive::Bivalens),
                        terminator.span,
                        "branch condition is not bivalens",
                    );
                }
            }
            MirTerminatorKind::Switch { value, cases, default } => {
                self.validate_operand(scope, value, terminator.span);
                self.require_block(scope, *default, terminator.span);
                for case in cases {
                    self.require_block(scope, case.target, terminator.span);
                }
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
                self.validate_callee(scope, callee, span);
                for arg in args {
                    self.validate_operand(scope, arg, span);
                }
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
            MirIntrinsic::Collection(op) => self.validate_collection_call(*op, call, span),
            MirIntrinsic::Panic => {
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

    fn validate_collection_call(&mut self, op: MirCollectionOp, call: &MirRuntimeCall, span: Span) {
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
            MirCollectionOp::Append | MirCollectionOp::AppendImmutable | MirCollectionOp::Index => {}
        }
    }

    fn validate_aggregate(&mut self, scope: &FunctionScope<'_>, aggregate: &MirAggregate, span: Span) {
        self.validate_mir_type(aggregate.ty, span);
        match (&aggregate.kind, &aggregate.fields) {
            (
                MirAggregateKind::Tuple | MirAggregateKind::Array | MirAggregateKind::Set,
                MirAggregateFields::Ordered(items),
            ) => {
                for item in items {
                    match item {
                        MirAggregateItem::Operand(value) | MirAggregateItem::Spread(value) => {
                            self.validate_operand(scope, value, span);
                        }
                    }
                }
            }
            (MirAggregateKind::Map, MirAggregateFields::Keyed(items)) => {
                for item in items {
                    self.validate_operand(scope, &item.key, span);
                    self.validate_operand(scope, &item.value, span);
                }
            }
            (MirAggregateKind::Struct(_), MirAggregateFields::Named(items))
            | (MirAggregateKind::EnumVariant(_), MirAggregateFields::Named(items)) => {
                for item in items {
                    self.validate_operand(scope, &item.value, span);
                }
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
                Some(MirFunctionSignature { return_ty: function.return_ty, error_ty: function.error_ty })
            }
            MirCallee::Definition(def_id) => self
                .context
                .functions
                .get(def_id)
                .copied()
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
            .map(|function| MirFunctionSignature { return_ty: function.return_ty, error_ty: function.error_ty })
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
