/**
 * Faber Code Generator - PraeparaBlock and CuraStatement
 */

import type { PraeparaBlock, CuraStatement } from '../../../parser/ast';
import type { FabGenerator } from '../generator';
import { genBlockStatement } from './functio';

export function genPraeparaBlock(node: PraeparaBlock, g: FabGenerator): string {
    // Determine keyword based on timing and async
    let keyword: string;
    if (node.timing === 'praepara') {
        keyword = node.async ? 'praeparabit' : 'praepara';
    } else {
        keyword = node.async ? 'postparabit' : 'postpara';
    }
    const omnia = node.omnia ? ' omnia' : '';
    return `${g.ind()}${keyword}${omnia} ${genBlockStatement(node.body, g)}`;
}

export function genCuraStatement(node: CuraStatement, g: FabGenerator): string {
    const parts: string[] = ['cura'];

    // Curator kind (arena/page)
    if (node.curatorKind) {
        parts.push(node.curatorKind);
    }

    // Resource expression
    if (node.resource) {
        parts.push(g.genExpression(node.resource));
    }

    // Binding verb
    parts.push(node.async ? 'fiet' : 'fit');

    // Type annotation
    if (node.typeAnnotation) {
        parts.push(g.genType(node.typeAnnotation));
    }

    // Binding name
    parts.push(node.binding.name);

    // Body
    parts.push(genBlockStatement(node.body, g));

    let result = `${g.ind()}${parts.join(' ')}`;

    if (node.catchClause) {
        result += ` cape ${node.catchClause.param.name} ${genBlockStatement(node.catchClause.body, g)}`;
    }

    return result;
}
