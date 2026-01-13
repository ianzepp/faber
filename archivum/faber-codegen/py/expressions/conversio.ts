/**
 * Python Code Generator - Conversion Expression (type conversion)
 *
 * TRANSFORMS:
 *   "42" numeratum -> int("42")
 *   "ff" numeratum<i32, Hex> -> int("ff", 16)
 *   "42" numeratum vel 0 -> int("42") if "42".lstrip('-').isdigit() else 0
 *   "3.14" fractatum -> float("3.14")
 *   42 textatum -> str(42)
 *   x bivalentum -> bool(x)
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

const RADIX_VALUES: Record<string, number> = {
    Dec: 10,
    Hex: 16,
    Oct: 8,
    Bin: 2,
};

export function genConversionExpression(node: ConversionExpression, g: PyGenerator): string {
    const expr = g.genExpression(node.expression);
    const fallback = node.fallback ? g.genExpression(node.fallback) : null;

    switch (node.conversion) {
        case 'numeratum': {
            const radix = node.radix ? RADIX_VALUES[node.radix] : 10;
            if (radix === 10) {
                if (fallback) {
                    // WHY: Python int() throws on invalid input, need try/except pattern
                    return `(int(${expr}) if str(${expr}).lstrip('-').isdigit() else ${fallback})`;
                }
                return `int(${expr})`;
            }
            // WHY: Non-decimal radix uses second argument
            if (fallback) {
                return `(int(${expr}, ${radix}) if ${expr} else ${fallback})`;
            }
            return `int(${expr}, ${radix})`;
        }

        case 'fractatum': {
            if (fallback) {
                // WHY: Python float() throws on invalid input
                return `(float(${expr}) if ${expr}.replace('.', '', 1).lstrip('-').isdigit() else ${fallback})`;
            }
            return `float(${expr})`;
        }

        case 'textatum':
            return `str(${expr})`;

        case 'bivalentum':
            return `bool(${expr})`;
    }
}
