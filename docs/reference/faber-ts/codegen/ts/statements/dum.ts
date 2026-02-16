/**
 * TypeScript Code Generator - DumStatement
 *
 * TRANSFORMS:
 *   dum (conditio) { ... } -> while (conditio) { ... }
 *   dum (conditio) { ... } cape e { ... } -> try { while (conditio) { ... } } catch (e) { ... }
 */

import type { DumStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genDumStatement(node: DumStatement, g: TsGenerator): string {
    const test = g.genExpression(node.test);
    const body = genBlockStatement(node.body, g);

    if (node.catchClause) {
        let result = `${g.ind()}try {\n`;

        g.depth++;
        result += `${g.ind()}while (${test}) ${body}`;
        g.depth--;
        result += `\n${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;

        return result;
    }

    return `${g.ind()}while (${test}) ${body}`;
}
