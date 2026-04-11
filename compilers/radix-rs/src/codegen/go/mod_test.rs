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
fn ad_is_rejected_for_go_targets() {
    let session = Session::new(Config::default().with_target(Target::Go));
    let result = crate::driver::compile(
        &session,
        "<test>",
        r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro content {
    scribe content
  }
}"#,
    );

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.is_error() && d.message.contains("ad is not supported for Go targets")));
}

#[test]
fn ad_with_catch_is_rejected_for_go_targets() {
    let session = Session::new(Config::default().with_target(Target::Go));
    let result = crate::driver::compile(
        &session,
        "<test>",
        r#"incipit {
  ad "fasciculus:lege" ("hello.txt") → textus pro content {
    scribe content
  } cape err {
    scribe err
  }
}"#,
    );

    assert!(!result.success());
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.is_error() && d.message.contains("ad is not supported for Go targets")));
}

#[test]
fn nested_object_honors_enclosing_map_value_type() {
    let code = compile_go(
        r#"incipit {
  fixum nested ← { outer: { inner: 1 } }
  scribe nested
}"#,
    );

    assert!(code.contains(r#"nested := map[string]map[string]int{"outer": map[string]int{"inner": 1}}"#));
    assert!(!code.contains(r#"map[string]map[string]any{"outer": map[string]any{"inner": 1}}"#));
}

#[test]
fn map_member_access_asserts_precise_types_when_map_values_are_any() {
    let code = compile_go(
        r#"incipit {
  fixum nested ← { outer: { inner: { deep: "found" } } }
  fixum data ← { items: ["first", "second", "third"] }
  scribe nested.outer.inner.deep
  scribe data.items[0]
}"#,
    );

    assert!(code.contains(r#"nested := map[string]map[string]map[string]string{"outer": map[string]map[string]string{"inner": map[string]string{"deep": "found"}}}"#));
    assert!(code.contains(r#"nested["outer"]["inner"]["deep"]"#));
    assert!(code.contains(r#"data := map[string][]any{"items": []any{"first", "second", "third"}}"#));
    assert!(code.contains(r#"data["items"][0]"#));
}

#[test]
fn optional_map_members_deref_pointer_maps_and_unknown_maps_explicitly() {
    let code = compile_go(
        r#"incipit {
  fixum maybe ← { present: { value: 100 } }
  scribe maybe?.present?.value
  fixum empty ← nihil
  scribe empty?.missing
}"#,
    );

    assert!(code.contains("func() *int"));
    assert!(code.contains("base := *v; value, ok := base[\"value\"]"));
    assert!(code.contains("m, ok := v.(map[string]any); if !ok { return nil }; value, ok := m[\"missing\"]"));
    assert!(!code.contains("&v.Missing"));
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
fn inferred_optional_nil_local_emits_typed_var() {
    let code = compile_go(
        r#"incipit {
  varia maybe ← nihil ⇢ si textus
  scribe maybe
}"#,
    );

    assert!(code.contains("var maybe *string = nil"));
    assert!(!code.contains("maybe := nil"));
}

#[test]
fn ternary_optional_result_coerces_branches_once() {
    let code = compile_go(
        r#"incipit {
  varia maybe ← nihil ⇢ si textus
  fixum result ← nonnihil maybe sic maybe secus "default"
  scribe result
}"#,
    );

    assert!(code.contains("result := func() *string"));
    assert!(code.contains("return maybe"));
    assert!(code.contains(r#"return func() *string { v := "default"; return &v }()"#));
    assert!(!code.contains("v := func() *string"));
    assert!(!code.contains("func() **string"));
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

#[test]
fn ego_fields_emit_self_access_in_methods() {
    let code = compile_go(
        r#"genus Counter {
  numerus value: 0

  functio increment() {
    ego.value ← ego.value + 1
  }

  functio get() → numerus {
    redde ego.value
  }
}"#,
    );

    assert!(code.contains("self.Value = (self.Value + 1)"));
    assert!(code.contains("return self.Value"));
    assert!(!code.contains("Counter.Value"));
}

#[test]
fn returning_ego_and_chaining_uses_value_return_with_pointer_temps() {
    let code = compile_go(
        r#"genus Calculator {
  numerus value: 0

  functio setValue(numerus n) → Calculator {
    ego.value ← n
    redde ego
  }

  functio double() → Calculator {
    ego.value ← ego.value * 2
    redde ego
  }

  functio getResult() → numerus {
    redde ego.value
  }
}

incipit {
  varia calc ← {} ⇢ Calculator
  fixum result ← calc.setValue(5).double().getResult()
  scribe result
}"#,
    );

    assert!(code.contains("return *self"));
    assert!(code.contains("calc.SetValue(5)"));
    assert!(code.contains("return &v"));
    assert!(code.contains("().Double(); return &v"));
    assert!(code.contains("().GetResult()"));
}

#[test]
fn finge_and_ordo_variants_emit_struct_values() {
    let code = compile_go(
        r#"discretio Status {
  Active,
  Click { numerus x, numerus y }
}

incipit {
  fixum Status s ← finge Active ⇢ Status
  fixum Status e ← finge Click { x: 1, y: 2 } ⇢ Status
}"#,
    );

    assert!(code.contains("s := Active{}"));
    assert!(code.contains("e := Click{X: 1, Y: 2}"));
    assert!(!code.contains(".(Status)"));
}

#[test]
fn translated_slice_helpers_emit_go_loops() {
    let code = compile_go(
        r#"incipit {
  varia items ← [1, 2, 3] ⇢ lista<numerus>
  fixum first ← items.primus()
  fixum doubled ← items.mappata(clausura numerus x: x * 2)
  fixum evens ← items.filtrata(clausura numerus x: x % 2 ≡ 0)
  fixum extended ← items.addita(4)
  fixum reversed ← items.inversa()
  fixum sorted ← items.ordinata()
  items.inverte()
  scribe first, doubled, evens, extended, reversed, sorted
}"#,
    );

    assert!(code.contains("first := items[0]"));
    assert!(code.contains("out[i] = mapper(value)"));
    assert!(code.contains("if pred(value) { out = append(out, value) }"));
    assert!(code.contains("out = append(out, 4)"));
    assert!(code.contains("out[i], out[j] = out[j], out[i]"));
    assert!(code.contains("sort.Slice(out, func(i, j int) bool { return out[i] < out[j] })"));
    assert!(code
        .contains("for i, j := 0, len(items)-1; i < j; i, j = i+1, j-1 { items[i], items[j] = items[j], items[i] }"));
}

#[test]
fn spread_calls_recover_fixed_arity_array_arguments() {
    let code = compile_go(
        r#"functio add(numerus a, numerus b) → numerus {
  redde a + b
}

incipit {
  fixum numerus[] numbers ← [3, 7]
  fixum total ← add(sparge numbers)
  scribe total
}"#,
    );

    assert!(code.contains("total := add(numbers[0], numbers[1])"));
}

#[test]
fn proba_names_are_sanitized_for_go_functions() {
    let code = compile_go(
        r#"proba "one plus one equals two" {
  adfirma 1 + 1 ≡ 2
}"#,
    );

    assert!(code.contains("func one_plus_one_equals_two()"));
}
