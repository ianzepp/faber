//! Type lowering
//!
//! Lowers AST type expressions to TypeIds.

use super::Lowerer;
use crate::semantic::{FuncSig, Mutability, ParamMode, ParamType, Primitive, Type, TypeId};
use crate::syntax::{Ident, TypeExpr, TypeExprKind};

impl<'a> Lowerer<'a> {
    /// Lower a type expression to TypeId
    pub fn lower_type(&mut self, ty: &TypeExpr) -> TypeId {
        self.current_span = ty.span;

        let mut ty_id = match &ty.kind {
            TypeExprKind::Named(name, params) => self.lower_named_type(name, params),
            TypeExprKind::Array(inner) => {
                let inner_id = self.lower_type(inner);
                self.types.array(inner_id)
            }
            TypeExprKind::Func(func) => {
                let params = func
                    .params
                    .iter()
                    .map(|param| ParamType {
                        ty: self.lower_type(param),
                        mode: ParamMode::Owned,
                        optional: false,
                    })
                    .collect();
                let ret = self.lower_type(&func.ret);
                self.types.function(FuncSig {
                    params,
                    ret,
                    is_async: false,
                    is_generator: false,
                })
            }
        };

        if let Some(mode) = ty.mode {
            let mutability = match mode {
                crate::syntax::TypeMode::Ref => Mutability::Immutable,
                crate::syntax::TypeMode::MutRef => Mutability::Mutable,
            };
            ty_id = self.types.reference(mutability, ty_id);
        }

        if ty.nullable {
            ty_id = self.types.option(ty_id);
        }

        ty_id
    }

    fn lower_named_type(&mut self, name: &Ident, params: &[TypeExpr]) -> TypeId {
        self.current_span = name.span;

        let name_str = self.interner.resolve(name.name);

        if let Some(prim) = primitive_from_name(name_str) {
            if !params.is_empty() {
                self.error("primitive type cannot accept parameters");
                return self.types.intern(Type::Error);
            }
            return self.types.primitive(prim);
        }

        if let Some(collection_id) = self.lower_collection_type(name_str, params) {
            return collection_id;
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

    fn lower_collection_type(&mut self, name: &str, params: &[TypeExpr]) -> Option<TypeId> {
        match name {
            "lista" => {
                if params.len() != 1 {
                    self.error("lista requires one type parameter");
                    return Some(self.types.intern(Type::Error));
                }
                Some(self.types.array(self.lower_type(&params[0])))
            }
            "tabula" => {
                if params.len() != 2 {
                    self.error("tabula requires two type parameters");
                    return Some(self.types.intern(Type::Error));
                }
                let key = self.lower_type(&params[0]);
                let value = self.lower_type(&params[1]);
                Some(self.types.map(key, value))
            }
            "copia" => {
                if params.len() != 1 {
                    self.error("copia requires one type parameter");
                    return Some(self.types.intern(Type::Error));
                }
                Some(self.types.set(self.lower_type(&params[0])))
            }
            _ => None,
        }
    }
}

fn primitive_from_name(name: &str) -> Option<Primitive> {
    match name {
        "textus" => Some(Primitive::Textus),
        "numerus" => Some(Primitive::Numerus),
        "fractus" => Some(Primitive::Fractus),
        "bivalens" => Some(Primitive::Bivalens),
        "nihil" => Some(Primitive::Nihil),
        "vacuum" => Some(Primitive::Vacuum),
        "numquam" => Some(Primitive::Numquam),
        "ignotum" => Some(Primitive::Ignotum),
        "octeti" => Some(Primitive::Octeti),
        _ => None,
    }
}
