/**
 * Rust Code Generator - TemptaStatement
 *
 * TRANSFORMS:
 *   tempta { ... } cape (e) { ... }
 *   -> // tempta (try) - Rust uses Result<T, E> instead
 *      { ... }
 *
 * WHY: Rust doesn't have try-catch; it uses Result<T, E> and the ? operator.
 *      We emit the block with a comment explaining the difference.
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genTemptaStatement(node: TemptaStatement, g: RsGenerator): string {
    const lines: string[] = [];
    lines.push(`${g.ind()}// tempta (try) - Rust uses Result<T, E> instead`);
    lines.push(`${g.ind()}${genBlockStatement(node.block, g)}`);

    return lines.join('\n');
}
