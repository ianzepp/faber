/**
 * Zig Code Generator - TemptaStatement (try/catch)
 *
 * TRANSFORMS:
 *   tempta { riskyOp() } cape err { handle(err) }
 *   -> // try block
 *      riskyOp();
 *      // catch handling would use: catch |err| { ... }
 *
 * TARGET: Zig handles errors differently - this is a simplified mapping.
 *         Real Zig would use catch |err| { } syntax on expressions.
 */

import type { TemptaStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genTemptaStatement(node: TemptaStatement, g: ZigGenerator): string {
    // Zig handles errors differently - this is a simplified mapping
    // Real Zig would use catch |err| { } syntax on expressions
    let result = `${g.ind()}// try block\n`;

    result += g.genBlockStatementContent(node.block);

    if (node.handler) {
        result += `\n${g.ind()}// catch handling would use: catch |${node.handler.param.name}| { ... }`;
    }

    return result;
}
