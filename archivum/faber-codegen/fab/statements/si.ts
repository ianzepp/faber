/**
 * Faber Code Generator - SiStatement
 *
 * STYLE: Uses sin/secus (canonical), not sin/secus
 */

import type { SiStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genSiStatement(node: SiStatement, g: FabGenerator): string {
    let result = `${g.ind()}si ${g.genExpression(node.test)} ${genBlockStatement(node.consequent, g)}`;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            // Use sin for else-if (canonical)
            result += ` sin ${g.genExpression(node.alternate.test)} ${genBlockStatement(node.alternate.consequent, g)}`;

            // Handle deeper chain
            if (node.alternate.alternate) {
                if (node.alternate.alternate.type === 'SiStatement') {
                    result += genSiAlternateChain(node.alternate.alternate, g);
                } else {
                    result += ` secus ${genBlockStatement(node.alternate.alternate, g)}`;
                }
            }
        } else {
            // Use secus for else (canonical)
            result += ` secus ${genBlockStatement(node.alternate, g)}`;
        }
    }

    // Handle catch clause
    if (node.catchClause) {
        result += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return result;
}

function genSiAlternateChain(node: SiStatement, g: FabGenerator): string {
    let result = ` sin ${g.genExpression(node.test)} ${genBlockStatement(node.consequent, g)}`;

    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            result += genSiAlternateChain(node.alternate, g);
        } else {
            result += ` secus ${genBlockStatement(node.alternate, g)}`;
        }
    }

    return result;
}
