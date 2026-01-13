/**
 * Rust Code Generator - PraeparaBlock and CuraStatement
 *
 * PraeparaBlock TRANSFORMS:
 *   praepara omnia { ... }
 *   -> // setup_all: { ... }
 *
 * CuraStatement TRANSFORMS:
 *   cura res = Resource.new() { ... }
 *   -> let res = Resource::new();
 *      { ... }
 *
 * WHY: Rust tests don't have setup/teardown hooks like Jest.
 *      We emit as helper functions or comments.
 * WHY: Rust uses RAII for resource management, so we emit a scoped block.
 */

import type { PraeparaBlock, CuraStatement } from '../../../parser/ast';
import type { RsGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genPraeparaBlock(node: PraeparaBlock, g: RsGenerator): string {
    const timing = node.timing === 'praepara' ? 'setup' : 'teardown';
    const scope = node.omnia ? 'all' : 'each';
    const asyncPrefix = node.async ? 'async_' : '';

    return `${g.ind()}// ${asyncPrefix}${timing}_${scope}: ${genBlockStatement(node.body, g)}`;
}

export function genCuraStatement(node: CuraStatement, g: RsGenerator): string {
    // For allocator curator kinds (arena/page), just emit the block contents
    // WHY: Rust has its own allocator mechanisms, we strip these for now
    if (node.curatorKind === 'arena' || node.curatorKind === 'page') {
        return g.genBlockStatementContent(node.body);
    }

    const binding = node.binding.name;
    const resource = node.resource ? g.genExpression(node.resource) : 'Default::default()';
    const awaitSuffix = node.async ? '.await' : '';
    const body = genBlockStatement(node.body, g);

    // Rust uses RAII, but we can emit a scoped block
    return `${g.ind()}let ${binding} = ${resource}${awaitSuffix};\n${g.ind()}${body}`;
}
