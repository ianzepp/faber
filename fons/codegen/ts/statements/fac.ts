/**
 * TypeScript Code Generator - FacBlockStatement
 *
 * TRANSFORMS:
 *   fac { x() } -> { x(); }
 *   fac { x() } cape e { y() } -> try { x(); } catch (e) { y(); }
 *
 * WHY: fac alone is just a scope block. With cape, it becomes try-catch.
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: TsGenerator): string {
    if (node.catchClause) {
        // With cape, emit as try-catch
        let result = `${g.ind()}try ${genBlockStatement(node.body, g)}`;
        result += ` catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body, g)}`;
        return result;
    }

    // Without cape, just emit the block
    return `${g.ind()}${genBlockStatement(node.body, g)}`;
}
