/**
 * Faber Code Generator - InStatement (with statement)
 */

import type { InStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genInStatement(node: InStatement, g: FabGenerator): string {
    return `${g.ind()}in ${g.genExpression(node.object)} ${genBlockStatement(node.body, g)}`;
}
