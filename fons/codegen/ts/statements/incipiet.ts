/**
 * TypeScript Code Generator - IncipietStatement (async entry point)
 *
 * TRANSFORMS:
 *   incipiet { body } -> (async () => { body })()
 *   incipiet ergo stmt -> (async () => { stmt })()
 *
 * TARGET: TypeScript/JavaScript needs an async IIFE for top-level await.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { TsGenerator } from '../generator';

export function genIncipietStatement(node: IncipietStatement, g: TsGenerator): string {
    const lines: string[] = [];
    const semi = g.semi ? ';' : '';
    lines.push(`${g.ind()}(async () => {`);
    g.depth++;
    if (node.ergoStatement) {
        lines.push(g.genStatement(node.ergoStatement));
    } else {
        lines.push(g.genBlockStatementContent(node.body!));
    }
    g.depth--;
    lines.push(`${g.ind()}})()${semi}`);
    return lines.join('\n');
}
