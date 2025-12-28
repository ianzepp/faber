/**
 * C++23 Code Generator - Core generator class
 *
 * Holds shared state and utilities for C++ code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';

/**
 * Map Latin type names to C++23 types.
 */
const typeMap: Record<string, string> = {
    textus: 'std::string',
    numerus: 'int64_t',
    fractus: 'double',
    decimus: 'double',
    bivalens: 'bool',
    nihil: 'void',
    vacuum: 'void',
    octeti: 'std::vector<uint8_t>',
    lista: 'std::vector',
    tabula: 'std::unordered_map',
    copia: 'std::unordered_set',
    promissum: 'std::future',
    erratum: 'std::runtime_error',
    objectum: 'std::any',
};

export class CppGenerator {
    depth = 0;
    inGenerator = false;

    // Track which headers are needed
    includes = new Set<string>();

    // Track whether we need the scope guard helper for demum (finally)
    needsScopeGuard = false;
    scopeGuardCounter = 0;

    constructor(public indent: string = '    ') {}

    /**
     * Generate indentation for current depth.
     */
    ind(): string {
        return this.indent.repeat(this.depth);
    }

    /**
     * Generate a statement. Dispatches to specific gen* functions.
     */
    genStatement(node: Statement): string {
        // TODO: Import and dispatch to individual statement handlers
        throw new Error(`genStatement not yet implemented for: ${node.type}`);
    }

    /**
     * Generate an expression. Dispatches to specific gen* functions.
     */
    genExpression(node: Expression): string {
        // TODO: Import and dispatch to individual expression handlers
        throw new Error(`genExpression not yet implemented for: ${node.type}`);
    }

    /**
     * Generate block statement content (statements only, no braces).
     */
    genBlockStatementContent(node: BlockStatement): string {
        return node.body.map(stmt => this.genStatement(stmt)).join('\n');
    }

    /**
     * Generate function parameter.
     *
     * WHY: C++ uses type-first syntax: "const std::string& name"
     *      Rest parameters use initializer_list or variadic templates.
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const type = node.typeAnnotation ? this.genType(node.typeAnnotation) : 'auto';

        // WHY: Pass strings by const reference, primitives by value
        if (type === 'std::string') {
            return `const ${type}& ${name}`;
        }

        // Rest parameters become initializer_list
        if (node.rest) {
            return `std::initializer_list<${type}> ${name}`;
        }

        return `${type} ${name}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Map Latin type name to C++ type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> std::optional<std::string>
        if (node.nullable) {
            this.includes.add('optional');
            result = `std::optional<${result}>`;
        }

        return result;
    }

    /**
     * Generate type parameter.
     */
    genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return this.genType(param);
        }

        // Numeric parameters (e.g., for fixed-size arrays)
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }
}
