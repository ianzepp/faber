/**
 * CLI Module Resolver - Load and parse modules for CLI extraction
 *
 * ARCHITECTURE
 * ============
 * This is a standalone module loader for CLI detection. It:
 * - Reuses the parser (not semantic analyzer)
 * - Follows imports to find mounted modules
 * - Extracts only CLI-relevant info (annotations, function signatures)
 * - No type checking, no symbol tables
 *
 * WHY: CLI detection is a codegen concern, not a semantic concern.
 *      Keeping it isolated prevents polluting the main compiler pipeline.
 *
 * @module codegen/cli/resolver
 */

import { resolve, dirname, extname } from 'node:path';
import { existsSync, readFileSync } from 'node:fs';
import { tokenize } from '../../tokenizer';
import { parse } from '../../parser';
import type {
    Program,
    ImportaDeclaration,
    IncipitStatement,
    FunctioDeclaration,
    Annotation,
    Parameter,
} from '../../parser/ast';

// =============================================================================
// TYPES
// =============================================================================

/** Extracted function info for CLI */
export interface CliFunctionInfo {
    name: string;
    annotations: Annotation[];
    params: CliParamInfo[];
}

/** Extracted parameter info for CLI */
export interface CliParamInfo {
    name: string;
    type: string;
    optional: boolean;
    shortFlag?: string;
    defaultValue?: string;
}

/** Extracted incipit info for CLI */
export interface CliIncipitInfo {
    annotations: Annotation[];
}

/** Import mapping: local alias -> module path */
export interface CliImportInfo {
    localName: string;
    sourcePath: string;
    isWildcard: boolean;
}

/** CLI-relevant info extracted from a module */
export interface CliModuleInfo {
    filePath: string;
    imports: CliImportInfo[];
    incipit?: CliIncipitInfo;
    functions: CliFunctionInfo[];
}

/** Context for CLI module resolution */
export interface CliResolverContext {
    /** Absolute path of the file doing the importing */
    basePath: string;
    /** Cache of already-parsed modules */
    cache: Map<string, CliModuleInfo>;
    /** Files being processed (cycle detection) */
    inProgress: Set<string>;
}

// =============================================================================
// PATH RESOLUTION
// =============================================================================

/**
 * Check if an import source is a local file import.
 */
export function isLocalImport(source: string): boolean {
    return source.startsWith('./') || source.startsWith('../');
}

/**
 * Resolve import source to absolute file path.
 * Returns null if file doesn't exist.
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
// EXTRACTION
// =============================================================================

/**
 * Extract parameter info from a function parameter.
 */
function extractParamInfo(param: Parameter): CliParamInfo {
    return {
        name: param.name.name,
        type: param.typeAnnotation?.name ?? 'textus',
        optional: param.optional === true,
        shortFlag: param.alias?.name,
        defaultValue: param.defaultValue?.type === 'Literal'
            ? String(param.defaultValue.value)
            : undefined,
    };
}

/**
 * Extract CLI-relevant info from a parsed program.
 */
function extractCliInfo(program: Program, filePath: string): CliModuleInfo {
    const imports: CliImportInfo[] = [];
    const functions: CliFunctionInfo[] = [];
    let incipit: CliIncipitInfo | undefined;

    for (const stmt of program.body) {
        switch (stmt.type) {
            case 'ImportaDeclaration': {
                const imp = stmt as ImportaDeclaration;
                const source = typeof imp.source === 'string' ? imp.source : imp.source.name;

                if (isLocalImport(source)) {
                    if (imp.wildcard && imp.wildcardAlias) {
                        // ex "./module" importa * ut alias
                        imports.push({
                            localName: imp.wildcardAlias.name,
                            sourcePath: source,
                            isWildcard: true,
                        });
                    }
                    else if (imp.specifiers) {
                        // ex "./module" importa Name, Other
                        for (const spec of imp.specifiers) {
                            imports.push({
                                localName: spec.local.name,
                                sourcePath: source,
                                isWildcard: false,
                            });
                        }
                    }
                }
                break;
            }

            case 'IncipitStatement': {
                const inc = stmt as IncipitStatement;
                if (inc.annotations && inc.annotations.length > 0) {
                    incipit = { annotations: inc.annotations };
                }
                break;
            }

            case 'FunctioDeclaration': {
                const fn = stmt as FunctioDeclaration;
                functions.push({
                    name: fn.name.name,
                    annotations: fn.annotations ?? [],
                    params: fn.params.map(extractParamInfo),
                });
                break;
            }
        }
    }

    return { filePath, imports, incipit, functions };
}

// =============================================================================
// MODULE LOADING
// =============================================================================

/**
 * Load and parse a module, extracting CLI-relevant info.
 * Returns null on parse error or file not found.
 */
export function loadCliModule(
    filePath: string,
    ctx: CliResolverContext
): CliModuleInfo | null {
    // Check cache
    if (ctx.cache.has(filePath)) {
        return ctx.cache.get(filePath)!;
    }

    // Cycle detection
    if (ctx.inProgress.has(filePath)) {
        // Return empty info for cycles (mirrors JS module behavior)
        return { filePath, imports: [], functions: [] };
    }

    // Check file exists
    if (!existsSync(filePath)) {
        return null;
    }

    ctx.inProgress.add(filePath);

    try {
        // Read and parse
        const source = readFileSync(filePath, 'utf-8');
        const tokenResult = tokenize(source);
        if (tokenResult.errors.length > 0) {
            return null;
        }

        const parseResult = parse(tokenResult.tokens);
        if (!parseResult.program) {
            return null;
        }

        // Extract CLI info
        const moduleInfo = extractCliInfo(parseResult.program, filePath);

        // Cache and return
        ctx.cache.set(filePath, moduleInfo);
        ctx.inProgress.delete(filePath);

        return moduleInfo;
    }
    catch {
        ctx.inProgress.delete(filePath);
        return null;
    }
}

/**
 * Create a new resolver context.
 */
export function createResolverContext(basePath: string): CliResolverContext {
    return {
        basePath,
        cache: new Map(),
        inProgress: new Set(),
    };
}

/**
 * Resolve a local import alias to module info.
 * Returns null if the module can't be found or parsed.
 */
export function resolveImportedModule(
    localName: string,
    moduleInfo: CliModuleInfo,
    ctx: CliResolverContext
): CliModuleInfo | null {
    // Find the import that matches this local name
    const imp = moduleInfo.imports.find(i => i.localName === localName);
    if (!imp) {
        return null;
    }

    // Resolve the path relative to the module's location
    const resolvedPath = resolveModulePath(imp.sourcePath, moduleInfo.filePath);
    if (!resolvedPath) {
        return null;
    }

    // Load the module
    return loadCliModule(resolvedPath, ctx);
}
