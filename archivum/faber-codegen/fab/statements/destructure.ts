/**
 * Faber Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   DestructureDeclaration -> ex source kind specifiers
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: FabGenerator): string {
    const source = g.genExpression(node.source);

    const specifiers = node.specifiers.map(spec => {
        const rest = spec.rest ? 'ceteri ' : '';
        if (spec.imported.name === spec.local.name) {
            return `${rest}${spec.imported.name}`;
        }
        return `${rest}${spec.imported.name} ut ${spec.local.name}`;
    });

    return `${g.ind()}ex ${source} ${node.kind} ${specifiers.join(', ')}`;
}
