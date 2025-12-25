/**
 * Semantic Analyzer - Type Resolution and Validation
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module performs semantic analysis on the AST produced by the parser.
 * It resolves types, builds symbol tables, validates type compatibility,
 * and annotates AST nodes with resolved type information.
 *
 * The analyzer walks the AST in a single pass:
 * 1. Statements are processed to build symbol tables
 * 2. Expressions are typed bottom-up
 * 3. Type errors are collected (not thrown) for multi-error reporting
 * 4. AST nodes are annotated with resolvedType field
 *
 * Key responsibilities:
 * - Variable type resolution (from annotation or initializer inference)
 * - Function signature recording
 * - Expression type inference
 * - Type compatibility checking
 * - Undefined variable detection
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST from parser
 * OUTPUT: SemanticResult with typed AST and error list
 * ERRORS: Type mismatches, undefined variables, redefinitions
 *
 * INVARIANTS
 * ==========
 * INV-1: All expressions have resolvedType after analysis (may be unknown)
 * INV-2: All variable references are validated against symbol table
 * INV-3: Analyzer never throws - errors collected in result
 *
 * @module semantic
 */

import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    TypeAliasDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    GuardStatement,
    AssertStatement,
    ReturnStatement,
    BlockStatement,
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrowFunctionExpression,
    AssignmentExpression,
    AwaitExpression,
    NewExpression,
    ArrayExpression,
    TypeAnnotation,
    ThrowStatement,
    ScribeStatement,
    TryStatement,
    FacBlockStatement,
    LambdaExpression,
} from '../parser/ast';
import type { Position } from '../tokenizer/types';
import type { Scope, Symbol } from './scope';
import { createGlobalScope, createScope, defineSymbol, lookupSymbol } from './scope';
import type { SemanticType } from './types';
import {
    genericType,
    functionType,
    unionType,
    userType,
    TEXTUS,
    NUMERUS,
    FRACTUS,
    DECIMUS,
    MAGNUS,
    BIVALENS,
    NIHIL,
    VACUUM,
    OCTETI,
    UNKNOWN,
    formatType,
    isAssignableTo,
} from './types';
import { SemanticErrorCode, SEMANTIC_ERRORS } from './errors';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Semantic error with source location.
 */
export interface SemanticError {
    message: string;
    position: Position;
}

/**
 * Result of semantic analysis.
 */
export interface SemanticResult {
    program: Program;
    errors: SemanticError[];
}

// =============================================================================
// LATIN TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to semantic types.
 */
const LATIN_TYPE_MAP: Record<string, SemanticType> = {
    textus: TEXTUS,
    numerus: NUMERUS,
    fractus: FRACTUS,
    decimus: DECIMUS,
    magnus: MAGNUS,
    bivalens: BIVALENS,
    nihil: NIHIL,
    vacuum: VACUUM,
    octeti: OCTETI,
};

/**
 * Generic types that take type parameters.
 *
 * CASE: Lowercase keys. Lookup normalizes input to lowercase.
 */
const GENERIC_TYPES = new Set(['lista', 'tabula', 'copia', 'promissum', 'cursor', 'fluxus']);

// =============================================================================
// STANDARD LIBRARY EXPORTS
// =============================================================================

/**
 * Norma standard library exports.
 *
 * Maps function names to their semantic types for import resolution.
 * When `ex norma importa X` is encountered, these symbols are added to scope.
 */
const NORMA_EXPORTS: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> = {
    // I/O
    scribe: { type: functionType([TEXTUS], VACUUM), kind: 'function' },
    vide: { type: functionType([TEXTUS], VACUUM), kind: 'function' },
    mone: { type: functionType([TEXTUS], VACUUM), kind: 'function' },
    lege: { type: functionType([], TEXTUS), kind: 'function' },

    // Iteration
    series: { type: functionType([NUMERUS], genericType('lista', [NUMERUS])), kind: 'function' },
    seriesAb: {
        type: functionType([NUMERUS, NUMERUS, NUMERUS], genericType('lista', [NUMERUS])),
        kind: 'function',
    },

    // Math functions
    fortuitus: { type: functionType([], NUMERUS), kind: 'function' },
    pavimentum: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    tectum: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    rotundus: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    absolutus: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    radix: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    potentia: { type: functionType([NUMERUS, NUMERUS], NUMERUS), kind: 'function' },
    minimus: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },
    maximus: { type: functionType([NUMERUS], NUMERUS), kind: 'function' },

    // Type conversion
    numerus: { type: functionType([UNKNOWN], NUMERUS), kind: 'function' },
    textus: { type: functionType([UNKNOWN], TEXTUS), kind: 'function' },

    // Constants
    PI: { type: NUMERUS, kind: 'variable' },
    E: { type: NUMERUS, kind: 'variable' },
};

/**
 * Norma/tempus module exports.
 *
 * Time operations: current time, sleeping, duration constants.
 * When `ex "norma/tempus" importa X` is encountered, these symbols are added to scope.
 */
const NORMA_TEMPUS_EXPORTS: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> = {
    // Current time functions
    nunc: { type: functionType([], NUMERUS), kind: 'function' },
    nunc_nano: { type: functionType([], NUMERUS), kind: 'function' },
    nunc_secunda: { type: functionType([], NUMERUS), kind: 'function' },

    // Sleep (async)
    dormi: {
        type: functionType([NUMERUS], genericType('promissum', [VACUUM])),
        kind: 'function',
    },

    // Duration constants (milliseconds)
    MILLISECUNDUM: { type: NUMERUS, kind: 'variable' },
    SECUNDUM: { type: NUMERUS, kind: 'variable' },
    MINUTUM: { type: NUMERUS, kind: 'variable' },
    HORA: { type: NUMERUS, kind: 'variable' },
    DIES: { type: NUMERUS, kind: 'variable' },
};

/**
 * Map of all norma submodule exports.
 */
const NORMA_SUBMODULES: Record<string, Record<string, { type: SemanticType; kind: 'function' | 'variable' }>> = {
    'norma/tempus': NORMA_TEMPUS_EXPORTS,
};

// =============================================================================
// MAIN ANALYZER
// =============================================================================

/**
 * Perform semantic analysis on a program.
 */
export function analyze(program: Program): SemanticResult {
    const errors: SemanticError[] = [];
    let currentScope: Scope = createGlobalScope();
    let currentFunctionReturnType: SemanticType | null = null;

    defineBuiltins();

    // ---------------------------------------------------------------------------
    // Built-in Definitions
    // ---------------------------------------------------------------------------

    /**
     * Define built-in functions (intrinsics) in global scope.
     *
     * Intrinsics are functions provided by the target runtime, not written in Faber.
     * They form the foundation that norma.fab builds upon.
     */
    function defineBuiltins(): void {
        const builtinPos = { line: 0, column: 0 };

        function defFn(name: string, params: SemanticType[], ret: SemanticType): void {
            currentScope.symbols.set(name, {
                name,
                type: functionType(params, ret),
                kind: 'function',
                mutable: false,
                position: builtinPos,
            });
        }

        // I/O Intrinsics (prefixed with _ for internal use by norma.fab)
        defFn('_scribe', [], VACUUM);
        defFn('_vide', [], VACUUM);
        defFn('_mone', [], VACUUM);
        defFn('_lege', [], TEXTUS);

        // Math Intrinsics (prefixed with _ for internal use by norma.fab)
        defFn('_fortuitus', [], NUMERUS);
        defFn('_pavimentum', [NUMERUS], NUMERUS);
        defFn('_tectum', [NUMERUS], NUMERUS);
        defFn('_radix', [NUMERUS], NUMERUS);
        defFn('_potentia', [NUMERUS, NUMERUS], NUMERUS);
    }

    // ---------------------------------------------------------------------------
    // Error Reporting
    // ---------------------------------------------------------------------------

    /**
     * Report a semantic error.
     */
    function error(message: string, position: Position): void {
        errors.push({ message, position });
    }

    // ---------------------------------------------------------------------------
    // Scope Management
    // ---------------------------------------------------------------------------

    /**
     * Enter a new scope.
     */
    function enterScope(kind: Scope['kind'] = 'block'): void {
        currentScope = createScope(currentScope, kind);
    }

    /**
     * Exit the current scope.
     */
    function exitScope(): void {
        if (currentScope.parent) {
            currentScope = currentScope.parent;
        }
    }

    /**
     * Define a symbol in current scope, reporting error if duplicate.
     */
    function define(symbol: Symbol): void {
        const err = defineSymbol(currentScope, symbol);

        if (err) {
            error(err, symbol.position);
        }
    }

    // ---------------------------------------------------------------------------
    // Import Resolution
    // ---------------------------------------------------------------------------

    /**
     * Analyze import declaration and add symbols to scope.
     *
     * WHY: Recognizes 'norma' base library and 'norma/*' submodules.
     *      Other modules pass through without type info for external JS/TS modules.
     */
    function analyzeImportDeclaration(node: ImportDeclaration): void {
        // Determine which export map to use
        let exports: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> | undefined;
        const moduleName = node.source;

        if (node.source === 'norma') {
            exports = NORMA_EXPORTS;
        } else if (node.source in NORMA_SUBMODULES) {
            exports = NORMA_SUBMODULES[node.source];
        } else {
            // Unknown module - imports pass through without type info
            // WHY: Allows importing from external JS/TS modules
            return;
        }

        if (node.wildcard) {
            // ex norma importa * - add all exports to scope
            for (const [name, { type, kind }] of Object.entries(exports)) {
                currentScope.symbols.set(name, {
                    name,
                    type,
                    kind,
                    mutable: false,
                    position: node.position,
                });
            }

            return;
        }

        // ex norma importa scribe, series - add specific exports
        for (const specifier of node.specifiers) {
            const exportInfo = exports[specifier.name];

            if (!exportInfo) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.NotExportedFromModule];

                error(`${text(specifier.name, moduleName)}\n${help}`, specifier.position);

                continue;
            }

            currentScope.symbols.set(specifier.name, {
                name: specifier.name,
                type: exportInfo.type,
                kind: exportInfo.kind,
                mutable: false,
                position: specifier.position,
            });
        }
    }

    // ---------------------------------------------------------------------------
    // Type Resolution
    // ---------------------------------------------------------------------------

    /**
     * Resolve a TypeAnnotation AST node to a SemanticType.
     *
     * WHY: Type parameters can be types or literals.
     *      - TypeAnnotation: Generic type params (lista<textus>), or sized types (numerus<i32>)
     *      - Literal: Numeric size params (numerus<32>)
     */
    function resolveTypeAnnotation(node: TypeAnnotation): SemanticType {
        // Handle union types
        if (node.union && node.union.length > 0) {
            const types = node.union.map(resolveTypeAnnotation);

            return unionType(types);
        }

        // Check for built-in primitive type (case-insensitive)
        const primitive = LATIN_TYPE_MAP[node.name.toLowerCase()];

        if (primitive) {
            const size = extractSizeFromTypeParams(node.typeParameters);

            if (node.nullable || size !== undefined) {
                return {
                    ...primitive,
                    nullable: node.nullable,
                    size,
                };
            }

            return primitive;
        }

        // Check for generic type (case-insensitive)
        if (GENERIC_TYPES.has(node.name.toLowerCase())) {
            // Filter only TypeAnnotation params for generic type parameters
            const typeParams = (node.typeParameters ?? [])
                .filter(p => p.type === 'TypeAnnotation')
                .map(p => resolveTypeAnnotation(p as TypeAnnotation));

            return genericType(node.name, typeParams, node.nullable);
        }

        // Check if it's a type alias in the symbol table
        const typeAlias = lookupSymbol(currentScope, node.name);

        if (typeAlias && typeAlias.kind === 'type') {
            if (node.nullable && !typeAlias.type.nullable) {
                return { ...typeAlias.type, nullable: true };
            }

            return typeAlias.type;
        }

        // User-defined type (not yet defined)
        return userType(node.name, node.nullable);
    }

    /**
     * Extract size from type parameters.
     *
     * WHY: Numeric literals in type parameters specify bit width (e.g., numerus<32>).
     */
    function extractSizeFromTypeParams(typeParams?: Array<TypeAnnotation | Literal>): number | undefined {
        if (!typeParams) {
            return undefined;
        }

        for (const param of typeParams) {
            if (param.type === 'Literal' && typeof param.value === 'number') {
                return param.value;
            }
        }

        return undefined;
    }

    /**
     * Infer type from a literal value.
     */
    function inferLiteralType(node: Literal): SemanticType {
        if (node.value === null) {
            return NIHIL;
        }

        if (typeof node.value === 'string') {
            return TEXTUS;
        }

        if (typeof node.value === 'number') {
            return NUMERUS;
        }

        if (typeof node.value === 'bigint') {
            return MAGNUS;
        }

        if (typeof node.value === 'boolean') {
            return BIVALENS;
        }

        return UNKNOWN;
    }

    // ---------------------------------------------------------------------------
    // Expression Type Resolution
    // ---------------------------------------------------------------------------

    /**
     * Resolve the type of an expression.
     */
    function resolveExpression(node: Expression): SemanticType {
        switch (node.type) {
            case 'Identifier':
                return resolveIdentifier(node);

            case 'Literal':
                return resolveLiteral(node);

            case 'TemplateLiteral':
                node.resolvedType = TEXTUS;
                return TEXTUS;

            case 'BinaryExpression':
                return resolveBinaryExpression(node);

            case 'UnaryExpression':
                return resolveUnaryExpression(node);

            case 'CallExpression':
                return resolveCallExpression(node);

            case 'MemberExpression':
                return resolveMemberExpression(node);

            case 'ArrowFunctionExpression':
                return resolveArrowFunction(node);

            case 'AssignmentExpression':
                return resolveAssignment(node);

            case 'AwaitExpression':
                return resolveAwait(node);

            case 'NewExpression':
                return resolveNew(node);

            case 'ConditionalExpression':
                return resolveConditional(node);

            case 'RangeExpression':
                return resolveRange(node);

            case 'ObjectExpression':
                return resolveObjectExpression(node);

            case 'ArrayExpression':
                return resolveArrayExpression(node);

            case 'LambdaExpression':
                return resolveLambdaExpression(node);

            default: {
                const _exhaustive: never = node;

                return UNKNOWN;
            }
        }
    }

    function resolveObjectExpression(node: Expression & { type: 'ObjectExpression' }): SemanticType {
        // Resolve each property value or spread argument
        for (const prop of node.properties) {
            if (prop.type === 'SpreadElement') {
                resolveExpression(prop.argument);
            }
            else {
                resolveExpression(prop.value);
            }
        }

        // WHY: Use Latin type name for consistency
        const objType = userType('objectum');

        node.resolvedType = objType;

        return objType;
    }

    function resolveArrayExpression(node: ArrayExpression): SemanticType {
        // Empty array - unknown element type
        if (node.elements.length === 0) {
            const arrType = genericType('lista', [UNKNOWN]);

            node.resolvedType = arrType;

            return arrType;
        }

        // Resolve all element types (SpreadElement resolves to its argument's type)
        const elementTypes: SemanticType[] = [];

        for (const element of node.elements) {
            if (element.type === 'SpreadElement') {
                // For spread, resolve the argument (should be an array) and use its element type
                const argType = resolveExpression(element.argument);
                // If spreading an array, use its element type; otherwise use unknown
                if (argType.kind === 'generic' && argType.name === 'lista' && argType.typeParams?.length) {
                    elementTypes.push(argType.typeParams[0]);
                }
                else {
                    elementTypes.push(UNKNOWN);
                }
            }
            else {
                elementTypes.push(resolveExpression(element));
            }
        }

        // Infer element type from first element
        const inferredElementType = elementTypes[0];

        // Validate all elements match the inferred type
        for (let i = 1; i < elementTypes.length; i++) {
            if (!isAssignableTo(elementTypes[i], inferredElementType)) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.TypeMismatch];

                error(
                    `${text(formatType(elementTypes[i]), formatType(inferredElementType))} in array element ${i + 1}\n${help}`,
                    node.elements[i].position,
                );
            }
        }

        const arrType = genericType('lista', [inferredElementType]);

        node.resolvedType = arrType;

        return arrType;
    }

    function resolveRange(node: Expression & { type: 'RangeExpression' }): SemanticType {
        resolveExpression(node.start);
        resolveExpression(node.end);

        if (node.step) {
            resolveExpression(node.step);
        }

        // Range produces an iterable of numbers
        const rangeType = genericType('lista', [NUMERUS]);

        node.resolvedType = rangeType;

        return rangeType;
    }

    function resolveIdentifier(node: Identifier): SemanticType {
        // Handle Latin boolean/null keywords
        if (node.name === 'verum' || node.name === 'falsum') {
            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        if (node.name === 'nihil') {
            node.resolvedType = NIHIL;

            return NIHIL;
        }

        // Look up in symbol table
        const symbol = lookupSymbol(currentScope, node.name);

        if (!symbol) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.UndefinedVariable];

            error(`${text(node.name)}\n${help}`, node.position);
            node.resolvedType = UNKNOWN;

            return UNKNOWN;
        }

        node.resolvedType = symbol.type;

        return symbol.type;
    }

    function resolveLiteral(node: Literal): SemanticType {
        const type = inferLiteralType(node);

        node.resolvedType = type;

        return type;
    }

    function resolveBinaryExpression(node: BinaryExpression): SemanticType {
        const leftType = resolveExpression(node.left);
        const rightType = resolveExpression(node.right);

        // Arithmetic operators: +, -, *, /, %
        if (['+', '-', '*', '/', '%'].includes(node.operator)) {
            // String concatenation with + operator
            if (node.operator === '+' && leftType.kind === 'primitive' && leftType.name === 'textus') {
                node.resolvedType = TEXTUS;

                return TEXTUS;
            }

            // Numeric arithmetic - preserve type when both operands are the same numeric type
            const numericTypes = ['numerus', 'fractus', 'decimus', 'magnus'];
            if (leftType.kind === 'primitive' && rightType.kind === 'primitive') {
                if (numericTypes.includes(leftType.name) && numericTypes.includes(rightType.name)) {
                    // Same type: preserve it
                    if (leftType.name === rightType.name) {
                        node.resolvedType = leftType;
                        return leftType;
                    }
                    // Mixed numeric types: use left type (caller's responsibility)
                    node.resolvedType = leftType;
                    return leftType;
                }
            }

            // Mixed or unknown - default to numerus
            node.resolvedType = NUMERUS;

            return NUMERUS;
        }

        // Comparison operators: <, >, <=, >=
        // WHY: Only allow comparison between compatible types (both numeric or both string)
        if (['<', '>', '<=', '>='].includes(node.operator)) {
            const leftPrim = leftType.kind === 'primitive' ? leftType.name : null;
            const rightPrim = rightType.kind === 'primitive' ? rightType.name : null;

            // Check for incompatible comparison (e.g., numerus > textus)
            if (leftPrim && rightPrim && leftPrim !== rightPrim) {
                // Allow unknown types to pass through
                if (leftPrim !== 'unknown' && rightPrim !== 'unknown') {
                    const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.IncompatibleComparison];

                    error(`${text(formatType(leftType), formatType(rightType), node.operator)}\n${help}`, node.position);
                }
            }

            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        // Equality operators: ==, !=
        if (['==', '!='].includes(node.operator)) {
            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        // Logical operators: &&, ||
        if (['&&', '||'].includes(node.operator)) {
            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        node.resolvedType = UNKNOWN;

        return UNKNOWN;
    }

    function resolveUnaryExpression(node: UnaryExpression): SemanticType {
        const argType = resolveExpression(node.argument);

        // Logical negation: !, non
        if (node.operator === '!' || node.operator === 'non') {
            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        // Null checks: nulla (is null/empty), nonnulla (is not null/non-empty)
        if (node.operator === 'nulla' || node.operator === 'nonnulla') {
            node.resolvedType = BIVALENS;

            return BIVALENS;
        }

        // Numeric negation - preserve the numeric type
        if (node.operator === '-') {
            // Keep the argument type for numeric types (numerus, fractus, decimus, magnus)
            if (argType.kind === 'primitive') {
                const numericTypes = ['numerus', 'fractus', 'decimus', 'magnus'];
                if (numericTypes.includes(argType.name)) {
                    node.resolvedType = argType;
                    return argType;
                }
            }
            node.resolvedType = NUMERUS;
            return NUMERUS;
        }

        node.resolvedType = argType;

        return argType;
    }

    function resolveCallExpression(node: CallExpression): SemanticType {
        const calleeType = resolveExpression(node.callee);

        // Resolve argument types (handle SpreadElement)
        for (const arg of node.arguments) {
            if (arg.type === 'SpreadElement') {
                resolveExpression(arg.argument);
            }
            else {
                resolveExpression(arg);
            }
        }

        // If callee is a function type, return its return type
        if (calleeType.kind === 'function') {
            node.resolvedType = calleeType.returnType;

            return calleeType.returnType;
        }

        node.resolvedType = UNKNOWN;

        return UNKNOWN;
    }

    function resolveMemberExpression(node: MemberExpression): SemanticType {
        resolveExpression(node.object);

        // TODO: Property access type depends on the object type
        node.resolvedType = UNKNOWN;

        return UNKNOWN;
    }

    function resolveArrowFunction(node: ArrowFunctionExpression): SemanticType {
        enterScope('function');

        // Define parameters
        const paramTypes: SemanticType[] = [];

        for (const param of node.params) {
            const paramType = param.typeAnnotation ? resolveTypeAnnotation(param.typeAnnotation) : UNKNOWN;

            paramTypes.push(paramType);
            define({
                name: param.name.name,
                type: paramType,
                kind: 'parameter',
                mutable: false,
                position: param.position,
            });
        }

        // Resolve body
        let returnType: SemanticType;

        if (node.body.type === 'BlockStatement') {
            analyzeBlock(node.body);
            returnType = VACUUM;
        } else {
            returnType = resolveExpression(node.body as Expression);
        }

        exitScope();

        const fnType = functionType(paramTypes, returnType, node.async);

        node.resolvedType = fnType;

        return fnType;
    }

    /**
     * Resolve lambda expression (pro ... redde or pro ... { }).
     *
     * WHY: LambdaExpression is simpler than ArrowFunction - params are just
     *      identifiers (no type annotations). Body can be expression or block.
     */
    function resolveLambdaExpression(node: LambdaExpression): SemanticType {
        enterScope('function');

        // Define parameters (untyped - infer from usage)
        const paramTypes: SemanticType[] = [];

        for (const param of node.params) {
            const paramType = UNKNOWN;
            paramTypes.push(paramType);
            define({
                name: param.name,
                type: paramType,
                kind: 'parameter',
                mutable: false,
                position: param.position,
            });
        }

        // Resolve body
        let returnType: SemanticType;

        if (node.body.type === 'BlockStatement') {
            analyzeBlock(node.body);
            returnType = VACUUM;
        } else {
            returnType = resolveExpression(node.body as Expression);
        }

        exitScope();

        const fnType = functionType(paramTypes, returnType, node.async);
        node.resolvedType = fnType;

        return fnType;
    }

    function resolveAssignment(node: AssignmentExpression): SemanticType {
        const rightType = resolveExpression(node.right);

        if (node.left.type === 'Identifier') {
            const symbol = lookupSymbol(currentScope, node.left.name);

            if (!symbol) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.UndefinedVariable];

                error(`${text(node.left.name)}\n${help}`, node.left.position);
            } else if (!symbol.mutable) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.ImmutableAssignment];

                error(`${text(node.left.name)}\n${help}`, node.position);
            } else if (!isAssignableTo(rightType, symbol.type)) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.TypeMismatch];

                error(`${text(formatType(rightType), formatType(symbol.type))}\n${help}`, node.position);
            }
        } else {
            resolveExpression(node.left);
        }

        node.resolvedType = rightType;

        return rightType;
    }

    function resolveAwait(node: AwaitExpression): SemanticType {
        const argType = resolveExpression(node.argument);

        // If awaiting a Promise, unwrap it
        if (argType.kind === 'generic' && argType.name === 'promissum') {
            const unwrapped = argType.typeParameters[0] ?? UNKNOWN;

            node.resolvedType = unwrapped;

            return unwrapped;
        }

        node.resolvedType = argType;

        return argType;
    }

    function resolveNew(node: NewExpression): SemanticType {
        // Resolve constructor arguments
        for (const arg of node.arguments) {
            resolveExpression(arg);
        }

        const type = userType(node.callee.name);

        node.resolvedType = type;

        return type;
    }

    function resolveConditional(node: Expression & { type: 'ConditionalExpression' }): SemanticType {
        resolveExpression(node.test);

        const consequentType = resolveExpression(node.consequent);
        const alternateType = resolveExpression(node.alternate);

        // Return union of both branches if different
        if (
            consequentType.kind === alternateType.kind &&
            consequentType.kind === 'primitive' &&
            (consequentType as any).name === (alternateType as any).name
        ) {
            node.resolvedType = consequentType;

            return consequentType;
        }

        const result = unionType([consequentType, alternateType]);

        node.resolvedType = result;

        return result;
    }

    // ---------------------------------------------------------------------------
    // Statement Analysis
    // ---------------------------------------------------------------------------

    function analyzeStatement(node: Statement): void {
        switch (node.type) {
            case 'ImportDeclaration':
                analyzeImportDeclaration(node);
                break;

            case 'VariableDeclaration':
                analyzeVariableDeclaration(node);
                break;

            case 'FunctionDeclaration':
                analyzeFunctionDeclaration(node);
                break;

            case 'TypeAliasDeclaration':
                analyzeTypeAliasDeclaration(node);
                break;

            case 'IfStatement':
                analyzeIfStatement(node);
                break;

            case 'WhileStatement':
                analyzeWhileStatement(node);
                break;

            case 'ForStatement':
                analyzeForStatement(node);
                break;

            case 'WithStatement':
                analyzeWithStatement(node);
                break;

            case 'SwitchStatement':
                analyzeSwitchStatement(node);
                break;

            case 'GuardStatement':
                analyzeGuardStatement(node);
                break;

            case 'AssertStatement':
                analyzeAssertStatement(node);
                break;

            case 'ReturnStatement':
                analyzeReturnStatement(node);
                break;

            case 'BlockStatement':
                analyzeBlock(node);
                break;

            case 'ExpressionStatement':
                resolveExpression(node.expression);
                break;

            case 'ThrowStatement':
                analyzeThrowStatement(node);
                break;

            case 'ScribeStatement':
                analyzeScribeStatement(node);
                break;

            case 'TryStatement':
                analyzeTryStatement(node);
                break;

            case 'FacBlockStatement':
                analyzeFacBlockStatement(node);
                break;

            default: {
                const _exhaustive: never = node;
                break;
            }
        }
    }

    function analyzeScribeStatement(node: ScribeStatement): void {
        for (const arg of node.arguments) {
            resolveExpression(arg);
        }
    }

    function analyzeVariableDeclaration(node: VariableDeclaration): void {
        // Handle object destructuring pattern
        if (node.name.type === 'ObjectPattern') {
            if (node.init) {
                resolveExpression(node.init);
            }

            // Define each property as a variable
            for (const prop of node.name.properties) {
                define({
                    name: prop.value.name,
                    type: UNKNOWN,
                    kind: 'variable',
                    mutable: node.kind === 'varia',
                    position: prop.position,
                });
            }

            return;
        }

        // Standard variable declaration - resolve type
        let type: SemanticType;

        if (node.typeAnnotation) {
            type = resolveTypeAnnotation(node.typeAnnotation);
        } else if (node.init) {
            type = resolveExpression(node.init);
        } else {
            type = UNKNOWN;

            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.NoTypeOrInitializer];

            error(`${text(node.name.name)}\n${help}`, node.position);
        }

        // Check initializer type compatibility
        if (node.typeAnnotation && node.init) {
            const initType = resolveExpression(node.init);

            if (!isAssignableTo(initType, type)) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.TypeMismatch];

                error(`${text(formatType(initType), formatType(type))}\n${help}`, node.position);
            }
        }

        // Define in symbol table
        define({
            name: node.name.name,
            type,
            kind: 'variable',
            mutable: node.kind === 'varia',
            position: node.position,
        });

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    function analyzeFunctionDeclaration(node: FunctionDeclaration): void {
        // Build function type from parameters and return type
        const paramTypes: SemanticType[] = node.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
        const returnType = node.returnType ? resolveTypeAnnotation(node.returnType) : VACUUM;

        const fnType = functionType(paramTypes, returnType, node.async);

        // Define function in current scope
        define({
            name: node.name.name,
            type: fnType,
            kind: 'function',
            mutable: false,
            position: node.position,
        });

        // Analyze function body in new scope
        enterScope('function');

        const previousReturnType = currentFunctionReturnType;

        currentFunctionReturnType = returnType;

        // Define parameters
        for (let i = 0; i < node.params.length; i++) {
            const param = node.params[i];

            define({
                name: param.name.name,
                type: paramTypes[i],
                kind: 'parameter',
                mutable: false,
                position: param.position,
            });
        }

        analyzeBlock(node.body);

        currentFunctionReturnType = previousReturnType;
        exitScope();

        node.resolvedType = fnType;
    }

    function analyzeTypeAliasDeclaration(node: TypeAliasDeclaration): void {
        const type = resolveTypeAnnotation(node.typeAnnotation);

        // WHY: Type aliases are stored as "type" symbols to distinguish from variables
        define({
            name: node.name.name,
            type,
            kind: 'type',
            mutable: false,
            position: node.position,
        });

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    function analyzeIfStatement(node: IfStatement): void {
        const testType = resolveExpression(node.test);

        // TODO: Warn but don't error - truthy/falsy is valid
        if (testType.kind === 'primitive' && testType.name !== 'bivalens') {
            // Could warn here about non-boolean in condition
        }

        enterScope();
        analyzeBlock(node.consequent);
        exitScope();

        if (node.alternate) {
            if (node.alternate.type === 'IfStatement') {
                analyzeIfStatement(node.alternate);
            } else {
                enterScope();
                analyzeBlock(node.alternate);
                exitScope();
            }
        }

        if (node.catchClause) {
            analyzeCatchClause(node.catchClause);
        }
    }

    function analyzeWhileStatement(node: WhileStatement): void {
        resolveExpression(node.test);

        enterScope();
        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCatchClause(node.catchClause);
        }
    }

    function analyzeForStatement(node: ForStatement): void {
        resolveExpression(node.iterable);

        enterScope();

        // TODO: Infer loop variable type from iterable element type
        define({
            name: node.variable.name,
            type: UNKNOWN,
            kind: 'variable',
            mutable: false,
            position: node.variable.position,
        });

        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCatchClause(node.catchClause);
        }
    }

    function analyzeWithStatement(node: WithStatement): void {
        resolveExpression(node.object);

        // WHY: Inside 'in' blocks, bare identifier assignments become property
        //      assignments on the context object. We don't validate these as
        //      variables since they'll be transformed at codegen time.
        enterScope();

        for (const stmt of node.body.body) {
            const isBareAssignment =
                stmt.type === 'ExpressionStatement' && stmt.expression.type === 'AssignmentExpression' && stmt.expression.left.type === 'Identifier';

            if (isBareAssignment) {
                // Skip validation for bare identifier assignments
                resolveExpression((stmt.expression as AssignmentExpression).right);
            } else {
                analyzeStatement(stmt);
            }
        }

        exitScope();
    }

    function analyzeSwitchStatement(node: SwitchStatement): void {
        resolveExpression(node.discriminant);

        for (const caseNode of node.cases) {
            resolveExpression(caseNode.test);

            enterScope();
            analyzeBlock(caseNode.consequent);
            exitScope();
        }

        if (node.defaultCase) {
            enterScope();
            analyzeBlock(node.defaultCase);
            exitScope();
        }

        if (node.catchClause) {
            analyzeCatchClause(node.catchClause);
        }
    }

    function analyzeGuardStatement(node: GuardStatement): void {
        for (const clause of node.clauses) {
            resolveExpression(clause.test);

            enterScope();
            analyzeBlock(clause.consequent);
            exitScope();
        }
    }

    function analyzeAssertStatement(node: AssertStatement): void {
        resolveExpression(node.test);

        if (node.message) {
            resolveExpression(node.message);
        }
    }

    function analyzeReturnStatement(node: ReturnStatement): void {
        if (!node.argument) {
            return;
        }

        const returnType = resolveExpression(node.argument);

        if (currentFunctionReturnType && !isAssignableTo(returnType, currentFunctionReturnType)) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.ReturnTypeMismatch];

            error(`${text(formatType(returnType), formatType(currentFunctionReturnType))}\n${help}`, node.position);
        }
    }

    function analyzeThrowStatement(node: ThrowStatement): void {
        resolveExpression(node.argument);
    }

    function analyzeTryStatement(node: TryStatement): void {
        enterScope();
        analyzeBlock(node.block);
        exitScope();

        if (node.handler) {
            analyzeCatchClause(node.handler);
        }

        if (node.finalizer) {
            enterScope();
            analyzeBlock(node.finalizer);
            exitScope();
        }
    }

    function analyzeCatchClause(node: { param: Identifier; body: BlockStatement }): void {
        enterScope();

        define({
            name: node.param.name,
            type: userType('Error'),
            kind: 'parameter',
            mutable: false,
            position: node.param.position,
        });

        analyzeBlock(node.body);
        exitScope();
    }

    /**
     * Analyze fac block statement (explicit scope block).
     *
     * WHY: fac creates an explicit scope. With cape, it's like try-catch.
     */
    function analyzeFacBlockStatement(node: FacBlockStatement): void {
        enterScope();
        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCatchClause(node.catchClause);
        }
    }

    function analyzeBlock(node: BlockStatement): void {
        for (const stmt of node.body) {
            analyzeStatement(stmt);
        }
    }

    // ---------------------------------------------------------------------------
    // Main Analysis
    // ---------------------------------------------------------------------------

    for (const stmt of program.body) {
        analyzeStatement(stmt);
    }

    return { program, errors };
}

// Re-export types
export * from './types';
export * from './scope';
export * from './errors';
