/**
 * TypeScript Code Generator - Collection DSL Expression
 *
 * TRANSFORMS:
 *   ex items prima 5       -> items.slice(0, 5)
 *   ex items ultima 3      -> items.slice(-3)
 *   ex prices summa        -> prices.reduce((a, b) => a + b, 0)
 *   ex items prima 5, ultima 2 -> items.slice(0, 5).slice(-2)
 *
 * WHY: DSL expressions provide concise collection pipelines.
 */

import type { CollectionDSLExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { applyDSLTransforms } from '../statements/iteratio';

export function genCollectionDSLExpression(node: CollectionDSLExpression, g: TsGenerator): string {
    const source = g.genExpression(node.source);
    return applyDSLTransforms(source, node.transforms, g);
}
