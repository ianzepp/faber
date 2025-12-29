/**
 * TypeScript Code Generator - IaceStatement
 *
 * TRANSFORMS:
 *   iace "message" -> throw "message"
 *   iace error     -> throw error
 *   mori "message" -> throw new Panic("message")
 *   mori error     -> throw new Panic(String(error))
 *
 * FLUMINA (streams-first):
 *   iace "message" (in fit)   -> yield respond.error("EFAIL", "message"); return;
 *   iace "message" (in fiunt) -> yield respond.error("EFAIL", "message"); return;
 *   mori "message" (in any)   -> throw new Panic("message") (unchanged - panics are fatal)
 *
 * WHY: mori indicates unrecoverable errors (like Rust's panic! or Zig's @panic).
 *      Using a dedicated Panic class allows catching panics separately from
 *      regular errors if needed, and makes stack traces clearer.
 */

import type { IaceStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: TsGenerator, semi: boolean): string {
    const expr = g.genExpression(node.argument);

    // WHY: mori (fatal) always throws, even in flumina mode - panics are non-recoverable
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

    // WHY: In flumina/fiunt/fiet/fient mode, iace yields an error response and exits the generator
    if (g.inFlumina || g.inFiunt || g.inFiet || g.inFient) {
        // Extract message for error response
        let message: string;
        if (node.argument.type === 'Literal' && typeof node.argument.value === 'string') {
            message = JSON.stringify(node.argument.value);
        } else {
            // For non-string expressions, convert to string
            message = `String(${expr})`;
        }
        return `${g.ind()}yield respond.error("EFAIL", ${message})${semi ? ';' : ''}\n${g.ind()}return${semi ? ';' : ''}`;
    }

    return `${g.ind()}throw ${expr}${semi ? ';' : ''}`;
}
