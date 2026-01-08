/**
 * Module Resolution - Local File Import Resolution
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module handles resolution of local .fab file imports during semantic analysis.
 * It provides:
 *
 * - Path resolution (relative to importing file)
 * - File loading and parsing
 * - Export extraction from parsed modules
 * - Cycle detection during recursive imports
 * - Caching of already-analyzed modules
 *
 * Resolution rules:
 * - Paths starting with "./" or "../" are local file imports
 * - ".fab" extension is implicit and added automatically
 * - "norma" and "norma/*" are handled as compiler intrinsics (not files)
 * - Other paths pass through to target language (external packages)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Import source string, base file path, module cache
 * OUTPUT: ModuleExports with symbols and types, or error
 * ERRORS: ModuleNotFound, CircularImport, ModuleParseError
 *
 * @module semantic/modules
 */

import { resolve, dirname, extname } from 'node:path';
import { existsSync, readFileSync } from 'node:fs';
import { tokenize } from '../tokenizer';
import { parse } from '../parser';
import type {
    Program,
    Statement,
    FunctioDeclaration,
    GenusDeclaration,
    PactumDeclaration,
    OrdoDeclaration,
    DiscretioDeclaration,
    TypeAliasDeclaration,
    VariaDeclaration,
} from '../parser/ast';
import type { SemanticType, FunctionType, PrimitiveType, VariantInfo } from './types';
import { functionType, UNKNOWN, VACUUM, userType, enumType, genusType, pactumType, genericType, primitiveType, discretioType } from './types';
import type { TypeAnnotation } from '../parser/ast';

// =============================================================================
// TYPE RESOLUTION (SIMPLIFIED FOR MODULE EXPORTS)
// =============================================================================

/**
 * Latin primitive type names for module-level type resolution.
 */
const LATIN_PRIMITIVES: Record<string, PrimitiveType['name']> = {
    textus: 'textus',
    numerus: 'numerus',
    fractus: 'fractus',
    decimus: 'decimus',
    magnus: 'magnus',
    bivalens: 'bivalens',
    nihil: 'nihil',
    vacuum: 'vacuum',
    octeti: 'octeti',
};

/**
 * Generic collection type names.
 */
const GENERIC_TYPES = new Set(['lista', 'tabula', 'copia', 'promissum', 'cursor', 'fluxus']);

/**
 * Resolve a TypeAnnotation to a SemanticType (simplified for module exports).
 *
 * WHY: Module exports need type info for cross-file field access resolution.
 *      This is a simplified resolver that handles common cases without full
 *      semantic analysis infrastructure.
 */
function resolveTypeSimple(node: TypeAnnotation): SemanticType {
    const name = node.name.toLowerCase();

    // Check for primitives
    if (name in LATIN_PRIMITIVES) {
        return primitiveType(LATIN_PRIMITIVES[name]!, node.nullable);
    }

    // Check for generics (lista<T>, tabula<K,V>, etc.)
    if (GENERIC_TYPES.has(name)) {
        const params =
            node.typeParameters?.map(tp => {
                if (tp.type === 'TypeAnnotation') {
                    return resolveTypeSimple(tp);
                }
                return UNKNOWN;
            }) ?? [];
        return genericType(name, params, node.nullable);
    }

    // User-defined type (genus, pactum, etc.)
    return userType(node.name, node.nullable);
}

// =============================================================================
// TYPES
// =============================================================================

/**
 * Symbol exported from a module.
 */
export interface ModuleExport {
    name: string;
    type: SemanticType;
    kind: 'function' | 'variable' | 'type' | 'genus' | 'pactum' | 'ordo' | 'discretio';
}

/**
 * Exports extracted from a parsed module.
 */
export interface ModuleExports {
    /** Map of export name to export info */
    exports: Map<string, ModuleExport>;
    /** The parsed program (for future multi-file codegen) */
    program: Program;
    /** Absolute path to the module file */
    filePath: string;
}

/**
 * Result of module resolution.
 */
export type ModuleResult = { ok: true; module: ModuleExports } | { ok: false; error: 'not_found' | 'cycle' | 'parse_error'; message: string };

/**
 * Context for module resolution, passed through recursive imports.
 */
export interface ModuleContext {
    /** Absolute path of the file doing the importing */
    basePath: string;
    /** Cache of already-analyzed modules (path -> exports) */
    cache: Map<string, ModuleExports>;
    /** Set of files currently being analyzed (for cycle detection) */
    inProgress: Set<string>;
}

// =============================================================================
// PATH RESOLUTION
// =============================================================================

/**
 * Check if an import source is a local file import.
 *
 * WHY: Local imports start with "./" or "../" to distinguish from
 *      stdlib (norma/*) and external packages.
 */
export function isLocalImport(source: string): boolean {
    return source.startsWith('./') || source.startsWith('../');
}

/**
 * Resolve import source to absolute file path.
 *
 * Resolution rules:
 * 1. Relative paths resolved from basePath's directory
 * 2. ".fab" extension added if not present
 * 3. Returns null if file doesn't exist
 */
export function resolveModulePath(source: string, basePath: string): string | null {
    const baseDir = dirname(basePath);

    // Add .fab extension if not present
    let targetPath = source;
    if (extname(source) !== '.fab') {
        targetPath = source + '.fab';
    }

    // Resolve to absolute path
    const absolutePath = resolve(baseDir, targetPath);

    // Check if file exists
    if (!existsSync(absolutePath)) {
        return null;
    }

    return absolutePath;
}

// =============================================================================
// EXPORT EXTRACTION
// =============================================================================

/**
 * Extract exports from a parsed program.
 *
 * All top-level declarations become exports:
 * - functio declarations
 * - genus declarations
 * - pactum declarations
 * - ordo declarations
 * - discretio declarations
 * - typus aliases
 * - fixum/varia declarations (module constants/variables)
 */
export function extractExports(program: Program, filePath: string): ModuleExports {
    const exports = new Map<string, ModuleExport>();

    for (const stmt of program.body) {
        const extracted = extractStatementExport(stmt);
        if (extracted) {
            exports.set(extracted.name, extracted);
        }
    }

    return { exports, program, filePath };
}

/**
 * Extract export from a single statement, if it's exportable.
 */
function extractStatementExport(stmt: Statement): ModuleExport | null {
    switch (stmt.type) {
        case 'FunctioDeclaration':
            return extractFunctioExport(stmt);
        case 'GenusDeclaration':
            return extractGenusExport(stmt);
        case 'PactumDeclaration':
            return extractPactumExport(stmt);
        case 'OrdoDeclaration':
            return extractOrdoExport(stmt);
        case 'DiscretioDeclaration':
            return extractDiscretioExport(stmt);
        case 'TypeAliasDeclaration':
            return extractTypusExport(stmt);
        case 'VariaDeclaration':
            return extractVariaExport(stmt);
        default:
            return null;
    }
}

/**
 * Extract export from function declaration.
 */
function extractFunctioExport(stmt: FunctioDeclaration): ModuleExport {
    // WHY: Build function type from parameters and return type.
    // Parameter types use UNKNOWN since full resolution isn't needed for most cases.
    // Return types ARE resolved so that callers can access fields on the result.
    const paramTypes: SemanticType[] = stmt.params.map(() => UNKNOWN);

    const returnType = stmt.returnType ? resolveTypeSimple(stmt.returnType) : VACUUM;
    const isAsync = stmt.returnVerb === 'fiet' || stmt.returnVerb === 'fient';

    return {
        name: stmt.name.name,
        type: functionType(paramTypes, returnType, isAsync),
        kind: 'function',
    };
}

/**
 * Extract export from genus (class) declaration.
 */
function extractGenusExport(stmt: GenusDeclaration): ModuleExport {
    // WHY: Extract field types for cross-module field access resolution.
    // Without this, chained member expressions like `result.errors.longitudo()`
    // fail because the semantic analyzer can't resolve the field type.
    const fields = new Map<string, SemanticType>();

    for (const field of stmt.fields) {
        if (!field.isStatic) {
            fields.set(field.name.name, resolveTypeSimple(field.fieldType));
        }
    }

    return {
        name: stmt.name.name,
        type: genusType(stmt.name.name, fields, new Map(), new Map(), new Map()),
        kind: 'genus',
    };
}

/**
 * Extract export from pactum (interface) declaration.
 *
 * WHY: Extract method signatures for cross-module method call resolution.
 * Without this, accessing methods through imported pactum types fails because
 * the semantic analyzer can't resolve the method return types.
 */
function extractPactumExport(stmt: PactumDeclaration): ModuleExport {
    const methods = new Map<string, FunctionType>();

    for (const method of stmt.methods) {
        // Parameter types use UNKNOWN since full resolution isn't needed for most cases
        const paramTypes: SemanticType[] = method.params.map(() => UNKNOWN);
        const returnType = method.returnType ? resolveTypeSimple(method.returnType) : VACUUM;
        const fnType = functionType(paramTypes, returnType, method.async);
        methods.set(method.name.name, fnType);
    }

    return {
        name: stmt.name.name,
        type: pactumType(stmt.name.name, methods),
        kind: 'pactum',
    };
}

/**
 * Extract export from ordo (enum) declaration.
 */
function extractOrdoExport(stmt: OrdoDeclaration): ModuleExport {
    return {
        name: stmt.name.name,
        type: enumType(stmt.name.name, new Map()),
        kind: 'ordo',
    };
}

/**
 * Extract export from discretio (tagged union) declaration.
 */
function extractDiscretioExport(stmt: DiscretioDeclaration): ModuleExport {
    // WHY: Preserve variant field information for cross-module pattern matching.
    // Without this, variant aliases in `discerne` cannot be typed when the
    // discretio is imported from another file.
    const variants = new Map<string, VariantInfo>();

    for (const variant of stmt.variants) {
        variants.set(variant.name.name, {
            fields: variant.fields.map(f => ({
                name: f.name.name,
                type: resolveTypeSimple(f.fieldType),
            })),
        });
    }

    return {
        name: stmt.name.name,
        type: discretioType(stmt.name.name, variants),
        kind: 'discretio',
    };
}

/**
 * Extract export from typus (type alias) declaration.
 */
function extractTypusExport(stmt: TypeAliasDeclaration): ModuleExport {
    return {
        name: stmt.name.name,
        type: userType(stmt.name.name),
        kind: 'type',
    };
}

/**
 * Extract export from variable declaration.
 */
function extractVariaExport(stmt: VariaDeclaration): ModuleExport | null {
    // WHY: Only named identifiers are exportable, not array patterns
    if (stmt.name.type !== 'Identifier') {
        return null;
    }

    return {
        name: stmt.name.name,
        type: UNKNOWN, // Type will be resolved during semantic analysis
        kind: 'variable',
    };
}

// =============================================================================
// MODULE RESOLUTION
// =============================================================================

/**
 * Resolve and load a local module.
 *
 * This is the main entry point for local file import resolution.
 * It handles:
 * 1. Path resolution
 * 2. Cycle detection
 * 3. Cache lookup
 * 4. File loading and parsing
 * 5. Export extraction
 * 6. Recursive import resolution (for cycle detection)
 */
export function resolveModule(source: string, ctx: ModuleContext): ModuleResult {
    // Resolve to absolute path
    const absolutePath = resolveModulePath(source, ctx.basePath);

    if (!absolutePath) {
        return {
            ok: false,
            error: 'not_found',
            message: `Cannot find module '${source}' (resolved from ${ctx.basePath})`,
        };
    }

    // Check cache first
    const cached = ctx.cache.get(absolutePath);
    if (cached) {
        return { ok: true, module: cached };
    }

    // Check for cycles
    // WHY: JS/TS handle circular imports at runtime via import hoisting.
    // Instead of erroring, return empty exports - values resolve when module finishes loading.
    // This enables patterns like: index.fab exports genExpressia, sibling files import it,
    // index.fab imports siblings for dispatch. All valid in JS/TS.
    if (ctx.inProgress.has(absolutePath)) {
        return {
            ok: true,
            module: {
                exports: new Map(),
                program: null as unknown as Program,
                filePath: absolutePath,
            },
        };
    }

    // Mark as in-progress
    ctx.inProgress.add(absolutePath);

    try {
        // Load and parse the module
        const sourceCode = readFileSync(absolutePath, 'utf-8');
        const { tokens, errors: tokenErrors } = tokenize(sourceCode);

        if (tokenErrors.length > 0) {
            return {
                ok: false,
                error: 'parse_error',
                message: `Lexer errors in '${source}': ${tokenErrors.map(e => e.text).join(', ')}`,
            };
        }

        const { program, errors: parseErrors } = parse(tokens);

        if (parseErrors.length > 0 || !program) {
            return {
                ok: false,
                error: 'parse_error',
                message: `Parse errors in '${source}': ${parseErrors.map(e => e.message).join(', ')}`,
            };
        }

        // Extract exports
        const moduleExports = extractExports(program, absolutePath);

        // Cache the result before recursing to prevent re-parsing on diamond dependencies
        ctx.cache.set(absolutePath, moduleExports);

        // WHY: Recursively resolve imports from this module to detect cycles
        // We need a new context with the imported file as the base path
        for (const stmt of program.body) {
            if (stmt.type === 'ImportaDeclaration' && isLocalImport(stmt.source)) {
                const childCtx: ModuleContext = {
                    basePath: absolutePath,
                    cache: ctx.cache,
                    inProgress: ctx.inProgress,
                };
                const childResult = resolveModule(stmt.source, childCtx);
                if (!childResult.ok) {
                    // Propagate the error (especially cycles)
                    return childResult;
                }
            }
        }

        return { ok: true, module: moduleExports };
    } finally {
        // Remove from in-progress (even on error)
        ctx.inProgress.delete(absolutePath);
    }
}

/**
 * Create a new module context for compilation starting from a file.
 */
export function createModuleContext(entryPath: string): ModuleContext {
    return {
        basePath: resolve(entryPath),
        cache: new Map(),
        inProgress: new Set(),
    };
}
