use super::TsCodegen;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

pub fn type_to_ts(codegen: &TsCodegen<'_>, type_id: TypeId, types: &TypeTable) -> String {
    match types.get(type_id) {
        Type::Primitive(prim) => primitive_to_ts(*prim),
        Type::Array(elem) => format!("Array<{}>", type_to_ts(codegen, *elem, types)),
        Type::Map(key, value) => {
            format!(
                "Record<{}, {}>",
                type_to_ts(codegen, *key, types),
                type_to_ts(codegen, *value, types)
            )
        }
        Type::Set(elem) => format!("Set<{}>", type_to_ts(codegen, *elem, types)),
        Type::Option(inner) => format!("{} | null", type_to_ts(codegen, *inner, types)),
        Type::Ref(_, inner) => type_to_ts(codegen, *inner, types),
        Type::Struct(def_id) | Type::Enum(def_id) | Type::Interface(def_id) => codegen.resolve_def(*def_id).to_owned(),
        Type::Alias(def_id, _) => codegen.resolve_def(*def_id).to_owned(),
        Type::Func(sig) => {
            let params: Vec<String> = sig
                .params
                .iter()
                .enumerate()
                .map(|(idx, param)| {
                    let optional = if param.optional { "?" } else { "" };
                    format!("p{}{}: {}", idx + 1, optional, type_to_ts(codegen, param.ty, types))
                })
                .collect();
            format!("({}) => {}", params.join(", "), type_to_ts(codegen, sig.ret, types))
        }
        Type::Param(name) => match codegen.resolve_symbol(*name) {
            "ignotum" => "unknown".to_owned(),
            "quidlibet" => "any".to_owned(),
            other => other.to_owned(),
        },
        Type::Applied(base, args) => {
            let base_str = type_to_ts(codegen, *base, types);
            let args_str: Vec<String> = args
                .iter()
                .map(|arg| type_to_ts(codegen, *arg, types))
                .collect();
            format!("{}<{}>", base_str, args_str.join(", "))
        }
        Type::Infer(_) => "unknown".to_owned(),
        Type::Union(members) => {
            if members.is_empty() {
                "never".to_owned()
            } else {
                members
                    .iter()
                    .map(|member| type_to_ts(codegen, *member, types))
                    .collect::<Vec<_>>()
                    .join(" | ")
            }
        }
        Type::Error => "unknown".to_owned(),
    }
}

fn primitive_to_ts(prim: Primitive) -> String {
    match prim {
        Primitive::Textus => "string",
        Primitive::Numerus => "number",
        Primitive::Fractus => "number",
        Primitive::Bivalens => "boolean",
        Primitive::Nihil => "null",
        Primitive::Vacuum => "void",
        Primitive::Numquam => "never",
        Primitive::Ignotum => "unknown",
        Primitive::Octeti => "Uint8Array",
        Primitive::Regex => "RegExp",
    }
    .to_owned()
}
