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

#[test]
fn optional_struct_fields_wrap_pointer_values() {
    let code = compile_go(
        r#"genus Address {
  textus city
  si textus state
}

genus User {
  textus name
  si Address address
}

incipit {
  fixum alice ← {
    name: "Alice",
    address: { city: "Roma", state: "Italia" } ⇢ Address
  } ⇢ User
}"#,
    );

    assert!(code.contains("Address: func() *Address"));
    assert!(code.contains("State: func() *string"));
}

#[test]
fn optional_chain_members_emit_nil_safe_access() {
    let code = compile_go(
        r#"genus Address {
  textus city
}

genus User {
  si Address address
}

incipit {
  fixum bob ← {} ⇢ User
  fixum city ← bob?.address?.city
  scribe city
}"#,
    );

    assert!(code.contains("if v == nil { return nil }"));
    assert!(code.contains("func() *Address"));
    assert!(code.contains("return &v.City"));
    assert!(!code.contains("func() **Address"));
}

#[test]
fn optional_chain_preserves_optional_fields_without_double_wrap() {
    let code = compile_go(
        r#"genus Address {
  si textus state
}

genus User {
  si Address address
}

incipit {
  fixum alice ← {} ⇢ User
  fixum state ← alice?.address?.state
  scribe state
}"#,
    );

    assert!(code.contains("func() *string"));
    assert!(code.contains("return v.State"));
    assert!(!code.contains("return &v.State"));
    assert!(!code.contains("func() **string"));
}

#[test]
fn optional_coalesce_returns_inner_value() {
    let code = compile_go(
        r#"genus Address {
  textus city
}

genus User {
  si Address address
}

incipit {
  fixum bob ← {} ⇢ User
  fixum city ← bob?.address?.city vel "Unknown"
  scribe city
}"#,
    );

    assert!(code.contains("func() string"));
    assert!(code.contains("if v != nil { return *v }"));
    assert!(code.contains(r#"return "Unknown""#));
}

#[test]
fn explicit_optional_local_initializers_wrap_pointer_values() {
    let code = compile_go(
        r#"incipit {
  fixum si textus name ← "Marcus"
  fixum display ← name vel "Anonymous"
  scribe display
}"#,
    );

    assert!(code.contains("name := func() *string"));
    assert!(code.contains("if v != nil { return *v }"));
}

#[test]
fn qua_primitive_and_array_conversions_emit_go_conversions() {
    let code = compile_go(
        r#"functio getData() → lista<numerus> {
  redde [1, 2, 3]
}

incipit {
  fixum data ← 42
  fixum asText ← data ⇢ textus
  fixum input ← "100"
  fixum asNum ← input ⇢ numerus
  fixum value ← 1
  fixum asBool ← value ⇢ bivalens
  fixum raw ← getData()
  fixum items ← raw ⇢ lista<textus>
  scribe asText, asNum, asBool, items
}"#,
    );

    assert!(code.contains("asText := fmt.Sprint(data)"));
    assert!(code.contains("asNum := func() int { v, _ := strconv.Atoi(fmt.Sprint(input)); return v }()"));
    assert!(code.contains("asBool := (value != 0)"));
    assert!(code.contains("out[i] = fmt.Sprint(value)"));
}

#[test]
fn innatum_empty_and_nonempty_lists_emit_typed_slices() {
    let code = compile_go(
        r#"incipit {
  fixum empty ← [] ⇢ lista<textus>
  fixum nums ← [1, 2, 3] ⇢ lista<numerus>
  scribe empty, nums
}"#,
    );

    assert!(code.contains("empty := []string{}"));
    assert!(code.contains("nums := []int{1, 2, 3}"));
}

#[test]
fn integer_division_promotes_when_expression_type_is_fractus() {
    let code = compile_go(
        r#"functio divide(numerus a, numerus b) → fractus {
  redde a / b
}"#,
    );

    assert!(code.contains("return (float64(a) / float64(b))"));
}

#[test]
fn custodi_nested_guards_emit_statement_if_chain() {
    let code = compile_go(
        r#"functio processValue(numerus x) → numerus {
  custodi {
    si x < 0 {
      redde -1
    }
    si x > 100 {
      redde -1
    }
  }

  redde x * 2
}"#,
    );

    assert!(code.contains("if (x > 100) {"));
    assert!(code.contains("if (x < 0) {"));
    assert!(code.contains("return -1"));
    assert!(!code.contains("func() any"));
}
