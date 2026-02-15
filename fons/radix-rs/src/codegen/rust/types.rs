//! Rust Type Generation
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! Converts Faber type representations (TypeId) to Rust type syntax. Handles
//! primitives, collections, references, structs, enums, traits, and function types.
//!
//! COMPILER PHASE: Codegen (submodule)
//! INPUT: TypeId from TypeTable
//! OUTPUT: Rust type syntax string
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Primitive mapping: Faber types map to closest Rust equivalents.
//!   WHY: numerus -> i64, textus -> String, bivalens -> bool.
//! - Collection mapping: lista -> Vec, tabula -> HashMap, copia -> HashSet.
//!   WHY: Rust standard library collections are idiomatic.
//! - Reference translation: de T -> &T, in T -> &mut T.
//!   WHY: Direct mapping of Faber borrow modes to Rust references.

use super::RustCodegen;
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};

/// Convert a Faber type to Rust syntax.
///
/// TRANSFORMS:
///   numerus           -> i64
///   textus            -> String
///   lista<T>          -> Vec<T>
///   tabula<K, V>      -> HashMap<K, V>
///   de T              -> &T
///   in T              -> &mut T
///   si T              -> Option<T>
///   futura functio    -> impl Future<Output = T>
///
/// TARGET: Rust-specific type mappings; ignotum -> Box<dyn Any>.
pub fn type_to_rust(codegen: &RustCodegen<'_>, type_id: TypeId, types: &TypeTable) -> String {
    let ty = types.get(type_id);

    match ty {
        Type::Primitive(prim) => primitive_to_rust(*prim),

        Type::Array(elem) => {
            format!("Vec<{}>", type_to_rust(codegen, *elem, types))
        }

        Type::Map(key, value) => {
            format!(
                "HashMap<{}, {}>",
                type_to_rust(codegen, *key, types),
                type_to_rust(codegen, *value, types)
            )
        }

        Type::Set(elem) => {
            format!("HashSet<{}>", type_to_rust(codegen, *elem, types))
        }

        Type::Option(inner) => {
            format!("Option<{}>", type_to_rust(codegen, *inner, types))
        }

        Type::Ref(mutability, inner) => {
            let inner_str = type_to_rust(codegen, *inner, types);
            match mutability {
                Mutability::Immutable => format!("&{}", inner_str),
                Mutability::Mutable => format!("&mut {}", inner_str),
            }
        }

        Type::Struct(def_id) => codegen.resolve_def(*def_id).to_owned(),

        Type::Enum(def_id) => codegen.resolve_def(*def_id).to_owned(),

        Type::Interface(def_id) => {
            format!("dyn {}", codegen.resolve_def(*def_id))
        }

        Type::Alias(def_id, resolved) => {
            codegen.resolve_def(*def_id);
            type_to_rust(codegen, *resolved, types)
        }

        Type::Func(sig) => {
            let params: Vec<String> = sig
                .params
                .iter()
                .map(|p| type_to_rust(codegen, p.ty, types))
                .collect();
            let ret = type_to_rust(codegen, sig.ret, types);

            if sig.is_async {
                format!("impl Future<Output = {}>", ret)
            } else {
                format!("fn({}) -> {}", params.join(", "), ret)
            }
        }

        Type::Param(name) => codegen.resolve_symbol(*name).to_owned(),

        Type::Applied(base, args) => {
            let base_str = type_to_rust(codegen, *base, types);
            let args_str: Vec<String> = args
                .iter()
                .map(|a| type_to_rust(codegen, *a, types))
                .collect();
            format!("{}<{}>", base_str, args_str.join(", "))
        }

        Type::Infer(_) => "_".to_owned(),

        Type::Union(variants) => {
            // Rust doesn't have ad-hoc union types, use enum or trait object
            if variants.is_empty() {
                "!".to_owned() // never type
            } else {
                "Box<dyn std::any::Any>".to_owned()
            }
        }

        Type::Error => "/* error */".to_owned(),
    }
}

/// Map Faber primitive types to Rust types.
///
/// TRANSFORMS:
///   Textus   -> String
///   Numerus  -> i64
///   Fractus  -> f64
///   Bivalens -> bool
///   Nihil    -> ()
///   Vacuum   -> ()
///   Numquam  -> ! (never type)
///   Ignotum  -> Box<dyn Any>
///   Octeti   -> Vec<u8>
///
/// TARGET: Rust primitive and standard library types.
fn primitive_to_rust(prim: Primitive) -> String {
    match prim {
        Primitive::Textus => "String",
        Primitive::Numerus => "i64",
        Primitive::Fractus => "f64",
        Primitive::Bivalens => "bool",
        Primitive::Nihil => "()", // or Option::None
        Primitive::Vacuum => "()",
        Primitive::Numquam => "!",
        Primitive::Ignotum => "Box<dyn std::any::Any>",
        Primitive::Octeti => "Vec<u8>",
    }
    .to_owned()
}
