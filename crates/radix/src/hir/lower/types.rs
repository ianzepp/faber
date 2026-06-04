//! Type syntax lowering for the HIR boundary.
//!
//! Parser type expressions carry surface spelling, spans, modifiers, and generic
//! argument syntax. This module turns that syntax into interned semantic
//! [`TypeId`] values that the resolver, typechecker, and code generators can
//! compare cheaply and share consistently.
//!
//! INVARIANTS
//! ==========
//! - Primitive and collection names are recognized by spelling before ordinary
//!   symbol lookup so `textus`, `lista<T>`, and friends remain language types,
//!   not user definitions.
//! - Named user types must resolve to type-bearing symbols: structs, enums,
//!   interfaces, aliases, or type parameters.
//! - Nullable unions use the canonical `T ∪ nihil` boundary: `nihil` members
//!   are stripped, the remaining members are deduplicated, and presence of
//!   `nihil` wraps the result in `Option`.
//! - `ignotum` is treated as an ordinary primitive/top type spelling here, not
//!   as nullability sugar.
//! - Syntax errors in type positions lower to `Type::Error` after recording a
//!   diagnostic so later phases can continue without guessing.
//!
//! MODIFIER ORDER
//! ==============
//! References (`de`/`in`) are applied to the lowered base type before the legacy
//! nullable flag is applied. That preserves the AST contract exactly; newer
//! source should prefer explicit union nullability where the grammar allows it.

use super::Lowerer;
use crate::semantic::{CollectionKind, FuncSig, InferVar, Mutability, ParamMode, ParamType, Primitive, Type, TypeId};
use crate::syntax::{Ident, TypeExpr, TypeExprKind};

impl<'a> Lowerer<'a> {
    /// Lower a parser type expression into the interned semantic type table.
    ///
    /// This is the only HIR-lowering entry point that may turn type syntax into
    /// reusable `TypeId`s. It preserves source span context for diagnostics,
    /// normalizes nullable unions, and applies AST modifiers in parser order
    /// without resolving type compatibility.
    pub fn lower_type(&mut self, ty: &TypeExpr) -> TypeId {
        self.current_span = ty.span;

        let mut ty_id = match &ty.kind {
            TypeExprKind::Infer => {
                let infer_id = self.next_def_id().0;
                self.types.intern(Type::Infer(InferVar(infer_id)))
            }
            TypeExprKind::Named(name, params) => self.lower_named_type(name, params),
            TypeExprKind::Array(inner) => {
                let inner_id = self.lower_type(inner);
                self.types.array(inner_id)
            }
            TypeExprKind::Func(func) => {
                let params = func
                    .params
                    .iter()
                    .map(|param| ParamType { ty: self.lower_type(param), mode: ParamMode::Owned, optional: false })
                    .collect();
                let ret = self.lower_type(&func.ret);
                let err = func.err.as_ref().map(|err| self.lower_type(err));
                self.types
                    .function(FuncSig { params, ret, err, is_async: false, is_generator: false })
            }
            TypeExprKind::Union(members) => {
                if members.is_empty() {
                    self.error("empty union type");
                    self.types.intern(Type::Error)
                } else {
                    // WHY: `T ∪ nihil` is the source spelling for nullable
                    // value types. HIR stores that as Option<T> so typecheck
                    // and codegen have one nullability shape to reason about.
                    let mut seen = std::collections::HashSet::new();
                    let mut cleaned: Vec<TypeId> = Vec::new();
                    let mut had_nihil = false;

                    for m in members {
                        let lowered = self.lower_type(m);
                        if let Type::Primitive(Primitive::Nihil) = self.types.get(lowered) {
                            had_nihil = true;
                            continue;
                        }
                        if seen.insert(lowered) {
                            cleaned.push(lowered);
                        }
                    }

                    if cleaned.is_empty() {
                        self.error("union type cannot consist only of 'nihil' (use the literal 'nihil' directly)");
                        self.types.intern(Type::Error)
                    } else if had_nihil {
                        // Only wrap in Option when nihil was actually present
                        if cleaned.len() == 1 {
                            self.types.option(cleaned[0])
                        } else {
                            let union_ty = self.types.intern(Type::Union(cleaned));
                            self.types.option(union_ty)
                        }
                    } else {
                        // Plain union with no nihil
                        if cleaned.len() == 1 {
                            cleaned[0]
                        } else {
                            self.types.intern(Type::Union(cleaned))
                        }
                    }
                }
            }
        };

        if let Some(mode) = ty.mode {
            let mutability = match mode {
                crate::syntax::TypeMode::De => Mutability::Immutable,
                crate::syntax::TypeMode::In => Mutability::Mutable,
            };
            ty_id = self.types.reference(mutability, ty_id);
        }

        if ty.nullable {
            ty_id = self.types.option(ty_id);
        }

        ty_id
    }

    /// Resolve a named type spelling before generic application.
    ///
    /// Primitive and collection names are language-owned spellings with fixed
    /// arity rules. User names must resolve to symbols that actually denote
    /// types; values, functions, and unresolved names become `Type::Error`
    /// rather than leaking expression-level definitions into type syntax.
    fn lower_named_type(&mut self, name: &Ident, params: &[TypeExpr]) -> TypeId {
        self.current_span = name.span;

        let name_str = self.interner.resolve(name.name);

        if let Some(prim) = Primitive::from_name(name_str) {
            if !params.is_empty() {
                self.error("primitive type cannot accept parameters");
                return self.types.intern(Type::Error);
            }
            return self.types.primitive(prim);
        }

        if let Some(collection) = CollectionKind::from_name(name_str) {
            if params.len() != collection.arity() {
                self.error(collection.arity_error());
                return self.types.intern(Type::Error);
            }
            let lowered = params
                .iter()
                .map(|param| self.lower_type(param))
                .collect::<Vec<_>>();
            return collection.lower(self.types, &lowered);
        }

        let Some(def_id) = self.resolver.lookup(name.name) else {
            self.error("unknown type name");
            return self.types.intern(Type::Error);
        };

        let Some(symbol) = self.resolver.get_symbol(def_id) else {
            self.error("missing type symbol information");
            return self.types.intern(Type::Error);
        };

        let base = match symbol.kind {
            crate::semantic::SymbolKind::Struct => self.types.intern(Type::Struct(def_id)),
            crate::semantic::SymbolKind::Enum => self.types.intern(Type::Enum(def_id)),
            crate::semantic::SymbolKind::Interface => self.types.intern(Type::Interface(def_id)),
            crate::semantic::SymbolKind::TypeAlias => match symbol.ty {
                Some(resolved) => self.types.intern(Type::Alias(def_id, resolved)),
                None => {
                    self.error("type alias is missing resolved type");
                    self.types.intern(Type::Error)
                }
            },
            crate::semantic::SymbolKind::TypeParam => self.types.intern(Type::Param(symbol.name)),
            _ => {
                self.error("type name does not refer to a type");
                self.types.intern(Type::Error)
            }
        };

        if params.is_empty() {
            return base;
        }

        let args = params.iter().map(|param| self.lower_type(param)).collect();
        self.types.intern(Type::Applied(base, args))
    }
}
