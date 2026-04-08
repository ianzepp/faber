use crate::codegen::Target;
use crate::driver::{Config, Session};

fn compile_go(source: &str) -> String {
    let session = Session::new(Config::default().with_target(Target::Go));
    let result = crate::driver::compile(&session, "<test>", source);
    assert!(result.success(), "compilation failed: {:?}", result.diagnostics);
    match result.output {
        Some(crate::Output::Go(out)) => out.code,
        other => panic!("expected Go output, got {:?}", other.map(|_| "other")),
    }
}

#[test]
fn empty_incipit_emits_main() {
    let code = compile_go("incipit {}");
    assert!(code.contains("package main"));
    assert!(code.contains("func main()"));
}

#[test]
fn function_emits_func() {
    let code = compile_go("functio salve() { scribe(1) }");
    assert!(code.contains("func salve()"));
}

#[test]
fn scribe_emits_println() {
    let code = compile_go("incipit { scribe(42) }");
    assert!(code.contains("fmt.Println(42)"));
}

#[test]
fn tempta_catch_binds_recovered_value() {
    let code = compile_go(
        r#"incipit {
  tempta {
    iace "ignis"
  } cape err {
    scribe err
  }
}"#,
    );

    assert!(code.contains("err := r"));
    assert!(!code.contains("var err any"));
}

#[test]
fn ad_with_binding_emits_dispatch_and_bound_body() {
    let code = compile_go(
        r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro content {
    scribe content
  }
}"#,
    );

    assert!(code.contains("func radixAd[T any]("));
    assert!(code.contains(
        "if __radixResult, __radixErr := radixAd[string](\"fasciculus:lege\", \"hello.txt\"); __radixErr != nil"
    ));
    assert!(code.contains("content := __radixResult"));
    assert!(code.contains("fmt.Println(content)"));
}

#[test]
fn ad_with_catch_binds_error_value() {
    let code = compile_go(
        r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro content {
    scribe content
  } cape err {
    scribe err
  }
}"#,
    );

    assert!(code.contains("err := __radixErr"));
    assert!(!code.contains("var err any"));
}
