/**
 * C++23 Code Generator - PactumDeclaration
 *
 * TRANSFORMS:
 *   pactum Iterabilis { functio sequens() -> textus? }
 *   -> template<typename T>
 *      concept Iterabilis = requires(T t) {
 *          { t.sequens() } -> std::same_as<std::optional<std::string>>;
 *      };
 */

import type { PactumDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genPactumDeclaration(node: PactumDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    g.includes.add('<concepts>');

    lines.push(`${g.ind()}template<typename T>`);
    lines.push(`${g.ind()}concept ${name} = requires(T t) {`);
    g.depth++;

    for (const method of node.methods) {
        const methodName = method.name.name;
        const returnType = method.returnType ? g.genType(method.returnType) : 'void';

        // Generate parameter types for the requires expression
        const paramTypes = method.params.map(p => {
            if (p.typeAnnotation) {
                return g.genType(p.typeAnnotation);
            }

            return 'auto';
        });

        if (paramTypes.length === 0) {
            lines.push(`${g.ind()}{ t.${methodName}() } -> std::same_as<${returnType}>;`);
        } else {
            lines.push(`${g.ind()}{ t.${methodName}(std::declval<${paramTypes.join('>(), std::declval<')}>()) } -> std::same_as<${returnType}>;`);
        }
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    return lines.join('\n');
}
