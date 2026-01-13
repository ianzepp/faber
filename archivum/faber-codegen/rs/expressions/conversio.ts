/**
 * Rust Code Generator - Conversion Expression (type conversion)
 *
 * TRANSFORMS:
 *   "42" numeratum -> "42".parse::<i64>().unwrap()
 *   "ff" numeratum<i32, Hex> -> i32::from_str_radix("ff", 16).unwrap()
 *   "42" numeratum vel 0 -> "42".parse::<i64>().unwrap_or(0)
 *   "3.14" fractatum -> "3.14".parse::<f64>().unwrap()
 *   42 textatum -> 42.to_string()
 *   x bivalentum -> x != 0  (for numbers) or !x.is_empty() (for strings)
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

const RADIX_VALUES: Record<string, number> = {
    Dec: 10,
    Hex: 16,
    Oct: 8,
    Bin: 2,
};

export function genConversionExpression(node: ConversionExpression, g: RsGenerator): string {
    const expr = g.genExpression(node.expression);
    const fallback = node.fallback ? g.genExpression(node.fallback) : null;

    switch (node.conversion) {
        case 'numeratum': {
            // WHY: Rust uses different types for integers
            const targetType = node.targetType ? g.genType(node.targetType) : 'i64';

            if (node.radix && node.radix !== 'Dec') {
                // WHY: Non-decimal radix requires from_str_radix
                const radix = RADIX_VALUES[node.radix];
                if (fallback) {
                    return `${targetType}::from_str_radix(${expr}, ${radix}).unwrap_or(${fallback})`;
                }
                return `${targetType}::from_str_radix(${expr}, ${radix}).unwrap()`;
            }

            if (fallback) {
                return `${expr}.parse::<${targetType}>().unwrap_or(${fallback})`;
            }
            return `${expr}.parse::<${targetType}>().unwrap()`;
        }

        case 'fractatum': {
            const targetType = node.targetType ? g.genType(node.targetType) : 'f64';
            if (fallback) {
                return `${expr}.parse::<${targetType}>().unwrap_or(${fallback})`;
            }
            return `${expr}.parse::<${targetType}>().unwrap()`;
        }

        case 'textatum':
            return `${expr}.to_string()`;

        case 'bivalentum':
            // WHY: Rust doesn't have truthiness, need explicit comparison
            return `(${expr} != 0)`;
    }
}
