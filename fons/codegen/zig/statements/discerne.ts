/**
 * Zig Code Generator - DiscerneStatement (pattern matching on variants)
 *
 * TRANSFORMS:
 *   discerne event { si Click pro x, y { use(x, y) } si Quit { exit() } }
 *   -> switch (event) { .click => |c| { use(c.x, c.y); }, .quit => { exit(); } }
 *
 * WHY: Zig's switch on tagged unions is idiomatic and efficient.
 */

import type { DiscerneStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genDiscerneStatement(node: DiscerneStatement, g: ZigGenerator): string {
    const discriminant = g.genExpression(node.discriminant);

    let result = `${g.ind()}switch (${discriminant}) {\n`;
    g.depth++;

    for (const caseNode of node.cases) {
        const variantName = caseNode.variant.name.toLowerCase();

        if (caseNode.bindings.length > 0) {
            // Capture payload: .click => |c| { ... }
            result += `${g.ind()}.${variantName} => |payload| {\n`;
            g.depth++;

            // Bind individual fields
            for (const binding of caseNode.bindings) {
                result += `${g.ind()}const ${binding.name} = payload.${binding.name};\n`;
            }

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
        } else {
            // No payload: .quit => { ... }
            result += `${g.ind()}.${variantName} => {\n`;
            g.depth++;

            for (const stmt of caseNode.consequent.body) {
                result += g.genStatement(stmt) + '\n';
            }

            g.depth--;
            result += `${g.ind()}},\n`;
        }
    }

    g.depth--;
    result += `${g.ind()}}`;

    return result;
}
