//! Type system

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
        let mut table = Self {
            types: Vec::new(),
            primitives: FxHashMap::default(),
        };

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
        // Structural equality check would go here
        false
    }

    /// Check if `from` is assignable to `to`
    pub fn assignable(&self, from: TypeId, to: TypeId) -> bool {
        if self.equals(from, to) {
            return true;
        }

        let from_ty = self.get(from);
        let to_ty = self.get(to);

        match (from_ty, to_ty) {
            // nil is assignable to Option<T>
            (Type::Primitive(Primitive::Nihil), Type::Option(_)) => true,

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
