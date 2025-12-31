/**
 * Faber Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   FunctioDeclaration -> functio name(params) modifier* -> returnType { body }
 *
 * STYLE: Modifiers (futura, cursor, curata) come after params, before return type.
 *        Uses -> for return type (canonical), not fit/fiet/fiunt/fient
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';

export function genFunctioDeclaration(node: FunctioDeclaration, g: FabGenerator): string {
    const parts: string[] = [];

    // Abstract modifier (stays at front - it's a declaration modifier, not a function modifier)
    if (node.isAbstract) {
        parts.push('abstractus');
    }

    parts.push('functio');

    // Function name
    parts.push(node.name.name);

    // Type parameters (inline: prae typus T)
    const typeParams = node.typeParams ? g.genInlineTypeParams(node.typeParams) : '';

    // Parameters
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    parts[parts.length - 1] += `(${typeParams}${params})`;

    // Function modifiers (after params): futura, cursor, curata NAME
    if (node.async) {
        parts.push('futura');
    }
    if (node.generator) {
        parts.push('cursor');
    }
    if (node.curatorName) {
        parts.push('curata');
        parts.push(node.curatorName);
    }

    // Return type (canonical: use -> arrow syntax)
    if (node.returnType) {
        parts.push('->');
        parts.push(g.genType(node.returnType));
    }

    // Body
    if (node.body) {
        parts.push(genBlockStatement(node.body, g));
    }

    return `${g.ind()}${parts.join(' ')}`;
}

/**
 * Generate block statement.
 */
export function genBlockStatement(node: BlockStatement, g: FabGenerator): string {
    if (node.body.length === 0) {
        return '{}';
    }

    g.depth++;
    const body = node.body.map(stmt => g.genStatement(stmt)).join('\n');
    g.depth--;

    return `{\n${body}\n${g.ind()}}`;
}
