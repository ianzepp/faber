/**
 * Faber Code Generator - CallExpression
 */

import type { CallExpression } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { applyNamespaceTemplate, getNamespaceTranslation, isNamespaceCall } from '../../shared/norma-namespace';

export function genCallExpression(node: CallExpression, g: FabGenerator): string {
    const argsArray = node.arguments.map(arg => {
        if (arg.type === 'SpreadElement') {
            return `sparge ${g.genExpression(arg.argument)}`;
        }
        return g.genExpression(arg);
    });
    const args = argsArray.join(', ');

    if (isNamespaceCall(node)) {
        const translation = getNamespaceTranslation(node.callee, 'fab');
        if (translation) {
            if (translation.method) {
                return `${translation.method}(${args})`;
            }
            if (translation.template) {
                return applyNamespaceTemplate(translation.template, [...argsArray]);
            }
        }
    }

    const callee = g.genExpression(node.callee);
    const op = node.optional ? '?(' : node.nonNull ? '!(' : '(';
    return `${callee}${op}${args})`;
}
