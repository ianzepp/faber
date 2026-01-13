/**
 * Faber Code Generator - TemptaStatement (try-catch-finally)
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genTemptaStatement(node: TemptaStatement, g: FabGenerator): string {
    let result = `${g.ind()}tempta ${genBlockStatement(node.block, g)}`;

    if (node.handler) {
        result += ` cape ${node.handler.param.name} ${genBlockStatement(node.handler.body, g)}`;
    }

    if (node.finalizer) {
        result += ` demum ${genBlockStatement(node.finalizer, g)}`;
    }

    return result;
}
