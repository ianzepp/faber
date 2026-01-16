/**
 * AST Node Definitions - Abstract Syntax Tree type definitions
 *
 * COMPILER PHASE
 * ==============
 * syntactic
 *
 * ARCHITECTURE
 * ============
 * This module defines the Abstract Syntax Tree (AST) node types produced by the
 * parser. The AST is a structured representation of Latin source code that preserves
 * syntactic information while abstracting away lexical details like whitespace.
 *
 * The AST design follows several key principles:
 * 1. All nodes extend BaseNode to carry source position for error reporting
 * 2. Discriminated unions (via 'type' field) enable exhaustive pattern matching
 * 3. Latin keywords are preserved as literals (varia, fixum) for semantic analysis
 * 4. Optional morphology info on Identifiers enables case-aware transformations
 *
 * This AST sits between the parser and semantic analyzer in the pipeline. It
 * deliberately preserves Latin-specific syntax (like prepositional parameters)
 * that will be transformed into target language constructs during code generation.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (type definitions only)
 * OUTPUT: Type definitions exported for parser and codegen phases
 * ERRORS: N/A (compile-time type checking only)
 *
 * INVARIANTS
 * ==========
 * INV-1: All AST nodes MUST include a position field for error reporting
 * INV-2: Discriminated union types MUST have unique 'type' string literals
 * INV-3: Optional fields use ? notation, never null/undefined unions
 * INV-4: Node types preserve Latin syntax, NOT target language semantics
 *
 * @module parser/ast
 */

// =============================================================================
// RE-EXPORTS
// =============================================================================

// Base types
export type { CommentType, Comment, BaseNode, Visibility } from './base';

// Type system
export type { TypeParameter, TypeAnnotation } from './types';

// Expressions
export type {
    Expression,
    Identifier,
    EgoExpression,
    Literal,
    TemplateLiteral,
    RegexLiteral,
    ArrayExpression,
    SpreadElement,
    ObjectExpression,
    ObjectProperty,
    RangeExpression,
    BinaryExpression,
    UnaryExpression,
    EstExpression,
    QuaExpression,
    InnatumExpression,
    ConversionExpression,
    ShiftExpression,
    CallExpression,
    MemberExpression,
    AssignmentExpression,
    ConditionalExpression,
    CedeExpression,
    NovumExpression,
    FingeExpression,
    LambdaExpression,
    PraefixumExpression,
    CollectionDSLExpression,
    AbExpression,
    ScriptumExpression,
    LegeExpression,
} from './expressions';

// Declarations
export type {
    ImportSpecifier,
    ImportaDeclaration,
    DestructureDeclaration,
    ObjectPattern,
    ObjectPatternProperty,
    ArrayPattern,
    ArrayPatternElement,
    VariaDeclaration,
    TypeParameterDeclaration,
    ReturnVerb,
    FunctioModifier,
    CurataModifier,
    ErrataModifier,
    ExitusModifier,
    ImmutataModifier,
    IacitModifier,
    FunctioDeclaration,
    Parameter,
    TypeAliasDeclaration,
    OrdoMember,
    OrdoDeclaration,
    VariantField,
    VariantDeclaration,
    DiscretioDeclaration,
    Annotation,
    FieldDeclaration,
    GenusDeclaration,
    PactumDeclaration,
    PactumMethod,
} from './declarations';

// Control flow
export type {
    BlockStatement,
    ExpressionStatement,
    SiStatement,
    DumStatement,
    CollectionDSLTransform,
    IteratioStatement,
    InStatement,
    EligeStatement,
    EligeCasus,
    DiscerneStatement,
    VariantPattern,
    VariantCase,
    CustodiStatement,
    CustodiClause,
    IncipitStatement,
    IncipietStatement,
} from './control';

// Actions
export type {
    AdfirmaStatement,
    ReddeStatement,
    RumpeStatement,
    PergeStatement,
    IaceStatement,
    TemptaStatement,
    CapeClause,
    OutputLevel,
    ScribeStatement,
    FacBlockStatement,
} from './actions';

// Testing
export type {
    ProbandumStatement,
    ProbaModifier,
    ProbaStatement,
    PraeparaTiming,
    CuratorKind,
    PraeparaBlock,
    CuraStatement,
} from './testing';

// Dispatch
export type {
    AdBindingVerb,
    AdBinding,
    AdStatement,
} from './dispatch';

// =============================================================================
// STATEMENT UNION
// =============================================================================

import type { ImportaDeclaration, DestructureDeclaration, VariaDeclaration, FunctioDeclaration, GenusDeclaration, PactumDeclaration, TypeAliasDeclaration, OrdoDeclaration, DiscretioDeclaration } from './declarations';
import type { ExpressionStatement, SiStatement, DumStatement, IteratioStatement, InStatement, EligeStatement, DiscerneStatement, CustodiStatement, BlockStatement, IncipitStatement, IncipietStatement } from './control';
import type { AdfirmaStatement, ReddeStatement, RumpeStatement, PergeStatement, IaceStatement, TemptaStatement, ScribeStatement, FacBlockStatement } from './actions';
import type { ProbandumStatement, ProbaStatement, PraeparaBlock, CuraStatement } from './testing';
import type { AdStatement } from './dispatch';
import type { BaseNode } from './base';

/**
 * Discriminated union of all statement types.
 *
 * DESIGN: TypeScript discriminated union enables exhaustive switch statements
 *         in visitors and transformers.
 */
export type Statement =
    | ImportaDeclaration
    | DestructureDeclaration
    | VariaDeclaration
    | FunctioDeclaration
    | GenusDeclaration
    | PactumDeclaration
    | TypeAliasDeclaration
    | OrdoDeclaration
    | DiscretioDeclaration
    | ExpressionStatement
    | SiStatement
    | DumStatement
    | IteratioStatement
    | InStatement
    | EligeStatement
    | DiscerneStatement
    | CustodiStatement
    | AdfirmaStatement
    | ReddeStatement
    | RumpeStatement
    | PergeStatement
    | BlockStatement
    | IaceStatement
    | TemptaStatement
    | ScribeStatement
    | FacBlockStatement
    | ProbandumStatement
    | ProbaStatement
    | PraeparaBlock
    | CuraStatement
    | AdStatement
    | IncipitStatement
    | IncipietStatement;

// =============================================================================
// PROGRAM (ROOT NODE)
// =============================================================================

/**
 * Program is the root node of the AST.
 *
 * INVARIANT: Body is always an array, never null (empty source = empty array).
 */
export interface Program extends BaseNode {
    type: 'Program';
    body: Statement[];
}
