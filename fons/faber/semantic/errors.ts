/**
 * Semantic Error Catalog
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module defines all error codes and messages for the semantic analysis
 * phase. Each error has:
 * - A unique code (S001, S002, etc.)
 * - User-facing text
 * - Helpful guidance that teaches proper usage
 *
 * Error messages should:
 * - Be clear and actionable
 * - Teach Latin syntax and semantics
 * - Point to specific locations in source
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  None (constant definitions)
 * OUTPUT: Error catalog for semantic analyzer
 * ERRORS: N/A
 *
 * @module semantic/errors
 */

// =============================================================================
// ERROR CODES
// =============================================================================

/**
 * Semantic error codes.
 *
 * WHY: S-prefix identifies semantic phase errors at a glance.
 */
export enum SemanticErrorCode {
    UndefinedVariable = 'S001',
    AlreadyDefined = 'S002',
    ImmutableAssignment = 'S003',
    TypeMismatch = 'S004',
    ReturnTypeMismatch = 'S005',
    NoTypeOrInitializer = 'S006',
    NotExportedFromModule = 'S007',
    IncompatibleComparison = 'S008',
    CedeOutsideAsyncOrGenerator = 'S009',
    AwaitOutsideAsync = 'S010',
    DefaultWithBorrowedParam = 'S011',
    RequiredAfterOptional = 'S015',
    MissingFunctionBody = 'S016',
    NonExhaustiveMatch = 'S017',
    DuplicateModifier = 'S018',
    InvalidModifierContext = 'S019',
    ConflictingModifiers = 'S020',
    ExitusRequiresVacuumReturn = 'S021',
    // Import resolution errors
    ModuleNotFound = 'S012',
    ModuleParseError = 'S014',
}

// =============================================================================
// ERROR CATALOG
// =============================================================================

/**
 * Error message catalog with text and help fields.
 *
 * DESIGN: Catalog is indexed by error code for O(1) lookup.
 *         Each entry provides user-facing text and helpful guidance.
 */
export const SEMANTIC_ERRORS = {
    [SemanticErrorCode.UndefinedVariable]: {
        text: (name: string) => `Undefined variable '${name}'`,
        help: "Variables must be declared with 'varia' or 'fixum' before use",
    },
    [SemanticErrorCode.AlreadyDefined]: {
        text: (name: string, line: number) => `'${name}' is already defined at line ${line}`,
        help: 'Each name can only be defined once in the same scope. Use a different name or assign to the existing variable.',
    },
    [SemanticErrorCode.ImmutableAssignment]: {
        text: (name: string) => `Cannot assign to immutable variable '${name}'`,
        help: "Variables declared with 'fixum' cannot be reassigned. Use 'varia' for mutable variables.",
    },
    [SemanticErrorCode.TypeMismatch]: {
        text: (sourceType: string, targetType: string) => `Type '${sourceType}' is not assignable to type '${targetType}'`,
        help: 'Ensure the types are compatible. You may need a type conversion or to change the variable type.',
    },
    [SemanticErrorCode.ReturnTypeMismatch]: {
        text: (returnType: string, functionType: string) => `Return type '${returnType}' is not assignable to function return type '${functionType}'`,
        help: 'The returned value must match the function return type annotation.',
    },
    [SemanticErrorCode.NoTypeOrInitializer]: {
        text: (name: string) => `Variable '${name}' has no type annotation or initializer`,
        help: "Variables must have either a type annotation (e.g., 'numerus x') or an initializer (e.g., 'varia x = 5').",
    },
    [SemanticErrorCode.NotExportedFromModule]: {
        text: (name: string, module: string) => `'${name}' is not exported from '${module}'`,
        help: 'Check the module documentation for available exports. You may have a typo in the import name.',
    },
    [SemanticErrorCode.IncompatibleComparison]: {
        text: (leftType: string, rightType: string, operator: string) => `Cannot compare '${leftType}' with '${rightType}' using '${operator}'`,
        help: 'Comparison operators require operands of the same type. Both sides must be numbers or both must be strings.',
    },
    [SemanticErrorCode.CedeOutsideAsyncOrGenerator]: {
        text: () => "'cede' requires an async or generator function context",
        help: "Use 'fiet' for async functions, 'fiunt' for generators, or 'fient' for async generators.",
    },
    [SemanticErrorCode.AwaitOutsideAsync]: {
        text: (keyword: string) => `'${keyword}' requires an async function context`,
        help: "Use 'fiet' or 'fient' to declare an async function.",
    },
    [SemanticErrorCode.DefaultWithBorrowedParam]: {
        text: (preposition: string) => `Cannot use default value with '${preposition}' (borrowed) parameter`,
        help: "Default values require owned parameters. Remove the 'de' or 'in' preposition, or remove the default value.",
    },
    [SemanticErrorCode.RequiredAfterOptional]: {
        text: (name: string) => `Required parameter '${name}' cannot follow optional parameters`,
        help: "Optional parameters (marked with 'si') must come after all required parameters.",
    },
    [SemanticErrorCode.ModuleNotFound]: {
        text: (path: string) => `Cannot find module '${path}'`,
        help: 'Verify the file path is correct and the file exists. Relative paths are resolved from the importing file.',
    },
    [SemanticErrorCode.ModuleParseError]: {
        text: (path: string) => `Failed to parse module '${path}'`,
        help: 'Fix the syntax errors in the imported module before importing it.',
    },
    [SemanticErrorCode.MissingFunctionBody]: {
        text: (name: string) => `Function '${name}' has no body`,
        help: "Regular functions require a body. Use '@ externa' annotation for external declarations.",
    },
    [SemanticErrorCode.NonExhaustiveMatch]: {
        text: (missing: string[]) => `Non-exhaustive match: missing variant${missing.length > 1 ? 's' : ''} ${missing.map(v => `'${v}'`).join(', ')}`,
        help: 'All variants of a discretio must be handled in a discerne statement.',
    },
    [SemanticErrorCode.DuplicateModifier]: {
        text: (name: string) => `Duplicate modifier '${name}'`,
        help: 'Each modifier can appear at most once in a function signature.',
    },
    [SemanticErrorCode.InvalidModifierContext]: {
        text: (name: string, context: string) => `Modifier '${name}' is only allowed on ${context}`,
        help: 'Remove the modifier or move it to a valid context.',
    },
    [SemanticErrorCode.ConflictingModifiers]: {
        text: (mod1: string, mod2: string) => `Modifiers '${mod1}' and '${mod2}' cannot be used together`,
        help: 'These modifiers have incompatible semantics. Remove one of them.',
    },
    [SemanticErrorCode.ExitusRequiresVacuumReturn]: {
        text: (returnType: string) => `Modifier 'exitus' requires vacuum return type, but function returns '${returnType}'`,
        help: "Functions with 'exitus' cannot have a return value. Remove the return type or use 'redde' without a value.",
    },
} as const;
