//! Type system
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Implements the semantic type system with type interning, assignability
//! checking, and support for primitives, collections, functions, and user-defined
//! types (structs, enums, interfaces).
//!
//! COMPILER PHASE: Semantic (used in all passes after collection)
//! INPUT: Type expressions from AST
//! OUTPUT: Interned TypeIds for type checking
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Type Interning: Types are stored in a vec and referenced by TypeId (u32),
//!   enabling cheap equality checks and small memory footprint
//! - Pre-Interned Primitives: Common types (textus, numerus, etc.) are interned
//!   once during TypeTable::new() and retrieved via primitive() method
//! - Structural Types: Collections (Array, Map, Set) are built compositionally
//!   from element/key/value types
//! - Inference Variables: Type::Infer represents unknown types during checking;
//!   unified to concrete types or reported as errors
//!
//! ASSIGNABILITY
//! =============
//! The assignable(from, to) method checks whether a value of type `from` can
//! be used where type `to` is expected:
//! - Exact match: textus assignable to textus
//! - nil assignable to Option<T>: nil becomes Some(nil) or None
//! - T assignable to Option<T>: value becomes Some(value)
//! - numerus assignable to fractus: integer widening
//! - Any assignable to ignotum: unknown sink (values flow in, not out)
//!
//! WHY: Allows implicit conversions (int → float) and nil-safety (nil → Option)
//! without explicit casts, reducing boilerplate.
//!
//! IGNOTUM TYPE
//! ============
//! `ignotum` is an unknown/any type that acts as a type-checking escape hatch:
//! - Values can flow INTO ignotum (any assignable to ignotum)
//! - Values cannot flow OUT of ignotum without explicit cast
//!
//! WHY: Provides flexibility for interop or prototyping while preventing
//! unsound type assumptions (ignotum assignable to nothing without cast).

use crate::hir::DefId;
use crate::lexer::Symbol as LexSymbol;
use rustc_hash::FxHashMap;

/// Type ID - reference into type table
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// Type table for interning types
pub struct TypeTable {
    types: Vec<Type>,
    primitives: FxHashMap<Primitive, TypeId>,
}

impl TypeTable {
    pub fn new() -> Self {
        let mut table = Self { types: Vec::new(), primitives: FxHashMap::default() };

        // Pre-intern primitives
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
        ] {
            let id = table.intern(Type::Primitive(prim));
            table.primitives.insert(prim, id);
        }

        table
    }

    /// Intern a type and return its ID
    pub fn intern(&mut self, ty: Type) -> TypeId {
        // Simple linear search for now - could use hash map for dedup
        let id = TypeId(self.types.len() as u32);
        self.types.push(ty);
        id
    }

    /// Get type by ID
    pub fn get(&self, id: TypeId) -> &Type {
        &self.types[id.0 as usize]
    }

    /// Get primitive type ID
    pub fn primitive(&self, prim: Primitive) -> TypeId {
        self.primitives[&prim]
    }

    /// Create an Option<T> type
    pub fn option(&mut self, inner: TypeId) -> TypeId {
        self.intern(Type::Option(inner))
    }

    /// Create a reference type
    pub fn reference(&mut self, mutability: Mutability, inner: TypeId) -> TypeId {
        self.intern(Type::Ref(mutability, inner))
    }

    /// Create an array type
    pub fn array(&mut self, element: TypeId) -> TypeId {
        self.intern(Type::Array(element))
    }

    /// Create a map type
    pub fn map(&mut self, key: TypeId, value: TypeId) -> TypeId {
        self.intern(Type::Map(key, value))
    }

    /// Create a set type
    pub fn set(&mut self, element: TypeId) -> TypeId {
        self.intern(Type::Set(element))
    }

    /// Create a function type
    pub fn function(&mut self, sig: FuncSig) -> TypeId {
        self.intern(Type::Func(sig))
    }

    /// Check if two types are equal
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

    /// Check if `from` is assignable to `to`
    pub fn assignable(&self, from: TypeId, to: TypeId) -> bool {
        if self.equals(from, to) {
            return true;
        }

        let from_ty = self.get(from);
        let to_ty = self.get(to);

        match (from_ty, to_ty) {
            // ignotum is an unknown sink: values can flow into it, but not out without cast/narrowing.
            (Type::Primitive(Primitive::Ignotum), _) => false,

            // Union source is assignable when every member can flow into destination.
            (Type::Union(from_types), _) => from_types.iter().all(|member| self.assignable(*member, to)),

            // Destination union accepts any source that can flow into at least one member.
            (_, Type::Union(to_types)) => to_types.iter().any(|member| self.assignable(from, *member)),

            // nil is assignable to Option<T>
            (Type::Primitive(Primitive::Nihil), Type::Option(_)) => true,

            // Option<S> is assignable to Option<T> if S is assignable to T
            (Type::Option(from_inner), Type::Option(to_inner)) => self.assignable(*from_inner, *to_inner),

            // T is assignable to Option<T>
            (_, Type::Option(inner)) => self.assignable(from, *inner),

            // numerus is assignable to fractus (widening)
            (Type::Primitive(Primitive::Numerus), Type::Primitive(Primitive::Fractus)) => true,

            // Any type is assignable to ignotum
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

/// Semantic type
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Primitive type
    Primitive(Primitive),

    /// Array type: lista<T>
    Array(TypeId),

    /// Map type: tabula<K, V>
    Map(TypeId, TypeId),

    /// Set type: copia<T>
    Set(TypeId),

    /// Optional type: si T
    Option(TypeId),

    /// Reference type: de T / in T
    Ref(Mutability, TypeId),

    /// Struct type
    Struct(DefId),

    /// Enum type
    Enum(DefId),

    /// Interface type
    Interface(DefId),

    /// Type alias (resolved)
    Alias(DefId, TypeId),

    /// Function type
    Func(FuncSig),

    /// Type parameter (unbound)
    Param(LexSymbol),

    /// Generic instantiation
    Applied(TypeId, Vec<TypeId>),

    /// Inference variable
    Infer(InferVar),

    /// Union type
    Union(Vec<TypeId>),

    /// Error type (for recovery)
    Error,
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Primitive {
    Textus,   // string
    Numerus,  // integer
    Fractus,  // float
    Bivalens, // boolean
    Nihil,    // null
    Vacuum,   // void
    Numquam,  // never
    Ignotum,  // unknown
    Octeti,   // bytes
    Regex,    // regex literal
}

impl Primitive {
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
            "objectum" | "quidlibet" | "curator" => Some(Self::Ignotum),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionKind {
    Lista,
    Tabula,
    Copia,
}

impl CollectionKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "lista" => Some(Self::Lista),
            "tabula" => Some(Self::Tabula),
            "copia" => Some(Self::Copia),
            _ => None,
        }
    }

    pub fn arity(self) -> usize {
        match self {
            Self::Lista | Self::Copia => 1,
            Self::Tabula => 2,
        }
    }

    pub fn arity_error(self) -> &'static str {
        match self {
            Self::Lista => "lista requires one type parameter",
            Self::Tabula => "tabula requires two type parameters",
            Self::Copia => "copia requires one type parameter",
        }
    }

    pub fn lower(self, types: &mut TypeTable, params: &[TypeId]) -> TypeId {
        debug_assert_eq!(params.len(), self.arity());
        match self {
            Self::Lista => types.array(params[0]),
            Self::Tabula => types.map(params[0], params[1]),
            Self::Copia => types.set(params[0]),
        }
    }
}

/// Mutability for references
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

/// Function signature
#[derive(Debug, Clone, PartialEq)]
pub struct FuncSig {
    pub params: Vec<ParamType>,
    pub ret: TypeId,
    pub is_async: bool,
    pub is_generator: bool,
}

/// Parameter type info
#[derive(Debug, Clone, PartialEq)]
pub struct ParamType {
    pub ty: TypeId,
    pub mode: ParamMode,
    pub optional: bool,
}

/// Parameter passing mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    Owned,
    Ref,
    MutRef,
    Move,
}

/// Inference variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InferVar(pub u32);

#[cfg(test)]
#[path = "types_test.rs"]
mod tests;
