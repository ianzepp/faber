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

#[test]
fn nested_object_honors_enclosing_map_value_type() {
    let code = compile_go(
        r#"incipit {
  fixum nested ← { outer: { inner: 1 } }
  scribe nested
}"#,
    );

    assert!(code.contains(r#"nested := map[string]map[string]any{"outer": map[string]any{"inner": 1}}"#));
    assert!(!code.contains(r#"map[string]map[string]any{"outer": map[string]int{"inner": 1}}"#));
}

#[test]
fn discerne_variants_emit_type_switch_with_returns() {
    let code = compile_go(
        r#"discretio Status {
  Active,
  Inactive
}

functio describe(Status status) → textus {
  discerne status {
    casu Active { redde "active" }
    casu Inactive { redde "inactive" }
  }
}"#,
    );

    assert!(code.contains("switch any(status).(type) {"));
    assert!(code.contains("case Active:"));
    assert!(code.contains(r#"return "active""#));
    assert!(code.contains("default:"));
}

#[test]
fn discerne_variant_bindings_extract_fields() {
    let code = compile_go(
        r#"discretio Event {
  Click { numerus x, numerus y },
  Quit
}

functio handle(Event event) {
  discerne event {
    casu Click fixum x, y {
      scribe x
      scribe y
    }
    casu Quit {
      scribe "quit"
    }
  }
}"#,
    );

    assert!(code.contains("case Click:"));
    assert!(code.contains("x := __radixCase.X"));
    assert!(code.contains("y := __radixCase.Y"));
}
