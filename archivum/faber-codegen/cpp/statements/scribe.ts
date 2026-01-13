/**
 * C++23 Code Generator - ScribeStatement
 *
 * TRANSFORMS:
 *   scribe "hello" -> std::print("hello\n");
 *   scribe x, y -> std::print("{} {}\n", x, y);
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: CppGenerator): string {
    g.includes.add('<print>');

    const prefix = node.level === 'debug' ? '[DEBUG] ' : node.level === 'warn' ? '[WARN] ' : '';

    if (node.arguments.length === 0) {
        return `${g.ind()}std::print("${prefix}\\n");`;
    }

    // Build format string with {} placeholders
    const formatParts = node.arguments.map(() => '{}');
    const format = prefix + formatParts.join(' ') + '\\n';
    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');

    return `${g.ind()}std::print("${format}", ${args});`;
}
