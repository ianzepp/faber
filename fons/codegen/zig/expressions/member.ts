/**
 * Zig Code Generator - Member Expression
 *
 * TRANSFORMS:
 *   obj.prop  -> obj.prop
 *   obj[key]  -> obj[key]
 *   obj?.prop -> if (obj) |o| o.prop else null (simplified)
 *   obj!.prop -> obj.?.prop (unwrap optional)
 */

import type { MemberExpression, RangeExpression, UnaryExpression, Literal, Identifier, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genMemberExpression(node: MemberExpression, g: ZigGenerator): string {
    const obj = g.genExpression(node.object);

    // GUARD: Computed access with slice syntax
    if (node.computed && node.property.type === 'RangeExpression') {
        return genSliceExpression(obj, node.property, g);
    }

    // GUARD: Computed access with negative index
    if (node.computed && isNegativeIndex(node.property)) {
        const negExpr = node.property as UnaryExpression;
        const absVal = g.genExpression(negExpr.argument);
        return `${obj}[${obj}.len - ${absVal}]`;
    }

    // GUARD: Computed access with non-null assertion
    if (node.computed && node.nonNull) {
        const prop = `[${g.genBareExpression(node.property)}]`;
        return `${obj}.?${prop}`;
    }

    // GUARD: Computed access with optional chaining
    if (node.computed && node.optional) {
        const prop = `[${g.genBareExpression(node.property)}]`;
        return `(if (${obj}) |_o| _o${prop} else null)`;
    }

    // GUARD: Computed access (standard)
    if (node.computed) {
        const prop = `[${g.genBareExpression(node.property)}]`;
        return `${obj}${prop}`;
    }

    const propName = (node.property as Identifier).name;

    // GUARD: Property access with non-null assertion
    if (node.nonNull) {
        return `${obj}.?.${propName}`;
    }

    // GUARD: Property access with optional chaining
    if (node.optional) {
        return `(if (${obj}) |_o| _o.${propName} else null)`;
    }

    // Standard property access
    return `${obj}.${propName}`;
}

/**
 * Check if an expression is a negative numeric literal.
 */
function isNegativeIndex(expr: Expression): boolean {
    if (expr.type === 'UnaryExpression' && expr.operator === '-' && expr.argument.type === 'Literal') {
        return typeof expr.argument.value === 'number';
    }
    if (expr.type === 'Literal' && typeof expr.value === 'number' && expr.value < 0) {
        return true;
    }
    return false;
}

/**
 * Generate slice expression from range inside brackets.
 *
 * TRANSFORMS:
 *   arr[1..3]       -> arr[1..3]
 *   arr[1 usque 3]  -> arr[1..4]  // inclusive adds 1 to end
 *   arr[-3..-1]     -> arr[arr.len-3..arr.len-1]
 *
 * WHY: Zig slices are exclusive like Faber's .. syntax.
 *      Inclusive ranges (usque) need end + 1 adjustment.
 *      Negative indices need conversion to len - n.
 */
function genSliceExpression(obj: string, range: RangeExpression, g: ZigGenerator): string {
    const start = genSliceIndex(obj, range.start, false, g);
    const end = genSliceIndex(obj, range.end, range.inclusive ?? false, g);
    return `${obj}[${start}..${end}]`;
}

/**
 * Generate a single slice index, handling negative indices and inclusive adjustment.
 */
function genSliceIndex(obj: string, expr: Expression, inclusive: boolean, g: ZigGenerator): string {
    // GUARD: Negative index with inclusive (e.g., usque -1 means to end)
    if (isNegativeIndex(expr) && inclusive) {
        const negExpr = expr as UnaryExpression;
        const absVal = (negExpr.argument as Literal).value as number;
        const adjusted = absVal - 1;
        if (adjusted === 0) {
            return `${obj}.len`;
        }
        return `${obj}.len - ${adjusted}`;
    }

    // GUARD: Negative index (exclusive)
    if (isNegativeIndex(expr)) {
        const negExpr = expr as UnaryExpression;
        const absVal = g.genExpression(negExpr.argument);
        return `${obj}.len - ${absVal}`;
    }

    // GUARD: Inclusive with literal - add 1 at compile time
    if (inclusive && expr.type === 'Literal' && typeof expr.value === 'number') {
        return String(expr.value + 1);
    }

    // GUARD: Inclusive with expression - add 1 at runtime
    if (inclusive) {
        return `${g.genExpression(expr)} + 1`;
    }

    // Standard index
    return g.genExpression(expr);
}
