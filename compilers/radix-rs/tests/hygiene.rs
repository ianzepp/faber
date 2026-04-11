#![allow(clippy::absurd_extreme_comparisons)]

use std::fs;
use std::path::{Path, PathBuf};

const MAX_UNWRAP: usize = 0;
const MAX_EXPECT: usize = 5;
const MAX_PANIC: usize = 0;
const MAX_UNREACHABLE: usize = 8;
const MAX_TODO: usize = 0;
const MAX_UNIMPLEMENTED: usize = 0;
const MAX_LET_UNDERSCORE: usize = 6;

#[derive(Clone)]
struct SourceFile {
    path: PathBuf,
    content: String,
    scrubbed: String,
}

fn source_files() -> Vec<SourceFile> {
    let mut files = Vec::new();
    collect_rs_files(Path::new("src"), &mut files);
    files
}

fn collect_rs_files(dir: &Path, out: &mut Vec<SourceFile>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.ends_with("_test.rs") || name.ends_with(".test.rs") {
            continue;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };
        let scrubbed = scrub_rust_source(&content);
        out.push(SourceFile { path, content, scrubbed });
    }
}

fn scrub_rust_source(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        String,
        Char,
    }

    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut state = State::Code;

    while let Some(ch) = chars.next() {
        match state {
            State::Code => match ch {
                '/' if chars.peek() == Some(&'/') => {
                    out.push(' ');
                    out.push(' ');
                    chars.next();
                    state = State::LineComment;
                }
                '/' if chars.peek() == Some(&'*') => {
                    out.push(' ');
                    out.push(' ');
                    chars.next();
                    state = State::BlockComment;
                }
                '"' => {
                    out.push(' ');
                    state = State::String;
                }
                '\'' => {
                    out.push(' ');
                    state = State::Char;
                }
                _ => out.push(ch),
            },
            State::LineComment => {
                if ch == '\n' {
                    out.push('\n');
                    state = State::Code;
                } else {
                    out.push(' ');
                }
            }
            State::BlockComment => {
                if ch == '*' && chars.peek() == Some(&'/') {
                    out.push(' ');
                    out.push(' ');
                    chars.next();
                    state = State::Code;
                } else if ch == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            State::String => {
                if ch == '\\' {
                    out.push(' ');
                    if let Some(escaped) = chars.next() {
                        out.push(if escaped == '\n' { '\n' } else { ' ' });
                    }
                } else if ch == '"' {
                    out.push(' ');
                    state = State::Code;
                } else if ch == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
            State::Char => {
                if ch == '\\' {
                    out.push(' ');
                    if let Some(escaped) = chars.next() {
                        out.push(if escaped == '\n' { '\n' } else { ' ' });
                    }
                } else if ch == '\'' {
                    out.push(' ');
                    state = State::Code;
                } else if ch == '\n' {
                    out.push('\n');
                } else {
                    out.push(' ');
                }
            }
        }
    }

    out
}

fn count_substring(haystack: &str, needle: &str) -> usize {
    haystack.match_indices(needle).count()
}

fn count_expect(file: &SourceFile) -> usize {
    count_substring(&file.scrubbed, ".expect(") - count_substring(&file.scrubbed, "self.expect(")
}

fn count_let_underscore(file: &SourceFile) -> usize {
    file.scrubbed
        .lines()
        .filter(|line| line.contains("let _ ="))
        .count()
}

fn companion_test_path(path: &Path) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_string_lossy();
    Some(path.with_file_name(format!("{stem}_test.rs")))
}

fn assert_budget(name: &str, observed: usize, budget: usize) {
    assert!(observed <= budget, "{name} budget exceeded: found {observed}, max {budget}.");
}

#[test]
fn unwrap_budget() {
    let count = source_files()
        .iter()
        .map(|file| count_substring(&file.scrubbed, ".unwrap()"))
        .sum();
    assert_budget(".unwrap()", count, MAX_UNWRAP);
}

#[test]
fn expect_budget() {
    let count = source_files().iter().map(count_expect).sum();
    assert_budget(".expect(", count, MAX_EXPECT);
}

#[test]
fn panic_budget() {
    let count = source_files()
        .iter()
        .map(|file| count_substring(&file.scrubbed, "panic!("))
        .sum();
    assert_budget("panic!(", count, MAX_PANIC);
}

#[test]
fn unreachable_budget() {
    let count = source_files()
        .iter()
        .map(|file| count_substring(&file.scrubbed, "unreachable!("))
        .sum();
    assert_budget("unreachable!(", count, MAX_UNREACHABLE);
}

#[test]
fn todo_budget() {
    let count = source_files()
        .iter()
        .map(|file| count_substring(&file.scrubbed, "todo!("))
        .sum();
    assert_budget("todo!(", count, MAX_TODO);
}

#[test]
fn unimplemented_budget() {
    let count = source_files()
        .iter()
        .map(|file| count_substring(&file.scrubbed, "unimplemented!("))
        .sum();
    assert_budget("unimplemented!(", count, MAX_UNIMPLEMENTED);
}

#[test]
fn let_underscore_budget() {
    let count = source_files().iter().map(count_let_underscore).sum();
    assert_budget("let _ =", count, MAX_LET_UNDERSCORE);
}

#[test]
fn companion_tests_use_cfg_path_module_convention() {
    let files = source_files();

    for file in &files {
        let Some(companion) = companion_test_path(&file.path) else {
            continue;
        };
        if !companion.exists() {
            continue;
        }

        let companion_name = companion.file_name().unwrap_or_default().to_string_lossy();
        let expected = format!("#[cfg(test)]\n#[path = \"{companion_name}\"]\nmod tests;");
        assert!(
            file.content.contains(&expected),
            "{} has companion test {}, but is missing the repo convention:\n{}",
            file.path.display(),
            companion.display(),
            expected
        );
    }
}
