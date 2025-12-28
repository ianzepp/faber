/**
 * Python Code Generator - PactumDeclaration
 *
 * TRANSFORMS:
 *   pactum Greeter { functio greet(textus name) -> textus }
 *   ->
 *   class Greeter(Protocol):
 *       def greet(self, name: str) -> str: ...
 *
 * WHY: Python uses typing.Protocol for structural subtyping (interfaces).
 *      Methods are declared with `...` as body (ellipsis = abstract signature).
 */

import type { PactumDeclaration, PactumMethod } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genPactumDeclaration(node: PactumDeclaration, g: PyGenerator): string {
    const name = node.name.name;
    const typeParams = node.typeParameters ? `[${node.typeParameters.map(p => p.name).join(', ')}]` : '';

    const lines: string[] = [];
    lines.push(`${g.ind()}class ${name}${typeParams}(Protocol):`);
    g.depth++;

    if (node.methods.length === 0) {
        lines.push(`${g.ind()}pass`);
    } else {
        for (const method of node.methods) {
            lines.push(genPactumMethod(method, g));
        }
    }

    g.depth--;
    return lines.join('\n');
}

/**
 * Generate pactum method signature.
 *
 * WHY: Protocol methods use `...` (ellipsis) as body to indicate abstract signature.
 *      Python requires `self` as first parameter for instance methods.
 */
function genPactumMethod(node: PactumMethod, g: PyGenerator): string {
    const asyncMod = node.async ? 'async ' : '';
    const name = node.name.name;
    const params = ['self', ...node.params.map(p => g.genParameter(p))].join(', ');

    let returnType = node.returnType ? g.genType(node.returnType) : 'None';
    if (node.async && node.generator) {
        returnType = `AsyncIterator[${returnType}]`;
    } else if (node.generator) {
        returnType = `Iterator[${returnType}]`;
    } else if (node.async) {
        returnType = `Awaitable[${returnType}]`;
    }

    return `${g.ind()}${asyncMod}def ${name}(${params}) -> ${returnType}: ...`;
}
