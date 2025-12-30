/**
 * Rust Code Generator - InitiumStatement (entry point)
 *
 * TRANSFORMS:
 *   initium { body } -> fn main() { body }
 *
 * TARGET: Rust uses fn main() as the program entry point.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genInitiumStatement(node: InitiumStatement, g: RsGenerator): string {
    const body = genBlockStatement(node.body, g);
    return `${g.ind()}fn main() ${body}`;
}
