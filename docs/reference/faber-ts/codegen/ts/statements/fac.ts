/**
 * TypeScript Code Generator - FacBlockStatement
 *
 * TRANSFORMS:
 *   fac { x() } -> { x(); }
 *   fac { x() } cape e { y() } -> try { x(); } catch (e) { y(); }
 *   fac { x() } dum cond -> do { x(); } while (cond)
 *   fac { x() } cape e { y() } dum cond -> try { do { x(); } while (cond) } catch (e) { y(); }
 *
 * WHY: fac alone is just a scope block. With cape, it becomes try-catch.
 *      With dum, it becomes a do-while loop (body executes first).
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: TsGenerator): string {
    if (node.test) {
        // Do-while loop: fac { body } dum condition
        const test = g.genExpression(node.test);
        const body = genBlockStatement(node.body, g);

        if (node.catchClause) {
            // With cape: try { do { } while () } catch (e) { }
            let result = `${g.ind()}try {\n`;

            g.depth++;
            result += `${g.ind()}do ${body} while (${test});\n`;
            g.depth--;
            result += `${g.ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;

            return result;
        }

        return `${g.ind()}do ${body} while (${test});`;
    }

    if (node.catchClause) {
        // With cape, emit as try-catch
        let result = `${g.ind()}try ${genBlockStatement(node.body, g)}`;
        result += ` catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;
        return result;
    }

    // Without cape, just emit the block
    return `${g.ind()}${genBlockStatement(node.body, g)}`;
}
