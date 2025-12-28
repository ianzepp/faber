/**
 * Zig Code Generator - Core generator class
 *
 * Holds shared state and utilities for Zig code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';

/**
 * Map Latin type names to Zig types.
 */
const typeMap: Record<string, string> = {
    // Primitives
    textus: '[]const u8',
    numerus: 'i64',
    fractus: 'f64',
    decimus: 'f128',
    magnus: 'i128',
    bivalens: 'bool',
    nihil: 'void',
    vacuum: 'void',
    octeti: '[]u8',
    // Meta types
    objectum: 'anytype',
    ignotum: 'anytype',
    numquam: 'noreturn',
    // Memory management
    curator: 'std.mem.Allocator',
};

export class ZigGenerator {
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
     * WHY: Zig requires type annotations on all parameters.
     *      Rest parameters use ... prefix syntax.
     */
    genParameter(node: Parameter): string {
        const name = node.name.name;
        const type = node.typeAnnotation ? this.genType(node.typeAnnotation) : 'anytype';
        // WHY: Zig uses ... before the type for variadic
        const prefix = node.rest ? '...' : '';
        return `${name}: ${prefix}${type}`;
    }

    /**
     * Generate type annotation from Latin type.
     */
    genType(node: TypeAnnotation): string {
        // Map Latin type name to Zig type
        const base = typeMap[node.name] ?? node.name;

        // Handle generic type parameters
        let result = base;
        if (node.typeParameters && node.typeParameters.length > 0) {
            const params = node.typeParameters.map(p => this.genTypeParameter(p)).filter((p): p is string => p !== null);

            // WHY: Zig generics use function call syntax: ArrayList(i32)
            if (params.length > 0) {
                result = `${base}(${params.join(', ')})`;
            }
        }

        // Handle nullable: textus? -> ?[]const u8
        if (node.nullable) {
            result = `?${result}`;
        }

        // WHY: Zig doesn't have union types in the same way
        // Union types would need to be defined as tagged unions

        return result;
    }

    /**
     * Generate type parameter.
     */
    genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return this.genType(param);
        }

        // Numeric parameters become comptime values
        if (param.type === 'Literal' && typeof param.value === 'number') {
            return String(param.value);
        }

        return null;
    }
}
