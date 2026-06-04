#[test]
fn user_defined_http_without_provenance_does_not_emit_norma_runtime_call() {
    let compiler = crate::Compiler::new(crate::Config::default());
    let source = r#"
pactum http {
    @ futura
    @ externa
    functio petet(textus url) → Replicatio

    @ futura
    @ externa
    functio mittet(textus url, textus corpus) → Replicatio

    @ futura
    @ externa
    functio ponet(textus url, textus corpus) → Replicatio

    @ futura
    @ externa
    functio delet(textus url) → Replicatio

    @ futura
    @ externa
    functio mutabit(textus url, textus corpus) → Replicatio

    @ futura
    @ externa
    functio rogabit(textus modus, textus url, tabula<textus, textus> capita, textus corpus) → Replicatio

    @ futura
    @ externa
    functio exspectabit(numerus portus, (Rogatio) → Replicatio handler) → Servitor

    @ externa
    functio replica(numerus status, tabula<textus, textus> capita, textus corpus) → Replicatio

    @ externa
    functio scribe(numerus status, textus corpus) → Replicatio

    @ externa
    functio funde(numerus status, octeti data) → Replicatio

    @ externa
    functio json(numerus status, valor data) → Replicatio

    @ externa
    functio redirige(textus url) → Replicatio
}

pactum Replicatio {
    @ externa
    functio status() → numerus

    @ externa
    functio corpus() → textus

    @ externa
    functio corpus_octeti() → octeti

    @ externa
    functio corpus_json() → valor

    @ externa
    functio capita() → tabula<textus, textus>

    @ externa
    functio caput(textus nomen) → textus ∪ nihil

    @ externa
    functio bene() → bivalens
}

pactum Rogatio {
    @ externa
    functio modus() → textus

    @ externa
    functio via() → textus

    @ externa
    functio corpus() → textus

    @ externa
    functio corpus_json() → valor

    @ externa
    functio capita() → tabula<textus, textus>

    @ externa
    functio caput(textus nomen) → textus ∪ nihil

    @ externa
    functio param(textus nomen) → textus ∪ nihil
}

pactum Servitor {
    @ externa
    functio siste() → vacuum

    @ externa
    functio portus() → numerus
}

incipiet {
    fixum _ responsum ← cede http.petet("http://127.0.0.1:9")
    nota responsum.status()
}
"#;

    let result = compiler.compile_str("http-hal-bridge.fab", source);
    let Some(crate::Output::Rust(rust)) = result.output else {
        panic!("expected Rust output, got diagnostics: {:?}", result.diagnostics);
    };

    assert!(!rust.code.contains("norma::hal::http::petet"));
    assert!(rust
        .code
        .contains("http.petet(\"http://127.0.0.1:9\".to_string()).await"));
    assert!(rust.code.contains("let responsum: dyn Replicatio ="));
    assert!(rust.code.contains("pub trait http"));
    assert!(rust.code.contains("pub trait Replicatio"));
}
