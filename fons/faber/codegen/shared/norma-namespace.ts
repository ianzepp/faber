/**
 * Namespace helpers for norma registry.
 */

import type { CallExpression, MemberExpression, Identifier } from '../../parser/ast';
import { getNormaTranslation, type VerteTranslation } from '../norma-registry';

export function isNamespaceCall(node: CallExpression): node is CallExpression & { callee: MemberExpression } {
    return node.callee.type === 'MemberExpression' && !node.callee.computed && node.callee.object.resolvedType?.kind === 'namespace';
}

export function getNamespaceTranslation(callee: MemberExpression, target: string): VerteTranslation | undefined {
    if (callee.object.resolvedType?.kind !== 'namespace') {
        return undefined;
    }

    if (callee.computed || callee.property.type !== 'Identifier') {
        return undefined;
    }

    const moduleName = callee.object.resolvedType.moduleName;
    const methodName = (callee.property as Identifier).name;
    return getNormaTranslation(target, moduleName, methodName);
}

export function applyNamespaceTemplate(template: string, args: string[]): string {
    let result = template;
    let implicitIdx = 0;

    result = result.replace(/§(\d+)?/g, (_match, indexStr) => {
        if (indexStr !== undefined) {
            const idx = Number.parseInt(indexStr, 10);
            return args[idx] ?? '';
        }
        return args[implicitIdx++] ?? '';
    });

    return result;
}
