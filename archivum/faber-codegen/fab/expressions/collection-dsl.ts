/**
 * Faber Code Generator - CollectionDSLExpression
 */

import type { CollectionDSLExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genCollectionDSLExpression(node: CollectionDSLExpression, g: FabGenerator): string {
    const source = g.genExpression(node.source);

    const transforms = node.transforms.map(t => {
        if (t.argument) {
            return `${t.verb} ${g.genExpression(t.argument)}`;
        }
        return t.verb;
    });

    return `ex ${source} ${transforms.join(' ')}`;
}
