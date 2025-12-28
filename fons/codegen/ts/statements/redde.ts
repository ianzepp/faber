/**
 * TypeScript Code Generator - ReddeStatement
 *
 * TRANSFORMS:
 *   redde x -> return x;
 *   redde -> return;
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: TsGenerator, semi: boolean): string {
    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)}${semi ? ';' : ''}`;
    }

    return `${g.ind()}return${semi ? ';' : ''}`;
}
