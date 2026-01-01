/**
 * TypeScript Code Generator - Scriptum Expression (format string)
 *
 * TRANSFORMS:
 *   scriptum("Hello, {}!", name) -> `Hello, ${name}!`
 *   scriptum("obj = {{ {} }}", val) -> `obj = { ${val} }`
 *
 * TARGET: TypeScript template literals for string interpolation.
 *
 * WHY: TS has native template literals, so we transform {} placeholders
 *      into ${arg} interpolations. This provides the most idiomatic output.
 *
 * ESCAPING:
 *   {{ -> literal {
 *   }} -> literal }
 *   {} -> placeholder for next argument
 *
 * NOTE: Placeholder count must match argument count. Extra args are ignored,
 *       missing args produce undefined. Faber does not validate at compile time.
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: TsGenerator): string {
    const format = node.format.value as string;

    if (node.arguments.length === 0) {
        // No args - just return the format string as a regular string
        // Still need to unescape {{ and }}
        const unescaped = format.replace(/\{\{/g, '{').replace(/\}\}/g, '}');
        return `"${unescaped}"`;
    }

    // Transform format string with {} placeholders into template literal
    const args = node.arguments.map(arg => g.genExpression(arg));
    let argIndex = 0;

    // Process the format string character by character to handle escaping
    let template = '';
    let i = 0;
    while (i < format.length) {
        if (format[i] === '{') {
            if (format[i + 1] === '{') {
                // {{ -> literal {
                template += '{';
                i += 2;
            } else if (format[i + 1] === '}') {
                // {} -> placeholder
                const arg = args[argIndex++] ?? 'undefined';
                template += `\${${arg}}`;
                i += 2;
            } else {
                // lone { - pass through
                template += '{';
                i++;
            }
        } else if (format[i] === '}') {
            if (format[i + 1] === '}') {
                // }} -> literal }
                template += '}';
                i += 2;
            } else {
                // lone } - pass through
                template += '}';
                i++;
            }
        } else {
            template += format[i];
            i++;
        }
    }

    return `\`${template}\``;
}
