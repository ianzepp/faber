/**
 * Zig Code Generator - Praefixum Expression (compile-time evaluation)
 *
 * TRANSFORMS:
 *   praefixum(256 * 4) -> comptime (256 * 4)
 *   praefixum { ... } -> comptime blk: { ... break :blk result; }
 *
 * TARGET: Zig's comptime keyword handles both forms:
 *         - Expression: comptime (expr)
 *         - Block: comptime blk: { ... break :blk result; }
 *
 * WHY: praefixum delegates compile-time evaluation to Zig's comptime.
 *      The Faber compiler doesn't interpret the code; Zig's compiler does.
 */

import type { PraefixumExpression, BlockStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genPraefixumExpression(node: PraefixumExpression, g: ZigGenerator): string {
    if (node.body.type === 'BlockStatement') {
        // Block form: comptime blk: { ... }
        // Note: The block should end with a redde (return) which becomes break :blk value
        const body = genComptimeBlockStatement(node.body, g);
        return `comptime ${body}`;
    }

    // Expression form: comptime (expr)
    const expr = g.genExpression(node.body);
    return `comptime (${expr})`;
}

/**
 * Generate a comptime block statement with labeled break.
 *
 * WHY: Zig's comptime blocks that return values need labeled breaks.
 *      We transform redde statements into break :blk value statements.
 */
function genComptimeBlockStatement(node: BlockStatement, g: ZigGenerator): string {
    if (node.body.length === 0) {
        return 'blk: {}';
    }

    g.depth++;
    const statements: string[] = [];

    for (const stmt of node.body) {
        if (stmt.type === 'ReddeStatement') {
            // Transform redde into break :blk
            const value = stmt.argument ? g.genExpression(stmt.argument) : 'void{}';
            statements.push(`${g.ind()}break :blk ${value};`);
        } else {
            statements.push(g.genStatement(stmt));
        }
    }

    g.depth--;

    return `blk: {\n${statements.join('\n')}\n${g.ind()}}`;
}
