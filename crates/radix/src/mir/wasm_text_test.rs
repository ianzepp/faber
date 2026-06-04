use crate::driver::{Config, Session};
use crate::{driver, Output, Target};
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_WAT_COUNTER: AtomicU64 = AtomicU64::new(0);

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
    assert!(output.contains("(func $f1 (export \"incipit\")"));
    assert!(output.contains("(local $l0 i64)"));
    assert!(output.contains("(local.set $t0 (call $adde (i64.const 1) (i64.const 2)))"));
    assert!(output.contains("(local.set $l0 (local.get $t0))"));
    assert!(output.contains("(call $__faber_diag_nota_i64 (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_top_level_const_entry_prefix_locals() {
    let output = compile_wasm_text(
        r#"
fixum _ age ← 25

incipit {
    nota age
}
"#,
    );

    assert!(output.contains("(local $l0 i64)"));
    assert!(output.contains("(local.set $l0 (i64.const 25))"));
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
fn wasm_target_emits_integer_bitwise_ops() {
    let source = r#"
incipit {
    fixum _ flags ← 10
    fixum _ mask ← 12
    nota flags ∧ mask
    nota flags ∨ mask
    nota flags ⊻ mask
    nota 1 ≪ 4
    nota 16 ≫ 2
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(i64.and (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(i64.or (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(i64.xor (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(i64.shl (i64.const 1) (i64.const 4))"));
    assert!(output.contains("(i64.shr_s (i64.const 16) (i64.const 2))"));
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
fn wasm_target_emits_predicate_unary_and_nil_tests() {
    let source = r#"
functio checks(numerus n, fractus f, bivalens flag, textus ∪ nihil maybe) → bivalens {
    fixum _ positive ← positivum n
    fixum _ negative ← negativum f
    fixum _ enabled ← verum flag
    fixum _ disabled ← falsum flag
    fixum _ absent ← maybe est nihil
    redde positive et negative et enabled et disabled et absent
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(i64.gt_s (local.get $l0) (i64.const 0))"));
    assert!(output.contains("(f64.lt (local.get $l1) (f64.const 0.0))"));
    assert!(output.contains("(i32.eq (local.get $l2) (i32.const 1))"));
    assert!(output.contains("(i32.eq (local.get $l2) (i32.const 0))"));
    assert!(output.contains("(i32.eqz (local.get $l3))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_option_coalesce_for_nullable_handles() {
    let source = r#"
incipit {
    fixum textus ∪ nihil maybe ← nihil
    fixum _ resolved ← maybe vel "fallback"
    nota resolved
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(local $l0 i32)"));
    assert!(output.contains("(local.set $l0 (i32.const 0))"));
    assert!(output.contains("(select (local.get $l0) (i32.const"));
    assert!(output.contains("(i32.ne (local.get $l0) (i32.const 0))"));
    assert!(output.contains("(call $__faber_diag_nota_text (local.get $l1))"));
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
fn wasm_target_emits_opaque_aggregate_handles() {
    let source = r#"
incipit {
    fixum lista<numerus> values ← [1, 2, 3]
    nota values
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_aggregate" "array_3_i64_i64_i64" (func $__faber_aggregate_array_3_i64_i64_i64 (param i64 i64 i64) (result i32)))"#
        )
    );
    assert!(
        output.contains(r#"(import "faber_diag" "nota_aggregate" (func $__faber_diag_nota_aggregate (param i32)))"#)
    );
    assert!(output.contains("(local $l0 i32)"));
    assert!(output.contains("(call $__faber_aggregate_array_3_i64_i64_i64 (i64.const 1) (i64.const 2) (i64.const 3))"));
    assert!(output.contains("(call $__faber_diag_nota_aggregate (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_aggregate_index_projection_reads() {
    let source = r#"
incipit {
    fixum lista<numerus> values ← [1, 2, 3]
    nota values[0]
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_aggregate" "index_i64_to_i64" (func $__faber_aggregate_index_i64_to_i64 (param i32 i64) (result i64)))"#
        )
    );
    assert!(output.contains("(call $__faber_aggregate_index_i64_to_i64 (local.get $l0) (i64.const 0))"));
    assert!(output.contains("(call $__faber_diag_nota_i64"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_aggregate_field_projection_reads() {
    let source = r#"
genus Punctum {
    numerus x
    textus label
}

incipit {
    fixum _ p ← Punctum {
        x = 10,
        label = "Roma"
    }
    nota p.x
    nota p.label
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_aggregate" "field_i32_to_i64" (func $__faber_aggregate_field_i32_to_i64 (param i32 i32) (result i64)))"#
        )
    );
    assert!(
        output.contains(
            r#"(import "faber_aggregate" "field_i32_to_text" (func $__faber_aggregate_field_i32_to_text (param i32 i32) (result i32)))"#
        )
    );
    assert!(output.contains("(call $__faber_aggregate_field_i32_to_i64 (local.get $l0) (i32.const"));
    assert!(output.contains("(call $__faber_aggregate_field_i32_to_text (local.get $l0) (i32.const"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_map_field_projection_reads() {
    let source = r#"
incipit {
    fixum _ codes ← praefixum({
        ok = 200,
        error = 500
    })
    nota codes.ok
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_aggregate" "field_i32_to_aggregate" (func $__faber_aggregate_field_i32_to_aggregate (param i32 i32) (result i32)))"#
        )
    );
    assert!(output.contains("(call $__faber_aggregate_field_i32_to_aggregate (local.get $l0) (i32.const"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_option_chain_projection_reads() {
    let source = r#"
genus Address {
    textus city
}

genus User {
    textus name
    Address address sponte
}

incipit {
    fixum _ user ← User {
        name = "Aurelia",
        address = Address { city = "Roma" }
    }
    fixum _ city ← user?.address?.city
    nota city
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(r#"(import "faber_aggregate" "field_i32_to_aggregate""#));
    assert!(output.contains(r#"(import "faber_aggregate" "field_i32_to_text""#));
    assert!(output.contains(";; option chain = (call $__faber_aggregate_field_i32_to_aggregate"));
    assert!(output.contains(";; option chain = (call $__faber_aggregate_field_i32_to_text"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_coerces_scalar_operands_into_union_carriers() {
    let source = r#"
incipit {
    fixum _ data ← { count = 100, active = verum }
    ex data varia count, active

    count ← 200
    active ← falsum

    nota count
    nota active
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(local.set $l1 (i32.wrap_i64 (i64.const 200)))"));
    assert!(output.contains("(local.set $l2 (i32.const 0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_runtime_conversion_imports() {
    let source = r#"
incipit {
    fixum _ parsed ← "42" ⇒ numerus
    fixum _ fallback ← "bad" ⇒ numerus vel 0
    fixum _ display ← parsed ⇒ textus
    nota fallback
    nota display
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_runtime" "convert_1_text_to_i64" (func $__faber_runtime_convert_1_text_to_i64 (param i32) (result i64)))"#
        )
    );
    assert!(
        output.contains(
            r#"(import "faber_runtime" "convert_2_text_i64_to_i64" (func $__faber_runtime_convert_2_text_i64_to_i64 (param i32 i64) (result i64)))"#
        )
    );
    assert!(
        output.contains(
            r#"(import "faber_runtime" "convert_1_i64_to_text" (func $__faber_runtime_convert_1_i64_to_text (param i64) (result i32)))"#
        )
    );
    assert!(output.contains("(call $__faber_runtime_convert_1_text_to_i64 (i32.const"));
    assert!(output.contains("(call $__faber_runtime_convert_2_text_i64_to_i64 (i32.const"));
    assert!(output.contains("(call $__faber_runtime_convert_1_i64_to_text (local.get $l0))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_assert_and_text_compare_imports() {
    let source = r#"
incipit {
    fixum _ name ← "Marcus"
    fixum _ count ← 10
    adfirma count > 0
    adfirma name ≠ "", "name must not be empty"
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(r#"(import "faber_text" "ne_text" (func $__faber_text_ne_text (param i32 i32) (result i32)))"#)
    );
    assert!(
        output.contains(r#"(import "faber_runtime" "assert_1_i32" (func $__faber_runtime_assert_1_i32 (param i32)))"#)
    );
    assert!(output.contains(
        r#"(import "faber_runtime" "assert_2_i32_text" (func $__faber_runtime_assert_2_i32_text (param i32 i32)))"#
    ));
    assert!(output.contains("(call $__faber_runtime_assert_1_i32"));
    assert!(output.contains("(call $__faber_text_ne_text"));
    assert!(output.contains("(call $__faber_runtime_assert_2_i32_text"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_collection_append_and_first_imports() {
    let source = r#"
incipit {
    varia lista<numerus> items ← vacua
    items.appende(1)
    nota items.primus()
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains(
        r#"(import "faber_runtime" "append_2_aggregate_i64" (func $__faber_runtime_append_2_aggregate_i64 (param i32 i64)))"#
    ));
    assert!(output.contains(
        r#"(import "faber_runtime" "first_1_aggregate_to_aggregate" (func $__faber_runtime_first_1_aggregate_to_aggregate (param i32) (result i32)))"#
    ));
    assert!(output.contains("(call $__faber_runtime_append_2_aggregate_i64"));
    assert!(output.contains("(call $__faber_runtime_first_1_aggregate_to_aggregate"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_panic_and_collection_length_imports() {
    let source = r#"
functio at(lista<numerus> items, numerus index) → numerus {
    si index ≥ items.longitudo() {
        mori "Index out of bounds"
    }
    redde items[index]
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(
            r#"(import "faber_runtime" "length_1_aggregate_to_i64" (func $__faber_runtime_length_1_aggregate_to_i64 (param i32) (result i64)))"#
        )
    );
    assert!(output.contains(r#"(import "faber_runtime" "panic_text" (func $__faber_runtime_panic_text (param i32)))"#));
    assert!(output.contains("(call $__faber_runtime_length_1_aggregate_to_i64 (local.get $l0))"));
    assert!(output.contains("(call $__faber_runtime_panic_text (i32.const"));
    assert!(output.contains("(unreachable)"));
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
fn wasm_target_emits_switch_dispatch_for_literal_elige() {
    let source = r#"
functio by_code(numerus code) → textus {
    elige code {
        casu 200 { redde "ok" }
        casu 404 { redde "missing" }
        ceterum { redde "other" }
    }
}

functio by_status(textus status) → numerus {
    elige status {
        casu "active" { redde 1 }
        casu "paused" { redde 2 }
        ceterum { redde 0 }
    }
}
"#;

    let output = compile_wasm_text(source);

    assert!(
        output.contains(r#"(import "faber_text" "eq_text" (func $__faber_text_eq_text (param i32 i32) (result i32)))"#)
    );
    assert!(output.contains("(i64.eq (local.get $l0) (i64.const 200))"));
    assert!(output.contains("(call $__faber_text_eq_text (local.get $l0) (i32.const"));
    assert!(output.contains("(local.set $__block"));
    assert!(output.contains("(br $__dispatch)"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_numeric_range_itera_ab_dispatch() {
    let source = r#"
incipit {
    itera ab 0‥8 per 2 fixum i {
        nota i
    }
}
"#;

    let output = compile_wasm_text(source);

    assert!(output.contains("(i64.const 8)"));
    assert!(output.contains("(i64.const 2)"));
    assert!(output.contains("(i64.gt_s (local.get $l2) (i64.const 0))"));
    assert!(output.contains("(i64.lt_s (local.get $l0) (local.get $l1))"));
    assert!(output.contains("(call $__faber_diag_nota_i64 (local.get $l0))"));
    assert!(output.contains("(i64.add (local.get $l0) (local.get $l2))"));
    validate_wat_if_available(&output);
}

#[test]
fn wasm_target_emits_aggregate_projection_assignments() {
    let output = compile_wasm_text(
        r#"
incipit {
    varia lista<numerus> values ← [1, 2, 3]
    values[0] ← 4
}
"#,
    );

    assert!(output.contains(r#"(import "faber_aggregate" "set_index_i64_i64""#));
    assert!(output.contains("(call $__faber_aggregate_set_index_i64_i64"));
    validate_wat_if_available(&output);
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
    let counter = TEMP_WAT_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("radix-wasm-text-test-{nanos}-{counter}.wat"))
}
