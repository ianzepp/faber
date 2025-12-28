/**
 * Python Code Generator - IaceStatement
 *
 * TRANSFORMS:
 *   iace "message" -> raise Exception("message")
 *   mori "message" -> raise SystemExit("message")
 */

import type { Expression, IaceStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: PyGenerator): string {
    const arg = g.genExpression(node.argument);
    const exceptionType = node.fatal ? 'SystemExit' : 'Exception';

    // If throwing a string literal, wrap in exception type
    if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
        return `${g.ind()}raise ${exceptionType}(${arg})`;
    }

    // If throwing a new Error, convert to exception type
    if (node.argument.type === 'NovumExpression') {
        const callee = node.argument.callee.name;
        if (callee === 'Error' || callee === 'erratum') {
            const args = node.argument.arguments
                .filter((arg): arg is Expression => arg.type !== 'SpreadElement')
                .map(a => g.genExpression(a))
                .join(', ');
            return `${g.ind()}raise ${exceptionType}(${args})`;
        }
    }

    if (node.fatal) {
        return `${g.ind()}raise SystemExit(${arg})`;
    }
    return `${g.ind()}raise ${arg}`;
}
