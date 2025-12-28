/**
 * Rust Code Generator - ExpressionStatement
 *
 * TRANSFORMS:
 *   someFunction() -> someFunction();
 */

import type { ExpressionStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genExpressionStatement(node: ExpressionStatement, g: RsGenerator): string {
    return `${g.ind()}${g.genExpression(node.expression)};`;
}
