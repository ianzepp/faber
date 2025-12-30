/**
 * C++23 Code Generator - GenusDeclaration
 *
 * TRANSFORMS:
 *   genus Persona { textus nomen: "anon" numerus aetas: 0 }
 *   -> struct Persona {
 *          std::string nomen = "anon";
 *          int64_t aetas = 0;
 *
 *          Persona() = default;
 *
 *          template<typename Overrides>
 *              requires std::is_aggregate_v<Overrides>
 *          Persona(const Overrides& o) {
 *              if constexpr (requires { o.nomen; }) nomen = o.nomen;
 *              if constexpr (requires { o.aetas; }) aetas = o.aetas;
 *              _creo();
 *          }
 *
 *          void _creo() { // user's creo body }
 *      };
 *
 * WHY: Auto-merge design - C++20's if constexpr (requires { ... }) pattern
 *      provides compile-time field checking, similar to Zig's @hasField.
 *      This allows `novum Persona { .nomen = "Marcus" }` to only override
 *      specified fields while keeping others at their defaults.
 */

import type { GenusDeclaration, FieldDeclaration, FunctioDeclaration } from '../../../parser/ast';
import type { CppGenerator } from '../generator';

export function genGenusDeclaration(node: GenusDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];

    lines.push(`${g.ind()}struct ${name} {`);
    g.depth++;

    // Fields
    for (const field of node.fields) {
        lines.push(genFieldDeclaration(field, g));
    }

    // Always generate auto-merge constructor for struct types with fields
    if (node.fields.length > 0) {
        lines.push('');
        lines.push(genAutoMergeConstructor(node, g));
    }

    // User's creo as _creo() private method
    if (node.constructor) {
        lines.push('');
        lines.push(genCreoMethod(node.constructor, g));
    }

    // Methods
    for (const method of node.methods) {
        lines.push('');
        lines.push(genMethodDeclaration(method, g));
    }

    g.depth--;
    lines.push(`${g.ind()}};`);

    // WHY: Auto-merge constructor requires <type_traits> for std::is_aggregate_v
    if (node.fields.length > 0) {
        g.includes.add('<type_traits>');
    }

    return lines.join('\n');
}

/**
 * Generate field declaration.
 */
function genFieldDeclaration(node: FieldDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const type = g.genType(node.fieldType);
    const init = node.init ? ` = ${g.genExpression(node.init)}` : '';

    return `${g.ind()}${type} ${name}${init};`;
}

/**
 * Generate auto-merge constructor using C++20 concepts.
 *
 * WHY: The template constructor accepts any aggregate type and uses
 *      if constexpr (requires { o.field; }) to check for field presence
 *      at compile time. This enables partial initialization like:
 *      `Persona{{.nomen = "Marcus"}}` - only nomen is overridden.
 */
function genAutoMergeConstructor(node: GenusDeclaration, g: CppGenerator): string {
    const name = node.name.name;
    const lines: string[] = [];
    const hasCreo = !!node.constructor;

    // Default constructor
    lines.push(`${g.ind()}${name}() = default;`);
    lines.push('');

    // Template auto-merge constructor
    lines.push(`${g.ind()}template<typename Overrides>`);
    lines.push(`${g.ind()}    requires std::is_aggregate_v<Overrides>`);
    lines.push(`${g.ind()}${name}(const Overrides& o) {`);
    g.depth++;

    // Apply each field override if present in the overrides struct
    for (const field of node.fields) {
        const fieldName = field.name.name;
        lines.push(`${g.ind()}if constexpr (requires { o.${fieldName}; }) ${fieldName} = o.${fieldName};`);
    }

    // Call _creo() if user defined it
    if (hasCreo) {
        lines.push(`${g.ind()}_creo();`);
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate user's creo as a private _creo() method.
 *
 * WHY: creo() is a post-initialization hook. By the time it runs,
 *      the struct already has merged field values. Named _creo to
 *      avoid conflicts with user-defined methods.
 */
function genCreoMethod(node: FunctioDeclaration, g: CppGenerator): string {
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for C++ target');
    }

    const lines: string[] = [];

    lines.push(`${g.ind()}void _creo() {`);
    g.depth++;

    for (const stmt of node.body.body) {
        const code = g.genStatement(stmt);
        // Replace ego. with this->
        lines.push(code.replace(/\bego\./g, 'this->'));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}

/**
 * Generate method declaration.
 */
function genMethodDeclaration(node: FunctioDeclaration, g: CppGenerator): string {
    if (!node.body) {
        throw new Error('Abstract methods not yet supported for C++ target');
    }

    const name = node.name.name;
    const params = node.params.map(p => g.genParameter(p)).join(', ');
    const returnType = node.returnType ? g.genType(node.returnType) : 'void';

    const lines: string[] = [];

    lines.push(`${g.ind()}${returnType} ${name}(${params}) {`);
    g.depth++;

    for (const stmt of node.body.body) {
        const code = g.genStatement(stmt);
        // Replace ego. with this->
        lines.push(code.replace(/\bego\./g, 'this->'));
    }

    g.depth--;
    lines.push(`${g.ind()}}`);

    return lines.join('\n');
}
