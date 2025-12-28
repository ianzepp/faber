/**
 * Zig Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x: numerus = 5 -> var x: i64 = 5;
 *   fixum y: textus = "hello" -> const m_y: []const u8 = "hello"; (module-level)
 *   fixum [a, b] = arr -> const a = arr[0]; const b = arr[1];
 *
 * TARGET: Zig requires explicit types for var (mutable) declarations.
 *         Const can infer but we add type for clarity.
 *         Zig doesn't have destructuring, so we expand to indexed access.
 *
 * NOTE: Object destructuring now uses DestructureDeclaration, not VariaDeclaration.
 *
 * WHY: Module-level constants use m_ prefix to avoid shadowing conflicts
 *      with function parameters. Zig forbids a param named 'x' if there's a
 *      module const 'x', but m_x doesn't conflict.
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

    // Check if this is a module-level const (depth 0 means we're at module level)
    const isModuleLevel = g.depth === 0 && kind === 'const';
    const zigName = isModuleLevel ? `m_${name}` : name;

    // Track module constants for reference generation
    if (isModuleLevel) {
        g.addModuleConstant(name);
    }

    // TARGET: Zig requires explicit types for var, we infer if not provided
    let typeAnno = '';

    if (node.typeAnnotation) {
        typeAnno = `: ${g.genType(node.typeAnnotation)}`;
    } else if (kind === 'var' && node.init) {
        typeAnno = `: ${g.inferZigType(node.init)}`;
    }

    const init = node.init ? ` = ${g.genExpression(node.init)}` : ' = undefined';

    return `${g.ind()}${kind} ${zigName}${typeAnno}${init};`;
}
