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
 * ERRORS: ModuleNotFound, ModuleParseError
 *
 * @module semantic/modules
 */

import { resolve, dirname, extname, relative } from 'node:path';
import { existsSync, readFileSync } from 'node:fs';
import { tokenize } from '../tokenizer';
import { parse } from '../parser';
import type { DiscretioDeclInfo } from './index';
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
import { functionType, UNKNOWN, VACUUM, userType, enumType, genusType, pactumType, genericType, primitiveType, discretioType, namespaceType } from './types';
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
    kind: 'function' | 'variable' | 'type' | 'genus' | 'pactum' | 'ordo' | 'discretio' | 'namespace';
}

/**
 * Exports extracted from a parsed module.
 */
export interface ModuleExports {
    /** Map of export name to export info */
    exports: Map<string, ModuleExport>;
    /** Discretio declarations by name (ordered variants) */
    discretios: Map<string, DiscretioDeclInfo>;
    /** The parsed program (for future multi-file codegen) */
    program: Program;
    /** Absolute path to the module file */
    filePath: string;
}

/**
 * Result of module resolution.
 */
export type ModuleResult = { ok: true; module: ModuleExports } | { ok: false; error: 'not_found' | 'parse_error'; message: string };

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
 * Context for resolving types within a module.
 * Contains all type definitions (genus, pactum, etc.) from the module.
 */
interface ModuleTypeContext {
    types: Map<string, SemanticType>;
}

/**
 * Resolve a type annotation, using in-module type definitions when available.
 *
 * WHY: When extracting exports, method return types may reference other types
 * defined in the same module. Without this, we'd get userType placeholders
 * that can't be resolved later for cross-module field access.
 */
function resolveTypeInModule(node: TypeAnnotation, ctx: ModuleTypeContext): SemanticType {
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
                    return resolveTypeInModule(tp, ctx);
                }
                return UNKNOWN;
            }) ?? [];
        return genericType(name, params, node.nullable);
    }

    // Check for in-module type definitions
    const moduleType = ctx.types.get(node.name);
    if (moduleType) {
        if (node.nullable && !moduleType.nullable) {
            return { ...moduleType, nullable: true };
        }
        return moduleType;
    }

    // User-defined type (not in this module)
    return userType(node.name, node.nullable);
}

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
 *
 * WHY: Two-pass extraction allows method return types to reference other
 * types defined in the same module, enabling proper cross-module type resolution.
 *
 * @param importedTypes - Types imported from other modules (for pactum method resolution)
 */
export function extractExports(program: Program, filePath: string, importedTypes: Map<string, SemanticType> = new Map()): ModuleExports {
    const exports = new Map<string, ModuleExport>();
    const discretios = new Map<string, DiscretioDeclInfo>();

    // Pass 1: Extract genus/ordo/discretio types first
    // These provide the type context for pactum method signatures
    // Start with imported types so we can reference them
    const typeCtx: ModuleTypeContext = { types: new Map(importedTypes) };
    for (const stmt of program.body) {
        if (stmt.type === 'GenusDeclaration') {
            // First pass: extract genus with simple resolution (field types as user types)
            const simpleCtx: ModuleTypeContext = { types: new Map() };
            const genusExport = extractGenusExport(stmt, simpleCtx);
            typeCtx.types.set(stmt.name.name, genusExport.type);
        } else if (stmt.type === 'OrdoDeclaration') {
            typeCtx.types.set(stmt.name.name, enumType(stmt.name.name, new Map()));
        } else if (stmt.type === 'DiscretioDeclaration') {
            // WHY: Extract discretio with full variant info from the start.
            // This ensures that genus fields referencing this discretio type
            // get the proper variant information for pattern matching.
            const discretioExports = extractDiscretioExports(stmt);
            // First export is always the union type
            typeCtx.types.set(stmt.name.name, discretioExports[0]!.type);
        }
    }

    // Pass 2: Re-extract genus types with full context for nested type resolution
    // This allows genus fields to reference other genus types defined in the same module
    for (const stmt of program.body) {
        if (stmt.type === 'GenusDeclaration') {
            const genusExport = extractGenusExport(stmt, typeCtx);
            typeCtx.types.set(stmt.name.name, genusExport.type);
        }
    }

    // Pass 3: Extract full exports using the complete type context
    for (const stmt of program.body) {
        if (stmt.type === 'DiscretioDeclaration') {
            discretios.set(stmt.name.name, {
                name: stmt.name.name,
                position: stmt.position,
                variants: stmt.variants.map(variant => ({
                    name: variant.name.name,
                    position: variant.position,
                })),
            });
        }
        const extracted = extractStatementExports(stmt, typeCtx);
        for (const exp of extracted) {
            exports.set(exp.name, exp);
        }
    }

    return { exports, discretios, program, filePath };
}

/**
 * Extract exports from a single statement, if it's exportable.
 * Returns array because discretio exports multiple types (union + variants).
 */
function extractStatementExports(stmt: Statement, ctx: ModuleTypeContext): ModuleExport[] {
    switch (stmt.type) {
        case 'FunctioDeclaration':
            return [extractFunctioExport(stmt, ctx)];
        case 'GenusDeclaration':
            return [extractGenusExport(stmt, ctx)];
        case 'PactumDeclaration':
            return [extractPactumExport(stmt, ctx)];
        case 'OrdoDeclaration':
            return [extractOrdoExport(stmt)];
        case 'DiscretioDeclaration':
            return extractDiscretioExports(stmt);
        case 'TypeAliasDeclaration':
            return [extractTypusExport(stmt)];
        case 'VariaDeclaration': {
            const exp = extractVariaExport(stmt);
            return exp ? [exp] : [];
        }
        default:
            return [];
    }
}

/**
 * Extract export from function declaration.
 */
function extractFunctioExport(stmt: FunctioDeclaration, ctx: ModuleTypeContext): ModuleExport {
    // WHY: Build function type from parameters and return type.
    // Parameter types use UNKNOWN since full resolution isn't needed for most cases.
    // Return types ARE resolved so that callers can access fields on the result.
    const paramTypes: SemanticType[] = stmt.params.map(() => UNKNOWN);

    const returnType = stmt.returnType ? resolveTypeInModule(stmt.returnType, ctx) : VACUUM;
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
function extractGenusExport(stmt: GenusDeclaration, ctx: ModuleTypeContext): ModuleExport {
    // WHY: Extract field types for cross-module field access resolution.
    // Without this, chained member expressions like `result.errors.longitudo()`
    // fail because the semantic analyzer can't resolve the field type.
    const fields = new Map<string, SemanticType>();

    for (const field of stmt.fields) {
        if (!field.isStatic) {
            fields.set(field.name.name, resolveTypeInModule(field.fieldType, ctx));
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
 * Without this, calling methods on pactum instances from other modules
 * fails because the return type can't be resolved.
 *
 * For HAL pactums with @subsidia, export as namespace instead of pactum.
 */
function extractPactumExport(stmt: PactumDeclaration, ctx: ModuleTypeContext): ModuleExport {
    // Check for @subsidia annotation (HAL interface)
    const subsidiaAnnotation = stmt.annotations?.find(a => a.name === 'subsidia');

    if (subsidiaAnnotation?.targetMappings) {
        // HAL pactum: export as namespace type
        return {
            name: stmt.name.name,
            type: namespaceType(stmt.name.name),
            kind: 'namespace',
        };
    }

    // Regular pactum: export as pactum type with methods
    const methods = new Map<string, FunctionType>();

    for (const method of stmt.methods) {
        const paramTypes = method.params.map(p => (p.typeAnnotation ? resolveTypeInModule(p.typeAnnotation, ctx) : UNKNOWN));
        const returnType = method.returnType ? resolveTypeInModule(method.returnType, ctx) : VACUUM;
        methods.set(method.name.name, functionType(paramTypes, returnType, method.async));
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
 * Extract exports from discretio (tagged union) declaration.
 *
 * WHY: Returns array because we export both the union type AND each variant type.
 * This allows code to import specific variants for use as parameter types:
 *   ex "./ast" importa Sententia, FunctioDeclaratio
 *   functio visit(FunctioDeclaratio f) -> vacuum { ... }
 */
function extractDiscretioExports(stmt: DiscretioDeclaration): ModuleExport[] {
    const exports: ModuleExport[] = [];

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

    const unionType = discretioType(stmt.name.name, variants);

    // Export the union type
    exports.push({
        name: stmt.name.name,
        type: unionType,
        kind: 'discretio',
    });

    // Export each variant as a separate type (Extract<Union, { tag: 'Variant' }>)
    // WHY: Allows importing specific variants for use as parameter types
    for (const variant of stmt.variants) {
        exports.push({
            name: variant.name.name,
            type: userType(variant.name.name), // Simplified - codegen produces Extract<>
            kind: 'type',
        });
    }

    return exports;
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
    // WHY: This is a TS-only bootstrap compiler whose job is to compile Rivus.
    // Circular imports are common in compiler codebases and are valid for JS/TS output.
    //
    // DESIGN: We detect cycles but do NOT treat them as semantic errors here.
    // Instead, return empty exports while the cycle is in progress.
    // This mirrors the JS module runtime model (values resolve when the module finishes loading).
    //
    // EDGE: This intentionally limits cross-module type information within the cycle.
    // Strict cycle diagnosis / SCC-aware resolution belongs in Rivus.

    if (ctx.inProgress.has(absolutePath)) {
        return {
            ok: true,
            module: {
                exports: new Map(),
                discretios: new Map(),
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
            const displayPath = relative(process.cwd(), absolutePath);
            const formatted = tokenErrors
                .map(e => `${displayPath}:${e.position.line}:${e.position.column} - ${e.text}`)
                .join('\n');
            return {
                ok: false,
                error: 'parse_error',
                message: formatted,
            };
        }

        const { program, errors: parseErrors } = parse(tokens);

        if (parseErrors.length > 0 || !program) {
            const displayPath = relative(process.cwd(), absolutePath);
            const formatted = parseErrors
                .map(e => `${displayPath}:${e.position.line}:${e.position.column} - ${e.message}`)
                .join('\n');
            return {
                ok: false,
                error: 'parse_error',
                message: formatted,
            };
        }

        // WHY: First resolve all imports to build full type context.
        // This allows pactum methods to reference types imported from other modules.
        const importedTypes = new Map<string, SemanticType>();
        for (const stmt of program.body) {
            if (stmt.type === 'ImportaDeclaration' && isLocalImport(stmt.source)) {
                const childCtx: ModuleContext = {
                    basePath: absolutePath,
                    cache: ctx.cache,
                    inProgress: ctx.inProgress,
                };
                const childResult = resolveModule(stmt.source, childCtx);
                if (!childResult.ok) {
                    // Propagate the error (not_found/parse_error)
                    return childResult;
                }
                // Add imported exports to type context
                for (const spec of stmt.specifiers) {
                    const exportInfo = childResult.module.exports.get(spec.imported.name);
                    if (
                        exportInfo &&
                        (exportInfo.kind === 'genus' || exportInfo.kind === 'pactum' || exportInfo.kind === 'ordo' || exportInfo.kind === 'discretio')
                    ) {
                        importedTypes.set(spec.local.name, exportInfo.type);
                    }
                }
            }
        }

        // Extract exports with imported types as context
        const moduleExports = extractExports(program, absolutePath, importedTypes);

        // Cache the result
        ctx.cache.set(absolutePath, moduleExports);

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
