/**
 * Rust Code Generator - Lege Expression (read stdin)
 *
 * TRANSFORMS:
 *   lege        -> { let mut buffer = String::new(); std::io::stdin().read_to_string(&mut buffer).unwrap_or(0); buffer }
 *   lege lineam -> { let mut line = String::new(); std::io::stdin().read_line(&mut line).unwrap_or(0); line.trim().to_string() }
 *
 * TARGET: Rust's std::io::stdin() for reading input as String.
 *
 * WHY: Bootstrap compiler needs to read source from stdin.
 *      Uses inline block expressions to manage mutable buffers.
 *      Returns empty string on error for consistency with other targets.
 */

import type { LegeExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genLegeExpression(node: LegeExpression, g: RsGenerator): string {
    // Track feature usage for preamble generation
    g.features.stdin = true;

    if (node.mode === 'line') {
        // WHY: read_line() appends to buffer including newline, so trim() removes it
        // Returns usize (bytes read) on success, we ignore and use the string content
        return '{ let mut line = String::new(); std::io::stdin().read_line(&mut line).unwrap_or(0); line.trim().to_string() }';
    }

    // Read all of stdin into string buffer
    // WHY: read_to_string() reads entire input stream until EOF
    return '{ let mut buffer = String::new(); std::io::stdin().read_to_string(&mut buffer).unwrap_or(0); buffer }';
}