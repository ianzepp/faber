/**
 * TypeScript Code Generator - EligeStatement
 *
 * TRANSFORMS:
 *   elige x { si 1 { a() } si 2 { b() } secus { c() } }
 *   -> if (x === 1) { a(); } else if (x === 2) { b(); } else { c(); }
 *
 * WHY: Always emit if/else for all targets. Simpler codegen, no type
 *      detection needed, and downstream compilers optimize anyway.
 */

import type { EligeStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genEligeStatement(node: EligeStatement, g: TsGenerator): string {
    const discriminant = g.genExpression(node.discriminant);
    let result = '';

    if (node.catchClause) {
        result += `${g.ind()}try {\n`;
        g.depth++;
    }

    // Generate if/else chain
    for (let i = 0; i < node.cases.length; i++) {
        const caseNode = node.cases[i]!;
        const keyword = i === 0 ? 'if' : 'else if';

        // Value matching: si expression { ... }
        const test = g.genExpression(caseNode.test);
        result += `${g.ind()}${keyword} (${discriminant} === ${test}) {\n`;
        g.depth++;

        for (const stmt of caseNode.consequent.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;

        // Add newline if more cases or default follows
        if (i < node.cases.length - 1 || node.defaultCase) {
            result += '\n';
        }
    }

    if (node.defaultCase) {
        // Add "else" for default case
        if (node.cases.length > 0) {
            result += `${g.ind()}else {\n`;
        } else {
            // No cases, just default - emit as bare block
            result += `${g.ind()}{\n`;
        }

        g.depth++;

        for (const stmt of node.defaultCase.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}}`;
    }

    if (node.catchClause) {
        g.depth--;
        result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return result;
}
