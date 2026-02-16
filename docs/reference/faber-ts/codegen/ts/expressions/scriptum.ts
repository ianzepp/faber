/**
 * TypeScript Code Generator - Scriptum Expression (format string)
 *
 * TRANSFORMS:
 *   scriptum("Hello, §!", name) -> `Hello, ${name}!`
 *   scriptum("§1 before §0", a, b) -> `${b} before ${a}` (indexed)
 *
 * TARGET: TypeScript template literals for string interpolation.
 *
 * WHY: TS has native template literals, so we transform § placeholders
 *      into ${arg} interpolations. This provides the most idiomatic output.
 *
 * NOTE: Supports both positional (§) and indexed (§0, §1) placeholders.
 *       Placeholder count must match argument count. Extra args are ignored,
 *       missing args produce undefined. Faber does not validate at compile time.
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: TsGenerator): string {
    const format = node.format.value as string;

    if (node.arguments.length === 0) {
        return `"${format}"`;
    }

    // Transform format string with § placeholders into template literal
    // Supports both § (positional) and §0, §1, etc. (indexed)
    const args = node.arguments.map(arg => g.genExpression(arg));
    let implicitIdx = 0;

    let template = '';
    for (let i = 0; i < format.length; i++) {
        const char = format[i]!;
        if (char === '§') {
            // Check for explicit index (§0, §1, etc.)
            let explicitIdx = '';
            while (i + 1 < format.length) {
                const nextChar = format[i + 1]!;
                if (nextChar >= '0' && nextChar <= '9') {
                    explicitIdx += nextChar;
                    i++;
                }
                else {
                    break;
                }
            }

            const idx = explicitIdx ? parseInt(explicitIdx, 10) : implicitIdx++;
            const arg = args[idx] ?? 'undefined';
            template += `\${${arg}}`;
        }
        else {
            template += char;
        }
    }

    return `\`${template}\``;
}
