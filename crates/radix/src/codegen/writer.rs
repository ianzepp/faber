//! Scoped, line-aware text writer for codegen backends.
//!
//! Backends emit target source incrementally while walking HIR. This helper is
//! the shared formatting boundary: it owns indentation state, line-start
//! detection, and the small scoped APIs that keep block-shaped output from
//! scattering manual whitespace bookkeeping through every target module.
//!
//! The writer deliberately stays text-oriented. It does not understand target
//! syntax, comments, or tokens; backend modules still decide what to write. Its
//! only policy is when indentation is inserted and how nested formatting scopes
//! are balanced.
//!
//! INVARIANTS
//! ==========
//! - Indentation is emitted only before the first non-newline character on a
//!   line.
//! - `indent` and `dedent` affect future line starts, not text already written
//!   on the current line.
//! - Scoped helpers restore indentation depth after their closure returns.
//! - Finalization consumes the writer so no later writes can silently diverge
//!   from the returned source string.

/// Indentation-aware code writer.
///
/// The struct is intentionally small and backend-agnostic. It provides enough
/// structure for Rust, TypeScript, Go, Faber, and the temporary MIR Rust probe
/// to share formatting mechanics without coupling them to one target's syntax.
pub struct CodeWriter {
    buffer: String,
    indent: usize,
    indent_str: &'static str,
    at_line_start: bool,
}

impl CodeWriter {
    /// Create an empty writer using four-space indentation.
    pub fn new() -> Self {
        Self { buffer: String::new(), indent: 0, indent_str: "    ", at_line_start: true }
    }

    /// Finish emission and return the accumulated source text.
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Write text while applying indentation at each line start.
    ///
    /// Newlines do not themselves receive indentation. The next non-newline
    /// character triggers the pending indentation, which avoids trailing spaces
    /// on intentionally blank lines.
    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.buffer.push('\n');
                self.at_line_start = true;
            } else {
                if self.at_line_start {
                    for _ in 0..self.indent {
                        self.buffer.push_str(self.indent_str);
                    }
                    self.at_line_start = false;
                }
                self.buffer.push(c);
            }
        }
    }

    /// Write preformatted [`std::fmt::Arguments`] through the same line policy.
    pub fn writef(&mut self, args: std::fmt::Arguments<'_>) {
        let s = format!("{}", args);
        self.write(&s);
    }

    /// Write a line (string + newline).
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    /// Write an empty line.
    pub fn newline(&mut self) {
        self.write("\n");
    }

    /// Increase indentation for subsequent lines.
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation for subsequent lines.
    ///
    /// Saturation keeps formatting helpers from underflowing after a mismatched
    /// manual `dedent`. Structural correctness still belongs to the caller; the
    /// writer only guarantees it will not panic while recovering.
    pub fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    /// Run a closure with one extra indentation level.
    ///
    /// This is the preferred API for nested emission because it makes the
    /// formatting scope local to the code that writes the nested body.
    pub fn indented<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent();
        f(self);
        self.dedent();
    }

    /// Write a brace-delimited block with a scoped indented body.
    ///
    /// The helper is intentionally punctuation-only: it fits Rust, TypeScript,
    /// and Go-style blocks, while targets with different delimiters can compose
    /// `writeln`, `indented`, and `write` directly.
    pub fn block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.writeln("{");
        self.indented(f);
        self.write("}");
    }
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Formatted-writing shorthand for [`CodeWriter`].
///
/// Backend code uses this macro at call sites where standard `write!` syntax
/// reads better than constructing `format_args!` manually, while still routing
/// through [`CodeWriter::write`] for line-start indentation.
#[macro_export]
macro_rules! write_code {
    ($w:expr, $($arg:tt)*) => {
        $w.writef(format_args!($($arg)*))
    };
}
