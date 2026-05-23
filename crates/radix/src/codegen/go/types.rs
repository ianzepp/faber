//! Type spelling for the Go backend.
//!
//! This module is the single place where semantic Faber types become Go type
//! text. It is intentionally a mapping layer, not a validator: unknown,
//! inference, union, and error-shaped types fall back to `any` as target escape
//! hatches because earlier phases own semantic rejection and recovery.
//!
//! TARGET CONTRACTS
//! ================
//! - Primitive Faber value types map to ordinary Go primitives where possible.
//! - Lists, maps, and sets map to slices, maps, and `map[T]struct{}` sets.
//! - Optional values map to pointers; nested options collapse to one pointer
//!   because Go's nil pointer does not encode option depth.
//! - Borrow/reference modes are no-ops in Go type text.
//! - User types resolve through the backend [`GoCodegen`] name catalog so
//!   declarations and uses share the same spelling.
//! - `ignotum`, `quidlibet`, unresolved inference, unions, and error recovery
//!   surfaces emit `any` rather than claiming stronger Go semantics.

use super::GoCodegen;
use crate::semantic::{Primitive, Type, TypeId, TypeTable};

/// Map a Faber type to its Go equivalent.
///
/// The mapping is target-oriented and deliberately lossy at Faber surfaces that
/// Go cannot represent directly. Callers should treat `any` results as escape
/// hatches, not as proof that the source type was semantically unconstrained.
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
        // Records have no dedicated generated struct shape yet, so the current
        // backend uses a string-keyed dynamic map.
        Type::Record(_) => "map[string]any".to_owned(),
        Type::Set(elem) => format!("map[{}]struct{{}}", type_to_go(codegen, *elem, types)),
        Type::Option(inner) => format!("*{}", non_option_type_to_go(codegen, *inner, types)),
        // Go has no lifetime or borrow-mode syntax; generated type text keeps
        // only the referent type.
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
        // Recovery and not-yet-represented type surfaces use Go's top type.
        Type::Infer(_) => "any".to_owned(),
        Type::Union(_) => "any".to_owned(),
        Type::Error => "any".to_owned(),
    }
}

fn non_option_type_to_go(codegen: &GoCodegen<'_>, type_id: TypeId, types: &TypeTable) -> String {
    // `**T` would imply observable nesting that Go nil pointers do not carry.
    // Collapse nested Faber options before adding the one pointer in
    // `type_to_go`.
    let mut current = type_id;
    loop {
        match types.get(current) {
            Type::Option(inner) => current = *inner,
            _ => return type_to_go(codegen, current, types),
        }
    }
}

fn primitive_to_go(prim: Primitive) -> String {
    // `vacuum` returns an empty string because Go omits return type text for
    // void functions. `nihil` remains `any` so functions that return Faber nil
    // can explicitly `return nil`.
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
        Primitive::Valor => "any", // TODO: proper valor type when Go runtime support exists
    }
    .to_owned()
}
