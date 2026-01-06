/**
 * Faber Code Generator - Conversion Expression (identity transform)
 *
 * TRANSFORMS:
 *   "42" numeratum -> "42" numeratum
 *   "ff" numeratum<i32, Hex> -> "ff" numeratum<i32, Hex>
 *   "42" numeratum vel 0 -> "42" numeratum vel 0
 *
 * WHY: Faber-to-Faber codegen is identity - re-emit the source syntax.
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genConversionExpression(node: ConversionExpression, g: FabGenerator): string {
    const expr = g.genExpression(node.expression);
    let result = `${expr} ${node.conversion}`;

    // WHY: Type parameters only valid for numeratum/fractatum
    if (node.targetType) {
        const targetType = g.genType(node.targetType);
        if (node.radix) {
            result += `<${targetType}, ${node.radix}>`;
        }
        else {
            result += `<${targetType}>`;
        }
    }

    if (node.fallback) {
        const fallback = g.genExpression(node.fallback);
        result += ` vel ${fallback}`;
    }

    return result;
}
