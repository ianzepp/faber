/**
 * Python Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x: numerus = 5 -> x: int = 5
 *   fixum y: textus = "hello" -> y: str = "hello"
 *   fixum [a, b, c] = coords -> a, b, c = coords
 *   figendum data = fetchData() -> data = await fetchData()
 *   variandum result = getResult() -> result = await getResult()
 *
 * WHY: Python has no const, so both varia and fixum become simple assignment.
 * WHY: Async bindings (figendum/variandum) imply await without explicit cede.
 *
 * NOTE: Object destructuring now uses DestructureDeclaration with ex-prefix:
 *       ex persona fixum nomen, aetas -> nomen, aetas = persona["nomen"], persona["aetas"]
 */

import type { VariaDeclaration } from '../../../parser/ast';
import type { PyGenerator } from '../generator';

export function genVariaDeclaration(node: VariaDeclaration, g: PyGenerator): string {
    // Check if this is an async binding (figendum/variandum)
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';

    // Handle array pattern destructuring
    // Python supports tuple unpacking: a, b, *rest = arr
    if (node.name.type === 'ArrayPattern') {
        const initExpr = node.init ? g.genExpression(node.init) : '[]';
        const awaitPrefix = isAsync ? 'await ' : '';

        // Build the pattern: a, b, *rest or _, b, _ for skips
        const parts = node.name.elements.map(elem => {
            if (elem.skip) {
                return '_'; // Python uses _ for ignored elements
            }
            if (elem.rest) {
                return `*${elem.name.name}`; // Python rest syntax
            }
            return elem.name.name;
        });

        return `${g.ind()}${parts.join(', ')} = ${awaitPrefix}${initExpr}`;
    }

    const name = node.name.name;
    const typeAnno = node.typeAnnotation ? `: ${g.genType(node.typeAnnotation)}` : '';

    if (isAsync && node.init) {
        const init = g.genExpression(node.init);
        return `${g.ind()}${name}${typeAnno} = await ${init}`;
    }

    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';
    return `${g.ind()}${name}${typeAnno}${init}`;
}
