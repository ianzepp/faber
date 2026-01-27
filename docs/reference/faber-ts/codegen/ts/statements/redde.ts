/**
 * TypeScript Code Generator - ReddeStatement
 *
 * TRANSFORMS:
 *   redde x -> return x;
 *   redde -> return;
 *
 * FLUMINA (streams-first):
 *   redde x (in fit)  -> yield respond.ok(x);
 *   redde x (in fiet) -> yield respond.ok(x);
 *   redde (in fit)    -> yield respond.ok(undefined);
 *   redde (in fiet)   -> yield respond.ok(undefined);
 */

import type { ReddeStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genReddeStatement(node: ReddeStatement, g: TsGenerator, semi: boolean): string {
    // WHY: In flumina mode (fit or fiet), redde yields a Responsum instead of returning directly
    if (g.inFlumina) {
        const value = node.argument ? g.genExpression(node.argument) : 'undefined';
        return `${g.ind()}yield respond.ok(${value})${semi ? ';' : ''}`;
    }

    if (node.argument) {
        return `${g.ind()}return ${g.genExpression(node.argument)}${semi ? ';' : ''}`;
    }

    return `${g.ind()}return${semi ? ';' : ''}`;
}
