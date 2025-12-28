/**
 * C++23 Code Generator - DumStatement
 *
 * TRANSFORMS:
 *   dum x > 0 { ... } -> while (x > 0) { ... }
 */

import type { DumStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genDumStatement(node: DumStatement, g: CppGenerator): string {
    const test = g.genExpression(node.test);
    const body = genBlockStatement(node.body, g);

    return `${g.ind()}while (${test}) ${body}`;
}
