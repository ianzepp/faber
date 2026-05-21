use crate::hir::DefId;
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    pub(super) fn flatten_option(&self, mut type_id: TypeId, types: &TypeTable) -> TypeId {
        while let Type::Option(inner) = types.get(type_id) {
            type_id = *inner;
        }
        type_id
    }
    /// Convert a TypeId to canonical Faber type syntax.
    ///
    /// TRANSFORMS:
    ///   Type::Primitive(Numerus) -> "numerus"
    ///   Type::Array(elem)        -> "lista<elem>"
    ///   Type::Ref(Immutable, T)  -> "de T"
    ///
    /// WHY: Type syntax must match Faber grammar exactly for round-trip validity.
    pub(super) fn type_to_faber(
        &self,
        type_id: TypeId,
        types: &TypeTable,
        names: &FxHashMap<DefId, Symbol>,
        interner: &Interner,
    ) -> String {
        let ty = types.get(type_id);

        match ty {
            Type::Primitive(prim) => match prim {
                Primitive::Textus => "textus",
                Primitive::Numerus => "numerus",
                Primitive::Fractus => "fractus",
                Primitive::Bivalens => "bivalens",
                Primitive::Nihil => "nihil",
                Primitive::Vacuum => "vacuum",
                Primitive::Numquam => "numquam",
                Primitive::Ignotum => "ignotum",
                Primitive::Valor => "valor",
                Primitive::Octeti => "octeti",
                Primitive::Regex => "regex",
            }
            .to_owned(),

            Type::Array(elem) => format!("lista<{}>", self.type_to_faber(*elem, types, names, interner)),

            Type::Map(key, value) => format!(
                "tabula<{}, {}>",
                self.type_to_faber(*key, types, names, interner),
                self.type_to_faber(*value, types, names, interner)
            ),

            Type::Record(_) => "ignotum".to_owned(),

            Type::Set(elem) => format!("copia<{}>", self.type_to_faber(*elem, types, names, interner)),

            Type::Option(inner) => format!(
                "si {}",
                self.type_to_faber(self.flatten_option(*inner, types), types, names, interner)
            ),

            Type::Ref(mutability, inner) => {
                let prefix = match mutability {
                    Mutability::Immutable => "de",
                    Mutability::Mutable => "in",
                };
                format!("{} {}", prefix, self.type_to_faber(*inner, types, names, interner))
            }

            Type::Struct(def_id) | Type::Enum(def_id) | Type::Interface(def_id) => {
                self.name_for_def(*def_id, names, interner)
            }

            Type::Alias(def_id, resolved) => names
                .get(def_id)
                .map(|sym| self.symbol_to_string(*sym, interner))
                .unwrap_or_else(|| self.type_to_faber(*resolved, types, names, interner)),

            Type::Func(sig) => {
                let params: Vec<String> = sig
                    .params
                    .iter()
                    .map(|p| self.type_to_faber(p.ty, types, names, interner))
                    .collect();
                let ret = self.type_to_faber(sig.ret, types, names, interner);
                format!("({}) → {}", params.join(", "), ret)
            }

            Type::Param(sym) => self.symbol_to_string(*sym, interner),

            Type::Applied(base, args) => {
                let base_str = self.type_to_faber(*base, types, names, interner);
                let args_str: Vec<String> = args
                    .iter()
                    .map(|a| self.type_to_faber(*a, types, names, interner))
                    .collect();
                format!("{}<{}>", base_str, args_str.join(", "))
            }

            // WHY: Canonical Faber output must stay inside real grammar even when
            // semantic precision is degraded. `ignotum` is the nearest legal
            // fallback for unresolved, union-shaped, or error-marker types.
            Type::Infer(_) | Type::Union(_) | Type::Error => "ignotum".to_owned(),
        }
    }
}
