/**
 * Zig Code Generator - DiscerneStatement (pattern matching on variants)
 *
 * TRANSFORMS:
 *   discerne event { casu Click ut c { use(c.x) } }
 *   -> switch (event) { .click => |c| { use(c.x); } }
 *
 *   discerne event { casu Click pro x, y { use(x, y) } casu Quit { exit() } }
 *   -> switch (event) { .click => |payload| { const x = payload.x; ... }, .quit => { exit(); } }
 *
 * WHY: Zig's switch on tagged unions is idiomatic and efficient.
 *
 * NOTE: Multi-discriminant matching not yet supported in Zig codegen.
 *       For now, only single-discriminant patterns are emitted.
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: ZigGenerator): string {
    // Use first discriminant only (multi-discriminant not yet supported)
    const discriminant = g.genExpression(node.discriminants[0]!);

    let result = `${g.ind()}switch (${discriminant}) {\n`;
    g.depth++;

    for (const caseNode of node.cases) {
        // Use first pattern only
        const pattern = caseNode.patterns[0];
        if (!pattern) continue;

        // Handle wildcard as else
        if (pattern.isWildcard) {
            result += `${g.ind()}else => {\n`;
            g.depth++;

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
            continue;
        }

        const variantName = pattern.variant.name.toLowerCase();

        if (pattern.alias) {
            // Alias binding: casu Click ut c { ... } -> .click => |c| { ... }
            result += `${g.ind()}.${variantName} => |${pattern.alias.name}| {\n`;
            g.depth++;

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
        } else if (pattern.bindings.length > 0) {
            // Positional bindings: casu Click pro x, y { ... }
            // Capture payload and destructure
            result += `${g.ind()}.${variantName} => |payload| {\n`;
            g.depth++;

            // Bind individual fields
            for (const binding of pattern.bindings) {
                result += `${g.ind()}const ${binding.name} = payload.${binding.name};\n`;
            }

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
        } else {
            // No bindings: .quit => { ... }
            result += `${g.ind()}.${variantName} => {\n`;
            g.depth++;

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
        }
    }

    // Generate default case (ceterum) as else
    if (node.defaultCase) {
        result += `${g.ind()}else => {\n`;
        g.depth++;

        for (const stmt of node.defaultCase.body) {
            result += g.genStatement(stmt) + '\n';
        }

        g.depth--;
        result += `${g.ind()}},\n`;
    }

    g.depth--;
    result += `${g.ind()}}`;

    return result;
}
