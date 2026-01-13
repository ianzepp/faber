/**
 * Rust Code Generator - IteratioStatement
 *
 * TRANSFORMS:
 *   pro i ex 0..10 { } -> for i in 0..10 { }
 *   pro i ex 0...10 { } -> for i in 0..=10 { }
 *   pro item ex items { } -> for item in items { }
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIteratioStatement(node: IteratioStatement, g: RsGenerator): string {
    const varName = node.variable.name;
    const body = genBlockStatement(node.body, g);

    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);
        const rangeOp = range.inclusive ? '..=' : '..';

        return `${g.ind()}for ${varName} in ${start}${rangeOp}${end} ${body}`;
    }

    const iterable = g.genExpression(node.iterable);

    return `${g.ind()}for ${varName} in ${iterable} ${body}`;
}
