use super::GoCodegen;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

/// Map a Faber type to its Go equivalent.
///
/// WHY: Go's type system is simpler than Rust's — no lifetimes, no borrows,
/// no Option<T> (uses *T pointers instead). This keeps the mapping straightforward.
pub fn type_to_go(codegen: &GoCodegen<'_>, type_id: TypeId, types: &TypeTable) -> String {
    match types.get(type_id) {
        Type::Primitive(prim) => primitive_to_go(*prim),
        Type::Array(elem) => format!("[]{}", type_to_go(codegen, *elem, types)),
        Type::Map(key, value) => {
            format!(
                "map[{}]{}",
                type_to_go(codegen, *key, types),
                type_to_go(codegen, *value, types)
            )
        }
        Type::Set(elem) => format!("map[{}]struct{{}}", type_to_go(codegen, *elem, types)),
        Type::Option(inner) => format!("*{}", non_option_type_to_go(codegen, *inner, types)),
        // WHY: Go is GC'd — de/in/ex borrow modes are irrelevant, just emit the inner type.
        Type::Ref(_, inner) => type_to_go(codegen, *inner, types),
        Type::Struct(def_id) | Type::Enum(def_id) | Type::Interface(def_id) => codegen.resolve_def(*def_id).to_owned(),
        Type::Alias(def_id, _) => codegen.resolve_def(*def_id).to_owned(),
        Type::Func(sig) => {
            let params: Vec<String> = sig
                .params
                .iter()
                .map(|param| type_to_go(codegen, param.ty, types))
                .collect();
            let ret = type_to_go(codegen, sig.ret, types);
            format!("func({}) {}", params.join(", "), ret)
        }
        Type::Param(name) => match codegen.resolve_symbol(*name) {
            "ignotum" => "any".to_owned(),
            "quidlibet" => "any".to_owned(),
            other => other.to_owned(),
        },
        Type::Applied(base, args) => {
            let base_str = type_to_go(codegen, *base, types);
            let args_str: Vec<String> = args
                .iter()
                .map(|arg| type_to_go(codegen, *arg, types))
                .collect();
            format!("{}[{}]", base_str, args_str.join(", "))
        }
        Type::Infer(_) => "any".to_owned(),
        Type::Union(_) => "any".to_owned(),
        Type::Error => "any".to_owned(),
    }
}

fn non_option_type_to_go(codegen: &GoCodegen<'_>, type_id: TypeId, types: &TypeTable) -> String {
    let mut current = type_id;
    loop {
        match types.get(current) {
            Type::Option(inner) => current = *inner,
            _ => return type_to_go(codegen, current, types),
        }
    }
}

fn primitive_to_go(prim: Primitive) -> String {
    match prim {
        Primitive::Textus => "string",
        Primitive::Numerus => "int",
        Primitive::Fractus => "float64",
        Primitive::Bivalens => "bool",
        Primitive::Nihil => "any",
        Primitive::Vacuum => "",
        Primitive::Numquam => "any",
        Primitive::Ignotum => "any",
        Primitive::Octeti => "[]byte",
        Primitive::Regex => "*regexp.Regexp",
    }
    .to_owned()
}
