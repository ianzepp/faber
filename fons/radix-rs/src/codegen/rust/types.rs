//! Rust type generation

use crate::semantic::{Mutability, Primitive, Type, TypeId, TypeTable};

/// Convert a Faber type to Rust syntax
pub fn type_to_rust(type_id: TypeId, types: &TypeTable) -> String {
    let ty = types.get(type_id);

    match ty {
        Type::Primitive(prim) => primitive_to_rust(*prim),

        Type::Array(elem) => {
            format!("Vec<{}>", type_to_rust(*elem, types))
        }

        Type::Map(key, value) => {
            format!(
                "HashMap<{}, {}>",
                type_to_rust(*key, types),
                type_to_rust(*value, types)
            )
        }

        Type::Set(elem) => {
            format!("HashSet<{}>", type_to_rust(*elem, types))
        }

        Type::Option(inner) => {
            format!("Option<{}>", type_to_rust(*inner, types))
        }

        Type::Ref(mutability, inner) => {
            let inner_str = type_to_rust(*inner, types);
            match mutability {
                Mutability::Immutable => format!("&{}", inner_str),
                Mutability::Mutable => format!("&mut {}", inner_str),
            }
        }

        Type::Struct(_def_id) => {
            // TODO: Look up struct name
            "TodoStruct".to_owned()
        }

        Type::Enum(_def_id) => {
            // TODO: Look up enum name
            "TodoEnum".to_owned()
        }

        Type::Interface(_def_id) => {
            // TODO: Look up trait name, use dyn Trait
            "dyn TodoTrait".to_owned()
        }

        Type::Alias(_def_id, resolved) => type_to_rust(*resolved, types),

        Type::Func(sig) => {
            let params: Vec<String> = sig
                .params
                .iter()
                .map(|p| type_to_rust(p.ty, types))
                .collect();
            let ret = type_to_rust(sig.ret, types);

            if sig.is_async {
                format!("impl Future<Output = {}>", ret)
            } else {
                format!("fn({}) -> {}", params.join(", "), ret)
            }
        }

        Type::Param(_name) => {
            // TODO: Look up type parameter name
            "T".to_owned()
        }

        Type::Applied(base, args) => {
            let base_str = type_to_rust(*base, types);
            let args_str: Vec<String> = args.iter().map(|a| type_to_rust(*a, types)).collect();
            format!("{}<{}>", base_str, args_str.join(", "))
        }

        Type::Infer(_) => "_".to_owned(),

        Type::Union(variants) => {
            // Rust doesn't have ad-hoc union types, use enum or trait object
            if variants.is_empty() {
                "!".to_owned() // never type
            } else {
                // TODO: This needs proper handling
                "TodoUnion".to_owned()
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
