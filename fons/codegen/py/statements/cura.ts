/**
 * Python Code Generator - CuraBlock and CuraStatement
 *
 * CuraStatement (resource management):
 *   cura arena { body } -> body (allocators are no-op for GC languages)
 *   cura page { body } -> body (allocators are no-op for GC languages)
 *
 *   cura resource fit r { body }
 *   -> with resource as r:
 *        body
 *
 * WHY: Python uses context managers (with statement) for resource management.
 *      Allocator curator kinds (arena/page) are no-ops since Python has GC.
 */

import type { CuraBlock, CuraStatement } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genCuraBlock(node: CuraBlock, g: PyGenerator): string {
    // Test hooks - Python doesn't have built-in test hooks, emit as comments
    const timing = node.timing === 'ante' ? 'before' : 'after';
    const scope = node.omnia ? 'all' : 'each';
    const lines: string[] = [];
    lines.push(`${g.ind()}# ${timing}_${scope}`);
    lines.push(g.genBlockStatementContent(node.body));
    return lines.join('\n');
}

export function genCuraStatement(node: CuraStatement, g: PyGenerator): string {
    // For allocator curator kinds (arena/page), just emit the block contents
    // WHY: GC targets don't need allocator management, memory is automatic
    if (node.curatorKind === 'arena' || node.curatorKind === 'page') {
        return g.genBlockStatementContent(node.body);
    }

    // Generic resource management with context manager
    const lines: string[] = [];
    const binding = node.binding.name;
    const resource = node.resource ? g.genExpression(node.resource) : 'None';

    // with <resource> as <binding>:
    lines.push(`${g.ind()}with ${resource} as ${binding}:`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
    g.depth--;

    return lines.join('\n');
}
