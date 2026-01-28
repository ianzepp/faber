//! Code writer with indentation support

use std::fmt::Write;

/// Indentation-aware code writer
pub struct CodeWriter {
    buffer: String,
    indent: usize,
    indent_str: &'static str,
    at_line_start: bool,
}

impl CodeWriter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent: 0,
            indent_str: "    ",
            at_line_start: true,
        }
    }

    /// Get the generated code
    pub fn finish(self) -> String {
        self.buffer
    }

    /// Write a string
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

    /// Write a formatted string
    pub fn writef(&mut self, args: std::fmt::Arguments<'_>) {
        let s = format!("{}", args);
        self.write(&s);
    }

    /// Write a line
    pub fn writeln(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    /// Write an empty line
    pub fn newline(&mut self) {
        self.write("\n");
    }

    /// Increase indentation
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    /// Decrease indentation
    pub fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    /// Write with temporary indent increase
    pub fn indented<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.indent();
        f(self);
        self.dedent();
    }

    /// Write a block with braces
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

/// Macro for formatted writing
#[macro_export]
macro_rules! write_code {
    ($w:expr, $($arg:tt)*) => {
        $w.writef(format_args!($($arg)*))
    };
}
