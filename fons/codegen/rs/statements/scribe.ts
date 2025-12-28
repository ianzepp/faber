/**
 * Rust Code Generator - ScribeStatement
 *
 * TRANSFORMS:
 *   scribe "hello" -> println!("hello");
 *   scribe.debug x -> dbg!(x);
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: RsGenerator): string {
    const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');
    const macro = node.level === 'debug' ? 'dbg!' : 'println!';

    return `${g.ind()}${macro}(${args});`;
}
