//! Dedicated test module for `crate::hal::http`.

use std::collections::HashMap;

use crate::datum::Valor;
use crate::hal::http::{rogabit, Replicatio, ERROR_HEADER};

#[test]
fn http_response_accessors_return_owned_snapshot_values() {
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "text/plain".to_string());
    let response = Replicatio::nova(201, headers, b"creatum".to_vec());

    assert_eq!(response.status(), 201);
    assert_eq!(response.corpus(), "creatum");
    assert_eq!(response.corpus_octeti(), b"creatum".to_vec());
    assert_eq!(
        response.capita().get("content-type"),
        Some(&"text/plain".to_string())
    );
    assert_eq!(
        response.caput("content-type"),
        Some("text/plain".to_string())
    );
    assert_eq!(
        response.caput("CONTENT-TYPE"),
        Some("text/plain".to_string())
    );
    assert!(response.bene());
}

#[test]
fn http_response_json_body_converts_to_valor() {
    let response = Replicatio::nova(
        200,
        HashMap::new(),
        br#"{"ok":true,"count":3,"items":["a"]}"#.to_vec(),
    );

    let Valor::Tabula(map) = response.corpus_json() else {
        panic!("expected JSON object as Valor::Tabula");
    };

    assert_eq!(map.get("ok"), Some(&Valor::Bivalens(true)));
    assert_eq!(map.get("count"), Some(&Valor::Numerus(3)));
}

#[test]
fn http_response_invalid_json_returns_nihil() {
    let response = Replicatio::nova(200, HashMap::new(), b"not json".to_vec());

    assert_eq!(response.corpus_json(), Valor::Nihil);
}

#[test]
fn http_error_response_is_deterministic() {
    let response = Replicatio::error("transport unavailable");

    assert_eq!(response.status(), 0);
    assert_eq!(
        response.caput(ERROR_HEADER),
        Some("transport unavailable".to_string())
    );
    assert!(!response.bene());
}

#[test]
fn http_invalid_method_returns_synthetic_response() {
    let response = futures::executor::block_on(rogabit(
        "not a method",
        "http://127.0.0.1:9",
        HashMap::new(),
        "",
    ));

    assert_eq!(response.status(), 0);
    assert!(response.caput(ERROR_HEADER).is_some());
    assert!(!response.bene());
}
