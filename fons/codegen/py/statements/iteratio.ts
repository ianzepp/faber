/**
 * Python Code Generator - IteratioStatement
 *
 * TRANSFORMS:
 *   ex 0..10 pro i { } -> for i in range(0, 10):
 *   ex items pro item { } -> for item in items:
 *   ex stream fiet chunk { } -> async for chunk in stream:
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIteratioStatement(node: IteratioStatement, g: PyGenerator): string {
    const lines: string[] = [];
    const varName = node.variable.name;
    const asyncKw = node.async ? 'async ' : '';

    if (node.catchClause) {
        lines.push(`${g.ind()}try:`);
        g.depth++;
    }

    // Check if iterable is a range expression
    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);
        // WHY: Python range() is exclusive, so add 1 for inclusive ranges
        const endExpr = range.inclusive ? `${end} + 1` : end;

        let rangeCall: string;
        if (range.step) {
            const step = g.genExpression(range.step);
            rangeCall = `range(${start}, ${endExpr}, ${step})`;
        } else {
            rangeCall = `range(${start}, ${endExpr})`;
        }

        lines.push(`${g.ind()}${asyncKw}for ${varName} in ${rangeCall}:`);
    } else {
        const iterable = g.genExpression(node.iterable);
        lines.push(`${g.ind()}${asyncKw}for ${varName} in ${iterable}:`);
    }

    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
    g.depth--;

    if (node.catchClause) {
        g.depth--;
        lines.push(`${g.ind()}except Exception as ${node.catchClause.param.name}:`);
        g.depth++;
        lines.push(g.genBlockStatementContent(node.catchClause.body));
        g.depth--;
    }

    return lines.join('\n');
}
