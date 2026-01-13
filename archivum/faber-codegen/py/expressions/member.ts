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
 * Optional chaining is flattened:
 *   a?.b.c?.d.e   -> (a.b.c.d.e if a is not None and a.b.c is not None else None)
 *
 * Also handles slice expressions:
 *   arr[1..3]       -> arr[1:3]
 *   arr[1 usque 3]  -> arr[1:4]  // inclusive adds 1 to end
 *   arr[-3..-1]     -> arr[-3:-1]
 *   arr[-3 usque -1] -> arr[-3:]  // inclusive of -1 means to end
 */

import type { MemberExpression, RangeExpression, Identifier, Expression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

/** Represents a segment in a member expression chain */
interface ChainSegment {
    property: string; // The property access string (e.g., ".prop" or "[key]")
    checkBefore: boolean; // Whether this segment has optional chaining (need to check object before access)
}

/**
 * Collect all segments of a member expression chain.
 * Returns the base object and an array of segments from left to right.
 *
 * WHY: Optional chaining like a?.b.c?.d.e should flatten to a single conditional
 *      rather than nested conditionals. We collect the chain to generate:
 *      (a.b.c.d.e if a is not None and a.b.c is not None else None)
 */
function collectChain(node: MemberExpression, g: PyGenerator): { base: Expression; segments: ChainSegment[] } {
    const segments: ChainSegment[] = [];
    let current: Expression = node;

    // Walk up the chain collecting segments
    while (current.type === 'MemberExpression') {
        const member = current as MemberExpression;

        // Build property access string
        let property: string;
        if (member.computed) {
            if (member.property.type === 'RangeExpression') {
                // Slice expressions handled separately
                break;
            }
            property = `[${g.genBareExpression(member.property)}]`;
        } else {
            const propName = (member.property as Identifier).name;
            // Handle .length -> len() conversion
            if (propName === 'length') {
                // Can't flatten length into chain, handle separately
                break;
            }
            property = `.${propName}`;
        }

        segments.unshift({
            property,
            checkBefore: member.optional ?? false,
        });

        current = member.object;
    }

    return { base: current, segments };
}

/**
 * Check if a member expression chain has any optional access.
 */
function hasOptionalInChain(node: MemberExpression): boolean {
    let current: Expression = node;
    while (current.type === 'MemberExpression') {
        if ((current as MemberExpression).optional) {
            return true;
        }
        current = (current as MemberExpression).object;
    }
    return false;
}

export function genMemberExpression(node: MemberExpression, g: PyGenerator): string {
    const objType = node.object.resolvedType;
    if (objType?.kind === 'namespace') {
        const obj = g.genExpression(node.object);
        if (node.computed) {
            const prop = g.genBareExpression(node.property);
            const access = `${obj}[${prop}]`;
            if (node.optional) {
                return `(${access} if ${obj} is not None else None)`;
            }
            return access;
        }

        const propName = (node.property as Identifier).name;
        const access = `${obj}.${propName}`;
        if (node.optional) {
            return `(${access} if ${obj} is not None else None)`;
        }
        return access;
    }

    // Special case: slice expressions
    if (node.computed && node.property.type === 'RangeExpression') {
        const obj = g.genExpression(node.object);
        return genSliceExpression(obj, node.property, g, node.optional);
    }

    // Special case: .length property (converts to len())
    if (!node.computed) {
        const propName = (node.property as Identifier).name;
        if (propName === 'length') {
            const obj = g.genExpression(node.object);
            if (node.optional) {
                return `(len(${obj}) if ${obj} is not None else None)`;
            }
            return `len(${obj})`;
        }
    }

    // For chains with optional access, flatten them
    if (hasOptionalInChain(node)) {
        const { base, segments } = collectChain(node, g);

        // If we couldn't collect any segments (e.g., slice or length), fall through
        if (segments.length === 0) {
            return genSimpleMember(node, g);
        }

        const baseStr = g.genExpression(base);

        // Build the full access chain
        let fullChain = baseStr;
        for (const seg of segments) {
            fullChain += seg.property;
        }

        // Collect all the null checks
        // For a?.b.c?.d, we need to check: a is not None, a.b.c is not None
        const checks: string[] = [];
        let checkPath = baseStr;
        for (const seg of segments) {
            if (seg.checkBefore) {
                checks.push(`${checkPath} is not None`);
            }
            checkPath += seg.property;
        }

        if (checks.length > 0) {
            const condition = checks.join(' and ');
            return `(${fullChain} if ${condition} else None)`;
        }

        return fullChain;
    }

    // Simple case: no optional chaining
    return genSimpleMember(node, g);
}

/**
 * Generate a simple member expression without chain flattening.
 */
function genSimpleMember(node: MemberExpression, g: PyGenerator): string {
    const obj = g.genExpression(node.object);

    if (node.computed) {
        const prop = g.genBareExpression(node.property);
        return `${obj}[${prop}]`;
    }

    const prop = (node.property as Identifier).name;
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
