/**
 * C++ Code Generator - CuraBlock and CuraStatement
 *
 * CuraStatement (resource management):
 *   cura arena { body } -> body (allocators are no-op for C++ in this context)
 *   cura page { body } -> body (allocators are no-op for C++ in this context)
 *
 *   cura resource fit r { body }
 *   -> {
 *        auto r = resource;
 *        body
 *      } // RAII handles cleanup
 *
 * WHY: C++ uses RAII for resource management.
 *      Allocator curator kinds (arena/page) are no-ops since we assume standard allocators.
 */

import type { CuraBlock, CuraStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genCuraBlock(node: CuraBlock, g: CppGenerator): string {
    // Test hooks - C++ doesn't have built-in test hooks, emit as comments
    const timing = node.timing === 'ante' ? 'before' : 'after';
    const scope = node.omnia ? 'all' : 'each';
    const lines: string[] = [];
    lines.push(`${g.ind()}// ${timing}_${scope}`);
    lines.push(g.genBlockStatementContent(node.body));
    return lines.join('\n');
}

export function genCuraStatement(node: CuraStatement, g: CppGenerator): string {
    // For allocator curator kinds (arena/page), just emit the block contents
    // WHY: Standard C++ doesn't need explicit allocator management for simple cases
    if (node.curatorKind === 'arena' || node.curatorKind === 'page') {
        return g.genBlockStatementContent(node.body);
    }

    // Generic resource management with RAII scope
    const lines: string[] = [];
    const binding = node.binding.name;
    const resource = node.resource ? g.genExpression(node.resource) : 'nullptr';

    // Scoped block for RAII
    lines.push(`${g.ind()}{`);
    g.depth++;
    lines.push(`${g.ind()}auto ${binding} = ${resource};`);
    lines.push(g.genBlockStatementContent(node.body));
    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
