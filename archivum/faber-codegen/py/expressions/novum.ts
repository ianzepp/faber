/**
 * Python Code Generator - NovumExpression
 *
 * TRANSFORMS:
 *   novum Persona() -> Persona()
 *   novum Persona { nomen: "X" } -> Persona({"nomen": "X"})
 *   novum Persona de props -> Persona(props)
 *
 * WHY: Python classes are called directly without `new`.
 */

import type { Expression, NovumExpression } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genNovumExpression(node: NovumExpression, g: PyGenerator): string {
    const callee = node.callee.name;
    const args: string[] = node.arguments.filter((arg): arg is Expression => arg.type !== 'SpreadElement').map(arg => g.genExpression(arg));

    if (node.withExpression) {
        // withExpression can be ObjectExpression (inline) or any Expression (de X)
        args.push(g.genExpression(node.withExpression));
    }

    return `${callee}(${args.join(', ')})`;
}
