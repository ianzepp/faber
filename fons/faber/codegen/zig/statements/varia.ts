/**
 * Zig Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x: numerus = 5 -> var x: i64 = 5;
 *   fixum y: textus = "hello" -> const y: []const u8 = "hello";
 *   fixum [a, b] = arr -> const a = arr[0]; const b = arr[1];
 *
 * TARGET: Zig requires explicit types for var (mutable) declarations.
 *         Const can infer but we add type for clarity.
 *         Zig doesn't have destructuring, so we expand to indexed access.
 *
 * NOTE: Object destructuring now uses DestructureDeclaration, not VariaDeclaration.
 */

import type { VariaDeclaration, Expression } from '../../../parser/ast';
import type { ZigGenerator } from '../generator';

export function genVariaDeclaration(node: VariaDeclaration, g: ZigGenerator): string {
    const kind = node.kind === 'varia' ? 'var' : 'const';

    // Handle array pattern destructuring
    // Zig doesn't have native destructuring, so we expand to indexed access
    // [a, b, ceteri rest] -> const a = arr[0]; const b = arr[1]; const rest = arr[2..];
    if (node.name.type === 'ArrayPattern') {
        const initExpr = node.init ? g.genExpression(node.init) : 'undefined';
        const lines: string[] = [];

        // Create a temp var to hold the array
        const tempVar = `_tmp`;
        lines.push(`${g.ind()}const ${tempVar} = ${initExpr};`);

        let idx = 0;
        for (const elem of node.name.elements) {
            if (elem.skip) {
                // Skip this position
                idx++;
                continue;
            }

            const localName = elem.name.name;

            if (elem.rest) {
                // Rest pattern: collect remaining elements as slice
                lines.push(`${g.ind()}${kind} ${localName} = ${tempVar}[${idx}..];`);
            } else {
                // Regular element: indexed access
                lines.push(`${g.ind()}${kind} ${localName} = ${tempVar}[${idx}];`);
                idx++;
            }
        }

        return lines.join('\n');
    }

    const name = node.name.name;

    // TARGET: Zig requires explicit types for var, we infer if not provided
    // EDGE: anytype is not valid for variable type annotations in Zig.
    //       If type is anytype, omit annotation and let Zig infer.
    let typeAnno = '';

    if (node.typeAnnotation) {
        const type = g.genType(node.typeAnnotation);
        if (type !== 'anytype') {
            typeAnno = `: ${type}`;
        }
    } else if (kind === 'var' && node.init) {
        const type = g.inferZigType(node.init);
        if (type !== 'anytype') {
            typeAnno = `: ${type}`;
        }
    }

    // EDGE: Array literal with lista<T> type needs Lista(T) construction
    // WHY: Faber's lista<T> maps to the stdlib Lista wrapper, not raw Zig arrays.
    //      Lista provides Latin-named methods and automatic allocator threading.
    const isListaType = node.typeAnnotation && (node.typeAnnotation.arrayShorthand || node.typeAnnotation.name === 'lista');

    if (node.init?.type === 'ArrayExpression' && isListaType) {
        g.features.lista = true;
        const curator = g.getCurator();
        const elementTypeNode = node.typeAnnotation!.typeParameters?.[0];
        const elementType = elementTypeNode && elementTypeNode.type === 'TypeAnnotation' ? g.genType(elementTypeNode) : 'anytype';

        if (node.init.elements.length === 0) {
            // Empty array: Lista(T).init(alloc)
            return `${g.ind()}${kind} ${name} = Lista(${elementType}).init(${curator});`;
        } else {
            // Non-empty array: Lista(T).fromItems(alloc, &.{...})
            const elements = node.init.elements
                .map(el => {
                    if (el.type === 'SpreadElement') {
                        return g.genExpression(el.argument);
                    }
                    return g.genExpression(el);
                })
                .join(', ');
            return `${g.ind()}${kind} ${name} = Lista(${elementType}).fromItems(${curator}, &.{ ${elements} });`;
        }
    }

    const init = node.init ? ` = ${g.genExpression(node.init)}` : ' = undefined';

    return `${g.ind()}${kind} ${name}${typeAnno}${init};`;
}
