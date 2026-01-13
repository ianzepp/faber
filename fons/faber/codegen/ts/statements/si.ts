/**
 * TypeScript Code Generator - SiStatement
 *
 * TRANSFORMS:
 *   si cond { block }                    -> if (cond) { block }
 *   si cond { block } secus { alt }      -> if (cond) { block } else { alt }
 *   si cond { block } cape err { hand }  -> if (cond) { try { block } catch (err) { hand } }
 *   si cond { b } cape e { h } secus { a } -> if (cond) { try { b } catch (e) { h } } else { a }
 *
 * WHY: Latin 'cape' clause wraps the consequent block in try-catch, not the entire if.
 *      This allows catching errors inside the if-true branch while still having else.
 */

import type { SiStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genSiStatement(node: SiStatement, g: TsGenerator): string {
    // Generate the core if-else chain without try-catch wrapper
    const genCoreIf = (): string => {
        let result = `${g.ind()}if (${g.genExpression(node.test)}) ${genBlockStatement(node.consequent, g)}`;

        if (node.alternate) {
            if (node.alternate.type === 'SiStatement') {
                result += ` else ${genSiStatement(node.alternate, g).trim()}`;
            }
            else {
                result += ` else ${genBlockStatement(node.alternate, g)}`;
            }
        }

        return result;
    };

    // WHY: 'cape' wraps the ENTIRE if statement in try-catch so we catch
    // errors from the condition evaluation, not just the consequent
    if (node.catchClause) {
        let result = `${g.ind()}try {\n`;
        g.depth++;
        result += genCoreIf() + '\n';
        g.depth--;
        result += `${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;
        return result;
    }

    return genCoreIf();
}
