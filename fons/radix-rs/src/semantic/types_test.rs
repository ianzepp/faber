use super::{Primitive, TypeTable};

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
