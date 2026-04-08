use crate::codegen::Target;
use crate::driver::{Config, Session};

fn compile_go(source: &str) -> String {
    let session = Session::new(Config::default().with_target(Target::Go));
    let result = crate::driver::compile(&session, "<test>", source);
    assert!(
        result.success(),
        "compilation failed: {:?}",
        result.diagnostics
    );
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
