use crate::driver::{Config, Session};
use crate::{driver, Output, Target};

#[test]
fn wasm_target_emits_wat_from_validated_mir() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}
"#;

    let result = driver::compile(&Session::new(Config::default().with_target(Target::Wasm)), "wasm.fab", source);
    let Some(Output::Wasm(output)) = result.output else {
        panic!("expected WASM text output");
    };

    assert!(output.code.contains("(module"));
    assert!(output
        .code
        .contains("(func $adde (export \"adde\") (param $l0 i64) (param $l1 i64) (result i64)"));
    assert!(output
        .code
        .contains("(i64.add (local.get $l0) (local.get $l1))"));
}

#[test]
fn wasm_target_rejects_unsupported_mir_shapes() {
    let source = r#"
functio label() → textus {
    redde "salve"
}
"#;

    let result = driver::compile(&Session::new(Config::default().with_target(Target::Wasm)), "wasm.fab", source);

    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MIR-to-WASM unsupported")));
}
