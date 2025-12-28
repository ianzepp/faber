/**
 * Python Code Generator - ScribeStatement
 *
 * TRANSFORMS:
 *   scribe "hello" -> print("hello")
 *   vide x         -> print("[DEBUG]", x)
 *   mone "oops"    -> print("[WARN]", "oops")
 */

import type { ScribeStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genScribeStatement(node: ScribeStatement, g: PyGenerator): string {
    const args = node.arguments.map(a => g.genExpression(a));
    if (node.level === 'debug') {
        args.unshift('"[DEBUG]"');
    } else if (node.level === 'warn') {
        args.unshift('"[WARN]"');
    }
    return `${g.ind()}print(${args.join(', ')})`;
}
