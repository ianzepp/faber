use super::{analyze, CliMode, CliType};
use crate::lexer::lex;
use crate::parser::parse;

fn analyze_source(source: &str) -> super::CliAnalysis {
    let result = parse(lex(source));
    assert!(
        result.errors.is_empty(),
        "expected parse success, got {:?}",
        result
            .errors
            .iter()
            .map(|err| err.message.as_str())
            .collect::<Vec<_>>()
    );
    let program = result.program.as_ref().expect("program");
    analyze(program, &result.interner)
}

fn messages(analysis: &super::CliAnalysis) -> Vec<&str> {
    analysis
        .errors
        .iter()
        .map(|err| err.message.as_str())
        .collect()
}

#[test]
fn detects_not_cli_programs() {
    let analysis = analyze_source("incipit {}");

    assert_eq!(analysis.mode, CliMode::NotCli);
    assert!(analysis.program.is_none());
    assert!(analysis.errors.is_empty());
}

#[test]
fn builds_single_command_cli_ir() {
    let analysis = analyze_source(
        r#"
@ cli "salve"
@ versio "1.0.0"
@ descriptio "Greets people"
@ optio name longum "name" typus textus vel "Roma"
@ optio loud brevis "l" longum "loud" typus bivalens
@ operandus ceteri textus words
incipit argumenta args {}
"#,
    );

    assert!(analysis.errors.is_empty(), "{:?}", messages(&analysis));
    let program = analysis.program.expect("cli program");
    assert_eq!(program.mode, CliMode::SingleCommand);
    assert_eq!(program.name, "salve");
    assert_eq!(program.entry_args, "args");
    assert_eq!(program.version.as_deref(), Some("1.0.0"));
    assert_eq!(program.options.len(), 2);
    assert_eq!(program.options[0].ty, CliType::Textus);
    assert_eq!(program.options[1].ty, CliType::Bivalens);
    assert!(program.options[1].flag);
    assert!(matches!(program.options[1].default, Some(super::CliDefault::Bool(false))));
    assert_eq!(program.operands.len(), 1);
    assert!(program.operands[0].rest);
}

#[test]
fn builds_subcommand_cli_ir_with_globals() {
    let analysis = analyze_source(
        r#"
@ cli "tool"
@ optio verbose brevis "v" longum "verbose" typus bivalens ubique
incipit argumenta args {}

@ imperium "jobs/list"
@ alias "ls"
@ descriptio "List jobs"
@ optio limit longum "limit" typus numerus vel 20
functio list() argumenta args {}
"#,
    );

    assert!(analysis.errors.is_empty(), "{:?}", messages(&analysis));
    let program = analysis.program.expect("cli program");
    assert_eq!(program.mode, CliMode::Subcommand);
    assert_eq!(program.global_options.len(), 1);
    assert_eq!(program.commands.len(), 1);
    assert_eq!(program.commands[0].path, vec!["jobs", "list"]);
    assert_eq!(program.commands[0].aliases, vec!["ls"]);
    assert_eq!(program.commands[0].args_binding.as_deref(), Some("args"));
}

#[test]
fn requires_cli_entry_argument_binding() {
    let analysis = analyze_source(
        r#"
@ cli "tool"
incipit {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("incipit argumenta")));
}

#[test]
fn validates_duplicate_flags_bindings_and_rest_ordering() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
@ optio left brevis "x"
@ optio right brevis "x"
@ operandus ceteri textus files
@ operandus textus trailing
incipit argumenta args {}
"#,
    );
    let messages = messages(&analysis);

    assert!(messages
        .iter()
        .any(|message| message.contains("duplicate short flag")));
    assert!(messages
        .iter()
        .any(|message| message.contains("ceteri operand must be the final operand")));
    assert!(messages
        .iter()
        .any(|message| message.contains("appears after ceteri operand")));
}

#[test]
fn rejects_global_local_collisions() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
@ optio config longum "config" ubique
incipit argumenta args {}

@ imperium "run"
@ optio config longum "local-config"
functio run() {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("collides with a global CLI binding")));
}

#[test]
fn rejects_single_command_global_local_collisions() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
@ optio config longum "config" ubique
@ optio config longum "local-config"
incipit argumenta args {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("single-command option 'config' collides with a global CLI binding")));
}

#[test]
fn rejects_non_empty_root_body_for_subcommand_cli() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
incipit argumenta args {
  scribe "setup"
}

@ imperium "run"
functio run() {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("empty root incipit body")));
}

#[test]
fn rejects_module_mounted_commands_until_phase_05() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
@ imperia "jobs" ex jobs
incipit argumenta args {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("Phase 05")));
}

#[test]
fn rejects_alias_collision_with_later_command_path() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
incipit argumenta args {}

@ imperium "one"
@ alias "two"
functio one() {}

@ imperium "two"
functio two() {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("command alias 'two' collides with a command path")));
}

#[test]
fn rejects_unsupported_cli_types() {
    let analysis = analyze_source(
        r#"
@ cli "bad"
@ optio output longum "output" typus tabula<textus, textus>
incipit argumenta args {}
"#,
    );

    assert!(messages(&analysis)
        .iter()
        .any(|message| message.contains("unsupported CLI type")));
}
