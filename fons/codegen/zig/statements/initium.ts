/**
 * Zig Code Generator - InitiumStatement (entry point)
 *
 * TRANSFORMS:
 *   initium { body } -> pub fn main() void { body }
 *
 * TARGET: Zig uses pub fn main() void as the program entry point.
 *         This is a straightforward wrapper - no magic injection.
 *         Source is responsible for allocator setup via cura if needed.
 */

import type { InitiumStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genInitiumStatement(node: InitiumStatement, g: ZigGenerator): string {
    const body = g.genBlockStatement(node.body);
    return `${g.ind()}pub fn main() void ${body}`;
}
