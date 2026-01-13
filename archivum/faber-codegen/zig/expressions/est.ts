/**
 * Zig Code Generator - Est Expression (type check)
 *
 * TRANSFORMS:
 *   x est textus      -> @TypeOf(x) == []const u8  (compile-time)
 *   x est numerus     -> @TypeOf(x) == i64         (compile-time)
 *   x est persona     -> @TypeOf(x) == persona     (compile-time)
 *   x non est textus  -> @TypeOf(x) != []const u8
 *
 * TARGET: Zig is statically typed - all type checks are compile-time.
 *         @TypeOf gives the type at comptime.
 *
 * LIMITATION: Runtime type checks don't exist in Zig. This emits comptime
 *             comparisons that the compiler will constant-fold to true/false.
 *
 * NOTE: For null checks, use `nihil x` or `nonnihil x` unary operators.
 */

import type { EstExpression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genEstExpression(node: EstExpression, g: ZigGenerator): string {
    const expr = g.genExpression(node.expression);
    const op = node.negated ? '!=' : '==';

    // Compile-time type comparison
    const targetType = g.genType(node.targetType);
    return `(@TypeOf(${expr}) ${op} ${targetType})`;
}
