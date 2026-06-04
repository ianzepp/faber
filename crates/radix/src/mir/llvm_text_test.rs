use crate::driver::{Config, Session};
use crate::{driver, Output, Target};

#[test]
fn llvm_text_target_emits_text_from_validated_mir() {
    let source = r#"
functio adde(numerus a, numerus b) → numerus {
    redde a + b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("define i64 @adde(i64 %l0, i64 %l1)"));
    assert!(output.code.contains("add i64 %l0, %l1"));
    assert!(output.code.contains("ret i64 %v"));
}

#[test]
fn llvm_text_target_emits_fractus_arithmetic_and_comparisons() {
    let source = r#"
functio media(fractus a, fractus b) → fractus {
    redde (a + b) / 2.0
}

functio minor(fractus a, fractus b) → bivalens {
    redde a < b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output
        .code
        .contains("define double @media(double %l0, double %l1)"));
    assert!(output.code.contains("fadd double %l0, %l1"));
    assert!(output.code.contains("fdiv double %v0, 2.0"));
    assert!(output
        .code
        .contains("define i1 @minor(double %l0, double %l1)"));
    assert!(output.code.contains("fcmp olt double %l0, %l1"));
}

#[test]
fn llvm_text_target_emits_boolean_unary_binary_and_equality() {
    let source = r#"
functio logicum(bivalens a, bivalens b) → bivalens {
    redde non a et (a aut b)
}

functio idem(bivalens a, bivalens b) → bivalens {
    redde a ≡ b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("xor i1 %l0, true"));
    assert!(output.code.contains("or i1 %l0, %l1"));
    assert!(output.code.contains("and i1 %v0, %v1"));
    assert!(output.code.contains("icmp eq i1 %l0, %l1"));
}

#[test]
fn llvm_text_target_emits_integer_comparisons() {
    let source = r#"
functio minor(numerus a, numerus b) → bivalens {
    redde a < b
}

functio idem(numerus a, numerus b) → bivalens {
    redde a ≡ b
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );
    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("icmp slt i64 %l0, %l1"));
    assert!(output.code.contains("icmp eq i64 %l0, %l1"));
}

#[test]
fn llvm_text_target_rejects_unsupported_mir_shapes() {
    let source = r#"
functio label() → textus {
    redde "salve"
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );

    assert!(result.output.is_none());
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MIR-to-LLVM unsupported")));
}

#[test]
fn llvm_text_target_rejects_multi_block_cfg_until_phase_004() {
    let source = r#"
functio ramus(bivalens flag) → numerus {
    si flag {
        redde 1
    }
    redde 0
}
"#;

    let result = driver::compile(
        &Session::new(Config::default().with_target(Target::LlvmText)),
        "llvm.fab",
        source,
    );

    assert!(result.output.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic
        .message
        .contains("MIR-to-LLVM unsupported: branch")));
}
