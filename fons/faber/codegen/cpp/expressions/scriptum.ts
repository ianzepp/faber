/**
 * C++23 Code Generator - Scriptum Expression (format string)
 *
 * TRANSFORMS:
 *   scriptum("Hello, §!", name) -> std::format("Hello, {}!", name)
 *   scriptum("§1 before §0", a, b) -> std::format("{1} before {0}", a, b) (indexed)
 *
 * TARGET: C++20's std::format for string formatting.
 *
 * WHY: C++20 introduced {} placeholder syntax matching Python/Rust,
 *      so the format string passes through directly after converting
 *      § placeholders to {}.
 *
 * NOTE: Requires C++20 or later. For older standards, would need sprintf.
 *       Supports both positional (§) and indexed (§0, §1) placeholders.
 *       C++20 std::format natively supports positional indices like {0}, {1}.
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: CppGenerator): string {
    // Convert § placeholders to {} for C++ std::format
    // §N becomes {N}, plain § becomes {}
    const format = (node.format.value as string).replace(/§(\d+)?/g, (_, idx) =>
        idx !== undefined ? `{${idx}}` : '{}',
    );

    if (node.arguments.length === 0) {
        // No args - just return the format string as a string literal
        return `"${format}"`;
    }

    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');

    return `std::format("${format}", ${args})`;
}
