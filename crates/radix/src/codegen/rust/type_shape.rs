//! Pure Rust-backend predicates over semantic type shapes.
//!
//! These helpers keep alias unwrapping and dynamic/optional policy consistent
//! across statement, declaration, and expression emission.

use crate::semantic::{Primitive, Type, TypeId, TypeTable};

pub(super) fn resolve_type(type_id: TypeId, types: &TypeTable) -> Type {
    match types.get(type_id) {
        Type::Alias(_, resolved) => resolve_type(*resolved, types),
        ty => ty.clone(),
    }
}

pub(super) fn type_id_is_option(type_id: TypeId, types: &TypeTable) -> bool {
    optional_value_type(type_id, types).is_some()
}

pub(super) fn type_is_option_or_nihil(type_id: TypeId, types: &TypeTable) -> bool {
    optional_value_type(type_id, types).is_some()
        || matches!(resolve_type(type_id, types), Type::Primitive(Primitive::Nihil))
}

pub(super) fn optional_value_type(type_id: TypeId, types: &TypeTable) -> Option<TypeId> {
    match resolve_type(type_id, types) {
        Type::Option(inner) => Some(inner),
        Type::Union(variants) => nullable_union_value_type(&variants, types),
        _ => None,
    }
}

pub(super) fn option_inner_or_self(type_id: TypeId, types: &TypeTable) -> TypeId {
    optional_value_type(type_id, types).unwrap_or(type_id)
}

pub(super) fn type_id_is_faber_value(type_id: TypeId, types: &TypeTable) -> bool {
    match resolve_type(type_id, types) {
        Type::Primitive(Primitive::Ignotum) => true,
        Type::Union(variants) => nullable_union_value_type(&variants, types).is_none() && !variants.is_empty(),
        _ => false,
    }
}

fn nullable_union_value_type(variants: &[TypeId], types: &TypeTable) -> Option<TypeId> {
    let mut value_ty = None;
    let mut has_nil = false;
    for variant in variants {
        if matches!(resolve_type(*variant, types), Type::Primitive(Primitive::Nihil)) {
            has_nil = true;
        } else if value_ty.replace(*variant).is_some() {
            return None;
        }
    }
    has_nil.then_some(value_ty).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hir::DefId;

    #[test]
    fn resolves_aliases_for_optional_and_dynamic_shapes() {
        let mut types = TypeTable::new();
        let numerus = types.primitive(Primitive::Numerus);
        let opt_numerus = types.option(numerus);
        let opt_alias = types.intern(Type::Alias(DefId(1), opt_numerus));
        let nil = types.primitive(Primitive::Nihil);
        let nullable_union = types.intern(Type::Union(vec![numerus, nil]));
        let ignotum = types.primitive(Primitive::Ignotum);
        let dynamic_alias = types.intern(Type::Alias(DefId(2), ignotum));

        assert!(type_id_is_option(opt_alias, &types));
        assert_eq!(option_inner_or_self(opt_alias, &types), numerus);
        assert!(type_id_is_option(nullable_union, &types));
        assert_eq!(option_inner_or_self(nullable_union, &types), numerus);
        assert!(!type_id_is_faber_value(nullable_union, &types));
        assert!(type_id_is_faber_value(dynamic_alias, &types));
    }

    #[test]
    fn distinguishes_nil_from_plain_values() {
        let types = TypeTable::new();
        let nihil = types.primitive(Primitive::Nihil);
        let textus = types.primitive(Primitive::Textus);

        assert!(type_is_option_or_nihil(nihil, &types));
        assert!(!type_is_option_or_nihil(textus, &types));
    }
}
