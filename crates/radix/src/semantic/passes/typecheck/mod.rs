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

mod access;
mod aggregate;
mod call;
mod collect;
mod control;
mod convert;
mod expr;
mod finalize;
mod infer;
mod item;
mod lookup;
mod ops;
mod pattern;
mod stmt;

#[derive(Clone, Copy)]
struct BindingInfo {
    ty: TypeId,
    mutable: bool,
}

struct StructInfo {
    fields: FxHashMap<Symbol, StructFieldInfo>,
    methods: FxHashMap<Symbol, FuncSig>,
}

#[derive(Clone, Copy)]
struct StructFieldInfo {
    ty: TypeId,
    required: bool,
    span: crate::lexer::Span,
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

    fn check_program(&mut self, hir: &mut HirProgram) {
        for item in &mut hir.items {
            self.check_item(item);
        }

        if let Some(entry) = &mut hir.entry {
            self.check_block(entry, None);
        }

        self.finalize_hir(hir);
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
#[path = "../typecheck_test.rs"]
mod tests;
