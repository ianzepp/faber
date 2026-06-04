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
    assert!(output.code.contains("%l0.addr = alloca i64"));
    assert!(output.code.contains("%load0 = load i64, ptr %l0.addr"));
    assert!(output.code.contains("%load1 = load i64, ptr %l1.addr"));
    assert!(output.code.contains("add i64 %load0, %load1"));
    assert!(output.code.contains("ret i64 %load"));
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
    assert!(output.code.contains("fadd double %load0, %load1"));
    assert!(output.code.contains("fdiv double %load2, 2.0"));
    assert!(output
        .code
        .contains("define i1 @minor(double %l0, double %l1)"));
    assert!(output.code.contains("fcmp olt double %load0, %load1"));
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

    assert!(output.code.contains("xor i1 %load0, true"));
    assert!(output.code.contains("or i1 %load1, %load2"));
    assert!(output.code.contains("and i1 %load3, %load4"));
    assert!(output.code.contains("icmp eq i1 %load0, %load1"));
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

    assert!(output.code.contains("icmp slt i64 %load0, %load1"));
    assert!(output.code.contains("icmp eq i64 %load0, %load1"));
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
fn llvm_text_target_emits_branch_return_cfg() {
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

    let Some(Output::LlvmText(output)) = result.output else {
        panic!("expected LLVM text output");
    };

    assert!(output.code.contains("bb0:"));
    assert!(output.code.contains("br i1 %load0, label %bb1, label %bb2"));
    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("ret i64 %load1"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("ret i64 %load2"));
}

#[test]
fn llvm_text_target_emits_branch_join_cfg() {
    let source = r#"
functio positum(numerus n) → numerus {
    fixum numerus x ← n > 0 sic 1 secus 0
    redde x
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

    assert!(output.code.contains("bb0:"));
    assert!(output.code.contains("br i1 %load1, label %bb1, label %bb2"));
    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("br label %bb3"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("br label %bb3"));
    assert!(output.code.contains("bb3:"));
    assert!(output.code.contains("ret i64 %load2"));
}

#[test]
fn llvm_text_target_emits_simple_loop_cfg() {
    let source = r#"
functio countdown(numerus n) → numerus {
    varia numerus x ← n
    dum x > 0 {
        x ← x - 1
    }
    redde x
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

    assert!(output.code.contains("bb1:"));
    assert!(output.code.contains("br i1 %load2, label %bb2, label %bb3"));
    assert!(output.code.contains("bb2:"));
    assert!(output.code.contains("sub i64 %load3, 1"));
    assert!(output.code.contains("br label %bb1"));
    assert!(output.code.contains("bb3:"));
    assert!(output.code.contains("ret i64 %load5"));
}

#[test]
fn llvm_text_target_still_rejects_switch_cfg() {
    let source = r#"
functio status(numerus code) → numerus {
    elige code {
        casu 200 {
            redde 1
        }
        ceterum {
            redde 0
        }
    }
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
        .contains("MIR-to-LLVM unsupported: switch")));
}
