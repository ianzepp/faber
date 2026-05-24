//! Rust type rendering for semantic type identifiers.
//!
//! Codegen receives `TypeId` values that were already assigned by semantic
//! analysis. This module renders those types as Rust syntax for declarations,
//! local annotations, conversions, and generated helper code. It is intentionally
//! a renderer over the existing type table: it resolves names and chooses Rust
//! spellings, but it does not infer missing types or redo compatibility checks.
//!
//! MAPPING CONTRACTS
//! =================
//! - Faber primitives map to the Rust primitives and runtime types selected by
//!   the backend (`textus` to `String`, `numerus` to `i64`, `valor` to
//!   `norma::datum::Valor`, and so on).
//! - Collections map to standard Rust containers: `lista` to `Vec`, `tabula`
//!   to `HashMap`, and `copia` to `HashSet`. The enclosing Rust module is
//!   responsible for importing collection names when emitted output needs them.
//! - Nullable semantic types render as `Option<T>`. Voluntary declaration slots
//!   that are represented as optional storage are handled by declaration and
//!   statement emission around this type renderer.
//! - Faber borrow modes render as Rust references: immutable `de` as `&T` and
//!   mutable `in` as `&mut T`; owned `ex` has already become the inner type.
//! - User types resolve through the codegen name catalog so generated Rust uses
//!   collision-safe symbols.
//!
//! ESCAPE HATCHES
//! ==============
//! `ignotum` and non-empty ad-hoc unions render as `FaberValue`. That preserves
//! an explicit dynamic boundary for constructs the Rust backend cannot model
//! precisely as a static Rust type while keeping generated single-file Rust
//! printable and cloneable under the direct `rustc` e2e harness.

use super::RustCodegen;
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};

/// Convert a semantic Faber type to Rust syntax.
///
/// Callers should pass only type identifiers that have already gone through the
/// semantic pipeline. The returned string may mention Rust library types such
/// as `HashMap`, `HashSet`, `Future`, `regex::Regex`, or
/// `norma::datum::Valor`; import collection is handled outside this renderer.
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

        Type::Record(_) => "CliArgs".to_owned(),

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
            // Interfaces lower to trait objects in type position. Call sites
            // and declarations decide whether an additional reference or box is
            // needed for a valid Rust value shape.
            format!("dyn {}", codegen.resolve_def(*def_id))
        }

        Type::Alias(def_id, resolved) => {
            // Resolve the alias name for catalog consistency, then render the
            // target type. Alias declarations themselves decide whether to
            // expose a Rust `type` item.
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
            // Rust has no anonymous sum type equivalent for Faber ad-hoc
            // unions. Empty unions can use the never type; other unions cross
            // an explicit dynamic boundary.
            if variants.is_empty() {
                "!".to_owned()
            } else {
                "FaberValue".to_owned()
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
///   Ignotum  -> FaberValue
///   Valor    -> norma::datum::Valor
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
        Primitive::Ignotum => "FaberValue",
        Primitive::Octeti => "Vec<u8>",
        Primitive::Regex => "regex::Regex",
        Primitive::Valor => "norma::datum::Valor",
    }
    .to_owned()
}
