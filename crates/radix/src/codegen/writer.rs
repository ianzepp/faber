//! Indentation-aware code writer for target languages
//!
//! ARCHITECTURE OVERVIEW
//! =====================
//! CodeWriter manages string accumulation with automatic indentation tracking.
//! All codegen backends use this to emit properly formatted source code.
//!
//! COMPILER PHASE: Codegen (helper utility)
//! INPUT: Strings and formatting commands (indent, dedent, write)
//! OUTPUT: Properly indented source code string
//!
//! DESIGN PHILOSOPHY
//! =================
//! - Stateful indentation: Tracks current indent level and applies it automatically
//!   to every new line. This eliminates manual indentation management in backend code.
//!
//! - Line-aware writing: Only applies indentation after newlines, avoiding spurious
//!   whitespace in the middle of expressions.
//!
//! - Zero-copy finish: Consumes self to return the final string, preventing
//!   accidental continued writes after finalization.

/// Indentation-aware code writer.
///
/// WHY: Codegen backends emit complex nested structures (functions, classes,
/// blocks). Manual indentation is error-prone; this writer automates it by
/// tracking indent depth and inserting whitespace after every newline.
///
/// INVARIANTS:
/// ----------
/// INV-1: Indentation is only applied at the start of a new line (after '\n')
/// INV-2: indent/dedent calls affect future lines, not the current line
pub struct CodeWriter {
    buffer: String,
    indent: usize,
    indent_str: &'static str,
    at_line_start: bool,
}

impl CodeWriter {
    pub fn new() -> Self {
        Self { buffer: String::new(), indent: 0, indent_str: "    ", at_line_start: true }
    }

    /// Get the generated code.
    ///
    /// WHY: Consumes self to prevent accidental writes after finalization.
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Write a string, applying indentation to new lines.
    ///
    /// WHY: Character-by-character iteration ensures we detect every newline
    /// and apply indentation immediately, maintaining INV-1.
    pub fn write(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.buffer.push('\n');
                self.at_line_start = true;
            } else {
                // WHY: Apply pending indentation before first non-newline character
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

    /// Write a formatted string.
    ///
    /// WHY: Convenience wrapper for format!() arguments, avoiding repeated
    /// format!() calls at call sites.
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
    /// WHY: Saturates at zero to prevent underflow from mismatched indent/dedent pairs.
    pub fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    /// Write with temporary indent increase.
    ///
    /// WHY: Scoped indentation ensures indent/dedent pairs are always balanced,
    /// even when early returns or errors occur inside the closure.
    pub fn indented<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent();
        f(self);
        self.dedent();
    }

    /// Write a block with braces.
    ///
    /// WHY: Common pattern in Rust/C-like targets. Combines brace writing with
    /// automatic indentation management.
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

/// Macro for formatted writing.
///
/// WHY: Provides ergonomic write!()-style syntax for CodeWriter, matching
/// standard Rust formatting conventions.
#[macro_export]
macro_rules! write_code {
    ($w:expr, $($arg:tt)*) => {
        $w.writef(format_args!($($arg)*))
    };
}
