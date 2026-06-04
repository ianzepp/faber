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
    assert!(result
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("MIR-to-LLVM unsupported: branch")));
}
