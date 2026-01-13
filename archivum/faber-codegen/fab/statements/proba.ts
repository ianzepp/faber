/**
 * Faber Code Generator - ProbaStatement (test case)
 */

import type { ProbaStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genProbaStatement(node: ProbaStatement, g: FabGenerator): string {
    let modifier = '';
    if (node.modifier) {
        modifier = ` ${node.modifier} "${node.modifierReason}"`;
    }

    return `${g.ind()}proba${modifier} "${node.name}" ${genBlockStatement(node.body, g)}`;
}
