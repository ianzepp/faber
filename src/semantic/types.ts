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
 * - Primitive types (Textus, Numerus, Bivalens, Nihil)
 * - Generic types (Lista<T>, Tabula<K,V>, Promissum<T>)
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
  nullable?: boolean
}

/**
 * Primitive type (Textus, Numerus, Bivalens, Nihil, Vacuum).
 */
export interface PrimitiveType extends BaseType {
  kind: "primitive"
  name: "Textus" | "Numerus" | "Bivalens" | "Nihil" | "Vacuum"
}

/**
 * Generic type with type parameters (Lista<T>, Tabula<K,V>, Promissum<T>).
 */
export interface GenericType extends BaseType {
  kind: "generic"
  name: string
  typeParameters: SemanticType[]
}

/**
 * Function type with parameter types and return type.
 */
export interface FunctionType extends BaseType {
  kind: "function"
  parameterTypes: SemanticType[]
  returnType: SemanticType
  async: boolean
}

/**
 * Union type (A | B | C).
 */
export interface UnionType extends BaseType {
  kind: "union"
  types: SemanticType[]
}

/**
 * Unknown type for unresolved cases.
 *
 * WHY: Allows semantic analysis to continue even when types can't be resolved.
 *      Codegen can handle unknown types with fallback behavior.
 */
export interface UnknownType extends BaseType {
  kind: "unknown"
  reason?: string
}

/**
 * User-defined type (class, interface, etc.).
 *
 * WHY: Placeholder for future class/struct support.
 */
export interface UserType extends BaseType {
  kind: "user"
  name: string
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
  | UserType

// =============================================================================
// TYPE CONSTRUCTORS
// =============================================================================

/**
 * Create a primitive type.
 */
export function primitiveType(name: PrimitiveType["name"], nullable?: boolean): PrimitiveType {
  return { kind: "primitive", name, nullable }
}

/**
 * Create a generic type.
 */
export function genericType(name: string, typeParameters: SemanticType[], nullable?: boolean): GenericType {
  return { kind: "generic", name, typeParameters, nullable }
}

/**
 * Create a function type.
 */
export function functionType(parameterTypes: SemanticType[], returnType: SemanticType, async = false): FunctionType {
  return { kind: "function", parameterTypes, returnType, async }
}

/**
 * Create a union type.
 */
export function unionType(types: SemanticType[]): UnionType {
  return { kind: "union", types }
}

/**
 * Create an unknown type.
 */
export function unknownType(reason?: string): UnknownType {
  return { kind: "unknown", reason }
}

/**
 * Create a user-defined type.
 */
export function userType(name: string, nullable?: boolean): UserType {
  return { kind: "user", name, nullable }
}

// =============================================================================
// COMMON TYPE CONSTANTS
// =============================================================================

export const TEXTUS: PrimitiveType = primitiveType("Textus")
export const NUMERUS: PrimitiveType = primitiveType("Numerus")
export const BIVALENS: PrimitiveType = primitiveType("Bivalens")
export const NIHIL: PrimitiveType = primitiveType("Nihil")
export const VACUUM: PrimitiveType = primitiveType("Vacuum")
export const UNKNOWN: UnknownType = unknownType()

// =============================================================================
// TYPE UTILITIES
// =============================================================================

/**
 * Check if two types are equal.
 *
 * WHY: Used for type checking assignments, function calls, etc.
 */
export function typesEqual(a: SemanticType, b: SemanticType): boolean {
  if (a.kind !== b.kind) return false

  switch (a.kind) {
    case "primitive":
      return a.name === (b as PrimitiveType).name
    case "generic": {
      const bg = b as GenericType
      if (a.name !== bg.name) return false
      if (a.typeParameters.length !== bg.typeParameters.length) return false
      return a.typeParameters.every((t, i) => typesEqual(t, bg.typeParameters[i]))
    }
    case "function": {
      const bf = b as FunctionType
      if (a.async !== bf.async) return false
      if (a.parameterTypes.length !== bf.parameterTypes.length) return false
      if (!typesEqual(a.returnType, bf.returnType)) return false
      return a.parameterTypes.every((t, i) => typesEqual(t, bf.parameterTypes[i]))
    }
    case "union": {
      const bu = b as UnionType
      if (a.types.length !== bu.types.length) return false
      return a.types.every((t, i) => typesEqual(t, bu.types[i]))
    }
    case "unknown":
      return true
    case "user":
      return a.name === (b as UserType).name
  }
}

/**
 * Check if a type is assignable to another type.
 *
 * WHY: More permissive than equality - handles nullability, unknown, etc.
 */
export function isAssignableTo(source: SemanticType, target: SemanticType): boolean {
  // Unknown is assignable to anything
  if (source.kind === "unknown") return true
  if (target.kind === "unknown") return true

  // Nihil is assignable to nullable types
  if (source.kind === "primitive" && source.name === "Nihil") {
    return target.nullable === true
  }

  // Check if source is in target union
  if (target.kind === "union") {
    return target.types.some(t => isAssignableTo(source, t))
  }

  // Source union must have all types assignable
  if (source.kind === "union") {
    return source.types.every(t => isAssignableTo(t, target))
  }

  // Otherwise check equality
  return typesEqual(source, target)
}

/**
 * Format a type for error messages.
 */
export function formatType(type: SemanticType): string {
  switch (type.kind) {
    case "primitive":
      return type.name + (type.nullable ? "?" : "")
    case "generic": {
      const params = type.typeParameters.map(formatType).join(", ")
      return `${type.name}<${params}>` + (type.nullable ? "?" : "")
    }
    case "function": {
      const params = type.parameterTypes.map(formatType).join(", ")
      const async = type.async ? "futura " : ""
      return `${async}(${params}) -> ${formatType(type.returnType)}`
    }
    case "union":
      return type.types.map(formatType).join(" | ")
    case "unknown":
      return type.reason ? `unknown(${type.reason})` : "unknown"
    case "user":
      return type.name + (type.nullable ? "?" : "")
  }
}
