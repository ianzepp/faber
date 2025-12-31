/**
 * Rust Code Generator - IncipietStatement (async entry point)
 *
 * TRANSFORMS:
 *   incipiet { body } -> #[tokio::main] async fn main() { body }
 *   incipiet ergo stmt -> #[tokio::main] async fn main() { stmt }
 *
 * TARGET: Rust uses #[tokio::main] attribute for async main.
 *         Requires tokio runtime dependency.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genIncipietStatement(node: IncipietStatement, g: RsGenerator): string {
    if (node.ergoStatement) {
        const lines: string[] = [];
        lines.push(`${g.ind()}#[tokio::main]`);
        lines.push(`${g.ind()}async fn main() {`);
        g.depth++;
        lines.push(g.genStatement(node.ergoStatement));
        g.depth--;
        lines.push(`${g.ind()}}`);
        return lines.join('\n');
    }
    const body = genBlockStatement(node.body!, g);
    return `${g.ind()}#[tokio::main]\n${g.ind()}async fn main() ${body}`;
}
