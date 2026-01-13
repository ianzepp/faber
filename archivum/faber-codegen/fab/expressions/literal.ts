/**
 * Faber Code Generator - Literal
 */

import type { Literal } from '../../../parser/ast';

export function genLiteral(node: Literal): string {
    // Use raw for faithful representation
    if (node.raw) {
        return node.raw;
    }

    // Fallback
    if (node.value === null) {
        return 'nihil';
    }
    if (node.value === true) {
        return 'verum';
    }
    if (node.value === false) {
        return 'falsum';
    }
    if (typeof node.value === 'string') {
        return `"${node.value}"`;
    }
    if (typeof node.value === 'bigint') {
        return `${node.value}n`;
    }
    return String(node.value);
}
