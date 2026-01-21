/**
 * TypeScript Code Generator - Novum Expression
 *
 * TRANSFORMS:
 *   novum Persona() -> new Persona()
 *   novum Persona(nomen: "Marcus") -> new Persona({ nomen: "Marcus" })
 *
 * WHY: novum creates new class instances, maps to new keyword.
 */

import type { NovumExpression, Expression, ObjectExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { genObjectExpression } from './object';

export function genNovumExpression(node: NovumExpression, g: TsGenerator): string {
    const callee = node.callee.name;
    const args: string[] = node.arguments.filter((arg): arg is Expression => arg.type !== 'SpreadElement').map(a => g.genExpression(a));

    if (node.withExpression) {
        args.push(genObjectExpression(node.withExpression as ObjectExpression, g));
    }

    const argsText = args.join(', ');

    return `new ${callee}(${argsText})`;
}
