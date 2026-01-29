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
    next_infer: u32,
    infer_ids: FxHashMap<crate::semantic::InferVar, TypeId>,
    substitutions: FxHashMap<crate::semantic::InferVar, TypeId>,
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
            next_infer: 0,
            infer_ids: FxHashMap::default(),
            substitutions: FxHashMap::default(),
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
        let ret = func.ret_ty.unwrap_or_else(|| self.fresh_infer());
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
            self.check_block(entry, None);
        }

        self.finalize_hir(hir);
    }

    fn finalize_hir(&mut self, hir: &mut HirProgram) {
        for item in &mut hir.items {
            self.finalize_item(item);
        }

        if let Some(entry) = &mut hir.entry {
            self.finalize_block(entry);
        }
    }

    fn finalize_item(&mut self, item: &mut HirItem) {
        match &mut item.kind {
            HirItemKind::Function(func) => self.finalize_function(func),
            HirItemKind::Const(const_item) => {
                if let Some(ty) = const_item.ty {
                    let resolved = self.resolve_type(ty);
                    if self.is_infer(resolved) {
                        self.error(
                            SemanticErrorKind::MissingTypeAnnotation,
                            "cannot infer constant type",
                            const_item.value.span,
                        );
                    }
                    const_item.ty = Some(resolved);
                }
                self.finalize_expr(&mut const_item.value);
            }
            HirItemKind::Struct(struct_item) => {
                for method in &mut struct_item.methods {
                    self.finalize_function(&mut method.func);
                }
            }
            _ => {}
        }
    }

    fn finalize_function(&mut self, func: &mut HirFunction) {
        if let Some(ret) = func.ret_ty {
            let resolved = self.resolve_type(ret);
            if self.is_infer(resolved) {
                let span = func.body.as_ref().map(|body| body.span).unwrap_or_default();
                self.error(
                    SemanticErrorKind::MissingTypeAnnotation,
                    "cannot infer return type",
                    span,
                );
            }
            func.ret_ty = Some(resolved);
        }

        if let Some(body) = &mut func.body {
            self.finalize_block(body);
        }
    }

    fn finalize_block(&mut self, block: &mut HirBlock) {
        for stmt in &mut block.stmts {
            self.finalize_stmt(stmt);
        }
        if let Some(expr) = &mut block.expr {
            self.finalize_expr(expr);
        }
    }

    fn finalize_stmt(&mut self, stmt: &mut HirStmt) {
        match &mut stmt.kind {
            HirStmtKind::Local(local) => {
                if let Some(ty) = local.ty {
                    let resolved = self.resolve_type(ty);
                    if self.is_infer(resolved) {
                        self.error(
                            SemanticErrorKind::MissingTypeAnnotation,
                            "cannot infer variable type",
                            local
                                .init
                                .as_ref()
                                .map(|expr| expr.span)
                                .unwrap_or_default(),
                        );
                    }
                    local.ty = Some(resolved);
                }
                if let Some(init) = &mut local.init {
                    self.finalize_expr(init);
                }
            }
            HirStmtKind::Expr(expr) => self.finalize_expr(expr),
            HirStmtKind::Redde(value) => {
                if let Some(expr) = value {
                    self.finalize_expr(expr);
                }
            }
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    fn finalize_expr(&mut self, expr: &mut HirExpr) {
        let resolved = expr.ty.map(|ty| self.resolve_type(ty));
        if let Some(ty) = resolved {
            if self.is_infer(ty) {
                self.error(
                    SemanticErrorKind::MissingTypeAnnotation,
                    "cannot infer expression type",
                    expr.span,
                );
            } else {
                expr.ty = Some(ty);
            }
        }

        match &mut expr.kind {
            HirExprKind::Binary(_, lhs, rhs) => {
                self.finalize_expr(lhs);
                self.finalize_expr(rhs);
            }
            HirExprKind::Unary(_, operand) => self.finalize_expr(operand),
            HirExprKind::Call(callee, args) => {
                self.finalize_expr(callee);
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::MethodCall(receiver, _, args) => {
                self.finalize_expr(receiver);
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::Field(object, _) => self.finalize_expr(object),
            HirExprKind::Index(object, index) => {
                self.finalize_expr(object);
                self.finalize_expr(index);
            }
            HirExprKind::Block(block) => self.finalize_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.finalize_expr(cond);
                self.finalize_block(then_block);
                if let Some(block) = else_block {
                    self.finalize_block(block);
                }
            }
            HirExprKind::Discerne(scrutinee, arms) => {
                self.finalize_expr(scrutinee);
                for arm in arms {
                    if let Some(guard) = &mut arm.guard {
                        self.finalize_expr(guard);
                    }
                    self.finalize_expr(&mut arm.body);
                }
            }
            HirExprKind::Loop(block) => self.finalize_block(block),
            HirExprKind::Dum(cond, block) => {
                self.finalize_expr(cond);
                self.finalize_block(block);
            }
            HirExprKind::Itera(_, iter, block) => {
                self.finalize_expr(iter);
                self.finalize_block(block);
            }
            HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.finalize_expr(lhs);
                self.finalize_expr(rhs);
            }
            HirExprKind::Array(elements) | HirExprKind::Tuple(elements) => {
                for element in elements {
                    self.finalize_expr(element);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.finalize_expr(value);
                }
            }
            HirExprKind::Clausura(_, _, body) => self.finalize_expr(body),
            HirExprKind::Cede(expr)
            | HirExprKind::Qua(expr, _)
            | HirExprKind::Ref(_, expr)
            | HirExprKind::Deref(expr) => self.finalize_expr(expr),
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
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
            self.unify(
                value_ty,
                annotated,
                const_item.value.span,
                "constant value does not match annotation",
            );
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
            self.check_block(body, None);
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

    fn check_block(&mut self, block: &mut HirBlock, expected: Option<TypeId>) -> TypeId {
        self.push_scope();
        for stmt in &mut block.stmts {
            self.check_stmt(stmt);
        }
        let ty = if let Some(expr) = &mut block.expr {
            self.check_expr_with_expected(expr, expected)
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
                let init_ty = self.check_expr_with_expected(init, Some(*ty));
                self.unify(
                    init_ty,
                    *ty,
                    init.span,
                    "initializer does not match annotation",
                );
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
                self.fresh_infer()
            }
        };

        if local.ty.is_none() {
            local.ty = Some(inferred);
        }

        self.insert_binding(local.def_id, inferred, local.mutable);
    }

    fn check_return(&mut self, value: Option<&mut HirExpr>, span: crate::lexer::Span) {
        let value_ty = match value {
            Some(expr) => {
                if let Some(expected) = self.current_return {
                    self.check_expr_with_expected(expr, Some(expected))
                } else {
                    self.check_expr(expr)
                }
            }
            None => self.vacuum_type(),
        };

        if let Some(expected) = self.current_return {
            self.unify(
                value_ty,
                expected,
                span,
                "return type does not match function signature",
            );
            return;
        }

        match self.inferred_return {
            None => self.inferred_return = Some(value_ty),
            Some(existing) => {
                self.unify(value_ty, existing, span, "incompatible return types");
            }
        }
    }

    fn check_expr(&mut self, expr: &mut HirExpr) -> TypeId {
        self.check_expr_with_expected(expr, None)
    }

    fn check_expr_with_expected(&mut self, expr: &mut HirExpr, expected: Option<TypeId>) -> TypeId {
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
            HirExprKind::Block(block) => self.check_block(block, expected),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_if(cond, then_block, else_block.as_mut(), expected)
            }
            HirExprKind::Discerne(scrutinee, arms) => self.check_match(scrutinee, arms, expected),
            HirExprKind::Loop(block) => {
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Dum(cond, block) => {
                self.check_condition(cond);
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Itera(_binding, iter, block) => {
                self.check_expr(iter);
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Assign(target, value) => self.check_assign(target, value),
            HirExprKind::AssignOp(op, target, value) => self.check_assign_op(*op, target, value),
            HirExprKind::Array(elements) => self.check_array(elements, expr.span, expected),
            HirExprKind::Struct(def_id, fields) => self.check_struct_literal(*def_id, fields),
            HirExprKind::Tuple(items) => self.check_tuple(items),
            HirExprKind::Clausura(params, ret, body) => {
                self.check_closure(params, ret.as_mut(), body, expected)
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

        let ty = if let Some(expected) = expected {
            self.unify(ty, expected, expr.span, "expression type mismatch")
        } else {
            ty
        };
        expr.ty = Some(self.resolve_type(ty));
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
                self.unify(lhs_ty, rhs_ty, lhs.span, "incompatible operands");
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
                if self.is_infer(self.resolve_type(operand_ty)) {
                    return self.unify(
                        operand_ty,
                        self.numerus_type(),
                        operand.span,
                        "numeric operand required",
                    );
                }
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
                if self.is_infer(self.resolve_type(operand_ty)) {
                    self.unify(
                        operand_ty,
                        self.bool_type(),
                        operand.span,
                        "boolean operand required",
                    );
                    return self.bool_type();
                }
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
                if self.is_infer(self.resolve_type(operand_ty)) {
                    self.unify(
                        operand_ty,
                        self.numerus_type(),
                        operand.span,
                        "integer operand required",
                    );
                    return self.numerus_type();
                }
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

        let resolved = self.resolve_type(callee_ty);
        if let Some(sig) = self.function_signature_from_type(resolved) {
            self.check_call_args(&sig, args, callee.span);
            return self.resolve_type(sig.ret);
        }

        if self.is_infer(resolved) {
            let sig = self.build_call_signature(args);
            let func_ty = self.types.function(sig.clone());
            self.unify(resolved, func_ty, callee.span, "callee is not callable");
            self.check_call_args(&sig, args, callee.span);
            return sig.ret;
        }

        self.error(
            SemanticErrorKind::NotCallable,
            "callee is not callable",
            callee.span,
        );
        self.error_type
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
            self.unify(arg_ty, param.ty, arg.span, "argument type mismatch");
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
        match self.types.get(self.resolve_type(obj_ty)) {
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
                self.unify(idx_ty, *key, index.span, "map index type mismatch");
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
        expected: Option<TypeId>,
    ) -> TypeId {
        self.check_condition(cond);
        let then_ty = self.check_block(then_block, expected);
        let else_ty = else_block
            .map(|block| self.check_block(block, expected))
            .unwrap_or_else(|| self.vacuum_type());

        self.common_type(then_ty, else_ty, cond.span)
    }

    fn check_condition(&mut self, cond: &mut HirExpr) {
        let cond_ty = self.check_expr(cond);
        if self.is_infer(self.resolve_type(cond_ty)) {
            self.unify(
                cond_ty,
                self.bool_type(),
                cond.span,
                "condition must be bivalens",
            );
            return;
        }
        if !self.is_bool(cond_ty) {
            self.error(
                SemanticErrorKind::InvalidOperandTypes,
                "condition must be bivalens",
                cond.span,
            );
        }
    }

    fn check_match(
        &mut self,
        scrutinee: &mut HirExpr,
        arms: &mut [HirCasuArm],
        expected: Option<TypeId>,
    ) -> TypeId {
        let scrutinee_ty = self.check_expr(scrutinee);
        let mut result_ty = None;

        for arm in arms {
            self.push_scope();
            self.check_pattern(&arm.pattern, scrutinee_ty, arm.span);
            if let Some(guard) = &mut arm.guard {
                self.check_condition(guard);
            }
            let body_ty = self.check_expr_with_expected(&mut arm.body, expected);
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
                self.unify(lit_ty, expected, span, "pattern type mismatch");
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
        let value_ty = self.check_expr_with_expected(value, Some(target_ty));
        self.unify(value_ty, target_ty, value.span, "assignment type mismatch");
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

    fn check_array(
        &mut self,
        elements: &mut [HirExpr],
        span: crate::lexer::Span,
        expected: Option<TypeId>,
    ) -> TypeId {
        let expected_elem = expected.and_then(|ty| match self.types.get(self.resolve_type(ty)) {
            Type::Array(inner) => Some(*inner),
            _ => None,
        });

        if elements.is_empty() {
            if let Some(inner) = expected_elem {
                return self.types.array(inner);
            }
            self.error(
                SemanticErrorKind::MissingTypeAnnotation,
                "empty array needs type annotation",
                span,
            );
            return self.types.array(self.fresh_infer());
        }

        let mut element_ty = None;
        for element in elements {
            let ty = if let Some(expected) = expected_elem {
                self.check_expr_with_expected(element, Some(expected))
            } else {
                self.check_expr(element)
            };
            element_ty = Some(match element_ty {
                None => ty,
                Some(existing) => self.common_type(existing, ty, element.span),
            });
        }

        self.types
            .array(element_ty.unwrap_or_else(|| self.fresh_infer()))
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
            self.unify(
                value_ty,
                *field_ty,
                value.span,
                "field initializer type mismatch",
            );
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
        expected: Option<TypeId>,
    ) -> TypeId {
        let expected_sig = expected.and_then(|ty| self.function_signature_from_type(ty));

        self.push_scope();
        for (idx, param) in params.iter().enumerate() {
            let mutable = matches!(param.mode, HirParamMode::MutRef);
            if let Some(sig) = &expected_sig {
                if let Some(expected_param) = sig.params.get(idx) {
                    self.unify(
                        param.ty,
                        expected_param.ty,
                        param.span,
                        "closure parameter type mismatch",
                    );
                }
            }
            self.insert_binding(param.def_id, param.ty, mutable);
        }

        let expected_ret = ret
            .as_ref()
            .copied()
            .or_else(|| expected_sig.as_ref().map(|sig| sig.ret));
        let body_ty = self.check_expr_with_expected(body, expected_ret);
        let ret_ty = match ret {
            Some(ty) => {
                self.unify(body_ty, *ty, body.span, "closure return type mismatch");
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
        if self.is_infer(self.resolve_type(target)) {
            return self.unify(expr_ty, target, expr.span, "invalid cast");
        }
        if !self.types.assignable(expr_ty, target) && !self.types.assignable(target, expr_ty) {
            self.error(SemanticErrorKind::InvalidCast, "invalid cast", expr.span);
        }
        target
    }

    fn check_deref(&mut self, expr: &mut HirExpr, span: crate::lexer::Span) -> TypeId {
        let expr_ty = self.check_expr(expr);
        match self.types.get(self.resolve_type(expr_ty)) {
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
        let lhs_resolved = self.resolve_type(lhs);
        let rhs_resolved = self.resolve_type(rhs);
        if self.is_infer(lhs_resolved) {
            self.unify(lhs, self.numerus_type(), span, "numeric operands required");
        }
        if self.is_infer(rhs_resolved) {
            self.unify(rhs, self.numerus_type(), span, "numeric operands required");
        }
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
        let left = self.resolve_type(a);
        let right = self.resolve_type(b);
        if self.types.assignable(left, right) {
            return right;
        }
        if self.types.assignable(right, left) {
            return left;
        }
        if self.is_numeric(left) && self.is_numeric(right) {
            return self.numeric_bin(left, right, span);
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
        match self.types.get(self.resolve_type(ty)) {
            Type::Struct(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.struct_def_from_type(*inner),
            Type::Applied(base, _) => self.struct_def_from_type(*base),
            _ => None,
        }
    }

    fn enum_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
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

    fn fresh_infer(&mut self) -> TypeId {
        let var = crate::semantic::InferVar(self.next_infer);
        self.next_infer += 1;
        let id = self.types.intern(Type::Infer(var));
        self.infer_ids.insert(var, id);
        id
    }

    fn resolve_type(&self, ty: TypeId) -> TypeId {
        let mut current = ty;
        loop {
            if let Some(infer) = self.infer_var_of(current) {
                if let Some(subst) = self.substitutions.get(&infer) {
                    current = *subst;
                    continue;
                }
            }
            match self.types.get(current) {
                Type::Alias(_, resolved) => current = *resolved,
                _ => return current,
            }
        }
    }

    fn infer_var_of(&self, ty: TypeId) -> Option<crate::semantic::InferVar> {
        match self.types.get(ty) {
            Type::Infer(var) => Some(*var),
            _ => None,
        }
    }

    fn is_infer(&self, ty: TypeId) -> bool {
        self.infer_var_of(ty).is_some()
    }

    fn function_signature_from_type(&self, ty: TypeId) -> Option<FuncSig> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Func(sig) => Some(sig.clone()),
            _ => None,
        }
    }

    fn build_call_signature(&mut self, args: &mut [HirExpr]) -> FuncSig {
        let params = args
            .iter_mut()
            .map(|arg| ParamType {
                ty: self.check_expr(arg),
                mode: ParamMode::Owned,
                optional: false,
            })
            .collect();
        FuncSig {
            params,
            ret: self.fresh_infer(),
            is_async: false,
            is_generator: false,
        }
    }

    fn unify(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span, message: &str) -> TypeId {
        let left = self.resolve_type(a);
        let right = self.resolve_type(b);
        if left == right {
            return left;
        }

        if let Some(var) = self.infer_var_of(left) {
            return self.bind_infer(var, right, span, message);
        }
        if let Some(var) = self.infer_var_of(right) {
            return self.bind_infer(var, left, span, message);
        }

        match (self.types.get(left), self.types.get(right)) {
            (Type::Primitive(Primitive::Numerus), Type::Primitive(Primitive::Fractus))
            | (Type::Primitive(Primitive::Fractus), Type::Primitive(Primitive::Numerus)) => {
                return self.fractus_type();
            }
            (Type::Primitive(a), Type::Primitive(b)) if a == b => return left,
            (Type::Array(a), Type::Array(b)) => {
                let inner = self.unify(*a, *b, span, message);
                return self.types.array(inner);
            }
            (Type::Map(ka, va), Type::Map(kb, vb)) => {
                let key = self.unify(*ka, *kb, span, message);
                let value = self.unify(*va, *vb, span, message);
                return self.types.map(key, value);
            }
            (Type::Set(a), Type::Set(b)) => {
                let inner = self.unify(*a, *b, span, message);
                return self.types.set(inner);
            }
            (Type::Option(a), Type::Option(b)) => {
                let inner = self.unify(*a, *b, span, message);
                return self.types.option(inner);
            }
            (Type::Ref(ma, a), Type::Ref(mb, b)) if ma == mb => {
                let inner = self.unify(*a, *b, span, message);
                return self.types.reference(*ma, inner);
            }
            (Type::Func(sig_a), Type::Func(sig_b)) => {
                if sig_a.params.len() != sig_b.params.len() {
                    self.error(SemanticErrorKind::WrongArity, message, span);
                    return self.error_type;
                }
                for (param_a, param_b) in sig_a.params.iter().zip(sig_b.params.iter()) {
                    self.unify(param_a.ty, param_b.ty, span, message);
                }
                let ret = self.unify(sig_a.ret, sig_b.ret, span, message);
                return self.types.function(FuncSig {
                    params: sig_a.params.clone(),
                    ret,
                    is_async: sig_a.is_async || sig_b.is_async,
                    is_generator: sig_a.is_generator || sig_b.is_generator,
                });
            }
            _ => {
                if self.types.assignable(left, right) || self.types.assignable(right, left) {
                    return right;
                }
            }
        }

        self.error(SemanticErrorKind::TypeMismatch, message, span);
        self.error_type
    }

    fn bind_infer(
        &mut self,
        var: crate::semantic::InferVar,
        ty: TypeId,
        span: crate::lexer::Span,
        message: &str,
    ) -> TypeId {
        let resolved = self.resolve_type(ty);
        if let Some(existing) = self.substitutions.get(&var) {
            return self.unify(*existing, resolved, span, message);
        }

        if self.occurs_in(var, resolved) {
            self.error(SemanticErrorKind::TypeMismatch, message, span);
            return self.error_type;
        }

        self.substitutions.insert(var, resolved);
        resolved
    }

    fn occurs_in(&self, var: crate::semantic::InferVar, ty: TypeId) -> bool {
        let resolved = self.resolve_type(ty);
        if let Some(found) = self.infer_var_of(resolved) {
            return found == var;
        }
        match self.types.get(resolved) {
            Type::Array(inner) | Type::Option(inner) | Type::Ref(_, inner) => {
                self.occurs_in(var, *inner)
            }
            Type::Set(inner) => self.occurs_in(var, *inner),
            Type::Map(key, value) => self.occurs_in(var, *key) || self.occurs_in(var, *value),
            Type::Func(sig) => {
                sig.params.iter().any(|param| self.occurs_in(var, param.ty))
                    || self.occurs_in(var, sig.ret)
            }
            Type::Applied(base, args) => {
                self.occurs_in(var, *base) || args.iter().any(|arg| self.occurs_in(var, *arg))
            }
            Type::Union(types) => types.iter().any(|inner| self.occurs_in(var, *inner)),
            _ => false,
        }
    }

    fn is_numeric(&self, ty: TypeId) -> bool {
        self.is_integer(ty) || self.is_fractus(ty)
    }

    fn is_integer(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_type(ty)),
            Type::Primitive(Primitive::Numerus)
        )
    }

    fn is_fractus(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_type(ty)),
            Type::Primitive(Primitive::Fractus)
        )
    }

    fn is_bool(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get(self.resolve_type(ty)),
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
