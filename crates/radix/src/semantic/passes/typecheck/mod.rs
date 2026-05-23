//! HIR type inference and checking for Radix semantic analysis.
//!
//! This module owns the compiler phase that turns resolved HIR into typed HIR.
//! It runs after collection, name resolution, and AST-to-HIR lowering have
//! already assigned stable `DefId`s and lowered source type syntax into the
//! shared semantic [`TypeTable`]. Typecheck may synthesize additional table
//! entries while inferring expressions, but every `TypeId` it reads or writes is
//! table-local to the single `TypeTable` passed through semantic analysis.
//!
//! COMPILER PHASE: Semantic (Pass 4)
//! INPUT: HIR with resolved DefIds but no type information
//! OUTPUT: HIR with finalized expression/declaration `TypeId`s, or collected
//! diagnostics that explain why a type could not be trusted
//!
//! DESIGN PHILOSOPHY
//! =================
//! Typecheck is intentionally local and bidirectional. Expressions normally
//! synthesize a type from their children; surrounding constructs may also pass an
//! expected type downward so empty collections, call arguments, blocks, and
//! branch bodies can be checked against declared context. This is not HM-style
//! whole-program inference: function parameters and other boundary contracts
//! remain explicit so diagnostics point at the source contract that failed.
//!
//! INFERENCE MODEL
//! ===============
//! Fresh [`InferVar`] entries stand for unknown expression types during a single
//! `TypeChecker` run. They live in the shared type table as ordinary `TypeId`s,
//! but the authoritative substitution map is held by `TypeChecker`; callers must
//! use `resolve_type` before making policy decisions. Finalization walks the HIR
//! after checking and replaces resolved inference variables with concrete
//! `TypeId`s. Unresolved inference or error states produce diagnostics instead
//! of changing the language's nullability, escape, or coercion rules.
//!
//! NULLABILITY AND ESCAPES
//! =======================
//! `nihil` participates in optional and coalescing rules; it is not the same as
//! an unknown type. `ignotum` remains the explicit escape hatch for interop or
//! intentionally unchecked values. The checker follows [`TypeTable::assignable`]
//! for widening, optional acceptance, union membership, and the one-way
//! `ignotum` policy, then uses targeted operator/call/access checks for syntax
//! whose rules are stricter than general assignment.
//!
//! UNIFICATION
//! ===========
//! `unify(a, b)` is the central compatibility operation. It resolves existing
//! substitutions, binds inference variables with an occurs check, structurally
//! unifies collection/function/reference shapes, allows numeric widening and
//! assignment-compatible flows, and records a type mismatch when no rule applies.
//! It returns `Type::Error` through the table after hard failures so later checks
//! can continue and report additional source errors without trusting bad types.
//!
//! ERROR RECOVERY
//! ==============
//! Typecheck collects errors and keeps traversing. The checker uses a dedicated
//! table-local error type, remembers lowered error expressions so they are only
//! reported once, and finalizes unresolved inference variables as missing
//! annotations. This preserves the HIR shape for downstream diagnostics while
//! preventing codegen from treating speculative types as proven.

use crate::hir::{
    DefId, HirArrayElement, HirBinOp, HirBlock, HirCape, HirCasuArm, HirExpr, HirExprKind, HirFunction, HirId, HirItem,
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

/// Type information for a value currently visible in lexical scope.
#[derive(Clone, Copy)]
struct BindingInfo {
    /// Table-local type of the binding after declaration or pattern checking.
    ty: TypeId,

    /// Whether assignment through this binding is permitted.
    mutable: bool,
}

/// Struct member information collected before expression checking begins.
struct StructInfo {
    /// Field contracts keyed by lowered field symbol.
    fields: FxHashMap<Symbol, StructFieldInfo>,

    /// Method signatures keyed by method name; receiver handling is performed
    /// by lookup/call checking rather than encoded in the signature table.
    methods: FxHashMap<Symbol, FuncSig>,
}

/// Field contract used for struct literals and field access checks.
#[derive(Clone, Copy)]
struct StructFieldInfo {
    /// Declared field type in the shared type table.
    ty: TypeId,

    /// Whether a struct literal must provide this field explicitly.
    required: bool,

    /// Declaration span used when reporting missing required fields.
    span: crate::lexer::Span,
}

/// Current alternate-exit context for `iace` and handled failable calls.
#[derive(Clone, Copy)]
enum ErrorSink {
    /// Function-level alternate-exit type declared on the current function.
    Function(TypeId),

    /// Local handler type introduced by `fac/cape`, `tempta`, or handled blocks.
    Local(TypeId),
}

/// Mutable state for one HIR typecheck run.
///
/// The checker is both a visitor and an inference engine. Collection tables hold
/// declarations discovered before expression checking, while the scope stack
/// tracks lexical bindings introduced by blocks and patterns. Inference state is
/// deliberately separate from the shared type table: `Type::Infer` values are
/// stored in the table, but their substitutions are only meaningful through this
/// checker instance.
struct TypeChecker<'a> {
    /// Resolver snapshot from previous semantic passes. Kept with the checker so
    /// future lookup logic can consult resolved symbols without widening method
    /// signatures.
    #[allow(dead_code)]
    resolver: &'a Resolver,

    /// Shared semantic type arena. All `TypeId`s read or written here are local
    /// to this table and must not be compared against IDs from another run.
    types: &'a mut TypeTable,

    /// Hard typechecking diagnostics collected while traversal continues.
    errors: Vec<SemanticError>,

    /// Lexical binding stack keyed by resolver `DefId`, innermost scope last.
    scopes: Vec<FxHashMap<DefId, BindingInfo>>,

    /// Function signatures collected before body checking so calls can be
    /// checked regardless of declaration order.
    functions: FxHashMap<DefId, FuncSig>,

    /// Constant declaration types available to path lookup.
    consts: FxHashMap<DefId, TypeId>,

    /// Struct field and method contracts available to literals/access/calls.
    structs: FxHashMap<DefId, StructInfo>,

    /// Interface method contracts available to method lookup.
    interfaces: FxHashMap<DefId, FxHashMap<Symbol, FuncSig>>,

    /// Enum variant payload types keyed by variant definition.
    variant_fields: FxHashMap<DefId, Vec<TypeId>>,

    /// Parent enum for each variant constructor.
    variant_parent: FxHashMap<DefId, DefId>,

    /// Declared return type while checking a function body.
    current_return: Option<TypeId>,

    /// Active alternate-exit target while checking `iace` or failable calls.
    current_error: Option<ErrorSink>,

    /// Synthesized return type for functions without an explicit return type.
    inferred_return: Option<TypeId>,

    /// Monotonic counter for fresh inference variables within this checker.
    next_infer: u32,

    /// Reverse map from inference variable token to its table-local `TypeId`.
    infer_ids: FxHashMap<InferVar, TypeId>,

    /// Inference substitutions established by unification.
    substitutions: FxHashMap<InferVar, TypeId>,

    /// Lowering-produced error expressions already reported during checking.
    errored_exprs: FxHashSet<HirId>,

    /// Table-local sentinel used to keep traversing after hard failures.
    error_type: TypeId,
}

/// Typecheck resolved HIR and attach table-local types to declarations and expressions.
///
/// This entrypoint owns the full typecheck lifecycle: collect declaration
/// contracts, check item/entry bodies, then finalize HIR annotations by resolving
/// aliases and inference substitutions. It returns all hard diagnostics together
/// so callers can present a useful batch instead of stopping at the first
/// mismatch.
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
            current_error: None,
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
