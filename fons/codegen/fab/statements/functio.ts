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
    const annotationMods: string[] = [];

    // Preserve existing annotations
    if (node.annotations) {
        for (const ann of node.annotations) {
            annotationMods.push(...ann.modifiers);
        }
    }

    // Add futura/cursor if async/generator (and not already in annotations)
    const hasAsyncAnnotation = isAsyncFromAnnotations(node.annotations);
    const hasGeneratorAnnotation = isGeneratorFromAnnotations(node.annotations);

    if (node.async && !hasAsyncAnnotation) {
        annotationMods.push('futura');
    }
    if (node.generator && !hasGeneratorAnnotation) {
        annotationMods.push('cursor');
    }
    if (node.isAbstract && !annotationMods.some(m => m.startsWith('abstract'))) {
        annotationMods.push('abstracta');
    }

    // Emit annotation line if we have modifiers
    if (annotationMods.length > 0) {
        lines.push(`${g.ind()}@ ${annotationMods.join(' ')}`);
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
