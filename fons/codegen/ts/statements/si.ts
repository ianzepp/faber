/**
 * TypeScript Code Generator - SiStatement
 *
 * TRANSFORMS:
 *   si (conditio) { ... } -> if (conditio) { ... }
 *   si (conditio) { ... } secus { ... } -> if (conditio) { ... } else { ... }
 *
 * WHY: Latin if-statements can have optional catch clauses for exception handling.
 *      When present, we wrap the entire if in a try-catch block.
 */

import type { SiStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genSiStatement(node: SiStatement, g: TsGenerator): string {
    let result = '';

    // WHY: Latin allows 'capta' (catch) clause on if-statements for brevity
    if (node.catchClause) {
        result += `${g.ind()}try {\n`;
        g.depth++;
        result += `${g.ind()}if (${g.genExpression(node.test)}) ${genBlockStatement(node.consequent, g)}`;
        g.depth--;
        result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;
    } else {
        result += `${g.ind()}if (${g.genExpression(node.test)}) ${genBlockStatement(node.consequent, g)}`;
    }

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            result += ` else ${genSiStatement(node.alternate, g).trim()}`;
        } else {
            result += ` else ${genBlockStatement(node.alternate, g)}`;
        }
    }

    return result;
}
