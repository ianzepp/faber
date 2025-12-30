/**
 * C++23 Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   functio salve(textus nomen) -> nihil { }
 *   -> void salve(std::string nomen) { }
 */

import type { FunctioDeclaration, BlockStatement } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genFunctioDeclaration(node: FunctioDeclaration, g: CppGenerator): string {
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for C++ target');
    }

    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    const returnType = node.returnType ? g.genType(node.returnType) : 'void';

    const body = genBlockStatement(node.body, g);

    return `${g.ind()}${returnType} ${name}(${params}) ${body}`;
}

/**
 * Generate block statement with braces.
 */
export function genBlockStatement(node: BlockStatement, g: CppGenerator): string {
    if (node.body.length === 0) {
        return '{}';
    }

    g.depth++;
    const body = node.body.map(stmt => g.genStatement(stmt)).join('\n');

    g.depth--;

    return `{\n${body}\n${g.ind()}}`;
}
