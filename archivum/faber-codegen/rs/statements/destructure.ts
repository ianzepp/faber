/**
 * Rust Code Generator - DestructureDeclaration
 *
 * TRANSFORMS:
 *   ex persona fixum nomen, aetas -> let Persona { nomen, aetas, .. } = persona;
 *   ex persona fixum nomen ut n -> let Persona { nomen: n, .. } = persona;
 *   ex promise figendum result -> let { result, .. } = promise.await;
 *
 * WHY: Rust struct destructuring requires type name or field access.
 *      Since we don't always have type info, we emit field access as fallback.
 */

import type { DestructureDeclaration } from '../../../parser/ast';
import type { RsGenerator } from '../generator';

export function genDestructureDeclaration(node: DestructureDeclaration, g: RsGenerator): string {
    const isAsync = node.kind === 'figendum' || node.kind === 'variandum';
    const mutKeyword = node.kind === 'varia' || node.kind === 'variandum' ? 'mut ' : '';
    const source = isAsync ? `${g.genExpression(node.source)}.await` : g.genExpression(node.source);

    // Check if source has resolved type info for struct name
    const structName = node.source.resolvedType?.kind === 'generic' ? node.source.resolvedType.name : null;

    if (structName) {
        // Full struct destructuring: let StructName { field, .. } = source;
        const props = node.specifiers.map(spec => {
            if (spec.rest) {
                return '..';
            }
            if (spec.imported.name !== spec.local.name) {
                return `${spec.imported.name}: ${mutKeyword}${spec.local.name}`;
            }
            return `${mutKeyword}${spec.imported.name}`;
        });

        // Add .. if no rest specifier to ignore other fields
        const hasRest = node.specifiers.some(s => s.rest);
        const pattern = hasRest ? props.join(', ') : `${props.join(', ')}, ..`;

        return `${g.ind()}let ${structName} { ${pattern} } = ${source};`;
    }

    // Fallback: emit individual field bindings
    const lines: string[] = [];
    for (const spec of node.specifiers) {
        if (spec.rest) {
            // Can't easily express rest in Rust without type info
            lines.push(`${g.ind()}// TODO: rest pattern requires type info`);
            continue;
        }
        const fieldName = spec.imported.name;
        const localName = spec.local.name;
        if (fieldName === localName) {
            lines.push(`${g.ind()}let ${mutKeyword}${localName} = ${source}.${fieldName};`);
        } else {
            lines.push(`${g.ind()}let ${mutKeyword}${localName} = ${source}.${fieldName};`);
        }
    }
    return lines.join('\n');
}
