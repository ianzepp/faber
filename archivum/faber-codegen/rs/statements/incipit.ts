/**
 * Rust Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> fn main() { body }
 *   incipit ergo stmt -> fn main() { stmt }
 *
 * TARGET: Rust uses fn main() as the program entry point.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIncipitStatement(node: IncipitStatement, g: RsGenerator): string {
    if (node.ergoStatement) {
        const lines: string[] = [];
        lines.push(`${g.ind()}fn main() {`);
        g.depth++;
        lines.push(g.genStatement(node.ergoStatement));
        g.depth--;
        lines.push(`${g.ind()}}`);
        return lines.join('\n');
    }
    const body = genBlockStatement(node.body!, g);
    return `${g.ind()}fn main() ${body}`;
}
