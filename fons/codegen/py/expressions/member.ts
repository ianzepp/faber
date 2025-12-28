/**
 * Python Code Generator - MemberExpression
 *
 * TRANSFORMS:
 *   obj.prop      -> obj.prop
 *   obj?.prop     -> (obj.prop if obj is not None else None)
 *   obj!.prop     -> obj.prop  (Python has no assertion, just access)
 *   obj[idx]      -> obj[idx]
 *   obj?[idx]     -> (obj[idx] if obj is not None else None)
 *   obj![idx]     -> obj[idx]
 *
 * Also handles slice expressions:
 *   arr[1..3]       -> arr[1:3]
 *   arr[1 usque 3]  -> arr[1:4]  // inclusive adds 1 to end
 *   arr[-3..-1]     -> arr[-3:-1]
 *   arr[-3 usque -1] -> arr[-3:]  // inclusive of -1 means to end
 */

import type { MemberExpression, RangeExpression, Identifier } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genMemberExpression(node: MemberExpression, g: PyGenerator): string {
    const obj = g.genExpression(node.object);

    if (node.computed) {
        // Check for slice syntax: arr[1..3] or arr[1 usque 3]
        // Python has native slice syntax: arr[1:3]
        if (node.property.type === 'RangeExpression') {
            return genSliceExpression(obj, node.property, g, node.optional);
        }

        const prop = g.genExpression(node.property);
        // WHY: Python has no native optional chaining; expand to conditional
        if (node.optional) {
            return `(${obj}[${prop}] if ${obj} is not None else None)`;
        }
        // WHY: Python has no non-null assertion; just access directly
        // WHY: Python natively supports negative indices, so no special handling needed
        return `${obj}[${prop}]`;
    }

    const prop = (node.property as Identifier).name;
    if (node.optional) {
        return `(${obj}.${prop} if ${obj} is not None else None)`;
    }
    return `${obj}.${prop}`;
}

/**
 * Generate slice expression from range inside brackets.
 *
 * TRANSFORMS:
 *   arr[1..3]       -> arr[1:3]
 *   arr[1 usque 3]  -> arr[1:4]  // inclusive adds 1 to end
 *   arr[-3..-1]     -> arr[-3:-1]
 *   arr[-3 usque -1] -> arr[-3:]  // inclusive of -1 means to end
 *
 * WHY: Python slice syntax is [start:end] with exclusive end (same as ..).
 *      Inclusive ranges (usque) need end + 1 adjustment.
 */
function genSliceExpression(obj: string, range: RangeExpression, g: PyGenerator, optional?: boolean): string {
    const start = g.genExpression(range.start);
    const end = g.genExpression(range.end);

    let sliceEnd: string;

    if (range.inclusive) {
        // Check if end is a literal number for simple +1
        if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
            const inclusiveEnd = range.end.value + 1;
            // If inclusive end is 0, it means "to the end" in Python
            sliceEnd = inclusiveEnd === 0 ? '' : String(inclusiveEnd);
        }
        // Check for negative literal in unary expression
        else if (
            range.end.type === 'UnaryExpression' &&
            range.end.operator === '-' &&
            range.end.argument.type === 'Literal' &&
            typeof range.end.argument.value === 'number'
        ) {
            const negVal = -range.end.argument.value;
            const inclusiveEnd = negVal + 1;
            sliceEnd = inclusiveEnd === 0 ? '' : String(inclusiveEnd);
        }
        // Dynamic end: need runtime +1
        else {
            sliceEnd = `${end} + 1`;
        }
    } else {
        sliceEnd = end;
    }

    const sliceExpr = `${obj}[${start}:${sliceEnd}]`;

    if (optional) {
        return `(${sliceExpr} if ${obj} is not None else None)`;
    }
    return sliceExpr;
}
