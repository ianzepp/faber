/**
 * TypeScript Code Generator - Member Expression
 *
 * TRANSFORMS:
 *   obj.prop          -> obj.prop
 *   obj[key]          -> obj[key]
 *   obj?.prop         -> obj?.prop
 *   obj?[key]         -> obj?.[key]   (TS requires dot before bracket)
 *   obj!.prop         -> obj!.prop
 *   obj![key]         -> obj![key]
 *   lista.longitudo   -> array.length (via norma)
 *   lista.primus      -> array[0]     (via norma)
 */

import type { MemberExpression, Identifier, Expression, RangeExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

// WHY: Use norma registry for stdlib property translations
import { getNormaTranslation, applyNormaTemplate } from '../../norma-registry';

export function genMemberExpression(node: MemberExpression, g: TsGenerator): string {
    const obj = g.genExpression(node.object);
    const objType = node.object.resolvedType;
    // WHY: Extract type name for norma lookup - works for both generics and primitives
    const collectionName = objType?.kind === 'generic' ? objType.name : objType?.kind === 'primitive' ? objType.name : null;

    if (objType?.kind === 'namespace') {
        if (node.computed) {
            const prop = g.genBareExpression(node.property);
            if (node.optional) {
                return `${obj}?.[${prop}]`;
            }
            if (node.nonNull) {
                return `${obj}![${prop}]`;
            }
            return `${obj}[${prop}]`;
        }

        const propName = (node.property as Identifier).name;
        if (node.optional) {
            return `${obj}?.${propName}`;
        }
        if (node.nonNull) {
            return `${obj}!.${propName}`;
        }
        return `${obj}.${propName}`;
    }

    if (node.computed) {
        // GUARD: tabula indexing uses Map.get()
        // WHY: tabula<K,V> maps to JS Map, which doesn't support bracket indexing.
        if (collectionName === 'tabula') {
            const prop = g.genBareExpression(node.property);
            if (node.optional) {
                return `${obj}?.get(${prop})`;
            }
            if (node.nonNull) {
                return `${obj}!.get(${prop})`;
            }
            return `${obj}.get(${prop})`;
        }

        // Check for slice syntax: arr[1..3] or arr[1 usque 3]
        if (node.property.type === 'RangeExpression') {
            return genSliceExpression(obj, node.property, g, node.optional);
        }

        // Check for negative index: arr[-1]
        if (isNegativeIndex(node.property)) {
            const idx = g.genExpression(node.property);
            if (node.optional) {
                return `${obj}?.at(${idx})`;
            }
            return `${obj}.at(${idx})`;
        }

        // WHY: Use genBareExpression to avoid unnecessary parens around index
        const prop = g.genBareExpression(node.property);

        // WHY: TypeScript requires ?. before [ for optional computed access
        if (node.optional) {
            return `${obj}?.[${prop}]`;
        }
        if (node.nonNull) {
            return `${obj}![${prop}]`;
        }
        return `${obj}[${prop}]`;
    }

    // WHY: Non-computed access always has Identifier property by grammar
    const prop = (node.property as Identifier).name;

    // WHY: Translate norma properties (longitudo -> length, primus -> [0], etc.)
    // Only applies to generic types with norma translations (lista, tabula, copia)
    if (collectionName) {
        const norma = getNormaTranslation('ts', collectionName, prop);
        if (norma?.template && norma?.params) {
            // Apply template with obj as 'ego', no additional args for properties
            const translated = applyNormaTemplate(norma.template, [...norma.params], obj, []);
            if (node.optional) {
                // WHY: Optional chaining must use `?.` for computed access.
                const suffix = translated.startsWith(obj) ? translated.slice(obj.length) : '';
                if (suffix.startsWith('[')) {
                    return `${obj}?.${suffix}`;
                }
                if (suffix.startsWith('.')) {
                    return `${obj}?${suffix}`;
                }
                // Fallback: preserve optional semantics even if template is exotic.
                return `(${obj} == null ? undefined : ${translated})`;
            }
            return translated;
        }
    }

    if (node.optional) {
        return `${obj}?.${prop}`;
    }
    if (node.nonNull) {
        return `${obj}!.${prop}`;
    }
    return `${obj}.${prop}`;
}

/**
 * Check if an expression is a negative numeric literal.
 *
 * WHY: JavaScript arrays don't support negative indices natively.
 *      We detect negative literals to emit .at() instead of bracket access.
 */
function isNegativeIndex(expr: Expression): boolean {
    // Direct negative literal: -1
    if (expr.type === 'UnaryExpression' && expr.operator === '-' && expr.argument.type === 'Literal') {
        return typeof expr.argument.value === 'number';
    }
    // Negative literal already parsed as negative number (rare)
    if (expr.type === 'Literal' && typeof expr.value === 'number' && expr.value < 0) {
        return true;
    }
    return false;
}

/**
 * Generate slice expression from range inside brackets.
 *
 * TRANSFORMS:
 *   arr[1..3]       -> arr.slice(1, 3)
 *   arr[1 usque 3]  -> arr.slice(1, 4)  // inclusive adds 1 to end
 *   arr[1..]        -> arr.slice(1)     // to end (if end is omitted)
 *   arr[..3]        -> arr.slice(0, 3)  // from start (if start is omitted)
 *   arr[-3..-1]     -> arr.slice(-3, -1)
 *   arr[-3 usque -1] -> arr.slice(-3)   // inclusive of -1 means to end
 *
 * WHY: JavaScript .slice() is always exclusive of end index, so inclusive
 *      ranges need adjustment. Negative indices work natively in slice().
 */
function genSliceExpression(obj: string, range: RangeExpression, g: TsGenerator, optional?: boolean): string {
    const start = g.genExpression(range.start);
    const end = g.genExpression(range.end);
    const optChain = optional ? '?' : '';

    // For inclusive ranges (usque), we need to add 1 to the end
    // unless end is negative (then we'd need different handling)
    if (range.inclusive) {
        // Check if end is a literal number for simple +1
        if (range.end.type === 'Literal' && typeof range.end.value === 'number') {
            const inclusiveEnd = range.end.value + 1;
            // If inclusive end is 0, it means "to the end"
            if (inclusiveEnd === 0) {
                return `${obj}${optChain}.slice(${start})`;
            }
            return `${obj}${optChain}.slice(${start}, ${inclusiveEnd})`;
        }
        // Check for negative literal in unary expression
        if (
            range.end.type === 'UnaryExpression' &&
            range.end.operator === '-' &&
            range.end.argument.type === 'Literal' &&
            typeof range.end.argument.value === 'number'
        ) {
            const negVal = -range.end.argument.value;
            const inclusiveEnd = negVal + 1;
            if (inclusiveEnd === 0) {
                return `${obj}${optChain}.slice(${start})`;
            }
            return `${obj}${optChain}.slice(${start}, ${inclusiveEnd})`;
        }
        // Dynamic end: need runtime +1
        return `${obj}${optChain}.slice(${start}, ${end} + 1)`;
    }

    // Exclusive range (.. or ante): direct slice
    return `${obj}${optChain}.slice(${start}, ${end})`;
}
