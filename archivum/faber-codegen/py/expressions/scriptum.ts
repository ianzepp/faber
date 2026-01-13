/**
 * Python Code Generator - Scriptum Expression (format string)
 *
 * TRANSFORMS:
 *   scriptum("Hello, §!", name) -> "Hello, {}!".format(name)
 *   scriptum("§1 before §0", a, b) -> "{1} before {0}".format(a, b) (indexed)
 *
 * TARGET: Python's str.format() method for string formatting.
 *
 * WHY: Python uses {} placeholders natively in format strings,
 *      so the format string passes through directly after converting
 *      § placeholders to {}.
 *
 * NOTE: Supports both positional (§) and indexed (§0, §1) placeholders.
 *       Python's .format() natively supports positional indices like {0}, {1}.
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: PyGenerator): string {
    // Convert § placeholders to {} for Python .format()
    // §N becomes {N}, plain § becomes {}
    const format = (node.format.value as string).replace(/§(\d+)?/g, (_, idx) =>
        idx !== undefined ? `{${idx}}` : '{}',
    );

    if (node.arguments.length === 0) {
        // No args - just return the format string as a string literal
        return `"${format}"`;
    }

    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');

    return `"${format}".format(${args})`;
}
