/**
 * Python Code Generator - FunctioDeclaration
 *
 * TRANSFORMS:
 *   functio salve(nomen: textus) -> textus { ... }
 *   -> def salve(nomen: str) -> str:
 *          ...
 *
 *   futura functio f() -> numerus { ... }
 *   -> async def f() -> int:
 *          ...
 *
 *   cursor functio f() -> numerus { ... }
 *   -> def f() -> Iterator[int]:
 *          ... (with yield)
 */

import type { FunctioDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';
import { isAsyncFromAnnotations, isGeneratorFromAnnotations } from '../../types';

export function genFunctioDeclaration(node: FunctioDeclaration, g: PyGenerator): string {
    // Derive async/generator from annotations OR node properties
    const isAsync = node.async || isAsyncFromAnnotations(node.annotations);
    const isGenerator = node.generator || isGeneratorFromAnnotations(node.annotations);

    const asyncMod = isAsync ? 'async ' : '';
    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');

    // Generate type parameters for generics (Python uses TypeVar)
    // prae typus T -> requires TypeVar('T') to be defined before function
    const typeParamDefs: string[] = [];
    if (node.typeParams && node.typeParams.length > 0) {
        for (const tp of node.typeParams) {
            typeParamDefs.push(`${g.ind()}${tp.name.name} = TypeVar('${tp.name.name}')`);
        }
    }

    // Build return type with generator/async wrapping
    let returnType = '';
    if (node.returnType) {
        let baseType = g.genType(node.returnType);
        if (isAsync && isGenerator) {
            baseType = `AsyncIterator[${baseType}]`;
        } else if (isGenerator) {
            baseType = `Iterator[${baseType}]`;
        } else if (isAsync) {
            baseType = `Awaitable[${baseType}]`;
        }
        returnType = ` -> ${baseType}`;
    }

    // Track generator context for cede -> yield vs await
    const prevInGenerator = g.inGenerator;
    g.inGenerator = isGenerator;

    // Guard: abstract methods not yet supported
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for Python target');
    }

    const header = `${g.ind()}${asyncMod}def ${name}(${params})${returnType}:`;
    g.depth++;
    const body = g.genBlockStatementContent(node.body);
    g.depth--;

    g.inGenerator = prevInGenerator;

    // Handle empty body
    if (node.body.body.length === 0) {
        const funcDef = `${header}\n${g.indent.repeat(g.depth + 1)}pass`;
        // Prepend TypeVar definitions if needed
        return typeParamDefs.length > 0 ? `${typeParamDefs.join('\n')}\n${funcDef}` : funcDef;
    }

    const funcDef = `${header}\n${body}`;
    // Prepend TypeVar definitions if needed
    return typeParamDefs.length > 0 ? `${typeParamDefs.join('\n')}\n${funcDef}` : funcDef;
}
