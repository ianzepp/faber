/**
 * TypeScript Code Generator - IaceStatement
 *
 * TRANSFORMS:
 *   iace "message" -> throw "message"
 *   iace error     -> throw error
 *   mori "message" -> throw new Panic("message")
 *   mori error     -> throw new Panic(String(error))
 *
 * WHY: mori indicates unrecoverable errors (like Rust's panic! or Zig's @panic).
 *      Using a dedicated Panic class allows catching panics separately from
 *      regular errors if needed, and makes stack traces clearer.
 */

import type { IaceStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: TsGenerator, semi: boolean): string {
    const expr = g.genExpression(node.argument);

    if (node.fatal) {
        // Track that we need the Panic class in preamble
        g.features.panic = true;

        // mori (panic) - wrap in Panic class
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            return `${g.ind()}throw new Panic(${JSON.stringify(node.argument.value)})${semi ? ';' : ''}`;
        }
        // Other expressions: convert to string
        return `${g.ind()}throw new Panic(String(${expr}))${semi ? ';' : ''}`;
    }

    return `${g.ind()}throw ${expr}${semi ? ';' : ''}`;
}
