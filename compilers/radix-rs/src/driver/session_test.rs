use super::{Config, Session};
use crate::codegen::Target;
use std::path::PathBuf;

#[test]
fn config_default_values_are_stable() {
    let config = Config::default();

    assert_eq!(config.target, Target::Rust);
    assert!(config.emit_comments);
    assert!(!config.strict);
    assert!(config.stdlib_path.is_none());
}

#[test]
fn config_builders_apply_expected_fields() {
    let stdlib = PathBuf::from("/tmp/norma");
    let config = Config::new()
        .with_target(Target::Faber)
        .with_stdlib(stdlib.clone())
        .strict();

    assert_eq!(config.target, Target::Faber);
    assert_eq!(config.stdlib_path, Some(stdlib));
    assert!(config.strict);
}

#[test]
fn session_wraps_config() {
    let config = Config::default().with_target(Target::Faber);
    let session = Session::new(config);

    assert_eq!(session.config.target, Target::Faber);
}
