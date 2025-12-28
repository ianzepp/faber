/**
 * Rust Code Generator - IaceStatement
 *
 * TRANSFORMS:
 *   iace "error" -> return Err("error");
 *   iace! "fatal" -> panic!("fatal");
 *
 * WHY: Rust uses Result<T, E> for error handling.
 *      Non-fatal iace returns Err(), fatal iace! uses panic!.
 */

import type { IaceStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genIaceStatement(node: IaceStatement, g: RsGenerator): string {
    const expr = g.genExpression(node.argument);

    if (node.fatal) {
        return `${g.ind()}panic!(${expr});`;
    }

    return `${g.ind()}return Err(${expr});`;
}
