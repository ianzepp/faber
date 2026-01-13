/**
 * C++23 Code Generator - IteratioStatement
 *
 * TRANSFORMS:
 *   ex items pro item { } -> for (auto& item : items) { }
 *   ex 0..10 pro i { } -> for (int64_t i = 0; i < 10; ++i) { }
 */

import type { IteratioStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIteratioStatement(node: IteratioStatement, g: CppGenerator): string {
    const varName = node.variable.name;
    const body = genBlockStatement(node.body, g);

    // Handle range expressions
    if (node.iterable.type === 'RangeExpression') {
        const range = node.iterable;
        const start = g.genExpression(range.start);
        const end = g.genExpression(range.end);
        const step = range.step ? g.genExpression(range.step) : '1';
        const cmp = range.inclusive ? '<=' : '<';

        return `${g.ind()}for (int64_t ${varName} = ${start}; ${varName} ${cmp} ${end}; ${varName} += ${step}) ${body}`;
    }

    const iterable = g.genExpression(node.iterable);

    // Range-based for
    return `${g.ind()}for (auto& ${varName} : ${iterable}) ${body}`;
}
