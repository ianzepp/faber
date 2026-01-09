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
 * The analyzer uses a three-phase approach to enable forward references:
 *
 * Phase 1a - Predeclaration:
 *   All top-level names (functions, types, etc.) are registered with
 *   placeholder types. This makes names visible before bodies are analyzed.
 *
 * Phase 1b - Signature Resolution:
 *   Type annotations are resolved now that all names exist. Placeholder
 *   types are updated to their real types.
 *
 * Phase 2 - Body Analysis:
 *   Function bodies and other statements are analyzed with the complete
 *   symbol table. Expressions are typed, compatibility is checked.
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
    ImportaDeclaration,
    DestructureDeclaration,
    VariaDeclaration,
    FunctioDeclaration,
    TypeAliasDeclaration,
    OrdoDeclaration,
    GenusDeclaration,
    PactumDeclaration,
    SiStatement,
    DumStatement,
    IteratioStatement,
    InStatement,
    EligeStatement,
    DiscerneStatement,
    CustodiStatement,
    AdfirmaStatement,
    ReddeStatement,
    BlockStatement,
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    AssignmentExpression,
    CedeExpression,
    NovumExpression,
    FingeExpression,
    ArrayExpression,
    ObjectExpression,
    TypeAnnotation,
    IaceStatement,
    ScribeStatement,
    TemptaStatement,
    FacBlockStatement,
    LambdaExpression,
    ProbandumStatement,
    ProbaStatement,
    PraeparaBlock,
    CuraStatement,
    DiscretioDeclaration,
    CollectionDSLExpression,
    IncipitStatement,
    IncipietStatement,
    AdStatement,
    ConversionExpression,
} from '../parser/ast';
import type { Position } from '../tokenizer/types';
import type { Scope, Symbol } from './scope';
import { createGlobalScope, createScope, defineSymbol, lookupSymbol, lookupSymbolLocal, updateSymbolType } from './scope';
import type { SemanticType, FunctionType, DiscretioType, VariantInfo, GenusType } from './types';
import {
    genericType,
    functionType,
    unionType,
    userType,
    enumType,
    genusType,
    pactumType,
    discretioType,
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
import { isLocalImport, resolveModule, createModuleContext, type ModuleContext, type ModuleExports } from './modules';
import type { Annotation } from '../parser/ast';

// =============================================================================
// ANNOTATION HELPERS
// =============================================================================

/**
 * Check if annotations include async modifier (futura).
 */
function isAsyncFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;
    for (const ann of annotations) {
        if (ann.name === 'futura') return true;
    }
    return false;
}

/**
 * Check if annotations include generator modifier (cursor).
 */
function isGeneratorFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;
    for (const ann of annotations) {
        if (ann.name === 'cursor') return true;
    }
    return false;
}

/**
 * Check if annotations include external declaration modifier (externa).
 *
 * WHY: External declarations tell the compiler the symbol exists but is
 *      provided elsewhere (runtime, FFI, linker). No initializer or body required.
 */
function isExternaFromAnnotations(annotations?: Annotation[]): boolean {
    if (!annotations) return false;
    for (const ann of annotations) {
        if (ann.name === 'externa') return true;
    }
    return false;
}

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

/**
 * Options for semantic analysis.
 */
export interface AnalyzeOptions {
    /** Absolute path to the file being analyzed (enables local imports) */
    filePath?: string;
    /** Module context for caching and cycle detection (created automatically if filePath provided) */
    moduleContext?: ModuleContext;
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
    ignotum: UNKNOWN,
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
 * Norma/mathesis module exports.
 *
 * Math operations: floor, ceil, sqrt, pow, trig, etc.
 * When `ex "norma/mathesis" importa X` is encountered, these symbols are added to scope.
 */
const NORMA_MATHESIS_EXPORTS: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> = {
    // Rounding
    pavimentum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    tectum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    rotundum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    truncatum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },

    // Powers and roots
    radix: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    potentia: { type: functionType([FRACTUS, FRACTUS], FRACTUS), kind: 'function' },
    logarithmus: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    logarithmus10: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    exponens: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },

    // Trigonometry
    sinus: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    cosinus: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    tangens: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },

    // Absolute and sign
    absolutum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },
    signum: { type: functionType([FRACTUS], FRACTUS), kind: 'function' },

    // Min/max/clamp
    minimus: { type: functionType([FRACTUS, FRACTUS], FRACTUS), kind: 'function' },
    maximus: { type: functionType([FRACTUS, FRACTUS], FRACTUS), kind: 'function' },
    constringens: { type: functionType([FRACTUS, FRACTUS, FRACTUS], FRACTUS), kind: 'function' },

    // Constants
    PI: { type: FRACTUS, kind: 'variable' },
    E: { type: FRACTUS, kind: 'variable' },
    TAU: { type: FRACTUS, kind: 'variable' },
};

/**
 * Norma/aleator module exports.
 *
 * Random number generation: floats, integers, bytes, UUIDs, collection operations.
 * When `ex "norma/aleator" importa X` is encountered, these symbols are added to scope.
 */
const NORMA_ALEATOR_EXPORTS: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> = {
    // Basic generation
    fractus: { type: functionType([], FRACTUS), kind: 'function' },
    inter: { type: functionType([NUMERUS, NUMERUS], NUMERUS), kind: 'function' },
    octeti: { type: functionType([NUMERUS], OCTETI), kind: 'function' },
    uuid: { type: functionType([], TEXTUS), kind: 'function' },

    // Collection operations (generic - takes lista<T>, returns T or lista<T>)
    selige: { type: functionType([UNKNOWN], UNKNOWN), kind: 'function' },
    miscita: { type: functionType([UNKNOWN], UNKNOWN), kind: 'function' },

    // Seeding
    semen: { type: functionType([NUMERUS], VACUUM), kind: 'function' },
};

/**
 * Norma/solum module exports.
 *
 * Local I/O operations: file reading/writing, path utilities.
 * When `ex "norma/solum" importa X` is encountered, these symbols are added to scope.
 */
const NORMA_SOLUM_EXPORTS: Record<string, { type: SemanticType; kind: 'function' | 'variable' }> = {
    // Reading - streaming
    legens: { type: functionType([TEXTUS], UNKNOWN), kind: 'function' },
    // Reading - batch
    leget: { type: functionType([TEXTUS], genericType('promissum', [TEXTUS])), kind: 'function' },
    lege: { type: functionType([TEXTUS], TEXTUS), kind: 'function' },
    // Stdin
    ausculta: { type: functionType([], UNKNOWN), kind: 'function' },
    hauri: { type: functionType([], TEXTUS), kind: 'function' },
    // Writing - streaming
    scribens: { type: functionType([TEXTUS], UNKNOWN), kind: 'function' },
    // Writing - batch
    scribet: { type: functionType([TEXTUS, TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    inscribe: { type: functionType([TEXTUS, TEXTUS], VACUUM), kind: 'function' },
    // Appending
    apponet: { type: functionType([TEXTUS, TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    appone: { type: functionType([TEXTUS, TEXTUS], VACUUM), kind: 'function' },
    // File info
    exstat: { type: functionType([TEXTUS], BIVALENS), kind: 'function' },
    inspice: { type: functionType([TEXTUS], UNKNOWN), kind: 'function' },
    // File operations
    dele: { type: functionType([TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    duplica: { type: functionType([TEXTUS, TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    move: { type: functionType([TEXTUS, TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    trunca: { type: functionType([TEXTUS, NUMERUS], genericType('promissum', [VACUUM])), kind: 'function' },
    tange: { type: functionType([TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    // Directory operations
    crea: { type: functionType([TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    elenca: { type: functionType([TEXTUS], genericType('promissum', [genericType('lista', [TEXTUS])])), kind: 'function' },
    ambula: { type: functionType([TEXTUS], UNKNOWN), kind: 'function' },
    vacua: { type: functionType([TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    deleArborem: { type: functionType([TEXTUS], genericType('promissum', [VACUUM])), kind: 'function' },
    // Path utilities
    iunge: { type: functionType([genericType('lista', [TEXTUS])], TEXTUS), kind: 'function' },
    dir: { type: functionType([TEXTUS], TEXTUS), kind: 'function' },
    basis: { type: functionType([TEXTUS], TEXTUS), kind: 'function' },
    extensio: { type: functionType([TEXTUS], TEXTUS), kind: 'function' },
    resolve: { type: functionType([TEXTUS], TEXTUS), kind: 'function' },
    domus: { type: functionType([], TEXTUS), kind: 'function' },
};

/**
 * Map of all norma submodule exports.
 */
const NORMA_SUBMODULES: Record<string, Record<string, { type: SemanticType; kind: 'function' | 'variable' }>> = {
    'norma/tempus': NORMA_TEMPUS_EXPORTS,
    'norma/mathesis': NORMA_MATHESIS_EXPORTS,
    'norma/aleator': NORMA_ALEATOR_EXPORTS,
    'norma/solum': NORMA_SOLUM_EXPORTS,
};

// =============================================================================
// MAIN ANALYZER
// =============================================================================

/**
 * Perform semantic analysis on a program.
 *
 * @param program - The parsed AST
 * @param options - Optional settings for local import resolution
 */
export function analyze(program: Program, options: AnalyzeOptions = {}): SemanticResult {
    const errors: SemanticError[] = [];
    let currentScope: Scope = createGlobalScope();
    let currentFunctionReturnType: SemanticType | null = null;
    let currentFunctionAsync = false;
    let currentFunctionGenerator = false;
    let currentGenusType: SemanticType | null = null;

    // WHY: Track type aliases currently being resolved to detect cycles
    const resolvingTypeAliases = new Set<string>();

    // WHY: Create module context for local import resolution if file path provided
    const moduleContext: ModuleContext | undefined = options.moduleContext ?? (options.filePath ? createModuleContext(options.filePath) : undefined);

    // WHY: Mark the current file as "in progress" for circular import detection
    // Without this, imports of the entry file wouldn't be detected as cycles
    if (moduleContext && options.filePath) {
        moduleContext.inProgress.add(moduleContext.basePath);
    }

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
        const builtinPos = { line: 0, column: 0, offset: 0 };

        function defFn(name: string, params: SemanticType[], ret: SemanticType): void {
            currentScope.symbols.set(name, {
                name,
                type: functionType(params, ret),
                kind: 'function',
                mutable: false,
                position: builtinPos,
            });
        }

        // I/O Intrinsics (always available - prefixed with _ for internal use)
        defFn('_scribe', [], VACUUM);
        defFn('_vide', [], VACUUM);
        defFn('_mone', [], VACUUM);
        defFn('_lege', [], TEXTUS);

        // NOTE: Math intrinsics removed - use norma/mathesis module instead
        // NOTE: Random intrinsics removed - use norma/aleator module instead
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
     * Resolution order:
     * 1. 'norma' base library -> compiler intrinsics
     * 2. 'norma/*' submodules -> compiler intrinsics
     * 3. Local imports ('./' or '../') -> parse and extract exports
     * 4. Other modules -> pass through to target language
     */
    function analyzeImportaDeclaration(node: ImportaDeclaration): void {
        const moduleName = node.source;

        // Check for norma stdlib first
        if (moduleName === 'norma') {
            analyzeNormaImport(node, NORMA_EXPORTS);
            return;
        }

        if (moduleName in NORMA_SUBMODULES) {
            analyzeNormaImport(node, NORMA_SUBMODULES[moduleName]!);
            return;
        }

        // Check for local file imports
        if (isLocalImport(moduleName) && moduleContext) {
            analyzeLocalImport(node);
            return;
        }

        // WHY: External packages pass through without type info
        // Target language compiler will validate them
    }

    /**
     * Handle imports from norma stdlib.
     */
    function analyzeNormaImport(node: ImportaDeclaration, exports: Record<string, { type: SemanticType; kind: 'function' | 'variable' }>): void {
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
            const importedName = specifier.imported.name;
            const localName = specifier.local.name;
            const exportInfo = exports[importedName];

            if (!exportInfo) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.NotExportedFromModule];
                error(`${text(importedName, node.source)}\n${help}`, specifier.position);
                continue;
            }

            currentScope.symbols.set(localName, {
                name: localName,
                type: exportInfo.type,
                kind: exportInfo.kind,
                mutable: false,
                position: specifier.position,
            });
        }
    }

    /**
     * Handle imports from local .fab files.
     */
    function analyzeLocalImport(node: ImportaDeclaration): void {
        if (!moduleContext) {
            return;
        }

        const result = resolveModule(node.source, moduleContext);

        if (!result.ok) {
            // Map module error to semantic error
            switch (result.error) {
                case 'not_found': {
                    const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.ModuleNotFound];
                    error(`${text(node.source)}\n${help}`, node.position);
                    break;
                }
                case 'cycle': {
                    const { help } = SEMANTIC_ERRORS[SemanticErrorCode.CircularImport];
                    // WHY: result.message already contains the full cycle path
                    error(`${result.message}\n${help}`, node.position);
                    break;
                }
                case 'parse_error': {
                    const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.ModuleParseError];
                    error(`${text(node.source)}: ${result.message}\n${help}`, node.position);
                    break;
                }
            }
            return;
        }

        const moduleExports = result.module.exports;

        if (node.wildcard) {
            // ex "./utils" importa * - add all exports to scope
            for (const [name, exportInfo] of moduleExports) {
                currentScope.symbols.set(name, {
                    name,
                    type: exportInfo.type,
                    kind: exportInfo.kind === 'function' || exportInfo.kind === 'variable' ? exportInfo.kind : 'variable',
                    mutable: false,
                    position: node.position,
                });
            }
            return;
        }

        // ex "./utils" importa foo, bar - add specific exports
        for (const specifier of node.specifiers) {
            const importedName = specifier.imported.name;
            const localName = specifier.local.name;
            const exportInfo = moduleExports.get(importedName);

            if (!exportInfo) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.NotExportedFromModule];
                error(`${text(importedName, node.source)}\n${help}`, specifier.position);
                continue;
            }

            currentScope.symbols.set(localName, {
                name: localName,
                type: exportInfo.type,
                kind: exportInfo.kind === 'function' || exportInfo.kind === 'variable' ? exportInfo.kind : 'variable',
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

        // Check for built-in primitive type
        const primitive = LATIN_TYPE_MAP[node.name];

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

        // Check for generic type
        if (GENERIC_TYPES.has(node.name)) {
            // Filter only TypeAnnotation params for generic type parameters
            const typeParams = (node.typeParameters ?? [])
                .filter(p => p.type === 'TypeAnnotation')
                .map(p => resolveTypeAnnotation(p as TypeAnnotation));

            return genericType(node.name, typeParams, node.nullable);
        }

        // Check if it's a type alias in the symbol table
        const typeAlias = lookupSymbol(currentScope, node.name);

        if (typeAlias && typeAlias.kind === 'type') {
            // WHY: Detect circular type alias references
            if (resolvingTypeAliases.has(node.name)) {
                error(`Circular type alias: '${node.name}' references itself`, node.position);

                return UNKNOWN;
            }

            if (node.nullable && !typeAlias.type.nullable) {
                return { ...typeAlias.type, nullable: true };
            }

            return typeAlias.type;
        }

        // WHY: If a named type is already in scope as a structured type,
        // return the full type instead of a placeholder userType.
        if (typeAlias) {
            const known = typeAlias.type;
            if (known.kind === 'genus' || known.kind === 'pactum' || known.kind === 'enum' || known.kind === 'discretio') {
                return node.nullable && !known.nullable ? { ...known, nullable: true } : known;
            }
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
            // WHY: Distinguish integer literals from float literals
            // 42 → numerus, 3.14 → fractus
            if (Number.isInteger(node.value)) {
                return NUMERUS;
            }
            return FRACTUS;
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

            case 'AssignmentExpression':
                return resolveAssignment(node);

            case 'CedeExpression':
                return resolveAwait(node);

            case 'NovumExpression':
                return resolveNew(node);

            case 'FingeExpression':
                return resolveFingeExpression(node);

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

            case 'EgoExpression':
                // WHY: 'ego' (this) resolves to the enclosing genus type
                if (currentGenusType) {
                    node.resolvedType = currentGenusType;
                    return currentGenusType;
                }
                node.resolvedType = UNKNOWN;
                return UNKNOWN;

            case 'QuaExpression': {
                // WHY: Type cast asserts a type, resolve inner and return target type
                resolveExpression(node.expression);
                const targetType = resolveTypeAnnotation(node.targetType);
                node.resolvedType = targetType;
                return targetType;
            }

            case 'InnatumExpression': {
                // WHY: Native type construction - validates literal matches builtin type
                // {} innatum tabula<K,V> -> constructs native Map/HashMap
                // [] innatum lista<T> -> constructs native array/Vec
                resolveExpression(node.expression);
                const targetType = resolveTypeAnnotation(node.targetType);

                // Validate: {} for tabula, [] for lista
                const typeName = node.targetType.name;
                const exprType = node.expression.type;

                if (typeName === 'tabula') {
                    if (exprType !== 'ObjectExpression') {
                        errors.push({ message: `innatum tabula requires object literal {}, got ${exprType}`, position: node.position });
                    }
                }
                else if (typeName === 'lista') {
                    if (exprType !== 'ArrayExpression') {
                        errors.push({ message: `innatum lista requires array literal [], got ${exprType}`, position: node.position });
                    }
                }
                else {
                    errors.push({ message: `innatum only supports builtin types (tabula, lista), got ${typeName}`, position: node.position });
                }

                node.resolvedType = targetType;
                return targetType;
            }

            case 'ConversionExpression': {
                // WHY: Type conversion operators transform values between primitive types
                // numeratum -> NUMERUS, fractatum -> FRACTUS, textatum -> TEXTUS, bivalentum -> BIVALENS
                resolveExpression(node.expression);

                // Determine result type based on conversion operator
                let resultType: SemanticType;
                switch (node.conversion) {
                    case 'numeratum':
                        // Use targetType if specified (e.g., i32, u64, magnus), otherwise default to NUMERUS
                        resultType = node.targetType ? resolveTypeAnnotation(node.targetType) : NUMERUS;
                        break;
                    case 'fractatum':
                        resultType = node.targetType ? resolveTypeAnnotation(node.targetType) : FRACTUS;
                        break;
                    case 'textatum':
                        resultType = TEXTUS;
                        break;
                    case 'bivalentum':
                        resultType = BIVALENS;
                        break;
                }

                // Validate fallback if present
                if (node.fallback) {
                    const fallbackType = resolveExpression(node.fallback);

                    // Check if fallback is nihil - result becomes nullable
                    if (node.fallback.type === 'Literal' && (node.fallback as Literal).value === null) {
                        resultType = { ...resultType, nullable: true };
                    }
                    else if (!isAssignableTo(fallbackType, resultType)) {
                        errors.push({
                            message: `Conversion fallback type ${formatType(fallbackType)} is not assignable to ${formatType(resultType)}`,
                            position: node.fallback.position,
                        });
                    }
                }

                node.resolvedType = resultType;
                return resultType;
            }

            case 'ShiftExpression': {
                // WHY: Bit shift operators preserve the type of the shifted expression
                // x dextratum 3 → x >> 3, x sinistratum 3 → x << 3
                const exprType = resolveExpression(node.expression);
                resolveExpression(node.amount);

                // Result type is the same as the expression being shifted
                node.resolvedType = exprType;
                return exprType;
            }

            case 'PraefixumExpression':
                // WHY: praefixum is compile-time evaluated, return underlying type
                if (node.body.type === 'BlockStatement') {
                    return UNKNOWN; // Block body requires tracing return
                }
                return resolveExpression(node.body);

            case 'EstExpression':
                // WHY: Type check returns boolean (bivalens)
                resolveExpression(node.expression);
                node.resolvedType = BIVALENS;
                return BIVALENS;

            case 'CollectionDSLExpression':
                // WHY: DSL expressions resolve to their source type after transforms
                // For now, return the source type (transforms don't change element type for prima/ultima)
                // summa would change to numerus, but we'd need smarter type inference for that
                resolveExpression(node.source);
                node.resolvedType = node.source.resolvedType || UNKNOWN;
                return node.resolvedType;

            case 'AbExpression':
                // WHY: Ab expressions (filtering DSL) resolve to their source type
                // Filtering doesn't change the element type, only reduces the collection
                resolveExpression(node.source);
                if (node.filter && node.filter.hasUbi) {
                    // Only resolve the condition if it's a full ubi expression
                    // Boolean property shorthand (hasUbi: false) is just a property name,
                    // not a variable reference, so we don't resolve it
                    resolveExpression(node.filter.condition);
                }
                node.resolvedType = node.source.resolvedType || UNKNOWN;
                return node.resolvedType;

            case 'ScriptumExpression':
                // WHY: Format string expression always returns textus
                // Resolve all argument expressions for type checking
                for (const arg of node.arguments) {
                    resolveExpression(arg);
                }
                node.resolvedType = TEXTUS;
                return TEXTUS;

            case 'LegeExpression':
                // WHY: lege (all) and lege lineam (one line) both return textus
                node.resolvedType = TEXTUS;
                return TEXTUS;

            case 'RegexLiteral':
                // WHY: Regex literals have no sub-expressions to resolve
                node.resolvedType = userType('Regex');
                return node.resolvedType;

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
            } else {
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
                if (argType.kind === 'generic' && argType.name === 'lista' && argType.typeParameters?.[0]) {
                    elementTypes.push(argType.typeParameters[0]);
                } else {
                    elementTypes.push(UNKNOWN);
                }
            } else {
                elementTypes.push(resolveExpression(element));
            }
        }

        // Infer element type from first element
        const inferredElementType = elementTypes[0] ?? UNKNOWN;

        // Validate all elements match the inferred type
        for (let i = 1; i < elementTypes.length; i++) {
            const elemType = elementTypes[i];
            const elemNode = node.elements[i];

            if (elemType && elemNode && !isAssignableTo(elemType, inferredElementType)) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.TypeMismatch];

                error(`${text(formatType(elemType), formatType(inferredElementType))} in array element ${i + 1}\n${help}`, elemNode.position);
            }
        }

        const arrType = genericType('lista', [inferredElementType]);

        node.resolvedType = arrType;

        return arrType;
    }

    function resolveRange(node: Expression & { type: 'RangeExpression' }): SemanticType {
        const startType = resolveExpression(node.start);
        const endType = resolveExpression(node.end);

        if (node.step) {
            resolveExpression(node.step);
        }

        // Determine element type from range endpoints
        // WHY: Ranges can be numeric (0..10) or character ("a".."z")
        // For character ranges used with intra, the range type is lista<textus>
        let elementType: SemanticType = NUMERUS;
        if (startType.kind === 'primitive' && startType.name === 'textus') {
            elementType = TEXTUS;
        } else if (endType.kind === 'primitive' && endType.name === 'textus') {
            elementType = TEXTUS;
        }

        const rangeType = genericType('lista', [elementType]);

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
            // WHY: leftPrim/rightPrim are null for non-primitives (including unknown),
            // so this check only fires when both are known primitive types
            if (leftPrim && rightPrim && leftPrim !== rightPrim) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.IncompatibleComparison];

                error(`${text(formatType(leftType), formatType(rightType), node.operator)}\n${help}`, node.position);
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

        // Range containment: intra
        // WHY: x intra range checks if x is within the range bounds
        // Supports both numeric ranges (x intra 0..100) and character ranges (c intra "a".."z")
        if (node.operator === 'intra') {
            // Right side should be a range expression
            if (node.right.type !== 'RangeExpression') {
                error(`intra operator requires a range expression on the right side, got ${node.right.type}`, node.position);
            }
            // Left side should be numeric or textus (for character range checks)
            const allowedTypes = ['numerus', 'fractus', 'decimus', 'magnus', 'textus'];
            if (leftType.kind === 'primitive' && !allowedTypes.includes(leftType.name)) {
                error(`intra operator left operand must be numeric or textus, got ${formatType(leftType)}`, node.position);
            }

            node.resolvedType = BIVALENS;
            return BIVALENS;
        }

        // Set membership: inter
        // WHY: x inter [a, b, c] checks if x equals any element in the array
        if (node.operator === 'inter') {
            // Right side should be an array (lista<T>)
            if (rightType.kind === 'generic' && rightType.name === 'lista') {
                // Type compatibility check: left type should match array element type
                const elementType = rightType.typeParameters[0];
                if (elementType && leftType.kind !== 'unknown' && elementType.kind !== 'unknown') {
                    if (!isAssignableTo(leftType, elementType)) {
                        error(`inter operator type mismatch: ${formatType(leftType)} inter ${formatType(rightType)}`, node.position);
                    }
                }
            } else if (rightType.kind !== 'unknown') {
                error(`inter operator requires an array on the right side, got ${formatType(rightType)}`, node.position);
            }

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
            } else {
                resolveExpression(arg);
            }
        }

        // If callee is a function type, return its return type
        if (calleeType.kind === 'function') {
            // WHY: Mark call for curator injection if function has curator param
            if (calleeType.hasCuratorParam) {
                node.needsCurator = true;
            }

            node.resolvedType = calleeType.returnType;

            return calleeType.returnType;
        }

        node.resolvedType = UNKNOWN;

        return UNKNOWN;
    }

    function resolveMemberExpression(node: MemberExpression): SemanticType {
        // Check if accessing a type member (enum, genus static field, etc.)
        if (node.object.type === 'Identifier' && !node.computed) {
            const symbol = lookupSymbol(currentScope, node.object.name);
            const propName = (node.property as Identifier).name;

            if (symbol) {
                // Enum member access: Status.pending
                if (symbol.kind === 'enum' && symbol.type.kind === 'enum') {
                    const memberType = symbol.type.members.get(propName);
                    if (memberType) {
                        node.resolvedType = memberType;
                        node.object.resolvedType = symbol.type;
                        return memberType;
                    }
                    // Unknown enum member
                    error(`Enum '${symbol.name}' has no member '${propName}'`, node.position);
                    node.resolvedType = UNKNOWN;
                    return UNKNOWN;
                }

                // Genus static field access: Config.VERSION
                if (symbol.kind === 'genus' && symbol.type.kind === 'genus') {
                    const staticFieldType = symbol.type.staticFields.get(propName);
                    if (staticFieldType) {
                        node.resolvedType = staticFieldType;
                        node.object.resolvedType = symbol.type;
                        return staticFieldType;
                    }
                    const staticMethodType = symbol.type.staticMethods.get(propName);
                    if (staticMethodType) {
                        node.resolvedType = staticMethodType;
                        node.object.resolvedType = symbol.type;
                        return staticMethodType;
                    }
                    // Fall through to instance member access if this is an instance
                }
            }
        }

        // Regular member access on an instance
        const objectType = resolveExpression(node.object);

        // If object is a genus instance, check for fields/methods
        if (objectType.kind === 'genus' && !node.computed) {
            const propName = (node.property as Identifier).name;

            const fieldType = objectType.fields.get(propName);
            if (fieldType) {
                node.resolvedType = fieldType;
                return fieldType;
            }

            const methodType = objectType.methods.get(propName);
            if (methodType) {
                node.resolvedType = methodType;
                return methodType;
            }
        }

        // If object is a pactum instance, check for methods
        if (objectType.kind === 'pactum' && !node.computed) {
            const propName = (node.property as Identifier).name;

            const methodType = objectType.methods.get(propName);
            if (methodType) {
                node.resolvedType = methodType;
                return methodType;
            }
        }

        // WHY: Handle user types by looking up the genus/pactum definition in scope.
        // This supports cross-module field access where the type is imported.
        if (objectType.kind === 'user' && !node.computed) {
            const symbol = lookupSymbol(currentScope, objectType.name);
            if (symbol && symbol.type.kind === 'genus') {
                const genusType = symbol.type;
                const propName = (node.property as Identifier).name;

                const fieldType = genusType.fields.get(propName);
                if (fieldType) {
                    node.resolvedType = fieldType;
                    return fieldType;
                }

                const methodType = genusType.methods.get(propName);
                if (methodType) {
                    node.resolvedType = methodType;
                    return methodType;
                }
            }

            // Handle pactum types imported from other modules
            if (symbol && symbol.type.kind === 'pactum') {
                const pactumType = symbol.type;
                const propName = (node.property as Identifier).name;

                const methodType = pactumType.methods.get(propName);
                if (methodType) {
                    node.resolvedType = methodType;
                    return methodType;
                }
            }
        }

        // Unknown property - return unknown for permissive behavior
        node.resolvedType = UNKNOWN;
        return UNKNOWN;
    }

    /**
     * Resolve lambda expression (pro ... redde or pro ... { }).
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

    function resolveAwait(node: CedeExpression): SemanticType {
        // Validate context: cede requires async or generator
        if (!currentFunctionAsync && !currentFunctionGenerator) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.CedeOutsideAsyncOrGenerator];
            error(`${text()}\n${help}`, node.position);
        }

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

    function resolveNew(node: NovumExpression): SemanticType {
        // Resolve constructor arguments
        for (const arg of node.arguments) {
            if (arg.type === 'SpreadElement') {
                resolveExpression(arg.argument);
            } else {
                resolveExpression(arg);
            }
        }

        const type = userType(node.callee.name);

        node.resolvedType = type;

        return type;
    }

    function resolveFingeExpression(node: FingeExpression): SemanticType {
        // Resolve field values if present
        if (node.fields) {
            resolveObjectExpression(node.fields);
        }

        // WHY: If explicit discretioType is provided via qua, look up the actual type
        //      from the symbol table to get the full discretio type (not just userType).
        //      This ensures type compatibility with function return types.
        if (node.discretioType) {
            const symbol = lookupSymbol(currentScope, node.discretioType.name);
            const type = symbol?.type ?? userType(node.discretioType.name);

            node.resolvedType = type;

            return type;
        }

        // TODO: Infer discretio type from context (variable declaration, return type, etc.)
        //       For now, mark as unknown - codegen will still work with explicit qua types
        node.resolvedType = UNKNOWN;

        return UNKNOWN;
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
            case 'ImportaDeclaration':
                analyzeImportaDeclaration(node);
                break;

            case 'VariaDeclaration':
                analyzeVariaDeclaration(node);
                break;

            case 'FunctioDeclaration':
                analyzeFunctioDeclaration(node);
                break;

            case 'TypeAliasDeclaration':
                analyzeTypeAliasDeclaration(node);
                break;

            case 'SiStatement':
                analyzeSiStatement(node);
                break;

            case 'DumStatement':
                analyzeDumStatement(node);
                break;

            case 'IteratioStatement':
                analyzeIteratioStatement(node);
                break;

            case 'InStatement':
                analyzeInStatement(node);
                break;

            case 'EligeStatement':
                analyzeEligeStatement(node);
                break;

            case 'DiscerneStatement':
                analyzeDiscerneStatement(node);
                break;

            case 'CustodiStatement':
                analyzeCustodiStatement(node);
                break;

            case 'AdfirmaStatement':
                analyzeAdfirmaStatement(node);
                break;

            case 'ReddeStatement':
                analyzeReddeStatement(node);
                break;

            case 'BlockStatement':
                analyzeBlock(node);
                break;

            case 'ExpressionStatement':
                resolveExpression(node.expression);
                break;

            case 'IaceStatement':
                analyzeIaceStatement(node);
                break;

            case 'ScribeStatement':
                analyzeScribeStatement(node);
                break;

            case 'TemptaStatement':
                analyzeTemptaStatement(node);
                break;

            case 'FacBlockStatement':
                analyzeFacBlockStatement(node);
                break;

            case 'OrdoDeclaration':
                analyzeOrdoDeclaration(node);
                break;

            case 'GenusDeclaration':
                analyzeGenusDeclaration(node);
                break;

            case 'PactumDeclaration':
                analyzePactumDeclaration(node);
                break;

            case 'DiscretioDeclaration':
                // TODO: Add proper semantic analysis for discretio
                // For now, just register the type name
                analyzeDiscretioDeclaration(node);
                break;

            case 'RumpeStatement':
            case 'PergeStatement':
                // No semantic analysis needed for break/continue
                break;

            case 'ProbandumStatement':
                // EDGE: Test statements are analyzed within test context
                analyzeProbandumStatement(node);
                break;

            case 'ProbaStatement':
                // Test body is analyzed in test scope
                analyzeProbaStatement(node);
                break;

            case 'PraeparaBlock':
                // Test setup/teardown block
                analyzePraeparaBlock(node);
                break;

            case 'CuraStatement':
                // Resource initialization statement
                analyzeCuraStatement(node);
                break;

            case 'DestructureDeclaration':
                // Object destructuring with ex-prefix syntax
                analyzeDestructureDeclaration(node);
                break;

            case 'IncipitStatement':
                analyzeIncipitStatement(node);
                break;

            case 'IncipietStatement':
                analyzeIncipietStatement(node);
                break;

            case 'AdStatement':
                analyzeAdStatement(node);
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

    function analyzeVariaDeclaration(node: VariaDeclaration): void {
        // Validate async binding context: figendum/variandum require async
        if ((node.kind === 'figendum' || node.kind === 'variandum') && !currentFunctionAsync) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.AwaitOutsideAsync];
            error(`${text(node.kind)}\n${help}`, node.position);
        }

        // Handle array destructuring pattern
        if (node.name.type === 'ArrayPattern') {
            if (node.init) {
                resolveExpression(node.init);
            }

            // Define each element as a variable (skip underscore elements)
            for (const elem of node.name.elements) {
                if (elem.skip) {
                    continue; // underscore means skip, no binding
                }

                define({
                    name: elem.name.name,
                    type: UNKNOWN,
                    kind: 'variable',
                    mutable: node.kind === 'varia' || node.kind === 'variandum',
                    position: elem.position,
                });
            }

            return;
        }

        // Standard variable declaration - resolve type
        let type: SemanticType;
        const isExterna = isExternaFromAnnotations(node.annotations);

        if (node.typeAnnotation) {
            type = resolveTypeAnnotation(node.typeAnnotation);
        } else if (node.init) {
            type = resolveExpression(node.init);
        } else {
            type = UNKNOWN;

            // External declarations require type annotation but no initializer
            // Non-externa without type or initializer is an error
            if (!isExterna) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.NoTypeOrInitializer];
                error(`${text(node.name.name)}\n${help}`, node.position);
            }
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

    function analyzeFunctioDeclaration(node: FunctioDeclaration): void {
        // Validate parameters
        let seenOptional = false;
        for (const param of node.params) {
            // Borrowed params (de/in) cannot have defaults
            if (param.defaultValue && (param.preposition === 'de' || param.preposition === 'in')) {
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.DefaultWithBorrowedParam];
                error(`${text(param.preposition)}\n${help}`, param.position);
            }

            // Required params cannot follow optional params
            if (param.optional || param.defaultValue) {
                seenOptional = true;
            } else if (seenOptional && !param.rest) {
                // Rest params (ceteri) can come after optional
                const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.RequiredAfterOptional];
                error(`${text(param.name.name)}\n${help}`, param.position);
            }
        }

        // Build function type from parameters and return type
        const paramTypes: SemanticType[] = node.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
        const returnType = node.returnType ? resolveTypeAnnotation(node.returnType) : VACUUM;

        // WHY: Detect curator param for allocator injection at call sites
        // Check both explicit curator param type and curata NAME syntax
        const hasCuratorParam = !!node.curatorName || node.params.some(p => p.typeAnnotation?.name.toLowerCase() === 'curator');

        const fnType = functionType(paramTypes, returnType, node.async, hasCuratorParam);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type: fnType,
                kind: 'function',
                mutable: false,
                position: node.position,
            });
        }

        // Analyze function body in new scope
        enterScope('function');

        // Save and set function context
        const previousReturnType = currentFunctionReturnType;
        const previousAsync = currentFunctionAsync;
        const previousGenerator = currentFunctionGenerator;

        currentFunctionReturnType = returnType;
        currentFunctionAsync = node.async || isAsyncFromAnnotations(node.annotations);
        currentFunctionGenerator = node.generator || isGeneratorFromAnnotations(node.annotations);

        // Define parameters (use alias if present for internal name)
        for (let i = 0; i < node.params.length; i++) {
            const param = node.params[i]!;
            const paramName = param.alias?.name ?? param.name.name;

            // Resolve default value expression if present
            if (param.defaultValue) {
                resolveExpression(param.defaultValue);
            }

            define({
                name: paramName,
                type: paramTypes[i]!,
                kind: 'parameter',
                mutable: false,
                position: param.position,
            });
        }

        // Functions must have body unless abstract or externa
        const isExterna = isExternaFromAnnotations(node.annotations);
        if (!node.body && !node.isAbstract && !isExterna) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.MissingFunctionBody];
            error(`${text(node.name.name)}\n${help}`, node.position);
        }

        // Analyze body if present
        if (node.body) {
            analyzeBlock(node.body);
        }

        // Restore function context
        currentFunctionReturnType = previousReturnType;
        currentFunctionAsync = previousAsync;
        currentFunctionGenerator = previousGenerator;
        exitScope();

        node.resolvedType = fnType;
    }

    function analyzeTypeAliasDeclaration(node: TypeAliasDeclaration): void {
        const type = resolveTypeAnnotation(node.typeAnnotation);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type,
                kind: 'type',
                mutable: false,
                position: node.position,
            });
        }

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    /**
     * Analyze enum (ordo) declaration.
     *
     * TRANSFORMS:
     *   ordo Status { pending, active, done }
     *   -> EnumType with members { pending: NUMERUS, active: NUMERUS, done: NUMERUS }
     */
    function analyzeOrdoDeclaration(node: OrdoDeclaration): void {
        const members = new Map<string, SemanticType>();

        for (const member of node.members) {
            // Determine member type from value or default to numerus
            let memberType: SemanticType = NUMERUS;

            if (member.value) {
                if (typeof member.value.value === 'string') {
                    memberType = TEXTUS;
                }
            }

            members.set(member.name.name, memberType);
        }

        const type = enumType(node.name.name, members);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type,
                kind: 'enum',
                mutable: false,
                position: node.position,
            });
        }

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    /**
     * Analyze genus (class/struct) declaration.
     *
     * WHY: Genus declarations create types with fields and methods.
     *      Static fields are tracked separately for member access resolution.
     */
    function analyzeGenusDeclaration(node: GenusDeclaration): void {
        const fields = new Map<string, SemanticType>();
        const methods = new Map<string, FunctionType>();
        const staticFields = new Map<string, SemanticType>();
        const staticMethods = new Map<string, FunctionType>();

        // Process fields
        for (const field of node.fields) {
            const fieldType = resolveTypeAnnotation(field.fieldType);

            if (field.isStatic) {
                staticFields.set(field.name.name, fieldType);
            } else {
                fields.set(field.name.name, fieldType);
            }
        }

        // Process methods (all instance methods for now - static methods not yet in parser)
        for (const method of node.methods) {
            const paramTypes = method.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
            const returnType = method.returnType ? resolveTypeAnnotation(method.returnType) : VACUUM;
            const fnType = functionType(paramTypes, returnType, method.async);

            methods.set(method.name.name, fnType);
        }

        const type = genusType(node.name.name, fields, methods, staticFields, staticMethods);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type,
                kind: 'genus',
                mutable: false,
                position: node.position,
            });
        }

        node.resolvedType = type;
        node.name.resolvedType = type;

        // WHY: Set genus context so 'ego' (this) resolves to the correct type
        const previousGenusType = currentGenusType;
        currentGenusType = type;

        // Analyze method bodies with genus context active
        for (const method of node.methods) {
            if (method.body) {
                analyzeMethodBody(method, type);
            }
        }

        // Restore previous genus context (for nested genus declarations)
        currentGenusType = previousGenusType;
    }

    /**
     * Analyze a method body within a genus context.
     *
     * WHY: Method bodies need access to 'ego' (this) and instance fields.
     *      This is similar to analyzeFunctioDeclaration but within a genus scope.
     */
    function analyzeMethodBody(method: FunctioDeclaration, genusType: GenusType): void {
        enterScope('function');

        // Save and set function context
        const previousReturnType = currentFunctionReturnType;
        const previousAsync = currentFunctionAsync;
        const previousGenerator = currentFunctionGenerator;

        const returnType = method.returnType ? resolveTypeAnnotation(method.returnType) : VACUUM;
        currentFunctionReturnType = returnType;
        currentFunctionAsync = method.async || isAsyncFromAnnotations(method.annotations);
        currentFunctionGenerator = method.generator || isGeneratorFromAnnotations(method.annotations);

        // Define parameters
        for (const param of method.params) {
            const paramName = param.alias?.name ?? param.name.name;
            const paramType = param.typeAnnotation ? resolveTypeAnnotation(param.typeAnnotation) : UNKNOWN;

            if (param.defaultValue) {
                resolveExpression(param.defaultValue);
            }

            define({
                name: paramName,
                type: paramType,
                kind: 'parameter',
                mutable: false,
                position: param.position,
            });
        }

        // Analyze body
        if (method.body) {
            analyzeBlock(method.body);
        }

        // Restore function context
        currentFunctionReturnType = previousReturnType;
        currentFunctionAsync = previousAsync;
        currentFunctionGenerator = previousGenerator;
        exitScope();
    }

    /**
     * Analyze pactum (interface) declaration.
     *
     * WHY: Pactum declarations define contracts that genus types can implement.
     */
    function analyzePactumDeclaration(node: PactumDeclaration): void {
        const methods = new Map<string, FunctionType>();

        for (const method of node.methods) {
            const paramTypes = method.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
            const returnType = method.returnType ? resolveTypeAnnotation(method.returnType) : VACUUM;
            const fnType = functionType(paramTypes, returnType, method.async);

            methods.set(method.name.name, fnType);
        }

        const type = pactumType(node.name.name, methods);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type,
                kind: 'pactum',
                mutable: false,
                position: node.position,
            });
        }

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    /**
     * Analyze discretio (tagged union) declaration.
     *
     * WHY: Discretio declarations define sum types with tagged variants.
     *      Each variant's fields are tracked to enable type inference
     *      for discerne pattern matching bindings.
     */
    function analyzeDiscretioDeclaration(node: DiscretioDeclaration): void {
        // Build variant info map with field types in declaration order
        const variants = new Map<string, VariantInfo>();

        for (const variant of node.variants) {
            const fields: { name: string; type: SemanticType }[] = [];

            for (const field of variant.fields) {
                const fieldType = resolveTypeAnnotation(field.fieldType);
                fields.push({ name: field.name.name, type: fieldType });
            }

            variants.set(variant.name.name, { fields });
        }

        const type = discretioType(node.name.name, variants);

        // WHY: Skip define if predeclared (two-pass analysis)
        if (!lookupSymbolLocal(currentScope, node.name.name)) {
            define({
                name: node.name.name,
                type,
                kind: 'type', // WHY: discretio is a type definition
                mutable: false,
                position: node.position,
            });
        } else {
            // Update the predeclared symbol with the full type
            updateSymbolType(currentScope, node.name.name, type);
        }

        node.resolvedType = type;
        node.name.resolvedType = type;
    }

    /**
     * Analyze destructure declaration (ex-prefix object destructuring).
     *
     * WHY: DestructureDeclaration extracts properties from an object into bindings.
     *      ex persona fixum nomen, aetas -> const { nomen, aetas } = persona
     */
    function analyzeDestructureDeclaration(node: DestructureDeclaration): void {
        // Resolve the source expression
        resolveExpression(node.source);

        // Define each specifier as a variable
        for (const specifier of node.specifiers) {
            const localName = specifier.local.name;

            define({
                name: localName,
                type: UNKNOWN, // TODO: Infer from source object type
                kind: 'variable',
                mutable: node.kind === 'varia' || node.kind === 'variandum',
                position: specifier.position,
            });
        }
    }

    function analyzeProbandumStatement(node: ProbandumStatement): void {
        // Test suite - analyze all test statements
        for (const stmt of node.body) {
            analyzeStatement(stmt);
        }
    }

    function analyzeProbaStatement(node: ProbaStatement): void {
        // Individual test case - analyze test body
        enterScope();
        analyzeBlock(node.body);
        exitScope();
    }

    function analyzePraeparaBlock(node: PraeparaBlock): void {
        // Test setup/teardown block
        enterScope();
        analyzeBlock(node.body);
        exitScope();
    }

    function analyzeCuraStatement(node: CuraStatement): void {
        // Resource binding statement - analyze resource and define binding
        // WHY: For allocator curator kinds (arena/page), resource is optional
        let resourceType: SemanticType;
        if (node.resource) {
            resourceType = resolveExpression(node.resource);
        } else {
            // Allocator binding - type is "allocator" (opaque for GC targets)
            resourceType = { kind: 'user', name: 'Allocator', nullable: false };
        }

        define({
            name: node.binding.name,
            type: resourceType,
            kind: 'variable',
            mutable: false,
            position: node.position,
        });

        enterScope();
        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeIncipitStatement(node: IncipitStatement): void {
        // WHY: Entry point contains either a body block or ergo-chained statement
        enterScope('function');
        if (node.body) {
            analyzeBlock(node.body);
        }
        if (node.ergoStatement) {
            analyzeStatement(node.ergoStatement);
        }
        exitScope();
    }

    function analyzeIncipietStatement(node: IncipietStatement): void {
        // WHY: Async entry point - same as incipit but marks async context
        enterScope('function');
        const previousAsync = currentFunctionAsync;
        currentFunctionAsync = true;

        if (node.body) {
            analyzeBlock(node.body);
        }
        if (node.ergoStatement) {
            analyzeStatement(node.ergoStatement);
        }

        currentFunctionAsync = previousAsync;
        exitScope();
    }

    function analyzeAdStatement(node: AdStatement): void {
        // WHY: Ad (goto-like label target) with optional binding and body
        for (const arg of node.arguments) {
            if (arg.type === 'SpreadElement') {
                resolveExpression(arg.argument);
            } else {
                resolveExpression(arg);
            }
        }

        if (node.binding) {
            enterScope();
            // Define the binding variable if present
            define({
                name: node.binding.name.name,
                type: UNKNOWN,
                kind: 'variable',
                mutable: false,
                position: node.binding.position,
            });
            if (node.body) {
                analyzeBlock(node.body);
            }
            exitScope();
        } else if (node.body) {
            enterScope();
            analyzeBlock(node.body);
            exitScope();
        }

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeSiStatement(node: SiStatement): void {
        const testType = resolveExpression(node.test);

        // TODO: Warn but don't error - truthy/falsy is valid
        if (testType.kind === 'primitive' && testType.name !== 'bivalens') {
            // Could warn here about non-boolean in condition
        }

        enterScope();
        analyzeBlock(node.consequent);
        exitScope();

        if (node.alternate) {
            if (node.alternate.type === 'SiStatement') {
                analyzeSiStatement(node.alternate);
            } else {
                enterScope();
                analyzeBlock(node.alternate);
                exitScope();
            }
        }

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeDumStatement(node: DumStatement): void {
        resolveExpression(node.test);

        enterScope();
        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeIteratioStatement(node: IteratioStatement): void {
        const iterableType = resolveExpression(node.iterable);

        enterScope();

        // WHY: Infer loop variable type from iterable element type.
        // This is required for norma stdlib lowering (e.g., elem.corpus.longitudo()).
        const inferredVarType = (() => {
            function elementTypeFromIterable(t: SemanticType): SemanticType {
                if (t.kind === 'union') {
                    return unionType(t.types.map(elementTypeFromIterable));
                }

                if (t.kind !== 'generic') {
                    return UNKNOWN;
                }

                // ex ... pro x  => iterate values
                if (node.kind === 'ex') {
                    switch (t.name) {
                        case 'lista':
                        case 'copia':
                        case 'cursor':
                        case 'fluxus':
                            return t.typeParameters[0] ?? UNKNOWN;
                        case 'tabula':
                            // ex tabula pro v => values (V)
                            return t.typeParameters[1] ?? UNKNOWN;
                        default:
                            return UNKNOWN;
                    }
                }

                // de ... pro k => iterate keys
                if (node.kind === 'in') {
                    if (t.name === 'tabula') {
                        return t.typeParameters[0] ?? UNKNOWN;
                    }
                }

                return UNKNOWN;
            }

            return elementTypeFromIterable(iterableType);
        })();

        define({
            name: node.variable.name,
            type: inferredVarType,
            kind: 'variable',
            mutable: false,
            position: node.variable.position,
        });
        node.variable.resolvedType = inferredVarType;

        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeInStatement(node: InStatement): void {
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

    function analyzeEligeStatement(node: EligeStatement): void {
        resolveExpression(node.discriminant);

        for (const caseNode of node.cases) {
            // Value matching: si expression { ... }
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
            analyzeCapeClause(node.catchClause);
        }
    }

    function analyzeDiscerneStatement(node: DiscerneStatement): void {
        // Resolve types for all discriminants
        const discriminantTypes: SemanticType[] = [];
        const discretios: (DiscretioType | null)[] = [];

        for (const discriminant of node.discriminants) {
            const discriminantType = resolveExpression(discriminant);
            discriminantTypes.push(discriminantType);

            // Try to get variant info if discriminant is a discretio type
            let discretio: DiscretioType | null = null;

            if (discriminantType.kind === 'discretio') {
                discretio = discriminantType;
            }
            else if (discriminantType.kind === 'user') {
                // Look up the type in scope — it might be a discretio
                const symbol = lookupSymbol(currentScope, discriminantType.name);

                if (symbol?.type.kind === 'discretio') {
                    discretio = symbol.type;
                }
            }

            discretios.push(discretio);
        }

        const numDiscriminants = node.discriminants.length;

        for (const caseNode of node.cases) {
            // Validate pattern count matches discriminant count
            if (caseNode.patterns.length !== numDiscriminants) {
                error(
                    `Pattern count mismatch: expected ${numDiscriminants} patterns, got ${caseNode.patterns.length}`,
                    caseNode.position
                );
            }

            // Each case introduces bindings into scope
            enterScope();

            // Process each pattern
            for (let i = 0; i < caseNode.patterns.length; i++) {
                const pattern = caseNode.patterns[i]!;
                const discriminantType = discriminantTypes[i] ?? UNKNOWN;
                const discretio = discretios[i];

                // Skip binding for wildcards
                if (pattern.isWildcard) continue;

                // Get variant field types if available
                const variantName = pattern.variant.name;
                const variantInfo = discretio?.variants.get(variantName);

                if (pattern.alias) {
                    // Alias binding: Click ut c
                    // WHY: Create a GenusType for the variant so field access works
                    // e.g., `casu MassaSententia ut m` binds m to a type with corpus field
                    let aliasType: SemanticType = discriminantType;

                    if (variantInfo) {
                        // Build fields map from variant info
                        const fields = new Map<string, SemanticType>();
                        for (const field of variantInfo.fields) {
                            fields.set(field.name, field.type);
                        }
                        // Create a GenusType for the variant
                        aliasType = genusType(
                            variantName,
                            fields,
                            new Map(), // methods
                            new Map(), // staticFields
                            new Map(), // staticMethods
                            false, // nullable
                        );
                    }

                    define({
                        name: pattern.alias.name,
                        type: aliasType,
                        kind: 'variable',
                        mutable: false,
                        position: pattern.position,
                    });

                    pattern.alias.resolvedType = aliasType;
                } else if (pattern.bindings.length > 0) {
                    // Positional bindings: Click pro x, y
                    for (let j = 0; j < pattern.bindings.length; j++) {
                        const binding = pattern.bindings[j]!;

                        // Infer type from variant field at same position
                        const fieldType = variantInfo?.fields[j]?.type ?? UNKNOWN;

                        define({
                            name: binding.name,
                            type: fieldType,
                            kind: 'variable',
                            mutable: false, // Pattern bindings are immutable
                            position: pattern.position,
                        });

                        // Also set the resolved type on the binding identifier
                        binding.resolvedType = fieldType;
                    }
                }
            }

            analyzeBlock(caseNode.consequent);
            exitScope();
        }
    }

    function analyzeCustodiStatement(node: CustodiStatement): void {
        for (const clause of node.clauses) {
            resolveExpression(clause.test);

            enterScope();
            analyzeBlock(clause.consequent);
            exitScope();
        }
    }

    function analyzeAdfirmaStatement(node: AdfirmaStatement): void {
        resolveExpression(node.test);

        if (node.message) {
            resolveExpression(node.message);
        }
    }

    function analyzeReddeStatement(node: ReddeStatement): void {
        if (!node.argument) {
            return;
        }

        const returnType = resolveExpression(node.argument);

        if (currentFunctionReturnType && !isAssignableTo(returnType, currentFunctionReturnType)) {
            const { text, help } = SEMANTIC_ERRORS[SemanticErrorCode.ReturnTypeMismatch];

            error(`${text(formatType(returnType), formatType(currentFunctionReturnType))}\n${help}`, node.position);
        }
    }

    function analyzeIaceStatement(node: IaceStatement): void {
        resolveExpression(node.argument);
    }

    function analyzeTemptaStatement(node: TemptaStatement): void {
        enterScope();
        analyzeBlock(node.block);
        exitScope();

        if (node.handler) {
            analyzeCapeClause(node.handler);
        }

        if (node.finalizer) {
            enterScope();
            analyzeBlock(node.finalizer);
            exitScope();
        }
    }

    function analyzeCapeClause(node: { param: Identifier; body: BlockStatement }): void {
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
     * Analyze fac block statement (explicit scope block or do-while loop).
     *
     * WHY: fac creates an explicit scope. With cape, it's like try-catch.
     *      With dum, it's a do-while loop (body executes first).
     */
    function analyzeFacBlockStatement(node: FacBlockStatement): void {
        enterScope();
        analyzeBlock(node.body);
        exitScope();

        if (node.catchClause) {
            analyzeCapeClause(node.catchClause);
        }

        if (node.test) {
            resolveExpression(node.test);
        }
    }

    function analyzeBlock(node: BlockStatement): void {
        for (const stmt of node.body) {
            analyzeStatement(stmt);
        }
    }

    // ---------------------------------------------------------------------------
    // Two-Pass Analysis: Predeclaration
    // ---------------------------------------------------------------------------

    /**
     * Predeclare a top-level statement by registering its name with a placeholder type.
     *
     * WHY: This enables forward references within a file. All names are visible
     *      before any bodies are analyzed.
     */
    function predeclareStatement(stmt: Statement): void {
        switch (stmt.type) {
            case 'FunctioDeclaration': {
                // WHY: Use placeholder types for params/return since referenced types
                // may not be defined yet. Real types are resolved in Phase 1b.
                const paramTypes: SemanticType[] = stmt.params.map(() => UNKNOWN);
                const returnType = stmt.returnType ? UNKNOWN : VACUUM;
                const isAsync = stmt.returnVerb === 'fiet' || stmt.returnVerb === 'fient';
                const fnType = functionType(paramTypes, returnType, isAsync);

                define({
                    name: stmt.name.name,
                    type: fnType,
                    kind: 'function',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            case 'GenusDeclaration': {
                // WHY: Create shell type - fields/methods resolved in Phase 1b
                const type = genusType(stmt.name.name, new Map(), new Map(), new Map(), new Map());

                define({
                    name: stmt.name.name,
                    type,
                    kind: 'genus',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            case 'PactumDeclaration': {
                // WHY: Create shell type - methods resolved in Phase 1b
                const type = pactumType(stmt.name.name, new Map());

                define({
                    name: stmt.name.name,
                    type,
                    kind: 'pactum',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            case 'OrdoDeclaration': {
                // WHY: Enum members don't reference other types, so we can fully resolve here
                const members = new Map<string, SemanticType>();

                for (const member of stmt.members) {
                    let memberType: SemanticType = NUMERUS;

                    if (member.value) {
                        if (typeof member.value.value === 'string') {
                            memberType = TEXTUS;
                        }
                    }

                    members.set(member.name.name, memberType);
                }

                const type = enumType(stmt.name.name, members);

                define({
                    name: stmt.name.name,
                    type,
                    kind: 'enum',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            case 'DiscretioDeclaration': {
                // WHY: Predeclare with empty variants — full resolution in analyzeDiscretioDeclaration
                const type = discretioType(stmt.name.name, new Map());

                define({
                    name: stmt.name.name,
                    type,
                    kind: 'type',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            case 'TypeAliasDeclaration': {
                // WHY: Type alias may reference forward types, use UNKNOWN placeholder
                define({
                    name: stmt.name.name,
                    type: UNKNOWN,
                    kind: 'type',
                    mutable: false,
                    position: stmt.position,
                });
                break;
            }

            // Other statement types don't need predeclaration
            default:
                break;
        }
    }

    // ---------------------------------------------------------------------------
    // Two-Pass Analysis: Signature Resolution
    // ---------------------------------------------------------------------------

    /**
     * Resolve signatures for predeclared symbols.
     *
     * WHY: Now that all names exist, we can resolve type annotations that
     *      reference forward-declared types. This updates the placeholder
     *      types to their real types.
     */
    function resolveSignature(stmt: Statement): void {
        switch (stmt.type) {
            case 'FunctioDeclaration': {
                const paramTypes: SemanticType[] = stmt.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
                const returnType = stmt.returnType ? resolveTypeAnnotation(stmt.returnType) : VACUUM;
                const hasCuratorParam = !!stmt.curatorName || stmt.params.some(p => p.typeAnnotation?.name.toLowerCase() === 'curator');
                const fnType = functionType(paramTypes, returnType, stmt.async, hasCuratorParam);

                updateSymbolType(currentScope, stmt.name.name, fnType);
                break;
            }

            case 'GenusDeclaration': {
                const fields = new Map<string, SemanticType>();
                const methods = new Map<string, FunctionType>();
                const staticFields = new Map<string, SemanticType>();
                const staticMethods = new Map<string, FunctionType>();

                for (const field of stmt.fields) {
                    const fieldType = resolveTypeAnnotation(field.fieldType);

                    if (field.isStatic) {
                        staticFields.set(field.name.name, fieldType);
                    } else {
                        fields.set(field.name.name, fieldType);
                    }
                }

                for (const method of stmt.methods) {
                    const paramTypes = method.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
                    const returnType = method.returnType ? resolveTypeAnnotation(method.returnType) : VACUUM;
                    const fnType = functionType(paramTypes, returnType, method.async);

                    methods.set(method.name.name, fnType);
                }

                const type = genusType(stmt.name.name, fields, methods, staticFields, staticMethods);

                updateSymbolType(currentScope, stmt.name.name, type);
                break;
            }

            case 'PactumDeclaration': {
                const methods = new Map<string, FunctionType>();

                for (const method of stmt.methods) {
                    const paramTypes = method.params.map(p => (p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN));
                    const returnType = method.returnType ? resolveTypeAnnotation(method.returnType) : VACUUM;
                    const fnType = functionType(paramTypes, returnType, method.async);

                    methods.set(method.name.name, fnType);
                }

                const type = pactumType(stmt.name.name, methods);

                updateSymbolType(currentScope, stmt.name.name, type);
                break;
            }

            case 'TypeAliasDeclaration': {
                // WHY: Track resolution to detect cycles (A -> B -> A)
                resolvingTypeAliases.add(stmt.name.name);
                const type = resolveTypeAnnotation(stmt.typeAnnotation);
                resolvingTypeAliases.delete(stmt.name.name);

                updateSymbolType(currentScope, stmt.name.name, type);
                break;
            }

            // OrdoDeclaration and DiscretioDeclaration are fully resolved in predeclare
            // Other statement types don't need signature resolution
            default:
                break;
        }
    }

    // ---------------------------------------------------------------------------
    // Main Analysis (Three-Phase)
    // ---------------------------------------------------------------------------

    // Phase 1a: Predeclare all top-level names with placeholder types
    // WHY: Enables forward references - all names are visible before body analysis
    for (const stmt of program.body) {
        predeclareStatement(stmt);
    }

    // Phase 1b: Resolve signatures with real types
    // WHY: Now that all names exist, type annotations can reference forward types
    for (const stmt of program.body) {
        resolveSignature(stmt);
    }

    // Phase 1c: Iteratively resolve type aliases until fixed point
    // WHY: Type alias chains like "A = B; B = C; C = numerus" need multiple passes
    //      when aliases are defined in forward-reference order
    let typeAliasProgress = true;

    while (typeAliasProgress) {
        typeAliasProgress = false;

        for (const stmt of program.body) {
            if (stmt.type === 'TypeAliasDeclaration') {
                const symbol = lookupSymbolLocal(currentScope, stmt.name.name);

                if (symbol && symbol.type.kind === 'unknown') {
                    // Try to resolve again
                    resolvingTypeAliases.add(stmt.name.name);
                    const type = resolveTypeAnnotation(stmt.typeAnnotation);
                    resolvingTypeAliases.delete(stmt.name.name);

                    if (type.kind !== 'unknown') {
                        updateSymbolType(currentScope, stmt.name.name, type);
                        typeAliasProgress = true;
                    }
                }
            }
        }
    }

    // Phase 1d: Detect type alias cycles (aliases that couldn't be resolved)
    // WHY: Circular aliases like "typus A = B; typus B = A" both remain UNKNOWN
    for (const stmt of program.body) {
        if (stmt.type === 'TypeAliasDeclaration') {
            const symbol = lookupSymbolLocal(currentScope, stmt.name.name);

            if (symbol && symbol.type.kind === 'unknown') {
                error(`Circular type alias: '${stmt.name.name}' cannot be resolved`, stmt.position);
            }
        }
    }

    // Phase 2: Analyze bodies with complete symbol table
    // WHY: Full type checking with all symbols and types available
    for (const stmt of program.body) {
        analyzeStatement(stmt);
    }

    return { program, errors };
}

// Re-export types
export * from './types';
export * from './scope';
export * from './errors';
