//! Pass 3: Type checking
//!
//! Bidirectional type inference and checking.

use crate::hir::{
    DefId, HirBinOp, HirBlock, HirCasuArm, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind,
    HirLiteral, HirLocal, HirParam, HirParamMode, HirPattern, HirProgram, HirStmt, HirStmtKind,
    HirStruct,
};
use crate::lexer::Symbol;
use crate::semantic::{
    FuncSig, ParamMode, ParamType, Primitive, Resolver, SemanticError, SemanticErrorKind, Type,
    TypeId, TypeTable,
};
use rustc_hash::FxHashMap;

#[derive(Clone, Copy)]
struct BindingInfo {
    ty: TypeId,
    mutable: bool,
}

struct StructInfo {
    fields: FxHashMap<Symbol, TypeId>,
    methods: FxHashMap<Symbol, FuncSig>,
}

struct TypeChecker<'a> {
    resolver: &'a Resolver,
    types: &'a mut TypeTable,
    errors: Vec<SemanticError>,
    scopes: Vec<FxHashMap<DefId, BindingInfo>>,
    functions: FxHashMap<DefId, FuncSig>,
    consts: FxHashMap<DefId, TypeId>,
    structs: FxHashMap<DefId, StructInfo>,
    variant_fields: FxHashMap<DefId, Vec<TypeId>>,
    variant_parent: FxHashMap<DefId, DefId>,
    current_return: Option<TypeId>,
    inferred_return: Option<TypeId>,
    error_type: TypeId,
}

/// Type check the HIR
pub fn typecheck(
    hir: &mut HirProgram,
    resolver: &Resolver,
    types: &mut TypeTable,
) -> Result<(), Vec<SemanticError>> {
    let mut checker = TypeChecker::new(resolver, types);
    checker.collect_items(hir);
    checker.check_program(hir);

    if checker.errors.is_empty() {
        Ok(())
    } else {
        Err(checker.errors)
    }
}

impl<'a> TypeChecker<'a> {
    fn new(resolver: &'a Resolver, types: &'a mut TypeTable) -> Self {
        let error_type = types.intern(Type::Error);
        Self {
            resolver,
            types,
            errors: Vec::new(),
            scopes: Vec::new(),
            functions: FxHashMap::default(),
            consts: FxHashMap::default(),
            structs: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            variant_parent: FxHashMap::default(),
            current_return: None,
            inferred_return: None,
            error_type,
        }
    }

    fn collect_items(&mut self, hir: &HirProgram) {
        for item in &hir.items {
            match &item.kind {
                HirItemKind::Function(func) => {
                    let sig = self.function_signature(func);
                    self.functions.insert(item.def_id, sig);
                }
                HirItemKind::Struct(struct_item) => self.collect_struct(item.def_id, struct_item),
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

    fn collect_struct(&mut self, def_id: DefId, struct_item: &HirStruct) {
        let mut fields = FxHashMap::default();
        let mut methods = FxHashMap::default();

        for field in &struct_item.fields {
            fields.insert(field.name, field.ty);
        }

        for method in &struct_item.methods {
            let sig = self.function_signature(&method.func);
            methods.insert(method.func.name, sig);
        }

        self.structs.insert(def_id, StructInfo { fields, methods });
    }

    fn function_signature(&mut self, func: &HirFunction) -> FuncSig {
        let params = func
            .params
            .iter()
            .map(|param| ParamType {
                ty: param.ty,
                mode: param_mode_from_hir(param.mode),
                optional: false,
            })
            .collect();
        let ret = func.ret_ty.unwrap_or(self.unknown_type());
        FuncSig {
            params,
            ret,
            is_async: func.is_async,
            is_generator: func.is_generator,
        }
    }

    fn check_program(&mut self, hir: &mut HirProgram) {
        for item in &mut hir.items {
            self.check_item(item);
        }

        if let Some(entry) = &mut hir.entry {
            self.check_block(entry);
        }
    }

    fn check_item(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(func) => self.check_function(item.def_id, func),
            HirItemKind::Const(const_item) => self.check_const(item.def_id, const_item),
            HirItemKind::Struct(struct_item) => {
                for method in &mut struct_item.methods {
                    self.check_function(method.def_id, &mut method.func);
                }
            }
            _ => {}
        }
    }

    fn check_const(&mut self, def_id: DefId, const_item: &mut crate::hir::HirConst) {
        let value_ty = self.check_expr(&mut const_item.value);

        let ty = if let Some(annotated) = const_item.ty {
            if !self.types.assignable(value_ty, annotated) {
                self.error(
                    SemanticErrorKind::TypeMismatch,
                    "constant value does not match annotation",
                    const_item.value.span,
                );
            }
            annotated
        } else {
            value_ty
        };

        const_item.ty = Some(ty);
        self.consts.insert(def_id, ty);
    }

    fn check_function(&mut self, def_id: DefId, func: &mut HirFunction) {
        self.push_scope();
        for param in &func.params {
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            self.insert_binding(param.def_id, param.ty, mutable);
        }

        let prev_return = self.current_return;
        let prev_inferred = self.inferred_return;
        self.current_return = func.ret_ty;
        self.inferred_return = None;

        if let Some(body) = &mut func.body {
            self.check_block(body);
        }

        let inferred = self.inferred_return.take();
        if func.ret_ty.is_none() {
            let ret_ty = inferred.unwrap_or(self.vacuum_type());
            func.ret_ty = Some(ret_ty);
            if let Some(sig) = self.functions.get_mut(&def_id) {
                sig.ret = ret_ty;
            }
        }

        self.current_return = prev_return;
        self.inferred_return = prev_inferred;
        self.pop_scope();
    }

    fn check_block(&mut self, block: &mut HirBlock) -> TypeId {
        self.push_scope();
        for stmt in &mut block.stmts {
            self.check_stmt(stmt);
        }
        let ty = if let Some(expr) = &mut block.expr {
            self.check_expr(expr)
        } else {
            self.vacuum_type()
        };
        self.pop_scope();
        ty
    }

    fn check_stmt(&mut self, stmt: &mut HirStmt) {
        match &mut stmt.kind {
            HirStmtKind::Local(local) => self.check_local(local),
            HirStmtKind::Expr(expr) => {
                self.check_expr(expr);
            }
            HirStmtKind::Redde(value) => self.check_return(value.as_mut(), stmt.span),
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    fn check_local(&mut self, local: &mut HirLocal) {
        let inferred = match (&local.ty, &mut local.init) {
            (Some(ty), Some(init)) => {
                let init_ty = self.check_expr(init);
                if !self.types.assignable(init_ty, *ty) {
                    self.error(
                        SemanticErrorKind::TypeMismatch,
                        "initializer does not match annotation",
                        init.span,
                    );
                }
                *ty
            }
            (Some(ty), None) => *ty,
            (None, Some(init)) => self.check_expr(init),
            (None, None) => {
                self.error(
                    SemanticErrorKind::MissingTypeAnnotation,
                    "variable declaration needs a type or initializer",
                    local
                        .init
                        .as_ref()
                        .map(|expr| expr.span)
                        .unwrap_or_default(),
                );
                self.unknown_type()
            }
        };

        if local.ty.is_none() {
            local.ty = Some(inferred);
        }

        self.insert_binding(local.def_id, inferred, local.mutable);
    }

    fn check_return(&mut self, value: Option<&mut HirExpr>, span: crate::lexer::Span) {
        let value_ty = match value {
            Some(expr) => self.check_expr(expr),
            None => self.vacuum_type(),
        };

        if let Some(expected) = self.current_return {
            if !self.types.assignable(value_ty, expected) {
                self.error(
                    SemanticErrorKind::TypeMismatch,
                    "return type does not match function signature",
                    span,
                );
            }
            return;
        }

        match self.inferred_return {
            None => self.inferred_return = Some(value_ty),
            Some(existing) => {
                if !self.types.assignable(value_ty, existing)
                    && !self.types.assignable(existing, value_ty)
                {
                    self.error(
                        SemanticErrorKind::TypeMismatch,
                        "incompatible return types",
                        span,
                    );
                }
            }
        }
    }

    fn check_expr(&mut self, expr: &mut HirExpr) -> TypeId {
        let ty = match &mut expr.kind {
            HirExprKind::Path(def_id) => self.check_path(*def_id, expr.span),
            HirExprKind::Literal(lit) => self.literal_type(lit),
            HirExprKind::Binary(op, lhs, rhs) => self.check_binary(*op, lhs, rhs),
            HirExprKind::Unary(op, operand) => self.check_unary(*op, operand),
            HirExprKind::Call(callee, args) => self.check_call(callee, args),
            HirExprKind::MethodCall(receiver, name, args) => {
                self.check_method_call(receiver, *name, args)
            }
            HirExprKind::Field(object, name) => self.check_field(object, *name),
            HirExprKind::Index(object, index) => self.check_index(object, index),
            HirExprKind::Block(block) => self.check_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_if(cond, then_block, else_block.as_mut())
            }
            HirExprKind::Discerne(scrutinee, arms) => self.check_match(scrutinee, arms),
            HirExprKind::Loop(block) => {
                self.check_block(block);
                self.vacuum_type()
            }
            HirExprKind::Dum(cond, block) => {
                self.check_condition(cond);
                self.check_block(block);
                self.vacuum_type()
            }
            HirExprKind::Itera(_binding, iter, block) => {
                self.check_expr(iter);
                self.check_block(block);
                self.vacuum_type()
            }
            HirExprKind::Assign(target, value) => self.check_assign(target, value),
            HirExprKind::AssignOp(op, target, value) => self.check_assign_op(*op, target, value),
            HirExprKind::Array(elements) => self.check_array(elements, expr.span),
            HirExprKind::Struct(def_id, fields) => self.check_struct_literal(*def_id, fields),
            HirExprKind::Tuple(items) => self.check_tuple(items),
            HirExprKind::Clausura(params, ret, body) => {
                self.check_closure(params, ret.as_mut(), body)
            }
            HirExprKind::Cede(inner) => self.check_expr(inner),
            HirExprKind::Qua(inner, target) => self.check_cast(inner, *target),
            HirExprKind::Ref(kind, inner) => {
                let inner_ty = self.check_expr(inner);
                let mutability = match kind {
                    crate::hir::HirRefKind::Shared => crate::semantic::Mutability::Immutable,
                    crate::hir::HirRefKind::Mutable => crate::semantic::Mutability::Mutable,
                };
                self.types.reference(mutability, inner_ty)
            }
            HirExprKind::Deref(inner) => self.check_deref(inner, expr.span),
            HirExprKind::Error => self.error_type,
        };

        expr.ty = Some(ty);
        ty
    }

    fn check_path(&mut self, def_id: DefId, span: crate::lexer::Span) -> TypeId {
        if let Some(binding) = self.lookup_binding(def_id) {
            return binding.ty;
        }

        if let Some(ty) = self.consts.get(&def_id) {
            return *ty;
        }

        if let Some(sig) = self.functions.get(&def_id) {
            return self.types.function(sig.clone());
        }

        self.error(
            SemanticErrorKind::UndefinedVariable,
            "unknown identifier",
            span,
        );
        self.error_type
    }

    fn check_binary(&mut self, op: HirBinOp, lhs: &mut HirExpr, rhs: &mut HirExpr) -> TypeId {
        let lhs_ty = self.check_expr(lhs);
        let rhs_ty = self.check_expr(rhs);

        match op {
            HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => {
                self.numeric_bin(lhs_ty, rhs_ty, lhs.span)
            }
            HirBinOp::Eq | HirBinOp::NotEq => {
                if !self.types.assignable(lhs_ty, rhs_ty) && !self.types.assignable(rhs_ty, lhs_ty)
                {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "incompatible operands",
                        lhs.span,
                    );
                }
                self.bool_type()
            }
            HirBinOp::Lt | HirBinOp::Gt | HirBinOp::LtEq | HirBinOp::GtEq => {
                self.numeric_bin(lhs_ty, rhs_ty, lhs.span);
                self.bool_type()
            }
            HirBinOp::And | HirBinOp::Or => {
                if !self.is_bool(lhs_ty) || !self.is_bool(rhs_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "boolean operands required",
                        lhs.span,
                    );
                }
                self.bool_type()
            }
            HirBinOp::BitAnd
            | HirBinOp::BitOr
            | HirBinOp::BitXor
            | HirBinOp::Shl
            | HirBinOp::Shr => {
                if !self.is_integer(lhs_ty) || !self.is_integer(rhs_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "integer operands required",
                        lhs.span,
                    );
                }
                self.numerus_type()
            }
        }
    }

    fn check_unary(&mut self, op: crate::hir::HirUnOp, operand: &mut HirExpr) -> TypeId {
        let operand_ty = self.check_expr(operand);
        match op {
            crate::hir::HirUnOp::Neg => {
                if !self.is_numeric(operand_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "numeric operand required",
                        operand.span,
                    );
                }
                operand_ty
            }
            crate::hir::HirUnOp::Not => {
                if !self.is_bool(operand_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "boolean operand required",
                        operand.span,
                    );
                }
                self.bool_type()
            }
            crate::hir::HirUnOp::BitNot => {
                if !self.is_integer(operand_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "integer operand required",
                        operand.span,
                    );
                }
                self.numerus_type()
            }
        }
    }

    fn check_call(&mut self, callee: &mut HirExpr, args: &mut [HirExpr]) -> TypeId {
        let callee_ty = self.check_expr(callee);

        let sig = match self.types.get(self.resolve_alias(callee_ty)) {
            Type::Func(sig) => Some(sig.clone()),
            _ => None,
        };

        let sig = match sig {
            Some(sig) => sig,
            None => {
                self.error(
                    SemanticErrorKind::NotCallable,
                    "callee is not callable",
                    callee.span,
                );
                return self.error_type;
            }
        };

        self.check_call_args(&sig, args, callee.span);
        sig.ret
    }

    fn check_method_call(
        &mut self,
        receiver: &mut HirExpr,
        name: Symbol,
        args: &mut [HirExpr],
    ) -> TypeId {
        let receiver_ty = self.check_expr(receiver);
        let Some(struct_def) = self.struct_def_from_type(receiver_ty) else {
            self.error(
                SemanticErrorKind::UndefinedMember,
                "method call on non-struct value",
                receiver.span,
            );
            return self.error_type;
        };

        let Some(info) = self.structs.get(&struct_def) else {
            return self.error_type;
        };

        let Some(sig) = info.methods.get(&name) else {
            self.error(
                SemanticErrorKind::UndefinedMember,
                "unknown method",
                receiver.span,
            );
            return self.error_type;
        };

        self.check_call_args(sig, args, receiver.span);
        sig.ret
    }

    fn check_call_args(&mut self, sig: &FuncSig, args: &mut [HirExpr], span: crate::lexer::Span) {
        if args.len() != sig.params.len() {
            self.error(
                SemanticErrorKind::WrongArity,
                "wrong number of arguments",
                span,
            );
        }

        for (arg, param) in args.iter_mut().zip(sig.params.iter()) {
            let arg_ty = self.check_expr(arg);
            if !self.types.assignable(arg_ty, param.ty) {
                self.error(
                    SemanticErrorKind::TypeMismatch,
                    "argument type mismatch",
                    arg.span,
                );
            }
        }
    }

    fn check_field(&mut self, object: &mut HirExpr, name: Symbol) -> TypeId {
        let obj_ty = self.check_expr(object);
        let Some(struct_def) = self.struct_def_from_type(obj_ty) else {
            self.error(
                SemanticErrorKind::UndefinedMember,
                "field access on non-struct value",
                object.span,
            );
            return self.error_type;
        };

        let Some(info) = self.structs.get(&struct_def) else {
            return self.error_type;
        };

        let Some(ty) = info.fields.get(&name) else {
            self.error(
                SemanticErrorKind::UndefinedMember,
                "unknown field",
                object.span,
            );
            return self.error_type;
        };

        *ty
    }

    fn check_index(&mut self, object: &mut HirExpr, index: &mut HirExpr) -> TypeId {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);
        match self.types.get(self.resolve_alias(obj_ty)) {
            Type::Array(elem) => {
                if !self.is_integer(idx_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "array index must be numerus",
                        index.span,
                    );
                }
                *elem
            }
            Type::Map(key, value) => {
                if !self.types.assignable(idx_ty, *key) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "map index type mismatch",
                        index.span,
                    );
                }
                *value
            }
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "indexing requires array or map",
                    object.span,
                );
                self.error_type
            }
        }
    }

    fn check_if(
        &mut self,
        cond: &mut HirExpr,
        then_block: &mut HirBlock,
        else_block: Option<&mut HirBlock>,
    ) -> TypeId {
        self.check_condition(cond);
        let then_ty = self.check_block(then_block);
        let else_ty = else_block
            .map(|block| self.check_block(block))
            .unwrap_or_else(|| self.vacuum_type());

        self.common_type(then_ty, else_ty, cond.span)
    }

    fn check_condition(&mut self, cond: &mut HirExpr) {
        let cond_ty = self.check_expr(cond);
        if !self.is_bool(cond_ty) {
            self.error(
                SemanticErrorKind::InvalidOperandTypes,
                "condition must be bivalens",
                cond.span,
            );
        }
    }

    fn check_match(&mut self, scrutinee: &mut HirExpr, arms: &mut [HirCasuArm]) -> TypeId {
        let scrutinee_ty = self.check_expr(scrutinee);
        let mut result_ty = None;

        for arm in arms {
            self.push_scope();
            self.check_pattern(&arm.pattern, scrutinee_ty, arm.span);
            if let Some(guard) = &mut arm.guard {
                self.check_condition(guard);
            }
            let body_ty = self.check_expr(&mut arm.body);
            result_ty = Some(match result_ty {
                None => body_ty,
                Some(existing) => self.common_type(existing, body_ty, arm.span),
            });
            self.pop_scope();
        }

        result_ty.unwrap_or_else(|| self.vacuum_type())
    }

    fn check_pattern(&mut self, pattern: &HirPattern, expected: TypeId, span: crate::lexer::Span) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Binding(def_id, _name) => {
                self.insert_binding(*def_id, expected, false);
            }
            HirPattern::Literal(lit) => {
                let lit_ty = self.literal_type(lit);
                if !self.types.assignable(lit_ty, expected)
                    && !self.types.assignable(expected, lit_ty)
                {
                    self.error(
                        SemanticErrorKind::TypeMismatch,
                        "pattern type mismatch",
                        span,
                    );
                }
            }
            HirPattern::Variant(variant_def, patterns) => {
                let Some(parent) = self.variant_parent.get(variant_def).copied() else {
                    self.error(
                        SemanticErrorKind::UndefinedVariable,
                        "unknown variant",
                        span,
                    );
                    return;
                };

                if let Some(expected_parent) = self.enum_def_from_type(expected) {
                    if expected_parent != parent {
                        self.error(
                            SemanticErrorKind::TypeMismatch,
                            "variant does not match scrutinee type",
                            span,
                        );
                    }
                }

                let fields = self
                    .variant_fields
                    .get(variant_def)
                    .cloned()
                    .unwrap_or_default();
                if fields.len() != patterns.len() {
                    self.error(
                        SemanticErrorKind::WrongArity,
                        "variant pattern arity mismatch",
                        span,
                    );
                }
                for (sub, field_ty) in patterns.iter().zip(fields.iter()) {
                    self.check_pattern(sub, *field_ty, span);
                }
            }
        }
    }

    fn check_assign(&mut self, target: &mut HirExpr, value: &mut HirExpr) -> TypeId {
        let target_ty = self.check_lvalue(target);
        let value_ty = self.check_expr(value);
        if !self.types.assignable(value_ty, target_ty) {
            self.error(
                SemanticErrorKind::TypeMismatch,
                "assignment type mismatch",
                value.span,
            );
        }
        target_ty
    }

    fn check_assign_op(
        &mut self,
        op: HirBinOp,
        target: &mut HirExpr,
        value: &mut HirExpr,
    ) -> TypeId {
        let target_ty = self.check_lvalue(target);
        let value_ty = self.check_expr(value);
        match op {
            HirBinOp::Add | HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => {
                self.numeric_bin(target_ty, value_ty, target.span);
            }
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "unsupported compound assignment",
                    target.span,
                );
            }
        }
        target_ty
    }

    fn check_lvalue(&mut self, target: &mut HirExpr) -> TypeId {
        match &mut target.kind {
            HirExprKind::Path(def_id) => {
                if let Some(binding) = self.lookup_binding(*def_id) {
                    if !binding.mutable {
                        self.error(
                            SemanticErrorKind::ImmutableAssignment,
                            "assignment to immutable binding",
                            target.span,
                        );
                    }
                    return binding.ty;
                }

                if let Some(ty) = self.consts.get(def_id) {
                    self.error(
                        SemanticErrorKind::ImmutableAssignment,
                        "assignment to constant",
                        target.span,
                    );
                    return *ty;
                }

                self.error(
                    SemanticErrorKind::InvalidAssignmentTarget,
                    "invalid assignment target",
                    target.span,
                );
                self.error_type
            }
            HirExprKind::Field(object, _name) => self.check_expr(object),
            HirExprKind::Index(object, _index) => self.check_expr(object),
            _ => {
                self.error(
                    SemanticErrorKind::InvalidAssignmentTarget,
                    "invalid assignment target",
                    target.span,
                );
                self.error_type
            }
        }
    }

    fn check_array(&mut self, elements: &mut [HirExpr], span: crate::lexer::Span) -> TypeId {
        if elements.is_empty() {
            self.error(
                SemanticErrorKind::MissingTypeAnnotation,
                "empty array needs type annotation",
                span,
            );
            return self.types.array(self.unknown_type());
        }

        let mut element_ty = None;
        for element in elements {
            let ty = self.check_expr(element);
            element_ty = Some(match element_ty {
                None => ty,
                Some(existing) => self.common_type(existing, ty, element.span),
            });
        }

        self.types
            .array(element_ty.unwrap_or_else(|| self.unknown_type()))
    }

    fn check_struct_literal(
        &mut self,
        def_id: DefId,
        fields: &mut Vec<(Symbol, HirExpr)>,
    ) -> TypeId {
        let Some(info) = self.structs.get(&def_id) else {
            self.error(
                SemanticErrorKind::UndefinedType,
                "unknown struct",
                fields
                    .first()
                    .map(|(_, expr)| expr.span)
                    .unwrap_or_default(),
            );
            return self.error_type;
        };

        for (name, value) in fields.iter_mut() {
            let Some(field_ty) = info.fields.get(name) else {
                self.error(
                    SemanticErrorKind::UndefinedMember,
                    "unknown field",
                    value.span,
                );
                continue;
            };
            let value_ty = self.check_expr(value);
            if !self.types.assignable(value_ty, *field_ty) {
                self.error(
                    SemanticErrorKind::TypeMismatch,
                    "field initializer type mismatch",
                    value.span,
                );
            }
        }

        self.types.intern(Type::Struct(def_id))
    }

    fn check_tuple(&mut self, items: &mut [HirExpr]) -> TypeId {
        let mut types = Vec::new();
        for item in items {
            types.push(self.check_expr(item));
        }
        self.types.intern(Type::Union(types))
    }

    fn check_closure(
        &mut self,
        params: &mut Vec<HirParam>,
        ret: Option<&mut TypeId>,
        body: &mut HirExpr,
    ) -> TypeId {
        self.push_scope();
        for param in params.iter() {
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            self.insert_binding(param.def_id, param.ty, mutable);
        }

        let body_ty = self.check_expr(body);
        let ret_ty = match ret {
            Some(ty) => {
                if !self.types.assignable(body_ty, *ty) {
                    self.error(
                        SemanticErrorKind::TypeMismatch,
                        "closure return type mismatch",
                        body.span,
                    );
                }
                *ty
            }
            None => body_ty,
        };

        self.pop_scope();

        let sig = FuncSig {
            params: params
                .iter()
                .map(|param| ParamType {
                    ty: param.ty,
                    mode: param_mode_from_hir(param.mode),
                    optional: false,
                })
                .collect(),
            ret: ret_ty,
            is_async: false,
            is_generator: false,
        };
        self.types.function(sig)
    }

    fn check_cast(&mut self, expr: &mut HirExpr, target: TypeId) -> TypeId {
        let expr_ty = self.check_expr(expr);
        if !self.types.assignable(expr_ty, target) && !self.types.assignable(target, expr_ty) {
            self.error(SemanticErrorKind::InvalidCast, "invalid cast", expr.span);
        }
        target
    }

    fn check_deref(&mut self, expr: &mut HirExpr, span: crate::lexer::Span) -> TypeId {
        let expr_ty = self.check_expr(expr);
        match self.types.get(self.resolve_alias(expr_ty)) {
            Type::Ref(_, inner) => *inner,
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "deref requires reference",
                    span,
                );
                self.error_type
            }
        }
    }

    fn numeric_bin(&mut self, lhs: TypeId, rhs: TypeId, span: crate::lexer::Span) -> TypeId {
        if !self.is_numeric(lhs) || !self.is_numeric(rhs) {
            self.error(
                SemanticErrorKind::InvalidOperandTypes,
                "numeric operands required",
                span,
            );
            return self.error_type;
        }

        if self.is_fractus(lhs) || self.is_fractus(rhs) {
            self.fractus_type()
        } else {
            self.numerus_type()
        }
    }

    fn common_type(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span) -> TypeId {
        if self.types.assignable(a, b) {
            return b;
        }
        if self.types.assignable(b, a) {
            return a;
        }
        self.error(SemanticErrorKind::TypeMismatch, "incompatible types", span);
        self.error_type
    }

    fn literal_type(&mut self, lit: &HirLiteral) -> TypeId {
        match lit {
            HirLiteral::Int(_) => self.numerus_type(),
            HirLiteral::Float(_) => self.fractus_type(),
            HirLiteral::String(_) => self.textus_type(),
            HirLiteral::Bool(_) => self.bool_type(),
            HirLiteral::Nil => self.nil_type(),
        }
    }

    fn struct_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_alias(ty)) {
            Type::Struct(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.struct_def_from_type(*inner),
            Type::Applied(base, _) => self.struct_def_from_type(*base),
            _ => None,
        }
    }

    fn enum_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_alias(ty)) {
            Type::Enum(def_id) => Some(*def_id),
            Type::Applied(base, _) => self.enum_def_from_type(*base),
            _ => None,
        }
    }

    fn resolve_alias(&self, ty: TypeId) -> TypeId {
        let mut current = ty;
        loop {
            match self.types.get(current) {
                Type::Alias(_, resolved) => current = *resolved,
                _ => return current,
            }
        }
    }

    fn is_numeric(&self, ty: TypeId) -> bool {
        self.is_integer(ty) || self.is_fractus(ty)
    }

    fn is_integer(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_alias(ty)),
            Type::Primitive(Primitive::Numerus)
        )
    }

    fn is_fractus(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_alias(ty)),
            Type::Primitive(Primitive::Fractus)
        )
    }

    fn is_bool(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_alias(ty)),
            Type::Primitive(Primitive::Bivalens)
        )
    }

    fn numerus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Numerus)
    }

    fn fractus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Fractus)
    }

    fn textus_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Textus)
    }

    fn bool_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Bivalens)
    }

    fn nil_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Nihil)
    }

    fn vacuum_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Vacuum)
    }

    fn unknown_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Ignotum)
    }

    fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_binding(&mut self, def_id: DefId, ty: TypeId, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(def_id, BindingInfo { ty, mutable });
        }
    }

    fn lookup_binding(&self, def_id: DefId) -> Option<BindingInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(&def_id) {
                return Some(*info);
            }
        }
        None
    }

    fn error(&mut self, kind: SemanticErrorKind, message: &str, span: crate::lexer::Span) {
        self.errors
            .push(SemanticError::new(kind, message.to_owned(), span));
    }
}

fn param_mode_from_hir(mode: HirParamMode) -> ParamMode {
    match mode {
        HirParamMode::Owned => ParamMode::Owned,
        HirParamMode::Ref => ParamMode::Ref,
        HirParamMode::MutRef => ParamMode::MutRef,
        HirParamMode::Move => ParamMode::Move,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::{
        HirBlock, HirExpr, HirExprKind, HirFunction, HirItem, HirItemKind, HirLiteral, HirLocal,
        HirProgram, HirStmt, HirStmtKind, HirStruct, HirTypeParam,
    };
    use crate::lexer::Span;

    fn span() -> Span {
        Span::default()
    }

    fn literal_int(id: u32, value: i64) -> HirExpr {
        HirExpr {
            id: crate::hir::HirId(id),
            kind: HirExprKind::Literal(HirLiteral::Int(value)),
            ty: None,
            span: span(),
        }
    }

    #[test]
    fn infers_function_return_type() {
        let mut types = TypeTable::new();
        let mut program = HirProgram {
            items: vec![HirItem {
                id: crate::hir::HirId(0),
                def_id: DefId(0),
                kind: HirItemKind::Function(HirFunction {
                    name: crate::lexer::Symbol(1),
                    type_params: Vec::new(),
                    params: Vec::new(),
                    ret_ty: None,
                    body: Some(HirBlock {
                        stmts: vec![HirStmt {
                            id: crate::hir::HirId(1),
                            kind: HirStmtKind::Redde(Some(literal_int(2, 42))),
                            span: span(),
                        }],
                        expr: None,
                        span: span(),
                    }),
                    is_async: false,
                    is_generator: false,
                }),
                span: span(),
            }],
            entry: None,
        };

        let resolver = Resolver::new();
        let result = typecheck(&mut program, &resolver, &mut types);
        assert!(result.is_ok());

        let item = &program.items[0];
        let HirItemKind::Function(func) = &item.kind else {
            panic!("expected function item");
        };
        assert_eq!(func.ret_ty, Some(types.primitive(Primitive::Numerus)));
    }

    #[test]
    fn reports_initializer_type_mismatch() {
        let mut types = TypeTable::new();
        let textus = types.primitive(Primitive::Textus);
        let mut program = HirProgram {
            items: Vec::new(),
            entry: Some(HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(0),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(0),
                        name: crate::lexer::Symbol(1),
                        ty: Some(textus),
                        init: Some(literal_int(1, 7)),
                        mutable: false,
                    }),
                    span: span(),
                }],
                expr: None,
                span: span(),
            }),
        };

        let resolver = Resolver::new();
        let result = typecheck(&mut program, &resolver, &mut types);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|err| err.kind == SemanticErrorKind::TypeMismatch));
    }

    #[test]
    fn resolves_method_call_type() {
        let mut types = TypeTable::new();
        let numerus = types.primitive(Primitive::Numerus);
        let struct_def = DefId(1);
        let struct_ty = types.intern(Type::Struct(struct_def));
        let method_name = crate::lexer::Symbol(3);
        let local_name = crate::lexer::Symbol(2);

        let mut program = HirProgram {
            items: vec![HirItem {
                id: crate::hir::HirId(0),
                def_id: struct_def,
                kind: HirItemKind::Struct(HirStruct {
                    name: crate::lexer::Symbol(1),
                    type_params: Vec::<HirTypeParam>::new(),
                    fields: Vec::new(),
                    methods: vec![crate::hir::HirMethod {
                        def_id: DefId(2),
                        func: HirFunction {
                            name: method_name,
                            type_params: Vec::new(),
                            params: Vec::new(),
                            ret_ty: Some(numerus),
                            body: None,
                            is_async: false,
                            is_generator: false,
                        },
                        receiver: crate::hir::HirReceiver::None,
                        span: span(),
                    }],
                    extends: None,
                    implements: Vec::new(),
                }),
                span: span(),
            }],
            entry: Some(HirBlock {
                stmts: vec![HirStmt {
                    id: crate::hir::HirId(10),
                    kind: HirStmtKind::Local(HirLocal {
                        def_id: DefId(10),
                        name: local_name,
                        ty: Some(struct_ty),
                        init: None,
                        mutable: false,
                    }),
                    span: span(),
                }],
                expr: Some(Box::new(HirExpr {
                    id: crate::hir::HirId(11),
                    kind: HirExprKind::MethodCall(
                        Box::new(HirExpr {
                            id: crate::hir::HirId(12),
                            kind: HirExprKind::Path(DefId(10)),
                            ty: None,
                            span: span(),
                        }),
                        method_name,
                        Vec::new(),
                    ),
                    ty: None,
                    span: span(),
                })),
                span: span(),
            }),
        };

        let resolver = Resolver::new();
        let result = typecheck(&mut program, &resolver, &mut types);
        assert!(result.is_ok());

        let entry_expr = program
            .entry
            .as_ref()
            .and_then(|block| block.expr.as_ref())
            .expect("expected entry expr");
        assert_eq!(entry_expr.ty, Some(numerus));
    }
}
