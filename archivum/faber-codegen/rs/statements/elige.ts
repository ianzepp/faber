/**
 * Rust Code Generator - EligeStatement
 *
 * TRANSFORMS:
 *   elige x { quando 1: ..., quando 2: ..., secus: ... }
 *   -> match x { 1 => ..., 2 => ..., _ => ... }
 */

import type { EligeStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatementInline } from './functio';

export function genEligeStatement(node: EligeStatement, g: RsGenerator): string {
    const discriminant = g.genExpression(node.discriminant);
    const lines: string[] = [];

    lines.push(`${g.ind()}match ${discriminant} {`);
    g.depth++;

    for (const caseNode of node.cases) {
        const test = g.genExpression(caseNode.test);
        const body = genBlockStatementInline(caseNode.consequent, g);
        lines.push(`${g.ind()}${test} => ${body},`);
    }

    if (node.defaultCase) {
        const body = genBlockStatementInline(node.defaultCase, g);
        lines.push(`${g.ind()}_ => ${body},`);
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
