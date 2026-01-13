/**
 * C++23 Code Generator - VariaDeclaration
 *
 * TRANSFORMS:
 *   varia x = 5 -> auto x = 5;
 *   fixum y = "hi" -> const auto y = "hi";
 *   varia numerus x = 5 -> int64_t x = 5;
 *   fixum textus y = "hi" -> const std::string y = "hi";
 */

import type { VariaDeclaration, Literal } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genVariaDeclaration(node: VariaDeclaration, g: CppGenerator): string {
    const isConst = node.kind === 'fixum';

    // Handle array destructuring (expand to indexed access)
    if (node.name.type === 'ArrayPattern') {
        return genArrayDestructuringDeclaration(node, g);
    }

    const name = node.name.name;
    const constPrefix = isConst ? 'const ' : '';

    let typeSpec: string;

    if (node.typeAnnotation) {
        typeSpec = g.genType(node.typeAnnotation);
    } else {
        typeSpec = 'auto';
    }

    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    return `${g.ind()}${constPrefix}${typeSpec} ${name}${init};`;
}

/**
 * Generate array destructuring as multiple declarations.
 *
 * TRANSFORMS:
 *   fixum [a, b, c] = arr -> const auto& _tmp = arr; const auto& a = _tmp[0]; ...
 *   [a, ceteri rest] = arr -> const auto& a = _tmp[0]; const auto rest = std::vector(_tmp.begin() + 1, _tmp.end());
 */
function genArrayDestructuringDeclaration(node: VariaDeclaration, g: CppGenerator): string {
    if (node.name.type !== 'ArrayPattern') {
        throw new Error('Expected ArrayPattern');
    }

    const lines: string[] = [];
    const isConst = node.kind === 'fixum';
    const constPrefix = isConst ? 'const ' : '';
    const initExpr = node.init ? g.genExpression(node.init) : 'undefined';

    // Create temp variable
    lines.push(`${g.ind()}${constPrefix}auto& _tmp = ${initExpr};`);

    let idx = 0;
    for (const elem of node.name.elements) {
        if (elem.skip) {
            // Skip this position
            idx++;
            continue;
        }

        const localName = elem.name.name;

        if (elem.rest) {
            // Rest pattern: collect remaining elements
            // Use vector constructor with iterators
            lines.push(`${g.ind()}${constPrefix}auto ${localName} = std::vector<decltype(_tmp)::value_type>(_tmp.begin() + ${idx}, _tmp.end());`);
        } else {
            // Regular element: indexed access
            lines.push(`${g.ind()}${constPrefix}auto& ${localName} = _tmp[${idx}];`);
            idx++;
        }
    }

    return lines.join('\n');
}
