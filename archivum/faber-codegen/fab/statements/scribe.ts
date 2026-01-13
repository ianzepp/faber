/**
 * Faber Code Generator - ScribeStatement (print)
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: FabGenerator): string {
    const keyword = node.level === 'debug' ? 'vide' : node.level === 'warn' ? 'mone' : 'scribe';
    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');
    return `${g.ind()}${keyword} ${args}`;
}
