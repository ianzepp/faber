/**
 * Semantic Types - Type System Definitions
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module defines the type system used during semantic analysis. Types are
 * resolved from Latin type annotations and inferred from expressions. The type
 * system supports:
 *
 * - Primitive types (textus, numerus, bivalens, nihil)
 * - Generic types (lista<T>, tabula<K,V>, promissum<T>)
 * - Function types (parameters and return type)
 * - Union types (A | B)
 * - Optional types (T?)
 * - Unknown type (for unresolved/error cases)
 *
 * The SemanticType discriminated union enables exhaustive pattern matching
 * in type checking and code generation.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (type definitions only)
 * OUTPUT: Type definitions exported for semantic analysis and codegen
 * ERRORS: N/A (compile-time type checking only)
 *
 * @module semantic/types
 */

// =============================================================================
// SEMANTIC TYPES
// =============================================================================

/**
 * Base type with common fields.
 */
interface BaseType {
    nullable?: boolean;
    size?: number; // Bit width for numerus<32>, etc.
}

/**
 * Primitive type (textus, numerus, bivalens, nihil, vacuum).
 *
 * WHY: Extended with size to support sized numeric types:
 *      - size: numeric bit width (numerus<32>)
 *
 * For explicit signed/unsigned, use type parameters:
 *      - numerus<i32> -> signed 32-bit
 *      - numerus<u64> -> unsigned 64-bit
 */
export interface PrimitiveType extends BaseType {
    kind: 'primitive';
    name: 'textus' | 'numerus' | 'bivalens' | 'nihil' | 'vacuum';
}

/**
 * Generic type with type parameters (lista<T>, tabula<K,V>, promissum<T>).
 */
export interface GenericType extends BaseType {
    kind: 'generic';
    name: string;
    typeParameters: SemanticType[];
}

/**
 * Function type with parameter types and return type.
 */
export interface FunctionType extends BaseType {
    kind: 'function';
    parameterTypes: SemanticType[];
    returnType: SemanticType;
    async: boolean;
}

/**
 * Union type (A | B | C).
 */
export interface UnionType extends BaseType {
    kind: 'union';
    types: SemanticType[];
}

/**
 * Unknown type for unresolved cases.
 *
 * WHY: Allows semantic analysis to continue even when types can't be resolved.
 *      Codegen can handle unknown types with fallback behavior.
 */
export interface UnknownType extends BaseType {
    kind: 'unknown';
    reason?: string;
}

/**
 * User-defined type (class, interface, etc.).
 *
 * WHY: Placeholder for future class/struct support.
 */
export interface UserType extends BaseType {
    kind: 'user';
    name: string;
}

/**
 * Discriminated union of all semantic types.
 */
export type SemanticType =
    | PrimitiveType
    | GenericType
    | FunctionType
    | UnionType
    | UnknownType
    | UserType;

// =============================================================================
// TYPE CONSTRUCTORS
// =============================================================================

/**
 * Create a primitive type.
 */
export function primitiveType(name: PrimitiveType['name'], nullable?: boolean): PrimitiveType {
    return { kind: 'primitive', name, nullable };
}

/**
 * Create a generic type.
 */
export function genericType(
    name: string,
    typeParameters: SemanticType[],
    nullable?: boolean,
): GenericType {
    return { kind: 'generic', name, typeParameters, nullable };
}

/**
 * Create a function type.
 */
export function functionType(
    parameterTypes: SemanticType[],
    returnType: SemanticType,
    async = false,
): FunctionType {
    return { kind: 'function', parameterTypes, returnType, async };
}

/**
 * Create a union type.
 */
export function unionType(types: SemanticType[]): UnionType {
    return { kind: 'union', types };
}

/**
 * Create an unknown type.
 */
export function unknownType(reason?: string): UnknownType {
    return { kind: 'unknown', reason };
}

/**
 * Create a user-defined type.
 */
export function userType(name: string, nullable?: boolean): UserType {
    return { kind: 'user', name, nullable };
}

// =============================================================================
// COMMON TYPE CONSTANTS
// =============================================================================

export const TEXTUS: PrimitiveType = primitiveType('textus');
export const NUMERUS: PrimitiveType = primitiveType('numerus');
export const BIVALENS: PrimitiveType = primitiveType('bivalens');
export const NIHIL: PrimitiveType = primitiveType('nihil');
export const VACUUM: PrimitiveType = primitiveType('vacuum');
export const UNKNOWN: UnknownType = unknownType();

// =============================================================================
// TYPE UTILITIES
// =============================================================================

/**
 * Check if two types are equal.
 *
 * WHY: Used for type checking assignments, function calls, etc.
 */
export function typesEqual(a: SemanticType, b: SemanticType): boolean {
    if (a.kind !== b.kind) {
        return false;
    }

    switch (a.kind) {
        case 'primitive':
            // Case-insensitive comparison (textus == textus == TEXTUS)
            return a.name.toLowerCase() === (b as PrimitiveType).name.toLowerCase();
        case 'generic': {
            const bg = b as GenericType;

            // Case-insensitive comparison
            if (a.name.toLowerCase() !== bg.name.toLowerCase()) {
                return false;
            }

            if (a.typeParameters.length !== bg.typeParameters.length) {
                return false;
            }

            return a.typeParameters.every((t, i) => typesEqual(t, bg.typeParameters[i]));
        }

        case 'function': {
            const bf = b as FunctionType;

            if (a.async !== bf.async) {
                return false;
            }

            if (a.parameterTypes.length !== bf.parameterTypes.length) {
                return false;
            }

            if (!typesEqual(a.returnType, bf.returnType)) {
                return false;
            }

            return a.parameterTypes.every((t, i) => typesEqual(t, bf.parameterTypes[i]));
        }

        case 'union': {
            const bu = b as UnionType;

            if (a.types.length !== bu.types.length) {
                return false;
            }

            return a.types.every((t, i) => typesEqual(t, bu.types[i]));
        }

        case 'unknown':
            return true;
        case 'user':
            return a.name === (b as UserType).name;
    }
}

/**
 * Check if a type is assignable to another type.
 *
 * WHY: More permissive than equality - handles nullability, unknown, etc.
 */
export function isAssignableTo(source: SemanticType, target: SemanticType): boolean {
    // Unknown is assignable to anything
    if (source.kind === 'unknown') {
        return true;
    }

    if (target.kind === 'unknown') {
        return true;
    }

    // nihil is assignable to nullable types (case-insensitive)
    if (source.kind === 'primitive' && source.name.toLowerCase() === 'nihil') {
        return target.nullable === true;
    }

    // Check if source is in target union
    if (target.kind === 'union') {
        return target.types.some(t => isAssignableTo(source, t));
    }

    // Source union must have all types assignable
    if (source.kind === 'union') {
        return source.types.every(t => isAssignableTo(t, target));
    }

    // Otherwise check equality
    return typesEqual(source, target);
}

/**
 * Format a type for error messages.
 */
export function formatType(type: SemanticType): string {
    switch (type.kind) {
        case 'primitive': {
            let result = type.name;

            // Add size parameter if present (e.g., numerus<32>)
            if (type.size !== undefined) {
                result += `<${type.size}>`;
            }

            return result + (type.nullable ? '?' : '');
        }

        case 'generic': {
            const params = type.typeParameters.map(formatType).join(', ');

            return `${type.name}<${params}>` + (type.nullable ? '?' : '');
        }

        case 'function': {
            const params = type.parameterTypes.map(formatType).join(', ');
            const async = type.async ? 'futura ' : '';

            return `${async}(${params}) -> ${formatType(type.returnType)}`;
        }

        case 'union':
            return type.types.map(formatType).join(' | ');
        case 'unknown':
            return type.reason ? `unknown(${type.reason})` : 'unknown';
        case 'user':
            return type.name + (type.nullable ? '?' : '');
    }
}
