/**
 * Rust Code Generator - Core generator class
 *
 * Holds shared state and utilities for Rust code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';

/**
 * Map Latin type names to Rust type names.
 */
const typeMap: Record<string, string> = {
    textus: 'String',
    numerus: 'f64',
    fractus: 'f64',
    magnus: 'i128',
    bivalens: 'bool',
    nihil: '()',
    vacuum: '()',
    lista: 'Vec',
    tabula: 'HashMap',
    copia: 'HashSet',
    promissum: 'Future',
    octeti: 'Vec<u8>',
};

export class RsGenerator {
    depth = 0;
    inGenerator = false;

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
     * WHY: Rust uses name: Type syntax like Latin.
     *      References (&) and mutability (mut) depend on usage.
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const type = node.typeAnnotation ? this.genType(node.typeAnnotation) : '_';

        // WHY: Strings typically passed as &str for borrowing
        if (type === 'String') {
            return `${name}: &str`;
        }

        return `${name}: ${type}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Map Latin type name to Rust type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> Option<String>
        if (node.nullable) {
            result = `Option<${result}>`;
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

        // Numeric parameters for const generics
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }
}
