/**
 * Zig Code Generator - CuraStatement
 *
 * CuraStatement TRANSFORMS:
 *   cura arena fit mem { ... }
 *   -> var mem_arena = std.heap.ArenaAllocator.init(std.heap.page_allocator);
 *      defer mem_arena.deinit();
 *      const mem = mem_arena.allocator();
 *      { ... }
 *
 *   cura page fit mem { ... }
 *   -> const mem = std.heap.page_allocator;
 *      { ... }
 *
 * WHY: Explicit allocator management for Zig.
 *      Push allocator name to curatorStack so nested code can reference it.
 */

import type { CuraStatement } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genCuraStatement(node: CuraStatement, g: ZigGenerator): string {
    const binding = node.binding.name;
    const lines: string[] = [];

    if (node.curatorKind === 'arena') {
        // Arena allocator: create arena, defer cleanup, get allocator
        // WHY: Arena frees all memory at once on scope exit
        const arenaVar = `${binding}_arena`;
        lines.push(`${g.ind()}var ${arenaVar} = std.heap.ArenaAllocator.init(std.heap.page_allocator);`);
        lines.push(`${g.ind()}defer ${arenaVar}.deinit();`);
        lines.push(`${g.ind()}const ${binding} = ${arenaVar}.allocator();`);

        // Push allocator name for nested code
        g.pushCurator(binding);

        // Generate block body
        lines.push(g.genBlockStatementContent(node.body));

        // Pop allocator
        g.popCurator();

        return lines.join('\n');
    }

    if (node.curatorKind === 'page') {
        // Page allocator: direct reference, no cleanup needed
        // WHY: Page allocator is global, no deallocation
        lines.push(`${g.ind()}const ${binding} = std.heap.page_allocator;`);

        // Push allocator name for nested code
        g.pushCurator(binding);

        // Generate block body
        lines.push(g.genBlockStatementContent(node.body));

        // Pop allocator
        g.popCurator();

        return lines.join('\n');
    }

    // Generic resource: emit expression with defer solve
    // WHY: For non-allocator resources, use defer for cleanup
    const resource = node.resource ? g.genExpression(node.resource) : 'undefined';
    const awaitPrefix = node.async ? 'try ' : '';

    lines.push(`${g.ind()}const ${binding} = ${awaitPrefix}${resource};`);
    lines.push(`${g.ind()}defer ${binding}.solve();`);
    lines.push(g.genBlockStatementContent(node.body));

    return lines.join('\n');
}
