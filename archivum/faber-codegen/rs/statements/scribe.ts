/**
 * Rust Code Generator - ScribeStatement
 *
 * TRANSFORMS:
 *   scribe "hello" -> println!("hello");
 *   scribe x, y -> println!("{} {}", x, y);
 *   scribe.debug x -> dbg!(x);
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: RsGenerator): string {
    // WHY: dbg! doesn't need format string, it prints expression and value
    if (node.level === 'debug') {
        const args = node.arguments.map(arg => g.genExpression(arg)).join(', ');
        return `${g.ind()}dbg!(${args});`;
    }

    // WHY: println! requires format string with {} placeholders
    const args = node.arguments.map(arg => g.genExpression(arg));
    const formatPlaceholders = args.map(() => '{}').join(' ');
    const formatArgs = args.length > 0 ? `, ${args.join(', ')}` : '';

    return `${g.ind()}println!("${formatPlaceholders}"${formatArgs});`;
}
