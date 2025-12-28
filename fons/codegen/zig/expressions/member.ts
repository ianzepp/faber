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

    if (node.computed) {
        // Check for slice syntax: arr[1..3] or arr[1 usque 3]
        // Zig has native slice syntax: arr[1..3]
        if (node.property.type === 'RangeExpression') {
            return genSliceExpression(obj, node.property, g);
        }

        // Check for negative index - Zig doesn't support negative indices
        // Need to compute arr.len - n
        if (isNegativeIndex(node.property)) {
            const negExpr = node.property as UnaryExpression;
            const absVal = g.genExpression(negExpr.argument);
            return `${obj}[${obj}.len - ${absVal}]`;
        }

        // WHY: Use genBareExpression to avoid unnecessary parens around index
        const prop = `[${g.genBareExpression(node.property)}]`;

        // WHY: Zig's optional unwrap uses .? syntax
        if (node.nonNull) {
            return `${obj}.?${prop}`;
        }

        // WHY: Zig's optional chaining requires if-else pattern
        if (node.optional) {
            return `(if (${obj}) |_o| _o${prop} else null)`;
        }

        return `${obj}${prop}`;
    }

    const prop = `.${(node.property as Identifier).name}`;

    // WHY: Zig's optional unwrap uses .? syntax
    if (node.nonNull) {
        return `${obj}.?${prop}`;
    }

    // WHY: Zig's optional chaining requires if-else pattern
    //      This is a simplified version; full impl would need temp vars
    if (node.optional) {
        const propName = (node.property as Identifier).name;
        return `(if (${obj}) |_o| _o.${propName} else null)`;
    }

    return `${obj}${prop}`;
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
    let start: string;
    let end: string;

    // Handle start - check for negative
    if (isNegativeIndex(range.start)) {
        const negExpr = range.start as UnaryExpression;
        const absVal = g.genExpression(negExpr.argument);
        start = `${obj}.len - ${absVal}`;
    } else {
        start = g.genExpression(range.start);
    }

    // Handle end - check for negative and inclusive
    if (isNegativeIndex(range.end)) {
        const negExpr = range.end as UnaryExpression;
        const absVal = (negExpr.argument as Literal).value as number;
        if (range.inclusive) {
            const inclusiveEnd = absVal - 1;
            if (inclusiveEnd === 0) {
                // Inclusive -1 means to the end
                end = `${obj}.len`;
            } else {
                end = `${obj}.len - ${inclusiveEnd}`;
            }
        } else {
            end = `${obj}.len - ${absVal}`;
        }
    } else if (range.inclusive) {
        // Positive inclusive end
        if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
            end = String(range.end.value + 1);
        } else {
            end = `${g.genExpression(range.end)} + 1`;
        }
    } else {
        end = g.genExpression(range.end);
    }

    return `${obj}[${start}..${end}]`;
}
