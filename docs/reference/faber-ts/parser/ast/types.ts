/**
 * AST Type Annotation Types - Type system node definitions
 *
 * @module parser/ast/types
 */

import type { BaseNode } from './base';
import type { Literal } from './expressions';

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

/**
 * Type parameter for parameterized types.
 *
 * DESIGN: Union type allows different parameter kinds:
 *         - TypeAnnotation: Generic type params (lista<textus>), or size specs (i32, u64)
 *         - Literal: Numeric params (numerus<32>)
 *
 * Examples:
 *   lista<textus> -> TypeAnnotation
 *   numerus<32> -> Literal (size in bits)
 *   numerus<i32> -> TypeAnnotation (explicit signed 32-bit)
 *   numerus<u64> -> TypeAnnotation (explicit unsigned 64-bit)
 */
export type TypeParameter = TypeAnnotation | Literal;

/**
 * Type annotation for variables, parameters, and return types.
 *
 * GRAMMAR (in EBNF):
 *   typeAnnotation := functionType | namedType
 *   functionType := '(' typeList? ')' '->' typeAnnotation
 *   namedType := ('de' | 'in')? IDENTIFIER typeParams? '?'? arrayBrackets*
 *   typeParams := '<' typeParameter (',' typeParameter)* '>'
 *   typeList := typeAnnotation (',' typeAnnotation)*
 *
 * INVARIANT: name is the base type name (textus, numerus, etc.).
 * INVARIANT: nullable indicates optional type with '?'.
 * INVARIANT: union contains multiple types for union types (unio<A, B>).
 * INVARIANT: typeParameters can contain types or literals.
 * INVARIANT: preposition encodes ownership for systems targets (Rust/Zig):
 *            de = borrowed/read-only (&T, []const u8)
 *            in = mutable borrow (&mut T, *T)
 * INVARIANT: parameterTypes + returnType indicate a function type (name='')
 *
 * Examples:
 *   textus -> name="textus"
 *   numerus? -> name="numerus", nullable=true
 *   lista<textus> -> name="lista", typeParameters=[TypeAnnotation]
 *   numerus<32> -> name="numerus", typeParameters=[Literal{value=32}]
 *   numerus<i32> -> name="numerus", typeParameters=[TypeAnnotation{name="i32"}]
 *   unio<textus, numerus> -> name="union", union=[{name="textus"}, {name="numerus"}]
 *   de textus -> name="textus", preposition="de" (borrowed)
 *   in textus -> name="textus", preposition="in" (mutable borrow)
 *   (T) -> bivalens -> name="", parameterTypes=[T], returnType=bivalens
 *   (A, B) -> C -> name="", parameterTypes=[A, B], returnType=C
 */
export interface TypeAnnotation extends BaseNode {
    type: 'TypeAnnotation';
    name: string;
    typeParameters?: TypeParameter[];
    nullable?: boolean;
    union?: TypeAnnotation[];
    arrayShorthand?: boolean; // true if parsed from [] syntax (e.g., numerus[] vs lista<numerus>)
    preposition?: string; // 'de' (borrowed) or 'in' (mutable) for systems targets
    parameterTypes?: TypeAnnotation[]; // for function types: (T, U) -> V
    returnType?: TypeAnnotation; // for function types: (T, U) -> V
}
