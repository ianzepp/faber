//! Rust type generation

use super::RustCodegen;
use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};

/// Convert a Faber type to Rust syntax
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

        Type::Struct(def_id) => {
            codegen.resolve_def(*def_id).to_owned()
        }

        Type::Enum(def_id) => {
            codegen.resolve_def(*def_id).to_owned()
        }

        Type::Interface(def_id) => {
            format!("dyn {}", codegen.resolve_def(*def_id))
        }

        Type::Alias(def_id, resolved) => {
            let _ = codegen.resolve_def(*def_id);
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

        Type::Param(name) => {
            codegen.resolve_symbol(*name).to_owned()
        }

        Type::Applied(base, args) => {
            let base_str = type_to_rust(codegen, *base, types);
            let args_str: Vec<String> = args.iter().map(|a| type_to_rust(codegen, *a, types)).collect();
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
