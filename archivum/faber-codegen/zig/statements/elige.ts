/**
 * Zig Code Generator - EligeStatement (switch on values)
 *
 * TRANSFORMS:
 *   elige x { si 1 { a() } si 2 { b() } secus { c() } }
 *   -> if (x == 1) { a(); } else if (x == 2) { b(); } else { c(); }
 *
 *   elige status { si "pending" { ... } secus { ... } }
 *   -> if (std.mem.eql(u8, status, "pending")) { ... } else { ... }
 *
 * WHY: For value matching, use if-else chains since Zig switch
 *      requires exhaustive comptime-known values.
 */

import type { EligeStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genEligeStatement(node: EligeStatement, g: ZigGenerator): string {
    const discriminant = g.genExpression(node.discriminant);

    // Value matching: use if-else chain
    // Check if comparing strings (need std.mem.eql)
    const hasStringCase = node.cases.some(c => c.test.type === 'Literal' && typeof c.test.value === 'string');
    const isString = g.isStringType(node.discriminant) || hasStringCase;

    let result = '';
    let first = true;

    for (const caseNode of node.cases) {
        const test = g.genExpression(caseNode.test);
        const prefix = first ? '' : ' else ';
        first = false;

        // Use std.mem.eql for strings, == for everything else
        const condition = isString ? `std.mem.eql(u8, ${discriminant}, ${test})` : `(${discriminant} == ${test})`;

        result += `${g.ind()}${prefix}if (${condition}) {\n`;
        g.depth++;

        for (const stmt of caseNode.consequent.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;
    }

    if (node.defaultCase) {
        result += ` else {\n`;
        g.depth++;

        for (const stmt of node.defaultCase.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;
    }

    return result;
}
