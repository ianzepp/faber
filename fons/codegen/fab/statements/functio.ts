/**
 * Faber Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   FunctioDeclaration -> @ annotations \n functio name(params) curata NAME? -> returnType { body }
 *
 * STYLE: Modifiers (futura, cursor) are emitted as @ annotations on preceding line.
 *        curata NAME stays inline (it binds a name).
 *        Uses -> for return type (canonical), not fit/fiet/fiunt/fient
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { isAsyncFromAnnotations, isGeneratorFromAnnotations } from '../../types';

export function genFunctioDeclaration(node: FunctioDeclaration, g: FabGenerator): string {
    const lines: string[] = [];

    // Build annotation modifiers
    // WHY: Combine existing annotations with async/generator derived from node properties
    // Preserve existing annotations (one per line)
    if (node.annotations) {
        for (const ann of node.annotations) {
            if (ann.argument) {
                lines.push(`${g.ind()}@ ${ann.name} ${g.genExpression(ann.argument)}`);
            } else {
                lines.push(`${g.ind()}@ ${ann.name}`);
            }
        }
    }

    // Add futura/cursor if async/generator (and not already in annotations)
    const hasAsyncAnnotation = isAsyncFromAnnotations(node.annotations);
    const hasGeneratorAnnotation = isGeneratorFromAnnotations(node.annotations);
    const hasAbstractAnnotation = node.annotations?.some(a => a.name.startsWith('abstract'));

    if (node.async && !hasAsyncAnnotation) {
        lines.push(`${g.ind()}@ futura`);
    }
    if (node.generator && !hasGeneratorAnnotation) {
        lines.push(`${g.ind()}@ cursor`);
    }
    if (node.isAbstract && !hasAbstractAnnotation) {
        lines.push(`${g.ind()}@ abstracta`);
    }

    const parts: string[] = [];

    parts.push('functio');

    // Function name
    parts.push(node.name.name);

    // Type parameters (inline: prae typus T)
    const typeParams = node.typeParams ? g.genInlineTypeParams(node.typeParams) : '';

    // Parameters
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    parts[parts.length - 1] += `(${typeParams}${params})`;

    // curata NAME stays inline (binds a name, not a simple modifier)
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

    lines.push(`${g.ind()}${parts.join(' ')}`);

    return lines.join('\n');
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
