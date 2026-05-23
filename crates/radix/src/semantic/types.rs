//! Semantic type model and shared type table.
//!
//! This module defines the compiler's semantic type vocabulary after parsing
//! has turned source type syntax into compiler data. The [`TypeTable`] is the
//! shared store passed through collection, resolution, lowering, typecheck, and
//! later analyses so every `TypeId` in a semantic result points into one table.
//!
//! TYPE TABLE ROLE
//! ===============
//! `TypeId` is an arena index, not a globally canonical type identity. Primitive
//! types are pre-seeded for stable access, while compound types are appended as
//! passes discover or synthesize them. Equality and assignability therefore use
//! structural comparison through the table rather than assuming matching IDs
//! for equivalent compound types.
//!
//! NULLABILITY AND ESCAPES
//! =======================
//! Nullable value types are represented as `Option(T)` or `Union([... Nihil])`
//! depending on the source construct and lowering path. `ignotum` is separate:
//! it is an escape hatch for unknown values, not the nullable marker.
//!
//! ASSIGNABILITY
//! =============
//! Assignment is intentionally more permissive than equality. It allows
//! numerus-to-fractus widening, values flowing into optional destinations, union
//! membership checks, and values flowing into `ignotum`. It does not allow
//! `ignotum` to flow out into a concrete type without an explicit narrowing or
//! cast elsewhere.

use crate::hir::DefId;
use crate::lexer::Symbol as LexSymbol;
use rustc_hash::FxHashMap;

/// Arena index into a [`TypeTable`].
///
/// A `TypeId` is meaningful only with the table that created it. Do not compare
/// IDs from different tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Per-analysis storage for semantic types.
///
/// The table preloads primitives so passes can cheaply request well-known
/// built-ins, then appends compound and inferred types as semantic information
/// becomes available. It deliberately exposes structural [`Self::equals`] and
/// [`Self::assignable`] helpers so callers do not need to know whether two
/// equivalent shapes were allocated at the same index.
pub struct TypeTable {
    types: Vec<Type>,
    primitives: FxHashMap<Primitive, TypeId>,
}

impl TypeTable {
    /// Build a type table with all primitive types available by name.
    pub fn new() -> Self {
        let mut table = Self { types: Vec::new(), primitives: FxHashMap::default() };

        for prim in [
            Primitive::Textus,
            Primitive::Numerus,
            Primitive::Fractus,
            Primitive::Bivalens,
            Primitive::Nihil,
            Primitive::Vacuum,
            Primitive::Numquam,
            Primitive::Ignotum,
            Primitive::Octeti,
            Primitive::Regex,
            Primitive::Valor,
        ] {
            let id = table.intern(Type::Primitive(prim));
            table.primitives.insert(prim, id);
        }

        table
    }

    /// Store a type in this table and return its local `TypeId`.
    pub fn intern(&mut self, ty: Type) -> TypeId {
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    /// Get a semantic type by table-local ID.
    pub fn get(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }

    /// Get the preloaded ID for a primitive type.
    pub fn primitive(&self, prim: Primitive) -> TypeId {
        self.primitives[&prim]
    }

    /// Allocate `T ∪ nihil` as an optional type.
    pub fn option(&mut self, inner: TypeId) -> TypeId {
        self.intern(Type::Option(inner))
    }

    /// Allocate a semantic reference type.
    pub fn reference(&mut self, mutability: Mutability, inner: TypeId) -> TypeId {
        self.intern(Type::Ref(mutability, inner))
    }

    /// Allocate a `lista<T>` semantic collection.
    pub fn array(&mut self, element: TypeId) -> TypeId {
        self.intern(Type::Array(element))
    }

    /// Allocate a `tabula<K, V>` semantic collection.
    pub fn map(&mut self, key: TypeId, value: TypeId) -> TypeId {
        self.intern(Type::Map(key, value))
    }

    /// Allocate a `copia<T>` semantic collection.
    pub fn set(&mut self, element: TypeId) -> TypeId {
        self.intern(Type::Set(element))
    }

    /// Allocate a function signature type.
    pub fn function(&mut self, sig: FuncSig) -> TypeId {
        self.intern(Type::Func(sig))
    }

    /// Return whether two table-local type IDs describe the same semantic shape.
    pub fn equals(&self, a: TypeId, b: TypeId) -> bool {
        if a == b {
            return true;
        }
        match (self.get(a), self.get(b)) {
            (Type::Primitive(pa), Type::Primitive(pb)) => pa == pb,
            (Type::Array(ae), Type::Array(be))
            | (Type::Option(ae), Type::Option(be))
            | (Type::Set(ae), Type::Set(be)) => self.equals(*ae, *be),
            (Type::Map(ak, av), Type::Map(bk, bv)) => self.equals(*ak, *bk) && self.equals(*av, *bv),
            (Type::Record(a_fields), Type::Record(b_fields)) => {
                a_fields.len() == b_fields.len()
                    && a_fields.iter().all(|(name, a_ty)| {
                        b_fields
                            .get(name)
                            .is_some_and(|b_ty| self.equals(*a_ty, *b_ty))
                    })
            }
            (Type::Ref(am, ai), Type::Ref(bm, bi)) => am == bm && self.equals(*ai, *bi),
            (Type::Struct(a_def), Type::Struct(b_def))
            | (Type::Enum(a_def), Type::Enum(b_def))
            | (Type::Interface(a_def), Type::Interface(b_def)) => a_def == b_def,
            (Type::Alias(_, a_inner), _) => self.equals(*a_inner, b),
            (_, Type::Alias(_, b_inner)) => self.equals(a, *b_inner),
            (Type::Func(a_sig), Type::Func(b_sig)) => {
                a_sig.params.len() == b_sig.params.len()
                    && a_sig
                        .params
                        .iter()
                        .zip(b_sig.params.iter())
                        .all(|(a_param, b_param)| {
                            a_param.mode == b_param.mode
                                && a_param.optional == b_param.optional
                                && self.equals(a_param.ty, b_param.ty)
                        })
                    && self.equals(a_sig.ret, b_sig.ret)
                    && match (a_sig.err, b_sig.err) {
                        (Some(a_err), Some(b_err)) => self.equals(a_err, b_err),
                        (None, None) => true,
                        _ => false,
                    }
                    && a_sig.is_async == b_sig.is_async
                    && a_sig.is_generator == b_sig.is_generator
            }
            (Type::Param(a_name), Type::Param(b_name)) => a_name == b_name,
            (Type::Applied(a_base, a_args), Type::Applied(b_base, b_args)) => {
                a_args.len() == b_args.len()
                    && self.equals(*a_base, *b_base)
                    && a_args
                        .iter()
                        .zip(b_args.iter())
                        .all(|(a_arg, b_arg)| self.equals(*a_arg, *b_arg))
            }
            (Type::Infer(a_var), Type::Infer(b_var)) => a_var == b_var,
            (Type::Union(a_types), Type::Union(b_types)) => {
                a_types.len() == b_types.len()
                    && a_types
                        .iter()
                        .zip(b_types.iter())
                        .all(|(a_ty, b_ty)| self.equals(*a_ty, *b_ty))
            }
            (Type::Error, Type::Error) => true,
            _ => false,
        }
    }

    /// Return whether a value of type `from` can be used where `to` is expected.
    ///
    /// This is a semantic compatibility relation, not identity. It encodes the
    /// language's implicit widening and optional/union acceptance rules while
    /// keeping `ignotum` one-way: concrete values may flow into it, but unknown
    /// values do not silently become concrete.
    pub fn assignable(&self, from: TypeId, to: TypeId) -> bool {
        if self.equals(from, to) {
            return true;
        }

        let from_ty = self.get(from);
        let to_ty = self.get(to);

        match (from_ty, to_ty) {
            // `ignotum` is an unknown sink: values can flow into it, but not out without cast/narrowing.
            (Type::Primitive(Primitive::Ignotum), _) => false,

            // Union source is assignable when every member can flow into destination.
            (Type::Union(from_types), _) => from_types.iter().all(|member| self.assignable(*member, to)),

            // Destination union accepts any source that can flow into at least one member.
            (_, Type::Union(to_types)) => to_types.iter().any(|member| self.assignable(from, *member)),

            // `nihil` can satisfy optional destinations without inventing a concrete inner value.
            (Type::Primitive(Primitive::Nihil), Type::Option(_)) => true,

            (Type::Option(from_inner), Type::Option(to_inner)) => self.assignable(*from_inner, *to_inner),

            (_, Type::Option(inner)) => self.assignable(from, *inner),

            // Numeric widening is implicit; narrowing remains an explicit operation.
            (Type::Primitive(Primitive::Numerus), Type::Primitive(Primitive::Fractus)) => true,

            (_, Type::Primitive(Primitive::Ignotum)) => true,

            _ => false,
        }
    }
}

impl Default for TypeTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Semantic type shapes understood by Radix after parsing.
///
/// Variants encode compiler meaning rather than one-to-one source spelling.
/// For example, collection names lower into structural collection variants,
/// and user-defined declarations are represented by `DefId` handles into the
/// resolver's symbol table.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Built-in scalar or special type.
    Primitive(Primitive),

    /// `lista<T>` collection.
    Array(TypeId),

    /// `tabula<K, V>` collection.
    Map(TypeId, TypeId),

    /// Structural record type used for compiler-synthesized contracts.
    Record(FxHashMap<LexSymbol, TypeId>),

    /// `copia<T>` collection.
    Set(TypeId),

    /// Optional value type written as `T ∪ nihil` in Faber type positions.
    Option(TypeId),

    /// Reference type used by parameter-passing and borrow analysis.
    Ref(Mutability, TypeId),

    /// User-defined struct declaration.
    Struct(DefId),

    /// User-defined enum declaration.
    Enum(DefId),

    /// User-defined interface declaration.
    Interface(DefId),

    /// Resolved type alias retaining the alias declaration identity.
    Alias(DefId, TypeId),

    /// Callable signature.
    Func(FuncSig),

    /// Generic type parameter that has not been substituted.
    Param(LexSymbol),

    /// Generic type applied to concrete type arguments.
    Applied(TypeId, Vec<TypeId>),

    /// Placeholder introduced during type inference.
    Infer(InferVar),

    /// Explicit union of possible value types.
    Union(Vec<TypeId>),

    /// Recovery sentinel used after an earlier type error.
    Error,
}

/// Built-in primitive and special semantic types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    /// Text/string value.
    Textus,

    /// Integer number.
    Numerus,

    /// Floating-point number.
    Fractus,

    /// Boolean truth value.
    Bivalens,

    /// Null value; nullable slots are modeled with option/union types.
    Nihil,

    /// No returned value.
    Vacuum,

    /// Never-returning control-flow type.
    Numquam,

    /// Unknown escape type for interop or intentionally unchecked values.
    Ignotum,

    /// Raw byte buffer.
    Octeti,

    /// Regular-expression literal value.
    Regex,

    /// Canonical dynamic data value for data formats, backed by `norma::datum::Valor`.
    Valor,
}

impl Primitive {
    /// Resolve a source-level primitive spelling to its semantic primitive.
    ///
    /// `objectum` and `quidlibet` remain accepted aliases for the unknown
    /// escape type; they do not introduce separate semantic types.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "textus" => Some(Self::Textus),
            "numerus" => Some(Self::Numerus),
            "fractus" => Some(Self::Fractus),
            "bivalens" => Some(Self::Bivalens),
            "nihil" => Some(Self::Nihil),
            "vacuum" => Some(Self::Vacuum),
            "numquam" => Some(Self::Numquam),
            "ignotum" => Some(Self::Ignotum),
            "octeti" => Some(Self::Octeti),
            "regex" => Some(Self::Regex),
            "objectum" | "quidlibet" => Some(Self::Ignotum),
            "valor" => Some(Self::Valor),
            _ => None,
        }
    }
}

/// Source-level collection families with fixed generic arity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionKind {
    Lista,
    Tabula,
    Copia,
}

impl CollectionKind {
    /// Resolve a Faber collection type name.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "lista" => Some(Self::Lista),
            "tabula" => Some(Self::Tabula),
            "copia" => Some(Self::Copia),
            _ => None,
        }
    }

    /// Number of type arguments required by this collection family.
    pub fn arity(self) -> usize {
        match self {
            Self::Lista | Self::Copia => 1,
            Self::Tabula => 2,
        }
    }

    /// Diagnostic text for incorrect generic arity.
    pub fn arity_error(self) -> &'static str {
        match self {
            Self::Lista => "lista requires one type parameter",
            Self::Tabula => "tabula requires two type parameters",
            Self::Copia => "copia requires one type parameter",
        }
    }

    /// Lower collection type arguments into the corresponding semantic type.
    pub fn lower(self, types: &mut TypeTable, params: &[TypeId]) -> TypeId {
        debug_assert_eq!(params.len(), self.arity());
        match self {
            Self::Lista => types.array(params[0]),
            Self::Tabula => types.map(params[0], params[1]),
            Self::Copia => types.set(params[0]),
        }
    }
}

/// Mutability carried by semantic reference types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Semantic callable contract.
///
/// The signature captures parameter passing modes, return type, optional error
/// type, and async/generator modifiers so backends and analyses can reason
/// about calls without re-reading syntax annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct FuncSig {
    pub params: Vec<ParamType>,
    pub ret: TypeId,
    pub err: Option<TypeId>,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Type and passing policy for one function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamType {
    pub ty: TypeId,
    pub mode: ParamMode,
    pub optional: bool,
}

/// Parameter ownership/borrowing mode as seen by semantic analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Owned,
    Ref,
    MutRef,
    Move,
}

/// Identifier for a type inference placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferVar(pub u32);

#[cfg(test)]
#[path = "types_test.rs"]
mod tests;
