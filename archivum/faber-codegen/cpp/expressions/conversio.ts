/**
 * C++ Code Generator - Conversion Expression (type conversion)
 *
 * TRANSFORMS:
 *   "42" numeratum -> std::stoll("42")
 *   "ff" numeratum<i32, Hex> -> std::stoi("ff", nullptr, 16)
 *   "42" numeratum vel 0 -> [&]{ try { return std::stoll("42"); } catch(...) { return 0LL; } }()
 *   "3.14" fractatum -> std::stod("3.14")
 *   42 textatum -> std::to_string(42)
 *   x bivalentum -> static_cast<bool>(x)
 */

import type { ConversionExpression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

const RADIX_VALUES: Record<string, number> = {
    Dec: 10,
    Hex: 16,
    Oct: 8,
    Bin: 2,
};

export function genConversionExpression(node: ConversionExpression, g: CppGenerator): string {
    const expr = g.genExpression(node.expression);
    const fallback = node.fallback ? g.genExpression(node.fallback) : null;

    switch (node.conversion) {
        case 'numeratum': {
            const radix = node.radix ? RADIX_VALUES[node.radix] : 10;
            // WHY: C++ uses different functions for different int sizes
            const func = radix === 10 ? 'std::stoll' : 'std::stoll';
            const radixArg = radix !== 10 ? `, nullptr, ${radix}` : '';

            if (fallback) {
                // WHY: C++ needs IIFE for try/catch in expression context
                return `[&]{ try { return ${func}(${expr}${radixArg}); } catch(...) { return static_cast<long long>(${fallback}); } }()`;
            }
            return `${func}(${expr}${radixArg})`;
        }

        case 'fractatum': {
            if (fallback) {
                return `[&]{ try { return std::stod(${expr}); } catch(...) { return static_cast<double>(${fallback}); } }()`;
            }
            return `std::stod(${expr})`;
        }

        case 'textatum':
            return `std::to_string(${expr})`;

        case 'bivalentum':
            return `static_cast<bool>(${expr})`;
    }
}
