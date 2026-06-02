use crate::driver::{Config, Session};
use crate::{driver, Output, Target};
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn wasm_target_emits_wat_from_validated_mir() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::WasmText)),
        "wasm.fab",
        source,
    );
    let Some(Output::WasmText(output)) = result.output else {
        panic!("expected WASM text output");
    };

    assert!(output.code.contains("(module"));
    assert!(output
        .code
        .contains("(func $adde (export \"adde\") (param $l0 i64) (param $l1 i64) (result i64)"));
    assert!(output
        .code
        .contains("(i64.add (local.get $l0) (local.get $l1))"));
    validate_wat_if_available(&output.code);
}

#[test]
fn wasm_target_emits_primitive_entry_calls_and_diagnostics() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}

incipit {
    fixum _ n ← adde(1, 2)
    nota n
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(r#"(import "faber_diag" "nota_i64" (func $__faber_diag_nota_i64 (param i64)))"#));
    assert!(output.contains("(func $f1 (export \"f1\")"));
    assert!(output.contains("(local $l0 i64)"));
    assert!(output.contains("(local.set $t0 (call $adde (i64.const 1) (i64.const 2)))"));
    assert!(output.contains("(local.set $l0 (local.get $t0))"));
    assert!(output.contains("(call $__faber_diag_nota_i64 (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_boolean_diagnostic_imports() {
    let source = r#"
incipit {
    fixum _ left ← verum
    fixum _ right ← falsum
    nota left aut right
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(r#"(import "faber_diag" "nota_i32" (func $__faber_diag_nota_i32 (param i32)))"#));
    assert!(output.contains("(i32.or (local.get $l0) (local.get $l1))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_text_handles_and_text_diagnostics() {
    let source = r#"
functio label() → textus {
    redde "salve" + "munde"
}

incipit {
    fixum _ greeting ← label()
    nota greeting
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(r#"(import "faber_diag" "nota_text" (func $__faber_diag_nota_text (param i32)))"#));
    assert!(
        output.contains(r#"(import "faber_text" "concat" (func $__faber_text_concat (param i32 i32) (result i32)))"#)
    );
    assert!(output.contains(r#"(func $label (export "label") (result i32)"#));
    assert!(output.contains("(call $__faber_text_concat (i32.const"));
    assert!(output.contains("(call $__faber_diag_nota_text (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_format_string_text_imports() {
    let source = r#"
incipit {
    fixum _ name ← "Marcus"
    fixum _ age ← 30
    fixum _ info ← "§ is § years old"(name, age)
    nota info
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_text" "format_2_text_i64" (func $__faber_text_format_2_text_i64 (param i32 i32 i64) (result i32)))"#
        )
    );
    assert!(output.contains("(call $__faber_text_format_2_text_i64 (i32.const"));
    assert!(output.contains("(call $__faber_diag_nota_text"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_primitive_unary_values() {
    let source = r#"
functio negative(numerus n) → numerus {
    redde -n
}

functio flipped(bivalens flag) → bivalens {
    redde non flag
}

incipit {
    nota negative(5)
    nota flipped(verum)
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(i64.sub (i64.const 0) (local.get $l0))"));
    assert!(output.contains("(i32.eqz (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_fractus_values_and_diagnostics() {
    let source = r#"
functio media(fractus a, fractus b) → fractus {
    redde (a + b) / 2.0
}

incipit {
    nota media(3.0, 7.0)
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(r#"(import "faber_diag" "nota_f64" (func $__faber_diag_nota_f64 (param f64)))"#));
    assert!(output.contains(r#"(func $media (export "media") (param $l0 f64) (param $l1 f64) (result f64)"#));
    assert!(output.contains("(f64.add (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(f64.div (local.get $t0) (f64.const 2.0))"));
    assert!(output.contains("(call $__faber_diag_nota_f64 (local.get $t0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_branch_dispatch_for_numeric_functions() {
    let source = r#"
functio clamp(numerus value, numerus min, numerus max) → numerus {
    si value < min {
        redde min
    }
    si value > max {
        redde max
    }
    redde value
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(local $__block i32)"));
    assert!(output.contains("(loop $__dispatch"));
    assert!(output.contains("(i64.lt_s (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(i64.gt_s (local.get $l0) (local.get $l2))"));
    assert!(output.contains("(br $__dispatch)"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_recursive_numeric_calls_with_branches() {
    let source = r#"
functio factorial(numerus n) → numerus {
    si n ≤ 1 {
        redde 1
    }
    redde n * factorial(n - 1)
}

incipit {
    nota factorial(5)
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(func $factorial (export \"factorial\")"));
    assert!(output.contains("(i64.le_s (local.get $l0) (i64.const 1))"));
    assert!(output.contains("(call $factorial (local.get $t2))"));
    assert!(output.contains("(call $__faber_diag_nota_i64 (local.get $t0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_rejects_unsupported_mir_shapes() {
    let source = r#"
incipit {
    fixum lista<numerus> values ← [1, 2, 3]
    nota values
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::WasmText)),
        "wasm.fab",
        source,
    );

    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MIR-to-WASM unsupported")));
}

fn compile_wasm_text(source: &str) -> String {
    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::WasmText)),
        "wasm.fab",
        source,
    );
    let Some(Output::WasmText(output)) = result.output else {
        let diagnostics = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect::<Vec<_>>()
            .join(" | ");
        panic!("expected WASM text output, got diagnostics: {diagnostics}");
    };
    output.code
}

fn validate_wat_if_available(wat: &str) {
    if Command::new("wasm-tools")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }

    let path = temp_wat_path();
    fs::write(&path, wat).expect("write temporary WAT");
    let output = Command::new("wasm-tools")
        .arg("validate")
        .arg(&path)
        .output()
        .expect("run wasm-tools validate");
    let _ = fs::remove_file(&path);

    assert!(
        output.status.success(),
        "wasm-tools validate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn temp_wat_path() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("radix-wasm-text-test-{nanos}.wat"))
}
