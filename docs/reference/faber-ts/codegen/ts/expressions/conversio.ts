/**
 * TypeScript Code Generator - Conversion Expression (type conversion)
 *
 * TRANSFORMS:
 *   "42" numeratum -> parseInt("42", 10)
 *   "ff" numeratum<i32, Hex> -> parseInt("ff", 16)
 *   "42" numeratum vel 0 -> parseInt("42", 10) || 0
 *   "3.14" fractatum -> parseFloat("3.14")
 *   42 textatum -> String(42)
 *   x bivalentum -> Boolean(x)
 *
 * WHY: TypeScript uses parseInt/parseFloat for parsing, String/Boolean for conversion.
 *      The `vel` fallback maps to logical OR since NaN is falsy in JavaScript.
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: Radix types map to numeric base values for parseInt
const RADIX_VALUES: Record<string, number> = {
    Dec: 10,
    Hex: 16,
    Oct: 8,
    Bin: 2,
};

export function genConversionExpression(node: ConversionExpression, g: TsGenerator): string {
    const expr = g.genExpression(node.expression);
    const fallback = node.fallback ? g.genExpression(node.fallback) : null;

    switch (node.conversion) {
        case 'numeratum': {
            const radix = node.radix ? RADIX_VALUES[node.radix] : 10;
            const parse = `parseInt(${expr}, ${radix})`;
            return fallback ? `(${parse} || ${fallback})` : parse;
        }

        case 'fractatum': {
            const parse = `parseFloat(${expr})`;
            return fallback ? `(${parse} || ${fallback})` : parse;
        }

        case 'textatum':
            // WHY: Infallible conversion, no fallback needed
            return `String(${expr})`;

        case 'bivalentum':
            // WHY: Infallible conversion following nonnulla semantics
            return `Boolean(${expr})`;
    }
}
