use super::{CollectionKind, Primitive, Type, TypeTable};

#[test]
fn allows_assigning_concrete_type_to_ignotum() {
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let ignotum = types.primitive(Primitive::Ignotum);

    assert!(types.assignable(textus, ignotum));
}

#[test]
fn rejects_assigning_ignotum_to_concrete_type() {
    let mut types = TypeTable::new();
    let ignotum = types.primitive(Primitive::Ignotum);
    let textus = types.primitive(Primitive::Textus);

    assert!(!types.assignable(ignotum, textus));
}

#[test]
fn still_assigns_same_type_id() {
    let mut types = TypeTable::new();
    let ignotum = types.primitive(Primitive::Ignotum);

    assert!(types.assignable(ignotum, ignotum));
}

#[test]
fn primitive_from_name_maps_builtin_aliases() {
    assert_eq!(Primitive::from_name("textus"), Some(Primitive::Textus));
    assert_eq!(Primitive::from_name("regex"), Some(Primitive::Regex));
    assert_eq!(Primitive::from_name("objectum"), Some(Primitive::Ignotum));
    assert_eq!(Primitive::from_name("quidlibet"), Some(Primitive::Ignotum));
    assert_eq!(Primitive::from_name("curator"), Some(Primitive::Ignotum));
    assert_eq!(Primitive::from_name("incognitus"), None);
}

#[test]
fn collection_kind_lowers_collection_types() {
    let mut types = TypeTable::new();
    let textus = types.primitive(Primitive::Textus);
    let numerus = types.primitive(Primitive::Numerus);

    let lista = CollectionKind::Lista.lower(&mut types, &[textus]);
    let tabula = CollectionKind::Tabula.lower(&mut types, &[textus, numerus]);
    let copia = CollectionKind::Copia.lower(&mut types, &[textus]);

    assert!(matches!(types.get(lista), Type::Array(inner) if *inner == textus));
    assert!(matches!(types.get(tabula), Type::Map(key, value) if *key == textus && *value == numerus));
    assert!(matches!(types.get(copia), Type::Set(inner) if *inner == textus));
}
