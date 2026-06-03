//! Source loading and location formatting for CLI commands.

use std::io::{self, Read};
use std::path::PathBuf;

/// Read source from a file argument or stdin.
///
/// This is the process-facing loader for single-file commands. It exits on I/O
/// failure because command handlers already report diagnostics through stderr,
/// while library callers should use `Compiler` directly for non-exiting flows.
pub fn read_source(args: &[String]) -> (String, String) {
    if args.is_empty() || args[0] == "-" {
        let mut source = String::new();
        io::stdin()
            .read_to_string(&mut source)
            .unwrap_or_else(|err| {
                eprintln!("error: failed to read stdin: {err}");
                std::process::exit(1);
            });
        ("<stdin>".to_owned(), source)
    } else {
        let path = PathBuf::from(&args[0]);
        let source = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            std::process::exit(1);
        });
        (args[0].clone(), source)
    }
}

pub(crate) fn source_file_from_input(name: String, source: String) -> crate::driver::SourceFile {
    crate::driver::SourceFile::inline(name, source)
}

/// Format an offset in a loaded source file for terminal diagnostics.
pub fn format_location(source_file: &crate::driver::SourceFile, offset: u32) -> String {
    let (line, column) = source_file.offset_to_line_col(offset);
    format!("{}:{}:{}", source_file.name.as_str(), line, column)
}

pub(crate) fn format_optional_location(
    source_file: &crate::driver::SourceFile,
    span: Option<crate::lexer::Span>,
) -> String {
    span.map(|span| format_location(source_file, span.start))
        .unwrap_or_else(|| source_file.name.clone())
}
