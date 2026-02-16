/**
 * TypeScript Code Generator - Assignment Expression
 *
 * TRANSFORMS:
 *   x = 5 -> x = 5
 *   x += 1 -> x += 1
 *   obj.prop = value -> obj.prop = value
 */

import type { AssignmentExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genAssignmentExpression(node: AssignmentExpression, g: TsGenerator): string {
    // GUARD: tabula indexing assignment uses Map.set()
    // WHY: tabula<K,V> maps to JS Map, so assignments to computed members need lowering.
    if (node.left.type === 'MemberExpression' && node.left.computed) {
        const objType = node.left.object.resolvedType;
        const collectionName = objType?.kind === 'generic' ? objType.name : null;

        if (collectionName === 'tabula') {
            const obj = g.genExpression(node.left.object);
            const prop = g.genBareExpression(node.left.property);
            const right = g.genBareExpression(node.right);

            const getExpr = node.left.optional
                ? `${obj}?.get(${prop})`
                : node.left.nonNull
                  ? `${obj}!.get(${prop})`
                  : `${obj}.get(${prop})`;

            const setStmt = node.left.optional
                ? `${obj}?.set(${prop}, __next)`
                : node.left.nonNull
                  ? `${obj}!.set(${prop}, __next)`
                  : `${obj}.set(${prop}, __next)`;

            if (node.operator === '=') {
                return `(() => { const __next = ${right}; ${setStmt}; return __next; })()`;
            }

            // EDGE: Preserve JS-like compound assignment behavior.
            // Example: m[k] += 1 -> m.set(k, (m.get(k) as any) + 1)
            const op = node.operator.endsWith('=') ? node.operator.slice(0, -1) : node.operator;
            return `(() => { const __next = (${getExpr} as any) ${op} ${right}; ${setStmt}; return __next; })()`;
        }
    }

    const left = node.left.type === 'Identifier' ? node.left.name : g.genBareExpression(node.left);
    return `${left} ${node.operator} ${g.genBareExpression(node.right)}`;
}
