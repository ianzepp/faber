/**
 * Faber Code Generator - IncipietStatement (canonical emit)
 *
 * TRANSFORMS:
 *   incipiet { body } -> incipiet { body }
 *   incipiet ergo stmt -> incipiet ergo stmt
 *
 * TARGET: Faber canonical form preserves the incipiet block.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIncipietStatement(node: IncipietStatement, g: FabGenerator): string {
    if (node.ergoStatement) {
        return `${g.ind()}incipiet ergo ${g.genStatement(node.ergoStatement)}`;
    }
    const body = genBlockStatement(node.body!, g);
    return `${g.ind()}incipiet ${body}`;
}
