//! Compilation driver
//!
//! Orchestrates the compilation pipeline from source to output.

mod session;
mod source;

pub use session::{Session, Config};
pub use source::SourceFile;

use crate::{CompileResult, Output};
use crate::lexer;
use crate::parser;
use crate::semantic::{self, PassConfig};
use crate::codegen::{self, Target};
use crate::diagnostics::Diagnostic;

/// Run the full compilation pipeline
pub fn compile(session: &Session, name: &str, source: &str) -> CompileResult {
    let mut diagnostics = Vec::new();

    // Phase 1: Lexing
    let lex_result = lexer::lex(source);
    if !lex_result.success() {
        for err in &lex_result.errors {
            diagnostics.push(Diagnostic::from_lex_error(name, source, err));
        }
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    // Phase 2: Parsing
    let parse_result = parser::parse(lex_result);
    if !parse_result.success() {
        for err in &parse_result.errors {
            diagnostics.push(Diagnostic::from_parse_error(name, source, err));
        }
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    let program = parse_result.program.unwrap();

    // Phase 3: Semantic analysis
    let pass_config = PassConfig::for_target(session.config.target);
    let semantic_result = semantic::analyze(&program, &pass_config);

    for err in &semantic_result.errors {
        diagnostics.push(Diagnostic::from_semantic_error(name, source, err));
    }

    if !semantic_result.success() {
        return CompileResult {
            output: None,
            diagnostics,
        };
    }

    let hir = semantic_result.hir.unwrap();

    // Phase 4: Code generation
    let crate_name = session.config.crate_name.as_deref().unwrap_or("output");
    match codegen::generate(session.config.target, &hir, &semantic_result.types, crate_name) {
        Ok(output) => CompileResult {
            output: Some(output),
            diagnostics,
        },
        Err(err) => {
            diagnostics.push(Diagnostic::codegen_error(&err.message));
            CompileResult {
                output: None,
                diagnostics,
            }
        }
    }
}
