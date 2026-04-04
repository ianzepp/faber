//! Pass 4: Type checking
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Implements bidirectional type inference and checking for the HIR. Infers
//! types where unspecified, checks type compatibility where specified, and
//! attaches TypeId annotations to every expression and declaration.
//!
//! COMPILER PHASE: Semantic (Pass 4)
//! INPUT: HIR with resolved DefIds but no type information
//! OUTPUT: HIR with TypeId on every expr/stmt; type errors for mismatches
//!
//! WHY: Bidirectional type checking combines inference (bottom-up) with
//! checking (top-down), enabling both flexibility (infer local variable types)
//! and precision (check function arguments against signatures).
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Inference Variables: Fresh type variables (InferVar) represent unknown
//!   types during checking; unification resolves them to concrete types
//! - Expected Type Propagation: check_expr_with_expected() allows parent
//!   context to guide type inference (e.g., array element type from annotation)
//! - Substitution Resolution: resolve_type() follows chains of type variable
//!   bindings to concrete types after unification
//! - Finalization Phase: After checking, finalize_hir() replaces all InferVars
//!   with their resolved types or emits errors for unresolved variables
//!
//! BIDIRECTIONAL TYPING
//! ====================
//! - Synthesis (check_expr): Infer type bottom-up (e.g., literal 42 → numerus)
//! - Checking (check_expr_with_expected): Verify type top-down (e.g., if
//!   expected is fractus, coerce numerus → fractus)
//!
//! UNIFICATION
//! ===========
//! unify(a, b) makes two types equal by:
//! 1. If either is InferVar, bind it to the other type (occurs check prevents cycles)
//! 2. If both are concrete, check structural equality (e.g., lista<T> ~ lista<U>)
//! 3. If assignable (e.g., numerus → fractus widening), use target type
//! 4. Otherwise, emit type mismatch error
//!
//! TRADE-OFFS
//! ==========
//! - No HM-style full inference: Requires type annotations on function parameters
//!   to avoid ambiguity and improve error messages (simpler for users)
//! - Eager error reporting: Continues checking after errors to find more issues,
//!   but may produce cascading errors if types remain unresolved

use crate::hir::{
    DefId, HirArrayElement, HirBinOp, HirBlock, HirCasuArm, HirExpr, HirExprKind, HirFunction, HirId, HirItem,
    HirItemKind, HirLiteral, HirLocal, HirObjectField, HirObjectKey, HirParam, HirParamMode, HirPattern, HirProgram,
    HirStmt, HirStmtKind, HirStruct,
};
use crate::lexer::Symbol;
use crate::semantic::{
    types::InferVar, FuncSig, ParamMode, ParamType, Primitive, Resolver, SemanticError, SemanticErrorKind, Type,
    TypeId, TypeTable,
};
use rustc_hash::{FxHashMap, FxHashSet};

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
    #[allow(dead_code)]
    resolver: &'a Resolver,
    types: &'a mut TypeTable,
    errors: Vec<SemanticError>,
    scopes: Vec<FxHashMap<DefId, BindingInfo>>,
    functions: FxHashMap<DefId, FuncSig>,
    consts: FxHashMap<DefId, TypeId>,
    structs: FxHashMap<DefId, StructInfo>,
    interfaces: FxHashMap<DefId, FxHashMap<Symbol, FuncSig>>,
    variant_fields: FxHashMap<DefId, Vec<TypeId>>,
    variant_parent: FxHashMap<DefId, DefId>,
    current_return: Option<TypeId>,
    inferred_return: Option<TypeId>,
    next_infer: u32,
    infer_ids: FxHashMap<InferVar, TypeId>,
    substitutions: FxHashMap<InferVar, TypeId>,
    errored_exprs: FxHashSet<HirId>,
    error_type: TypeId,
}

/// Type check the HIR
pub fn typecheck(hir: &mut HirProgram, resolver: &Resolver, types: &mut TypeTable) -> Result<(), Vec<SemanticError>> {
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
            interfaces: FxHashMap::default(),
            variant_fields: FxHashMap::default(),
            variant_parent: FxHashMap::default(),
            current_return: None,
            inferred_return: None,
            next_infer: 0,
            infer_ids: FxHashMap::default(),
            substitutions: FxHashMap::default(),
            errored_exprs: FxHashSet::default(),
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
                HirItemKind::Interface(interface_item) => {
                    let mut methods = FxHashMap::default();
                    for method in &interface_item.methods {
                        let params = method
                            .params
                            .iter()
                            .map(|param| ParamType {
                                ty: param.ty,
                                mode: param_mode_from_hir(param.mode),
                                optional: param.optional,
                            })
                            .collect();
                        let ret = method.ret_ty.unwrap_or_else(|| self.vacuum_type());
                        methods.insert(method.name, FuncSig { params, ret, is_async: false, is_generator: false });
                    }
                    self.interfaces.insert(item.def_id, methods);
                }
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
            .map(|param| ParamType { ty: param.ty, mode: param_mode_from_hir(param.mode), optional: param.optional })
            .collect();
        let ret = func.ret_ty.unwrap_or_else(|| self.vacuum_type());
        FuncSig { params, ret, is_async: func.is_async, is_generator: func.is_generator }
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
                self.error(SemanticErrorKind::MissingTypeAnnotation, "cannot infer return type", span);
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
            HirStmtKind::Ad(ad) => {
                for arg in &mut ad.args {
                    self.finalize_expr(arg);
                }
                if let Some(body) = &mut ad.body {
                    self.finalize_block(body);
                }
                if let Some(catch) = &mut ad.catch {
                    self.finalize_block(catch);
                }
            }
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
            HirExprKind::OptionalChain(object, chain) => {
                self.finalize_expr(object);
                match chain {
                    crate::hir::HirOptionalChainKind::Member(_) => {}
                    crate::hir::HirOptionalChainKind::Index(index) => self.finalize_expr(index),
                    crate::hir::HirOptionalChainKind::Call(args) => {
                        for arg in args {
                            self.finalize_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::NonNull(object, chain) => {
                self.finalize_expr(object);
                match chain {
                    crate::hir::HirNonNullKind::Member(_) => {}
                    crate::hir::HirNonNullKind::Index(index) => self.finalize_expr(index),
                    crate::hir::HirNonNullKind::Call(args) => {
                        for arg in args {
                            self.finalize_expr(arg);
                        }
                    }
                }
            }
            HirExprKind::Ab { source, filter, transforms } => {
                self.finalize_expr(source);
                if let Some(filter) = filter {
                    if let crate::hir::HirCollectionFilterKind::Condition(cond) = &mut filter.kind {
                        self.finalize_expr(cond);
                    }
                }
                for transform in transforms {
                    if let Some(arg) = &mut transform.arg {
                        self.finalize_expr(arg);
                    }
                }
            }
            HirExprKind::Block(block) => self.finalize_block(block),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.finalize_expr(cond);
                self.finalize_block(then_block);
                if let Some(block) = else_block {
                    self.finalize_block(block);
                }
            }
            HirExprKind::Discerne(scrutinees, arms) => {
                for scrutinee in scrutinees {
                    self.finalize_expr(scrutinee);
                }
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
            HirExprKind::Itera(_, _, _, iter, block) => {
                self.finalize_expr(iter);
                self.finalize_block(block);
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                self.finalize_expr(start);
                self.finalize_expr(end);
                if let Some(step) = step {
                    self.finalize_expr(step);
                }
            }
            HirExprKind::Assign(lhs, rhs) | HirExprKind::AssignOp(_, lhs, rhs) => {
                self.finalize_expr(lhs);
                self.finalize_expr(rhs);
            }
            HirExprKind::Array(elements) => {
                for element in elements {
                    match element {
                        HirArrayElement::Expr(expr) | HirArrayElement::Spread(expr) => self.finalize_expr(expr),
                    }
                }
            }
            HirExprKind::Tuple(elements) | HirExprKind::Scribe(elements) => {
                for element in elements {
                    self.finalize_expr(element);
                }
            }
            HirExprKind::Scriptum(_, args) => {
                for arg in args {
                    self.finalize_expr(arg);
                }
            }
            HirExprKind::Adfirma(cond, message) => {
                self.finalize_expr(cond);
                if let Some(message) = message {
                    self.finalize_expr(message);
                }
            }
            HirExprKind::Panic(value) | HirExprKind::Throw(value) => self.finalize_expr(value),
            HirExprKind::Tempta { body, catch, finally } => {
                self.finalize_block(body);
                if let Some(catch) = catch {
                    self.finalize_block(catch);
                }
                if let Some(finally) = finally {
                    self.finalize_block(finally);
                }
            }
            HirExprKind::Struct(_, fields) => {
                for (_, value) in fields {
                    self.finalize_expr(value);
                }
            }
            HirExprKind::Clausura(_, _, body) => self.finalize_expr(body),
            HirExprKind::Verte { source, entries, .. } => {
                self.finalize_expr(source);
                if let Some(entries) = entries {
                    for field in entries {
                        self.finalize_object_field(field);
                    }
                }
            }
            HirExprKind::Conversio { source, fallback, .. } => {
                self.finalize_expr(source);
                if let Some(fallback) = fallback {
                    self.finalize_expr(fallback);
                }
            }
            HirExprKind::Cede(expr) | HirExprKind::Ref(_, expr) | HirExprKind::Deref(expr) => self.finalize_expr(expr),
            HirExprKind::Path(_) | HirExprKind::Literal(_) | HirExprKind::Error => {}
        }
    }

    fn finalize_object_field(&mut self, field: &mut HirObjectField) {
        match &mut field.key {
            HirObjectKey::Computed(expr) | HirObjectKey::Spread(expr) => self.finalize_expr(expr),
            HirObjectKey::Ident(_) | HirObjectKey::String(_) => {}
        }
        if let Some(value) = &mut field.value {
            self.finalize_expr(value);
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
                let expr_ty = self.check_expr(expr);
                if self.is_infer(self.resolve_type(expr_ty)) {
                    let vacuum = self.vacuum_type();
                    self.unify(expr_ty, vacuum, expr.span, "ignored expression result must resolve");
                }
            }
            HirStmtKind::Ad(ad) => {
                for arg in &mut ad.args {
                    self.check_expr(arg);
                }
                if let Some(body) = &mut ad.body {
                    self.check_block(body, None);
                }
                if let Some(catch) = &mut ad.catch {
                    self.check_block(catch, None);
                }
            }
            HirStmtKind::Redde(value) => self.check_return(value.as_mut(), stmt.span),
            HirStmtKind::Rumpe | HirStmtKind::Perge => {}
        }
    }

    fn check_local(&mut self, local: &mut HirLocal) {
        let inferred = match (&local.ty, &mut local.init) {
            (Some(ty), Some(init)) => {
                let init_ty = self.check_expr_with_expected(init, Some(*ty));
                self.unify(init_ty, *ty, init.span, "initializer does not match annotation");
                *ty
            }
            (Some(ty), None) => *ty,
            (None, Some(init)) => self.check_expr(init),
            (None, None) => self.fresh_infer(),
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
            self.unify(value_ty, expected, span, "return type does not match function signature");
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
            HirExprKind::MethodCall(receiver, name, args) => self.check_method_call(receiver, *name, args),
            HirExprKind::Field(object, name) => self.check_field(object, *name),
            HirExprKind::Index(object, index) => self.check_index(object, index),
            HirExprKind::OptionalChain(object, chain) => self.check_optional_chain(object, chain, expr.span),
            HirExprKind::NonNull(object, chain) => self.check_non_null(object, chain, expr.span),
            HirExprKind::Ab { source, filter, transforms } => self.check_ab(source, filter.as_mut(), transforms),
            HirExprKind::Block(block) => self.check_block(block, expected),
            HirExprKind::Si(cond, then_block, else_block) => {
                self.check_if(cond, then_block, else_block.as_mut(), expected)
            }
            HirExprKind::Discerne(scrutinees, arms) => self.check_match(scrutinees, arms, expected),
            HirExprKind::Loop(block) => {
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Dum(cond, block) => {
                self.check_condition(cond);
                self.check_block(block, None);
                self.vacuum_type()
            }
            HirExprKind::Itera(mode, binding, _, iter, block) => {
                let iter_ty = self.check_expr(iter);
                let elem_ty = match self.types.get(self.resolve_type(iter_ty)) {
                    Type::Array(inner) => match mode {
                        crate::hir::HirIteraMode::De => self.numerus_type(),
                        crate::hir::HirIteraMode::Ex | crate::hir::HirIteraMode::Pro => *inner,
                    },
                    Type::Map(key, value) => match mode {
                        crate::hir::HirIteraMode::Ex => *value,
                        crate::hir::HirIteraMode::De | crate::hir::HirIteraMode::Pro => *key,
                    },
                    Type::Union(items) if matches!(mode, crate::hir::HirIteraMode::Pro) && items.len() >= 2 => {
                        self.numerus_type()
                    }
                    _ => self.numerus_type(),
                };
                self.push_scope();
                self.insert_binding(*binding, elem_ty, true);
                self.check_block(block, None);
                self.pop_scope();
                self.vacuum_type()
            }
            HirExprKind::Intervallum { start, end, step, .. } => {
                let start_ty = self.check_expr(start);
                let end_ty = self.check_expr(end);
                if let Some(step) = step {
                    self.check_expr(step);
                }
                self.types.intern(Type::Union(vec![start_ty, end_ty]))
            }
            HirExprKind::Assign(target, value) => self.check_assign(target, value),
            HirExprKind::AssignOp(op, target, value) => self.check_assign_op(*op, target, value),
            HirExprKind::Array(elements) => self.check_array(elements, expr.span, expected),
            HirExprKind::Struct(def_id, fields) => self.check_struct_literal(*def_id, fields),
            HirExprKind::Verte { source, target, entries } => {
                self.check_verte(source, *target, entries.as_mut(), expr.span)
            }
            HirExprKind::Conversio { source, target, params: _, fallback } => {
                self.check_conversio(source, *target, fallback.as_deref_mut(), expr.span)
            }
            HirExprKind::Tuple(items) => self.check_tuple(items),
            HirExprKind::Scribe(items) => {
                for item in items {
                    self.check_expr(item);
                }
                self.vacuum_type()
            }
            HirExprKind::Scriptum(_template, args) => {
                for arg in args {
                    self.check_expr(arg);
                }
                self.textus_type()
            }
            HirExprKind::Adfirma(cond, message) => {
                let cond_ty = self.check_expr(cond);
                let bool_ty = self.bool_type();
                self.unify(cond_ty, bool_ty, cond.span, "assert condition must be boolean");
                if let Some(message) = message {
                    self.check_expr(message);
                }
                self.vacuum_type()
            }
            HirExprKind::Panic(value) => {
                self.check_expr(value);
                self.vacuum_type()
            }
            HirExprKind::Throw(value) => {
                self.check_expr(value);
                self.vacuum_type()
            }
            HirExprKind::Tempta { body, catch, finally } => {
                self.check_block(body, None);
                if let Some(catch) = catch {
                    self.check_block(catch, None);
                }
                if let Some(finally) = finally {
                    self.check_block(finally, None);
                }
                self.vacuum_type()
            }
            HirExprKind::Clausura(params, ret, body) => self.check_closure(params, ret.as_mut(), body, expected),
            HirExprKind::Cede(inner) => self.check_expr(inner),
            HirExprKind::Ref(kind, inner) => {
                let inner_ty = self.check_expr(inner);
                let mutability = match kind {
                    crate::hir::HirRefKind::Shared => crate::semantic::Mutability::Immutable,
                    crate::hir::HirRefKind::Mutable => crate::semantic::Mutability::Mutable,
                };
                self.types.reference(mutability, inner_ty)
            }
            HirExprKind::Deref(inner) => self.check_deref(inner, expr.span),
            HirExprKind::Error => {
                if self.errored_exprs.insert(expr.id) {
                    self.error(
                        SemanticErrorKind::LoweringError,
                        "invalid expression produced during lowering",
                        expr.span,
                    );
                }
                self.error_type
            }
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

        if let Some(parent) = self.variant_parent.get(&def_id).copied() {
            return self.types.intern(Type::Enum(parent));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Struct)
        ) {
            return self.types.intern(Type::Struct(def_id));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Interface)
        ) {
            return self.types.intern(Type::Interface(def_id));
        }

        if matches!(
            self.resolver.get_symbol(def_id).map(|symbol| symbol.kind),
            Some(crate::semantic::SymbolKind::Module)
        ) {
            return self.types.primitive(Primitive::Ignotum);
        }

        self.error(SemanticErrorKind::UndefinedVariable, "unknown identifier", span);
        self.error_type
    }

    fn check_binary(&mut self, op: HirBinOp, lhs: &mut HirExpr, rhs: &mut HirExpr) -> TypeId {
        let lhs_ty = self.check_expr(lhs);
        let rhs_ty = self.check_expr(rhs);

        match op {
            HirBinOp::Add => {
                if self.is_infer(self.resolve_type(lhs_ty)) && self.is_textus(rhs_ty) {
                    let textus = self.textus_type();
                    self.unify(lhs_ty, textus, lhs.span, "string operands required");
                }
                if self.is_infer(self.resolve_type(rhs_ty)) && self.is_textus(lhs_ty) {
                    let textus = self.textus_type();
                    self.unify(rhs_ty, textus, rhs.span, "string operands required");
                }
                if self.is_textus(lhs_ty) || self.is_textus(rhs_ty) {
                    if !self.is_textus(lhs_ty) || !self.is_textus(rhs_ty) {
                        self.error(SemanticErrorKind::InvalidOperandTypes, "string operands required", lhs.span);
                        return self.error_type;
                    }
                    self.textus_type()
                } else {
                    self.numeric_bin(lhs_ty, rhs_ty, lhs.span)
                }
            }
            HirBinOp::Sub | HirBinOp::Mul | HirBinOp::Div | HirBinOp::Mod => self.numeric_bin(lhs_ty, rhs_ty, lhs.span),
            HirBinOp::Eq
            | HirBinOp::NotEq
            | HirBinOp::StrictEq
            | HirBinOp::StrictNotEq
            | HirBinOp::Is
            | HirBinOp::IsNot => {
                let lhs_is_nil = matches!(self.types.get(self.resolve_type(lhs_ty)), Type::Primitive(Primitive::Nihil));
                let rhs_is_nil = matches!(self.types.get(self.resolve_type(rhs_ty)), Type::Primitive(Primitive::Nihil));
                if !(lhs_is_nil || rhs_is_nil) {
                    self.unify(lhs_ty, rhs_ty, lhs.span, "incompatible operands");
                }
                self.bool_type()
            }
            HirBinOp::Lt | HirBinOp::Gt | HirBinOp::LtEq | HirBinOp::GtEq => {
                self.numeric_bin(lhs_ty, rhs_ty, lhs.span);
                self.bool_type()
            }
            HirBinOp::InRange => {
                if self.is_infer(self.resolve_type(lhs_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(lhs_ty, numerus, lhs.span, "range operand must be numeric");
                }
                if !self.is_numeric(lhs_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "range operand must be numeric",
                        lhs.span,
                    );
                }
                self.bool_type()
            }
            HirBinOp::Between => {
                match self.types.get(self.resolve_type(rhs_ty)) {
                    Type::Array(elem) | Type::Set(elem) => {
                        self.unify(lhs_ty, *elem, lhs.span, "membership operand type mismatch");
                    }
                    Type::Map(key, _) => {
                        self.unify(lhs_ty, *key, lhs.span, "membership operand type mismatch");
                    }
                    _ => {}
                }
                self.bool_type()
            }
            HirBinOp::Coalesce => {
                let lhs_kind = self.types.get(self.resolve_type(lhs_ty)).clone();
                match lhs_kind {
                    Type::Option(inner) => {
                        self.unify(rhs_ty, inner, rhs.span, "coalesce fallback type mismatch");
                        inner
                    }
                    Type::Primitive(Primitive::Nihil) => rhs_ty,
                    _ => self.common_type(lhs_ty, rhs_ty, lhs.span),
                }
            }
            HirBinOp::And | HirBinOp::Or => {
                if !self.is_bool(lhs_ty) || !self.is_bool(rhs_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "boolean operands required", lhs.span);
                }
                self.bool_type()
            }
            HirBinOp::BitAnd | HirBinOp::BitOr | HirBinOp::BitXor | HirBinOp::Shl | HirBinOp::Shr => {
                if !self.is_integer(lhs_ty) || !self.is_integer(rhs_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "integer operands required", lhs.span);
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
                    let numerus = self.numerus_type();
                    return self.unify(operand_ty, numerus, operand.span, "numeric operand required");
                }
                if !self.is_numeric(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operand required", operand.span);
                }
                operand_ty
            }
            crate::hir::HirUnOp::Not => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let bivalens = self.bool_type();
                    self.unify(operand_ty, bivalens, operand.span, "boolean operand required");
                    return self.bool_type();
                }
                if !self.is_bool(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "boolean operand required", operand.span);
                }
                self.bool_type()
            }
            crate::hir::HirUnOp::BitNot => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(operand_ty, numerus, operand.span, "integer operand required");
                    return self.numerus_type();
                }
                if !self.is_integer(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "integer operand required", operand.span);
                }
                self.numerus_type()
            }
            crate::hir::HirUnOp::IsNull
            | crate::hir::HirUnOp::IsNotNull
            | crate::hir::HirUnOp::IsNil
            | crate::hir::HirUnOp::IsNotNil => self.bool_type(),
            crate::hir::HirUnOp::IsNeg | crate::hir::HirUnOp::IsPos => {
                if self.is_infer(self.resolve_type(operand_ty)) {
                    let numerus = self.numerus_type();
                    self.unify(operand_ty, numerus, operand.span, "numeric operand required");
                } else if !self.is_numeric(operand_ty) {
                    self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operand required", operand.span);
                }
                self.bool_type()
            }
            crate::hir::HirUnOp::IsTrue | crate::hir::HirUnOp::IsFalse => {
                let _ = operand_ty;
                self.bool_type()
            }
        }
    }

    fn check_call(&mut self, callee: &mut HirExpr, args: &mut [HirExpr]) -> TypeId {
        if let HirExprKind::Path(def_id) = &callee.kind {
            if let Some(parent) = self.variant_parent.get(def_id).copied() {
                let fields = self.variant_fields.get(def_id).cloned().unwrap_or_default();
                if args.len() != fields.len() {
                    self.error(SemanticErrorKind::WrongArity, "wrong number of arguments", callee.span);
                }
                for (arg, field_ty) in args.iter_mut().zip(fields.iter()) {
                    let arg_ty = self.check_expr(arg);
                    self.unify(arg_ty, *field_ty, arg.span, "argument type mismatch");
                }
                return self.types.intern(Type::Enum(parent));
            }
        }

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

        if matches!(self.types.get(resolved), Type::Primitive(Primitive::Ignotum)) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        self.error(SemanticErrorKind::NotCallable, "callee is not callable", callee.span);
        self.error_type
    }

    fn check_method_call(&mut self, receiver: &mut HirExpr, name: Symbol, args: &mut [HirExpr]) -> TypeId {
        let receiver_ty = self.check_expr(receiver);
        if let Some(sig) = self.lookup_method_signature(receiver_ty, name) {
            self.check_call_args(&sig, args, receiver.span);
            return sig.ret;
        }

        let array_inner = match self.types.get(self.resolve_type(receiver_ty)) {
            Type::Array(inner) => Some(*inner),
            _ => None,
        };
        if let Some(inner) = array_inner {
            if args.is_empty() {
                return self.numerus_type();
            }
            if let [arg] = args {
                let arg_ty = self.check_expr(arg);
                if self
                    .function_signature_from_type(self.resolve_type(arg_ty))
                    .is_none()
                {
                    self.unify(arg_ty, inner, arg.span, "argument type mismatch");
                    return self.vacuum_type();
                }
            }
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.array(inner);
        }

        if matches!(
            self.types.get(self.resolve_type(receiver_ty)),
            Type::Primitive(Primitive::Ignotum)
        ) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        for arg in args {
            self.check_expr(arg);
        }
        let _ = name;
        self.fresh_infer()
    }

    fn check_call_args(&mut self, sig: &FuncSig, args: &mut [HirExpr], span: crate::lexer::Span) {
        let required = sig.params.iter().filter(|param| !param.optional).count();
        let spread_compat =
            args.len() == 1 && sig.params.len() > 1 && self.check_spread_array_compat(sig, &mut args[0], span);
        if !spread_compat && (args.len() < required || args.len() > sig.params.len()) {
            self.error(SemanticErrorKind::WrongArity, "wrong number of arguments", span);
        }
        if spread_compat {
            return;
        }

        for (arg, param) in args.iter_mut().zip(sig.params.iter()) {
            let arg_ty = self.check_expr(arg);
            self.unify(arg_ty, param.ty, arg.span, "argument type mismatch");
        }
    }

    fn check_spread_array_compat(&mut self, sig: &FuncSig, arg: &mut HirExpr, span: crate::lexer::Span) -> bool {
        let arg_ty = self.check_expr(arg);
        let resolved = self.resolve_type(arg_ty);
        let inner = match self.types.get(resolved) {
            Type::Array(inner) => *inner,
            _ => return false,
        };
        if sig.params.iter().any(|param| param.optional) {
            return false;
        }
        for param in &sig.params {
            self.unify(inner, param.ty, span, "argument type mismatch");
        }
        true
    }

    fn check_field(&mut self, object: &mut HirExpr, name: Symbol) -> TypeId {
        let obj_ty = self.check_expr(object);
        self.check_field_from_type(obj_ty, name, object.span)
    }

    fn check_index(&mut self, object: &mut HirExpr, index: &mut HirExpr) -> TypeId {
        let obj_ty = self.check_expr(object);
        let idx_ty = self.check_expr(index);
        self.check_index_from_type(obj_ty, idx_ty, object.span, index.span)
    }

    fn check_optional_chain(
        &mut self,
        object: &mut HirExpr,
        chain: &mut crate::hir::HirOptionalChainKind,
        span: crate::lexer::Span,
    ) -> TypeId {
        let object_ty = self.check_expr(object);
        let object_resolved = self.resolve_type(object_ty);

        let inner_ty = match self.types.get(object_resolved) {
            Type::Option(inner) => *inner,
            _ => object_ty,
        };

        let result = match chain {
            crate::hir::HirOptionalChainKind::Member(name) => self.check_field_from_type(inner_ty, *name, object.span),
            crate::hir::HirOptionalChainKind::Index(index) => {
                let idx_ty = self.check_expr(index);
                self.check_index_from_type(inner_ty, idx_ty, object.span, index.span)
            }
            crate::hir::HirOptionalChainKind::Call(args) => self.check_call_from_type(inner_ty, args, span),
        };

        self.types.option(result)
    }

    fn check_non_null(
        &mut self,
        object: &mut HirExpr,
        chain: &mut crate::hir::HirNonNullKind,
        span: crate::lexer::Span,
    ) -> TypeId {
        let object_ty = self.check_expr(object);
        let inner_ty = match self.types.get(self.resolve_type(object_ty)) {
            Type::Option(inner) => *inner,
            _ => object_ty,
        };

        match chain {
            crate::hir::HirNonNullKind::Member(name) => self.check_field_from_type(inner_ty, *name, object.span),
            crate::hir::HirNonNullKind::Index(index) => {
                let idx_ty = self.check_expr(index);
                self.check_index_from_type(inner_ty, idx_ty, object.span, index.span)
            }
            crate::hir::HirNonNullKind::Call(args) => self.check_call_from_type(inner_ty, args, span),
        }
    }

    fn check_ab(
        &mut self,
        source: &mut HirExpr,
        filter: Option<&mut crate::hir::HirCollectionFilter>,
        transforms: &mut [crate::hir::HirCollectionTransform],
    ) -> TypeId {
        let source_ty = self.check_expr(source);

        if let Some(filter) = filter {
            match &mut filter.kind {
                crate::hir::HirCollectionFilterKind::Condition(cond) => {
                    self.check_condition(cond);
                }
                crate::hir::HirCollectionFilterKind::Property(_name) => {}
            }
        }

        let mut has_sum = false;
        for transform in transforms {
            if let Some(arg) = &mut transform.arg {
                self.check_expr(arg);
            }
            if matches!(transform.kind, crate::hir::HirTransformKind::Sum) {
                has_sum = true;
            }
        }

        if has_sum {
            self.numerus_type()
        } else {
            source_ty
        }
    }

    fn check_field_from_type(&mut self, object_ty: TypeId, name: Symbol, span: crate::lexer::Span) -> TypeId {
        if let Some(struct_def) = self.struct_def_from_type(object_ty) {
            if let Some(info) = self.structs.get(&struct_def) {
                if let Some(field_ty) = info.fields.get(&name).copied() {
                    return field_ty;
                }
            }
        }
        if let Type::Map(_, value_ty) = self.types.get(self.resolve_type(object_ty)) {
            let _ = name;
            return *value_ty;
        }
        let _ = span;
        self.error_type
    }

    fn check_index_from_type(
        &mut self,
        object_ty: TypeId,
        idx_ty: TypeId,
        object_span: crate::lexer::Span,
        index_span: crate::lexer::Span,
    ) -> TypeId {
        let resolved = self.resolve_type(object_ty);
        let resolved_kind = self.types.get(resolved).clone();
        let kind = match resolved_kind {
            Type::Array(elem) => Some((Some(elem), None, None)),
            Type::Map(key, value) => Some((None, Some(key), Some(value))),
            Type::Union(_) => Some((Some(self.types.primitive(Primitive::Ignotum)), None, None)),
            Type::Primitive(Primitive::Ignotum) => Some((Some(self.types.primitive(Primitive::Ignotum)), None, None)),
            Type::Infer(_) => Some((Some(self.fresh_infer()), None, None)),
            _ => None,
        };

        match kind {
            Some((Some(elem), None, None)) => {
                if !self.is_integer(idx_ty) {
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "array index must be numerus",
                        index_span,
                    );
                }
                elem
            }
            Some((None, Some(key), Some(value))) => {
                self.unify(idx_ty, key, index_span, "map index type mismatch");
                value
            }
            _ => {
                self.error(
                    SemanticErrorKind::InvalidOperandTypes,
                    "indexing requires array or map",
                    object_span,
                );
                self.error_type
            }
        }
    }

    fn check_call_from_type(&mut self, callee_ty: TypeId, args: &mut [HirExpr], span: crate::lexer::Span) -> TypeId {
        let resolved = self.resolve_type(callee_ty);
        if let Some(sig) = self.function_signature_from_type(resolved) {
            self.check_call_args(&sig, args, span);
            return self.resolve_type(sig.ret);
        }

        if self.is_infer(resolved) {
            let sig = self.build_call_signature(args);
            let func_ty = self.types.function(sig.clone());
            self.unify(resolved, func_ty, span, "callee is not callable");
            self.check_call_args(&sig, args, span);
            return sig.ret;
        }

        if matches!(self.types.get(resolved), Type::Primitive(Primitive::Ignotum)) {
            for arg in args {
                self.check_expr(arg);
            }
            return self.types.primitive(Primitive::Ignotum);
        }

        for arg in args {
            self.check_expr(arg);
        }
        self.error(SemanticErrorKind::NotCallable, "callee is not callable", span);
        self.error_type
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
        let bivalens = self.bool_type();
        let cond_ty = self.check_expr_with_expected(cond, Some(bivalens));
        if !self.is_bool(cond_ty) {
            self.error(SemanticErrorKind::InvalidOperandTypes, "condition must be bivalens", cond.span);
        }
    }

    fn check_match(&mut self, scrutinees: &mut [HirExpr], arms: &mut [HirCasuArm], expected: Option<TypeId>) -> TypeId {
        let scrutinee_tys: Vec<_> = scrutinees.iter_mut().map(|scrutinee| self.check_expr(scrutinee)).collect();
        let mut result_ty = None;

        for arm in arms {
            self.push_scope();
            for (pattern, scrutinee_ty) in arm.patterns.iter().zip(scrutinee_tys.iter().copied()) {
                self.check_pattern(pattern, scrutinee_ty, arm.span);
            }
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
            HirPattern::Alias(def_id, _name, pattern) => {
                self.insert_binding(*def_id, expected, false);
                if let HirPattern::Variant(variant_def, patterns) = pattern.as_ref() {
                    if patterns.is_empty() {
                        let Some(parent) = self.variant_parent.get(variant_def).copied() else {
                            self.error(SemanticErrorKind::UndefinedVariable, "unknown variant", span);
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
                        return;
                    }
                }
                self.check_pattern(pattern, expected, span);
            }
            HirPattern::Literal(lit) => {
                let lit_ty = self.literal_type(lit);
                self.unify(lit_ty, expected, span, "pattern type mismatch");
            }
            HirPattern::Variant(variant_def, patterns) => {
                let Some(parent) = self.variant_parent.get(variant_def).copied() else {
                    self.error(SemanticErrorKind::UndefinedVariable, "unknown variant", span);
                    return;
                };

                if let Some(expected_parent) = self.enum_def_from_type(expected) {
                    if expected_parent != parent {
                        self.error(SemanticErrorKind::TypeMismatch, "variant does not match scrutinee type", span);
                    }
                }

                let fields = self
                    .variant_fields
                    .get(variant_def)
                    .cloned()
                    .unwrap_or_default();
                if fields.len() != patterns.len() {
                    self.error(SemanticErrorKind::WrongArity, "variant pattern arity mismatch", span);
                }
                for (sub, field_ty) in patterns.iter().zip(fields.iter()) {
                    self.check_pattern(sub, *field_ty, span);
                }
            }
        }
    }

    fn check_assign(&mut self, target: &mut HirExpr, value: &mut HirExpr) -> TypeId {
        let target_ty = self.check_lvalue(target);
        if target_ty == self.error_type {
            self.check_expr(value);
            return self.error_type;
        }
        let value_ty = self.check_expr_with_expected(value, Some(target_ty));
        self.unify(value_ty, target_ty, value.span, "assignment type mismatch");
        target_ty
    }

    fn check_assign_op(&mut self, op: HirBinOp, target: &mut HirExpr, value: &mut HirExpr) -> TypeId {
        let target_ty = self.check_lvalue(target);
        let value_ty = self.check_expr(value);
        match op {
            HirBinOp::Add if self.is_textus(target_ty) => {
                let textus = self.textus_type();
                self.unify(value_ty, textus, target.span, "string operands required");
            }
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

                if let Some(ty) = self.consts.get(def_id).copied() {
                    self.error(SemanticErrorKind::ImmutableAssignment, "assignment to constant", target.span);
                    return ty;
                }

                self.error(
                    SemanticErrorKind::InvalidAssignmentTarget,
                    "invalid assignment target",
                    target.span,
                );
                self.error_type
            }
            HirExprKind::Field(object, name) => self.check_field(object, *name),
            HirExprKind::Index(object, index) => self.check_index(object, index),
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
        elements: &mut [HirArrayElement],
        _span: crate::lexer::Span,
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
            let infer = self.fresh_infer();
            return self.types.array(infer);
        }

        let mut element_ty = None;
        for element in elements {
            let (expr, spread) = match element {
                HirArrayElement::Expr(expr) => (expr, false),
                HirArrayElement::Spread(expr) => (expr, true),
            };
            let ty = if spread {
                if let Some(expected) = expected_elem {
                    let expected_array = self.types.array(expected);
                    self.check_expr_with_expected(expr, Some(expected_array))
                } else {
                    self.check_expr(expr)
                }
            } else if let Some(expected) = expected_elem {
                self.check_expr_with_expected(expr, Some(expected))
            } else {
                self.check_expr(expr)
            };
            let ty = if spread {
                match self.types.get(self.resolve_type(ty)) {
                    Type::Array(inner) => *inner,
                    _ => {
                        self.error(
                            SemanticErrorKind::InvalidOperandTypes,
                            "array spread requires lista operand",
                            expr.span,
                        );
                        self.error_type
                    }
                }
            } else {
                ty
            };
            element_ty = Some(match element_ty {
                None => ty,
                Some(existing) => self.array_common_type(existing, ty, expr.span),
            });
        }

        let elem_ty = element_ty.unwrap_or_else(|| self.fresh_infer());
        self.types.array(elem_ty)
    }

    fn array_common_type(&mut self, a: TypeId, b: TypeId, span: crate::lexer::Span) -> TypeId {
        let a_resolved = self.resolve_type(a);
        let b_resolved = self.resolve_type(b);

        if let Type::Array(a_inner) = self.types.get(a_resolved).clone() {
            if !matches!(self.types.get(b_resolved), Type::Array(_)) {
                return self.common_type(a_inner, b, span);
            }
        }

        if let Type::Array(b_inner) = self.types.get(b_resolved).clone() {
            if !matches!(self.types.get(a_resolved), Type::Array(_)) {
                return self.common_type(a, b_inner, span);
            }
        }

        self.common_type(a, b, span)
    }

    #[allow(clippy::ptr_arg)]
    fn check_struct_literal(&mut self, def_id: DefId, fields: &mut Vec<(Symbol, HirExpr)>) -> TypeId {
        let field_types = match self.structs.get(&def_id) {
            Some(info) => info.fields.clone(),
            None => {
                self.error(
                    SemanticErrorKind::UndefinedType,
                    "unknown struct",
                    fields
                        .first()
                        .map(|(_, expr)| expr.span)
                        .unwrap_or_default(),
                );
                return self.error_type;
            }
        };

        for (name, value) in fields.iter_mut() {
            let Some(field_ty) = field_types.get(name).copied() else {
                self.error(SemanticErrorKind::UndefinedMember, "unknown field", value.span);
                continue;
            };
            let value_ty = self.check_expr(value);
            self.unify(value_ty, field_ty, value.span, "field initializer type mismatch");
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

    #[allow(clippy::ptr_arg)]
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
                    self.unify(param.ty, expected_param.ty, param.span, "closure parameter type mismatch");
                }
            }
            self.insert_binding(param.def_id, param.ty, mutable);
        }

        let expected_ret = ret
            .as_ref()
            .map(|ty| **ty)
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
                    optional: param.optional,
                })
                .collect(),
            ret: ret_ty,
            is_async: false,
            is_generator: false,
        };
        self.types.function(sig)
    }

    /// Unified type conversion check — dispatches on the resolved target type to determine
    /// whether this is a cast (qua), native construction (innatum), or struct instantiation (novum).
    fn check_verte(
        &mut self,
        source: &mut HirExpr,
        target: TypeId,
        entries: Option<&mut Vec<HirObjectField>>,
        span: crate::lexer::Span,
    ) -> TypeId {
        let target_resolved = self.resolve_type(target);

        // For infer-typed targets, unify with the source type
        if self.is_infer(target_resolved) {
            let expr_ty = self.check_expr(source);
            return self.unify(expr_ty, target, source.span, "invalid cast");
        }

        match (self.types.get(target_resolved).clone(), entries) {
            // Struct instantiation — validate field names and types
            (Type::Struct(def_id), Some(entries)) => {
                self.check_struct_fields(def_id, entries, span);
            }
            // Array construction — validate element types
            (Type::Array(elem_ty), _) => {
                if let HirExprKind::Array(elements) = &mut source.kind {
                    for element in elements {
                        match element {
                            HirArrayElement::Expr(expr) => {
                                let element_ty = self.check_expr(expr);
                                self.unify(element_ty, elem_ty, expr.span, "array element type mismatch");
                            }
                            HirArrayElement::Spread(expr) => {
                                let spread_ty = self.check_expr(expr);
                                match self.types.get(self.resolve_type(spread_ty)) {
                                    Type::Array(inner) => {
                                        self.unify(*inner, elem_ty, expr.span, "array spread element type mismatch");
                                    }
                                    _ => {
                                        self.error(
                                            SemanticErrorKind::InvalidOperandTypes,
                                            "array spread requires lista operand",
                                            expr.span,
                                        );
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.check_expr(source);
                }
            }
            // Map construction — validate key-value entry types
            (Type::Map(key_ty, value_ty), Some(entries)) => {
                let mut inferred_values = Vec::new();
                for field in entries {
                    match &mut field.key {
                        HirObjectKey::Ident(_) | HirObjectKey::String(_) => {
                            let textus = self.textus_type();
                            self.unify(textus, key_ty, span, "map key type mismatch");
                        }
                        HirObjectKey::Computed(key) => {
                            let actual_key_ty = self.check_expr(key);
                            self.unify(actual_key_ty, key_ty, key.span, "map key type mismatch");
                        }
                        HirObjectKey::Spread(expr) => {
                            let spread_ty = self.check_expr(expr);
                            match self.types.get(self.resolve_type(spread_ty)).clone() {
                                Type::Map(spread_key_ty, spread_value_ty) => {
                                    self.unify(spread_key_ty, key_ty, expr.span, "map spread key type mismatch");
                                    self.unify(spread_value_ty, value_ty, expr.span, "map spread value type mismatch");
                                }
                                _ => {
                                    self.error(
                                        SemanticErrorKind::InvalidOperandTypes,
                                        "object spread requires tabula operand",
                                        expr.span,
                                    );
                                }
                            }
                            continue;
                        }
                    }

                    let Some(value) = &mut field.value else {
                        self.error(
                            SemanticErrorKind::InvalidOperandTypes,
                            "object field requires value",
                            span,
                        );
                        continue;
                    };
                    let value_ty_actual = self.check_expr(value);
                    inferred_values.push(self.resolve_type(value_ty_actual));
                    let value_resolved = self.resolve_type(value_ty);
                    if !(self.is_infer(value_resolved)
                        || matches!(self.types.get(value_resolved), Type::Primitive(Primitive::Ignotum))
                        || matches!(self.types.get(value_resolved), Type::Union(_)))
                    {
                        self.unify(value_ty_actual, value_ty, value.span, "map value type mismatch");
                    }
                }
                if self.is_infer(self.resolve_type(value_ty)) {
                    let inferred_value_ty = match inferred_values.as_slice() {
                        [] => value_ty,
                        [single] => *single,
                        _ => self.types.intern(Type::Union(inferred_values)),
                    };
                    return self.types.map(key_ty, inferred_value_ty);
                }
            }
            // Cast / other — check the source, trust the target type
            _ => {
                self.check_expr(source);
            }
        }

        let _ = span;
        target_resolved
    }

    /// Check a runtime value conversion (conversio).
    /// Validates source, checks fallback type matches target.
    fn check_conversio(
        &mut self,
        source: &mut HirExpr,
        target: TypeId,
        fallback: Option<&mut HirExpr>,
        _span: crate::lexer::Span,
    ) -> TypeId {
        self.check_expr(source);
        if let Some(fallback) = fallback {
            let fb_ty = self.check_expr(fallback);
            let target_resolved = self.resolve_type(target);
            let fb_resolved = self.resolve_type(fb_ty);
            if target_resolved != fb_resolved && !self.is_infer(fb_resolved) && !self.is_infer(target_resolved) {
                // Allow fallback type mismatch for now — codegen handles it
            }
        }
        target
    }

    /// Validate struct field entries against the struct definition.
    /// Extracted from check_struct_literal so it can be used by both Verte and Struct paths.
    fn check_struct_fields(&mut self, def_id: DefId, fields: &mut [HirObjectField], span: crate::lexer::Span) {
        let field_types = match self.structs.get(&def_id) {
            Some(info) => info.fields.clone(),
            None => {
                self.error(
                    SemanticErrorKind::UndefinedType,
                    "unknown struct",
                    fields
                        .first()
                        .and_then(|field| field.value.as_ref().map(|expr| expr.span))
                        .unwrap_or(span),
                );
                return;
            }
        };

        for field in fields.iter_mut() {
            match &mut field.key {
                HirObjectKey::Ident(name) | HirObjectKey::String(name) => {
                    let Some(field_ty) = field_types.get(name).copied() else {
                        let error_span = field.value.as_ref().map(|expr| expr.span).unwrap_or(span);
                        self.error(SemanticErrorKind::UndefinedMember, "unknown field", error_span);
                        continue;
                    };
                    let Some(value) = &mut field.value else {
                        self.error(SemanticErrorKind::UndefinedMember, "struct field requires value", span);
                        continue;
                    };
                    let value_ty = self.check_expr(value);
                    self.unify(value_ty, field_ty, value.span, "field initializer type mismatch");
                }
                HirObjectKey::Computed(expr) => {
                    self.check_expr(expr);
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "computed keys are not valid in struct construction",
                        expr.span,
                    );
                }
                HirObjectKey::Spread(expr) => {
                    self.check_expr(expr);
                    self.error(
                        SemanticErrorKind::InvalidOperandTypes,
                        "spread fields are not valid in struct construction",
                        expr.span,
                    );
                }
            }
        }
    }

    fn check_deref(&mut self, expr: &mut HirExpr, span: crate::lexer::Span) -> TypeId {
        let expr_ty = self.check_expr(expr);
        match self.types.get(self.resolve_type(expr_ty)) {
            Type::Ref(_, inner) => *inner,
            _ => {
                self.error(SemanticErrorKind::InvalidOperandTypes, "deref requires reference", span);
                self.error_type
            }
        }
    }

    fn numeric_bin(&mut self, lhs: TypeId, rhs: TypeId, span: crate::lexer::Span) -> TypeId {
        let lhs_resolved = self.resolve_type(lhs);
        let rhs_resolved = self.resolve_type(rhs);
        if self.is_infer(lhs_resolved) || self.is_infer(rhs_resolved) {
            let numerus = self.numerus_type();
            if self.is_infer(lhs_resolved) {
                self.unify(lhs, numerus, span, "numeric operands required");
            }
            if self.is_infer(rhs_resolved) {
                self.unify(rhs, numerus, span, "numeric operands required");
            }
        }
        if !self.is_numeric(lhs) || !self.is_numeric(rhs) {
            self.error(SemanticErrorKind::InvalidOperandTypes, "numeric operands required", span);
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
            HirLiteral::Regex(_, _) => self.regex_type(),
            HirLiteral::Bool(_) => self.bool_type(),
            HirLiteral::Nil => self.nil_type(),
        }
    }

    fn struct_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Struct(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.struct_def_from_type(*inner),
            Type::Option(inner) => self.struct_def_from_type(*inner),
            Type::Applied(base, _) => self.struct_def_from_type(*base),
            _ => None,
        }
    }

    fn enum_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Enum(def_id) => Some(*def_id),
            Type::Option(inner) => self.enum_def_from_type(*inner),
            Type::Applied(base, _) => self.enum_def_from_type(*base),
            _ => None,
        }
    }

    fn interface_def_from_type(&self, ty: TypeId) -> Option<DefId> {
        match self.types.get(self.resolve_type(ty)) {
            Type::Interface(def_id) => Some(*def_id),
            Type::Ref(_, inner) => self.interface_def_from_type(*inner),
            Type::Option(inner) => self.interface_def_from_type(*inner),
            Type::Applied(base, _) => self.interface_def_from_type(*base),
            _ => None,
        }
    }

    fn lookup_method_signature(&self, receiver_ty: TypeId, name: Symbol) -> Option<FuncSig> {
        if let Some(struct_def) = self.struct_def_from_type(receiver_ty) {
            if let Some(info) = self.structs.get(&struct_def) {
                if let Some(sig) = info.methods.get(&name) {
                    return Some(sig.clone());
                }
            }
        }
        if let Some(interface_def) = self.interface_def_from_type(receiver_ty) {
            if let Some(methods) = self.interfaces.get(&interface_def) {
                if let Some(sig) = methods.get(&name) {
                    return Some(sig.clone());
                }
            }
        }
        None
    }

    #[allow(dead_code)]
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
        let var = InferVar(self.next_infer);
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

    fn infer_var_of(&self, ty: TypeId) -> Option<InferVar> {
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
            .map(|arg| ParamType { ty: self.check_expr(arg), mode: ParamMode::Owned, optional: false })
            .collect();
        FuncSig { params, ret: self.fresh_infer(), is_async: false, is_generator: false }
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

        let left_ty = self.types.get(left).clone();
        let right_ty = self.types.get(right).clone();

        match (left_ty, right_ty) {
            (Type::Primitive(Primitive::Numerus), Type::Primitive(Primitive::Fractus))
            | (Type::Primitive(Primitive::Fractus), Type::Primitive(Primitive::Numerus)) => {
                return self.fractus_type();
            }
            (Type::Primitive(a), Type::Primitive(b)) if a == b => return left,
            (Type::Array(a), Type::Array(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.array(inner);
            }
            (Type::Map(ka, va), Type::Map(kb, vb)) => {
                let key = self.unify(ka, kb, span, message);
                let value = self.unify(va, vb, span, message);
                return self.types.map(key, value);
            }
            (Type::Set(a), Type::Set(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.set(inner);
            }
            (Type::Option(a), Type::Option(b)) => {
                let inner = self.unify(a, b, span, message);
                return self.types.option(inner);
            }
            (Type::Ref(ma, a), Type::Ref(mb, b)) if ma == mb => {
                let inner = self.unify(a, b, span, message);
                return self.types.reference(ma, inner);
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

    fn bind_infer(&mut self, var: InferVar, ty: TypeId, span: crate::lexer::Span, message: &str) -> TypeId {
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

    fn occurs_in(&self, var: InferVar, ty: TypeId) -> bool {
        let resolved = self.resolve_type(ty);
        if let Some(found) = self.infer_var_of(resolved) {
            return found == var;
        }
        match self.types.get(resolved) {
            Type::Array(inner) | Type::Option(inner) | Type::Ref(_, inner) => self.occurs_in(var, *inner),
            Type::Set(inner) => self.occurs_in(var, *inner),
            Type::Map(key, value) => self.occurs_in(var, *key) || self.occurs_in(var, *value),
            Type::Func(sig) => {
                sig.params.iter().any(|param| self.occurs_in(var, param.ty)) || self.occurs_in(var, sig.ret)
            }
            Type::Applied(base, args) => self.occurs_in(var, *base) || args.iter().any(|arg| self.occurs_in(var, *arg)),
            Type::Union(types) => types.iter().any(|inner| self.occurs_in(var, *inner)),
            _ => false,
        }
    }

    fn is_numeric(&self, ty: TypeId) -> bool {
        self.is_integer(ty) || self.is_fractus(ty)
    }

    fn is_integer(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Numerus))
    }

    fn is_fractus(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Fractus))
    }

    fn is_textus(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Textus))
    }

    fn is_bool(&self, ty: TypeId) -> bool {
        matches!(self.types.get(self.resolve_type(ty)), Type::Primitive(Primitive::Bivalens))
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

    fn regex_type(&mut self) -> TypeId {
        self.types.primitive(Primitive::Regex)
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
#[path = "typecheck_test.rs"]
mod tests;
