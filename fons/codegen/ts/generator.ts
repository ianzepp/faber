/**
 * TypeScript Code Generator - Core generator class
 *
 * Holds shared state and utilities for TypeScript code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter, TypeParameterDeclaration } from '../../parser/ast';
import type { RequiredFeatures } from '../types';
import { createRequiredFeatures } from '../types';

/**
 * Map Latin type names to TypeScript types.
 */
const typeMap: Record<string, string> = {
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
    decimus: 'Decimal',
    magnus: 'bigint',
    bivalens: 'boolean',
    nihil: 'null',
    vacuum: 'void',
    numquam: 'never',
    octeti: 'Uint8Array',
    lista: 'Array',
    tabula: 'Map',
    copia: 'Set',
    promissum: 'Promise',
    erratum: 'Error',
    cursor: 'Iterator',
    objectum: 'object',
    object: 'object',
    ignotum: 'unknown',
};

export class TsGenerator {
    depth = 0;
    inGenerator = false;
    features: RequiredFeatures;

    constructor(public indent: string = '    ') {
        this.features = createRequiredFeatures();
    }

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
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${this.genType(node.typeAnnotation)}` : '';
        const prefix = node.rest ? '...' : '';
        return `${prefix}${name}${typeAnno}`;
    }

    /**
     * Generate type parameters (generics).
     *
     * WHY: TypeParameterDeclaration[] is an array of individual params,
     *      each with a name Identifier. We join them into <T, U, V> syntax.
     */
    genTypeParams(params: TypeParameterDeclaration[]): string {
        return `<${params.map(p => p.name.name).join(', ')}>`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Track feature usage for preamble
        if (node.name === 'decimus' || node.name === 'decim') {
            this.features.decimal = true;
        }

        // Map Latin type name to TypeScript type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> string | null
        if (node.nullable) {
            result = `${result} | null`;
        }

        // Handle union types
        if (node.union && node.union.length > 0) {
            result = node.union.map(t => this.genType(t)).join(' | ');
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

        // Ignore numeric parameters (e.g., numerus<32>)
        if (param.type === 'Literal') {
            return null;
        }

        return null;
    }
}
