/**
 * Zig Code Generator - Arrow Function Expression
 *
 * TRANSFORMS:
 *   (x) => x + 1 -> @compileError("Arrow functions not supported in Zig")
 *
 * TARGET: Zig doesn't have arrow functions or lambdas as first-class values.
 *         Arrow functions don't have return type annotations in Faber, so
 *         they cannot be compiled to Zig. Use pro syntax with return type instead.
 *
 * LIMITATION: Arrow functions should be converted to lambdas with return types
 *             or named functions for Zig target.
 */

import type { ArrowFunctionExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genArrowFunction(_node: ArrowFunctionExpression, _g: ZigGenerator): string {
    return `@compileError("Arrow functions not supported in Zig - use 'pro x -> Type: expr' syntax")`;
}
