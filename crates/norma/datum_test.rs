//! Dedicated test module for `crate::datum`.
//!
//! Tests live here (rather than inline in `datum.rs`) per project convention.

use crate::datum::Valor;

#[test]
fn json_roundtrip_basic_shapes() {
    let original = serde_json::json!({
        "null": null,
        "bool": true,
        "int": 42,
        "float": 3.14,
        "str": "hello",
        "arr": [1, "x", false],
        "obj": {"nested": {"k": 1}}
    });

    let valor = Valor::try_from(original.clone()).expect("json -> valor");
    let back = valor.try_to_json().expect("valor -> json");
    assert_eq!(back, original);
}

#[test]
fn toml_roundtrip_basic_shapes() {
    let original = toml::from_str::<toml::Value>(
        r#"
        str = "hi"
        int = 7
        float = 2.5
        bool = false
        arr = [1, 2]
        [tbl]
        inner = "x"
        "#,
    )
    .unwrap();

    let valor = Valor::try_from(original.clone()).expect("toml -> valor");
    let back = valor.try_to_toml().expect("valor -> toml");
    // Note: table key order may differ (BTreeMap), but values equal.
    assert_eq!(back, original);
}

#[test]
fn toml_datetime_becomes_tempus() {
    let v = toml::from_str::<toml::Value>(r#"dt = 1979-05-27T07:32:00Z"#).unwrap();
    let valor = Valor::try_from(v).expect("datetime conversion");
    // The table contains a Tempus entry.
    if let Valor::Tabula(m) = valor {
        match m.get("dt") {
            Some(Valor::Tempus(_)) => {}
            other => panic!("expected Tempus, got {:?}", other),
        }
    } else {
        panic!("expected tabula");
    }
}

#[test]
fn large_json_numbers_convert_via_fractus() {
    // All JSON numbers are accepted: exact i64 -> Numerus, else Fractus (f64 path).
    let big = serde_json::json!(1u64 << 60);
    let _v: Valor = Valor::try_from(big).expect("large int becomes fractus or numerus");
}

#[test]
fn nihil_and_collections_preserve() {
    let v = Valor::Lista(vec![Valor::Nihil, Valor::Bivalens(true)]);
    let j = v.try_to_json().expect("valor -> json");
    let back = Valor::try_from(j).expect("json -> valor");
    assert_eq!(back, v);
}

#[test]
fn fractus_nan_and_infinity_are_rejected_for_json() {
    let nan = Valor::Fractus(f64::NAN);
    assert!(nan.try_to_json().is_err());

    let inf = Valor::Fractus(f64::INFINITY);
    assert!(inf.try_to_json().is_err());
}

#[test]
fn nihil_is_rejected_for_toml() {
    let n = Valor::Nihil;
    assert!(n.try_to_toml().is_err());
}