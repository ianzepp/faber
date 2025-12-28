/**
 * C++23 Code Generator - MemberExpression
 *
 * TRANSFORMS:
 *   obj.prop      -> obj.prop
 *   obj?.prop     -> (obj ? obj->prop : std::nullopt)  (for pointers)
 *   obj!.prop     -> obj->prop  (assert not null, just dereference)
 *   obj[idx]      -> obj[idx]
 *   obj?[idx]     -> (obj ? (*obj)[idx] : std::nullopt)
 *   obj![idx]     -> (*obj)[idx]
 *   arr[1..3]     -> std::vector<T>(arr.begin() + 1, arr.begin() + 3)
 */

import type { MemberExpression, Identifier, RangeExpression, UnaryExpression, Literal, Expression } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genMemberExpression(node: MemberExpression, g: CppGenerator): string {
    const obj = g.genExpression(node.object);

    if (node.computed) {
        // Check for slice syntax: arr[1..3] or arr[1 usque 3]
        if (node.property.type === 'RangeExpression') {
            return genSliceExpression(obj, node.property, g);
        }

        // Check for negative index - C++ doesn't support negative indices
        if (isNegativeIndex(node.property)) {
            const negExpr = node.property as UnaryExpression;
            const absVal = g.genExpression(negExpr.argument);
            return `${obj}[${obj}.size() - ${absVal}]`;
        }

        // WHY: Use genBareExpression to avoid unnecessary parens around index
        const prop = g.genBareExpression(node.property);
        // WHY: For optional, use ternary with nullptr check
        if (node.optional) {
            g.includes.add('<optional>');
            return `(${obj} ? (*${obj})[${prop}] : std::nullopt)`;
        }
        // WHY: For non-null assertion, dereference and access
        if (node.nonNull) {
            return `(*${obj})[${prop}]`;
        }
        return `${obj}[${prop}]`;
    }

    const prop = (node.property as Identifier).name;

    // Handle ego -> this
    if (obj === 'this') {
        return `this->${prop}`;
    }

    if (node.optional) {
        g.includes.add('<optional>');
        return `(${obj} ? ${obj}->${prop} : std::nullopt)`;
    }
    if (node.nonNull) {
        return `${obj}->${prop}`;
    }
    return `${obj}.${prop}`;
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
 *   arr[1..3]       -> std::vector<T>(arr.begin() + 1, arr.begin() + 3)
 *   arr[1 usque 3]  -> std::vector<T>(arr.begin() + 1, arr.begin() + 4)
 *   arr[-3..-1]     -> std::vector<T>(arr.end() - 3, arr.end() - 1)
 *
 * WHY: C++ doesn't have native slice syntax; use vector range constructor.
 *      This creates a copy (which matches the semantics of other targets).
 */
function genSliceExpression(obj: string, range: RangeExpression, g: CppGenerator): string {
    g.includes.add('<vector>');

    let startExpr: string;
    let endExpr: string;

    // Handle start
    if (isNegativeIndex(range.start)) {
        const negExpr = range.start as UnaryExpression;
        const absVal = g.genExpression(negExpr.argument);
        startExpr = `${obj}.end() - ${absVal}`;
    } else {
        startExpr = `${obj}.begin() + ${g.genExpression(range.start)}`;
    }

    // Handle end
    if (isNegativeIndex(range.end)) {
        const negExpr = range.end as UnaryExpression;
        const absVal = (negExpr.argument as Literal).value as number;
        if (range.inclusive) {
            const inclusiveEnd = absVal - 1;
            if (inclusiveEnd === 0) {
                endExpr = `${obj}.end()`;
            } else {
                endExpr = `${obj}.end() - ${inclusiveEnd}`;
            }
        } else {
            endExpr = `${obj}.end() - ${absVal}`;
        }
    } else if (range.inclusive) {
        if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
            endExpr = `${obj}.begin() + ${range.end.value + 1}`;
        } else {
            endExpr = `${obj}.begin() + ${g.genExpression(range.end)} + 1`;
        }
    } else {
        endExpr = `${obj}.begin() + ${g.genExpression(range.end)}`;
    }

    // Use decltype to infer element type
    return `std::vector<typename decltype(${obj})::value_type>(${startExpr}, ${endExpr})`;
}
