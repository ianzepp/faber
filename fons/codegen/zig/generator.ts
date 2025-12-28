/**
 * Zig Code Generator - Core generator class
 *
 * Holds shared state and utilities for Zig code generation.
 * Individual gen* functions are in separate files under statements/ and expressions/.
 */

import type { Statement, Expression, BlockStatement, Parameter, TypeAnnotation, TypeParameter } from '../../parser/ast';
import type { SemanticType } from '../../semantic/types';

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

    // Track module-level constants (comptime values)
    moduleConstants = new Set<string>();

    // Track active allocator name for collection operations
    curatorStack: string[] = ['alloc'];

    constructor(public indent: string = '    ') {}

    // -------------------------------------------------------------------------
    // Allocator (curator) management
    // -------------------------------------------------------------------------

    /**
     * Get the current active allocator name.
     */
    getCurator(): string {
        return this.curatorStack[this.curatorStack.length - 1] ?? 'alloc';
    }

    /**
     * Push a new allocator name onto the stack.
     */
    pushCurator(name: string): void {
        this.curatorStack.push(name);
    }

    /**
     * Pop the current allocator from the stack.
     */
    popCurator(): void {
        if (this.curatorStack.length > 1) {
            this.curatorStack.pop();
        }
    }

    // -------------------------------------------------------------------------
    // Module constants tracking
    // -------------------------------------------------------------------------

    /**
     * Check if a name is a module-level constant.
     */
    hasModuleConstant(name: string): boolean {
        return this.moduleConstants.has(name);
    }

    /**
     * Add a module-level constant.
     */
    addModuleConstant(name: string): void {
        this.moduleConstants.add(name);
    }

    // -------------------------------------------------------------------------
    // Type inference helpers
    // -------------------------------------------------------------------------

    /**
     * Check if an expression has a string type.
     */
    isStringType(node: Expression): boolean {
        if (node.resolvedType?.kind === 'primitive' && node.resolvedType.name === 'textus') {
            return true;
        }
        if (node.type === 'Literal' && typeof node.value === 'string') {
            return true;
        }
        if (node.type === 'TemplateLiteral') {
            return true;
        }
        return false;
    }

    /**
     * Infer Zig type from expression.
     */
    inferZigType(node: Expression): string {
        // Use resolved type from semantic analysis if available
        if (node.resolvedType) {
            return this.semanticTypeToZig(node.resolvedType);
        }

        // Fallback: infer from literals
        if (node.type === 'Literal') {
            if (typeof node.value === 'number') {
                return Number.isInteger(node.value) ? 'i64' : 'f64';
            }
            if (typeof node.value === 'string') {
                return '[]const u8';
            }
            if (typeof node.value === 'boolean') {
                return 'bool';
            }
        }

        if (node.type === 'Identifier') {
            if (node.name === 'verum' || node.name === 'falsum') {
                return 'bool';
            }
            if (node.name === 'nihil') {
                return '?void';
            }
        }

        return 'anytype';
    }

    /**
     * Convert a semantic type to Zig type string.
     */
    semanticTypeToZig(type: SemanticType): string {
        const nullable = type.nullable ? '?' : '';

        switch (type.kind) {
            case 'primitive':
                switch (type.name) {
                    case 'textus':
                        return `${nullable}[]const u8`;
                    case 'numerus':
                        return `${nullable}i64`;
                    case 'bivalens':
                        return `${nullable}bool`;
                    case 'nihil':
                        return 'void';
                    case 'vacuum':
                        return 'void';
                }
                break;
            case 'generic':
                return 'anytype';
            case 'function':
                return 'anytype';
            case 'union':
                return 'anytype';
            case 'unknown':
                return 'anytype';
            case 'user':
                return type.name;
        }

        return 'anytype';
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
     * Generate block statement with braces.
     */
    genBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return '{}';
        }
        this.depth++;
        const body = this.genBlockStatementContent(node);
        this.depth--;
        return `{\n${body}\n${this.ind()}}`;
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
