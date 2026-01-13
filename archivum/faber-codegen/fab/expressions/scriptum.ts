/**
 * Faber Code Generator - ScriptumExpression
 */

import type { ScriptumExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genScriptumExpression(node: ScriptumExpression, g: FabGenerator): string {
    const format = node.format.raw;
    const args = node.arguments.map(arg => g.genExpression(arg));

    if (args.length === 0) {
        return `scriptum(${format})`;
    }

    return `scriptum(${format}, ${args.join(', ')})`;
}
