/**
 * Faber Code Generator - InitiumStatement (canonical emit)
 *
 * TRANSFORMS:
 *   initium { body } -> initium { body }
 *
 * TARGET: Faber canonical form preserves the initium block.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genInitiumStatement(node: InitiumStatement, g: FabGenerator): string {
    const body = genBlockStatement(node.body, g);
    return `${g.ind()}initium ${body}`;
}
