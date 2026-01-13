/**
 * Faber Code Generator - FacBlockStatement (do block or do-while loop)
 *
 * TRANSFORMS:
 *   fac { x() } -> fac { x() }
 *   fac { x() } cape e { y() } -> fac { x() } cape e { y() }
 *   fac { x() } dum cond -> fac { x() } dum cond
 *   fac { x() } cape e { y() } dum cond -> fac { x() } cape e { y() } dum cond
 */

import type { FacBlockStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genFacBlockStatement(node: FacBlockStatement, g: FabGenerator): string {
    let result = `${g.ind()}fac ${genBlockStatement(node.body, g)}`;

    if (node.catchClause) {
        result += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    if (node.test) {
        result += ` dum ${g.genExpression(node.test)}`;
    }

    return result;
}
