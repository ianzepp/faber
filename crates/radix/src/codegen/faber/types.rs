//! Canonical Faber type emission.
//!
//! The Faber backend prints semantic [`Type`] values back into source-level
//! type syntax. This is a grammar boundary: declarations elsewhere in the
//! backend rely on this module to produce type-first Faber spellings such as
//! `textus`, `lista<T>`, references as `de T`/`in T`, and function types with
//! Faber arrows.
//!
//! INVARIANTS
//! ==========
//! - Emitted type text should stay inside accepted Faber grammar unless a limit
//!   below names a current backend gap.
//! - Named structs, enums, interfaces, aliases, and type parameters prefer the
//!   resolved source name when one is available.
//! - Nested options are flattened before nullable spelling is emitted, avoiding
//!   stacked nullable wrappers in output.
//! - Inference holes remain `_`; record, union, and error-marker types degrade
//!   to `ignotum` rather than inventing unsupported type syntax.
//!
//! LIMITS
//! ======
//! This writer does not reconstruct aliases that were erased before semantic
//! typing or field-level record structure. It also still emits semantic options
//! as flattened legacy `si T` text, while the active grammar's canonical
//! nullable form is `T ∪ nihil`.

use crate::hir::DefId;
use crate::lexer::{Interner, Symbol};
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};
use rustc_hash::FxHashMap;

impl super::FaberCodegen {
    /// Strip nested semantic option wrappers before printing nullable syntax.
    ///
    /// HIR and type inference can accumulate option layers from several source
    /// constructs. This backend treats repeated nullability as one nullable
    /// type so output does not gain artificial precision or depth.
    pub(super) fn flatten_option(&self, mut type_id: TypeId, types: &TypeTable) -> TypeId {
        while let Type::Option(inner) = types.get(type_id) {
            type_id = *inner;
        }
        type_id
    }

    /// Convert a TypeId to the Faber backend's current type syntax.
    ///
    /// This is the single spelling policy for type positions in the Faber
    /// backend. It preserves named semantic identities when possible, emits
    /// collection and reference types in current Faber grammar, and falls back
    /// where the grammar lacks a source form for the semantic shape. Semantic
    /// options are the current exception: they are flattened, but still printed
    /// with the legacy `si T` spelling.
    ///
    /// WHY: Type syntax should match Faber grammar for round-trip validity;
    /// where that is not currently true, the gap belongs in this writer rather
    /// than being hidden by downstream declaration code.
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
                if let Some(err) = sig.err {
                    let err = self.type_to_faber(err, types, names, interner);
                    format!("({}) → {} ⇥ {}", params.join(", "), ret, err)
                } else {
                    format!("({}) → {}", params.join(", "), ret)
                }
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

            // WHY: Fallback output should stay inside real grammar even when semantic
            // precision is degraded. `_` preserves unresolved inference; `ignotum`
            // remains the nearest fallback for union-shaped or error-marker types.
            Type::Infer(_) => "_".to_owned(),
            Type::Union(_) | Type::Error => "ignotum".to_owned(),
        }
    }
}
