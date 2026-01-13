/**
 * Zig Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> pub fn main() void { body }
 *   incipit ergo stmt -> pub fn main() void { stmt }
 *
 * TARGET: Zig uses pub fn main() void as the program entry point.
 *         This is a straightforward wrapper - no magic injection.
 *         Source is responsible for allocator setup via cura if needed.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genIncipitStatement(node: IncipitStatement, g: ZigGenerator): string {
    if (node.ergoStatement) {
        const lines: string[] = [];
        lines.push(`${g.ind()}pub fn main() void {`);
        g.depth++;
        lines.push(g.genStatement(node.ergoStatement));
        g.depth--;
        lines.push(`${g.ind()}}`);
        return lines.join('\n');
    }
    const body = g.genBlockStatement(node.body!);
    return `${g.ind()}pub fn main() void ${body}`;
}
