/**
 * CLI Command Tree Detector - Build command tree from annotations
 *
 * ARCHITECTURE
 * ============
 * This module scans a program's AST and any mounted modules to build
 * a complete CLI command tree. It handles:
 *
 * - @ cli on incipit -> marks file as CLI program
 * - @ versio, @ descriptio on incipit -> program metadata
 * - @ imperium on functions -> leaf commands
 * - @ imperia on incipit -> mounted submodules (recursive)
 * - @ alias on functions -> command aliases
 *
 * @module codegen/cli/detector
 */

import type { Program, Annotation } from '../../parser/ast';
import type { CliProgram, CliCommandNode, CliParam } from '../ts/generator';
import {
    type CliModuleInfo,
    type CliResolverContext,
    type CliFunctionInfo,
    createResolverContext,
    loadCliModule,
    resolveImportedModule,
} from './resolver';

// =============================================================================
// ANNOTATION HELPERS
// =============================================================================

/**
 * Extract string value from annotation argument.
 */
function getAnnotationString(ann: Annotation): string | undefined {
    if (ann.argument?.type === 'Literal' && typeof ann.argument.value === 'string') {
        return ann.argument.value;
    }
    return undefined;
}

/**
 * Find annotation by name in annotation list.
 */
function findAnnotation(annotations: Annotation[] | undefined, name: string): Annotation | undefined {
    return annotations?.find(a => a.name === name);
}

// =============================================================================
// COMMAND TREE BUILDING
// =============================================================================

/**
 * Create a new command tree node.
 */
function createCommandNode(name: string, fullPath: string): CliCommandNode {
    return { name, fullPath, children: new Map() };
}

/**
 * Insert a command into the tree at the given path.
 * Creates intermediate nodes as needed.
 * Returns false if a duplicate command exists at that path.
 */
function insertCommand(
    root: CliCommandNode,
    path: string[],
    fullPath: string,
    functionName: string,
    params: CliParam[],
    alias?: string,
    modulePrefix?: string
): boolean {
    let current = root;

    // Navigate/create intermediate nodes
    for (let i = 0; i < path.length - 1; i++) {
        const segment = path[i]!;
        if (!current.children.has(segment)) {
            const intermediatePath = path.slice(0, i + 1).join('/');
            current.children.set(segment, createCommandNode(segment, intermediatePath));
        }
        current = current.children.get(segment)!;
    }

    // Create/update leaf node
    const leafName = path[path.length - 1]!;
    if (current.children.has(leafName)) {
        const existing = current.children.get(leafName)!;
        // Conflict: command already has a handler
        if (existing.functionName) {
            return false;
        }
    }
    else {
        current.children.set(leafName, createCommandNode(leafName, fullPath));
    }

    const leaf = current.children.get(leafName)!;
    leaf.functionName = functionName;
    leaf.params = params;
    leaf.alias = alias;
    leaf.modulePrefix = modulePrefix;
    return true;
}

/**
 * Get or create a subtree node at the given path.
 * Used for mounting submodules.
 */
function getOrCreateSubtree(root: CliCommandNode, path: string[]): CliCommandNode {
    let current = root;

    for (let i = 0; i < path.length; i++) {
        const segment = path[i]!;
        if (!current.children.has(segment)) {
            const intermediatePath = path.slice(0, i + 1).join('/');
            current.children.set(segment, createCommandNode(segment, intermediatePath));
        }
        current = current.children.get(segment)!;
    }

    return current;
}

// =============================================================================
// FUNCTION EXTRACTION
// =============================================================================

/**
 * Extract commands from functions with @ imperium annotation.
 * @param modulePrefix - For imported modules, the namespace to qualify function calls
 */
function extractCommands(
    functions: CliFunctionInfo[],
    root: CliCommandNode,
    pathPrefix: string[],
    errors: string[],
    modulePrefix?: string
): void {
    for (const fn of functions) {
        const imperiumAnn = findAnnotation(fn.annotations, 'imperium');
        if (!imperiumAnn) continue;

        const aliasAnn = findAnnotation(fn.annotations, 'alias');
        const commandPath = getAnnotationString(imperiumAnn) ?? fn.name;
        const pathParts = [...pathPrefix, ...commandPath.split('/')];
        const fullPath = pathParts.join('/');

        const params: CliParam[] = fn.params.map(p => ({
            name: p.name,
            type: p.type,
            optional: p.optional,
            shortFlag: p.shortFlag,
            defaultValue: p.defaultValue,
        }));

        const success = insertCommand(
            root,
            pathParts,
            fullPath,
            fn.name,
            params,
            aliasAnn ? getAnnotationString(aliasAnn) : undefined,
            modulePrefix
        );

        if (!success) {
            errors.push(`Duplicate command path: ${fullPath}`);
        }
    }
}

// =============================================================================
// MODULE MOUNTING (RECURSIVE)
// =============================================================================

/**
 * Track module imports needed for CLI dispatcher.
 * Maps module alias -> relative import path
 */
export type CliModuleImports = Map<string, string>;

/**
 * Generate a unique module alias for a file path.
 * Converts path to camelCase identifier.
 */
function generateModuleAlias(filePath: string, basePath: string): string {
    // Get relative path from base
    const { relative } = require('node:path');
    const relPath = relative(require('node:path').dirname(basePath), filePath);

    // Convert to camelCase identifier: ./commands/config/server.fab -> commandsConfigServer
    return relPath
        .replace(/\.fab$/, '')
        .replace(/^\.\//, '')
        .split('/')
        .map((part: string, i: number) =>
            i === 0 ? part : part.charAt(0).toUpperCase() + part.slice(1)
        )
        .join('')
        .replace(/[^a-zA-Z0-9]/g, '');
}

/**
 * Process @ imperia annotations and mount submodules.
 */
function mountSubmodules(
    moduleInfo: CliModuleInfo,
    root: CliCommandNode,
    pathPrefix: string[],
    ctx: CliResolverContext,
    errors: string[],
    visited: Set<string>,
    moduleImports: CliModuleImports,
    basePath: string
): void {
    if (!moduleInfo.incipit) return;

    // Find all @ imperia annotations on incipit
    for (const ann of moduleInfo.incipit.annotations) {
        if (ann.name !== 'imperia') continue;
        if (!ann.exClause) {
            errors.push(`@ imperia requires 'ex <module>' clause`);
            continue;
        }

        const mountPath = getAnnotationString(ann);
        if (!mountPath) {
            errors.push(`@ imperia requires a path argument`);
            continue;
        }

        const moduleName = ann.exClause.name;

        // Resolve the imported module
        const submodule = resolveImportedModule(moduleName, moduleInfo, ctx);
        if (!submodule) {
            errors.push(`Cannot resolve module '${moduleName}' for @ imperia "${mountPath}"`);
            continue;
        }

        // Cycle detection
        if (visited.has(submodule.filePath)) {
            errors.push(`Circular module reference: ${submodule.filePath}`);
            continue;
        }
        visited.add(submodule.filePath);

        // Calculate the full path for this mount point
        const mountPathParts = [...pathPrefix, ...mountPath.split('/')];

        // Get or create the subtree node for this mount point
        const subtree = getOrCreateSubtree(root, mountPathParts);

        // Extract description from submodule's incipit
        if (submodule.incipit) {
            const descAnn = findAnnotation(submodule.incipit.annotations, 'descriptio');
            if (descAnn) {
                subtree.description = getAnnotationString(descAnn);
            }
        }

        // Generate module alias and track for import generation
        const moduleAlias = generateModuleAlias(submodule.filePath, basePath);
        const { relative, dirname } = require('node:path');
        const relImportPath = './' + relative(dirname(basePath), submodule.filePath).replace(/\.fab$/, '');
        moduleImports.set(moduleAlias, relImportPath);

        // Extract commands from the submodule with module prefix
        extractCommands(submodule.functions, root, mountPathParts, errors, moduleAlias);

        // Recursively mount any sub-submodules
        mountSubmodules(submodule, root, mountPathParts, ctx, errors, visited, moduleImports, basePath);

        visited.delete(submodule.filePath);
    }
}

// =============================================================================
// MAIN DETECTION
// =============================================================================

/** Result of CLI detection */
export interface CliDetectionResult {
    cli?: CliProgram;
    /** Module imports needed for CLI dispatcher (alias -> relative path) */
    moduleImports: CliModuleImports;
    errors: string[];
}

/**
 * Detect CLI program from a parsed AST.
 *
 * @param program - Parsed program AST
 * @param filePath - Absolute path to the source file (for module resolution)
 * @returns CLI program metadata and any errors
 */
export function detectCliProgram(program: Program, filePath?: string): CliDetectionResult {
    const errors: string[] = [];

    // Extract CLI info from the main program
    const mainInfo: CliModuleInfo = {
        filePath: filePath ?? '',
        imports: [],
        functions: [],
    };

    // Extract imports, incipit, and functions
    for (const stmt of program.body) {
        switch (stmt.type) {
            case 'ImportaDeclaration': {
                const imp = stmt;
                const source = typeof imp.source === 'string' ? imp.source : imp.source.name;

                if (source.startsWith('./') || source.startsWith('../')) {
                    if (imp.wildcard && imp.wildcardAlias) {
                        mainInfo.imports.push({
                            localName: imp.wildcardAlias.name,
                            sourcePath: source,
                            isWildcard: true,
                        });
                    }
                    else if (imp.specifiers) {
                        for (const spec of imp.specifiers) {
                            mainInfo.imports.push({
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
                if (stmt.annotations && stmt.annotations.length > 0) {
                    mainInfo.incipit = { annotations: stmt.annotations };
                }
                break;
            }

            case 'FunctioDeclaration': {
                mainInfo.functions.push({
                    name: stmt.name.name,
                    annotations: stmt.annotations ?? [],
                    params: stmt.params.map(p => ({
                        name: p.name.name,
                        type: p.typeAnnotation?.name ?? 'textus',
                        optional: p.optional === true,
                        shortFlag: p.alias?.name,
                        defaultValue: p.defaultValue?.type === 'Literal'
                            ? String(p.defaultValue.value)
                            : undefined,
                    })),
                });
                break;
            }
        }
    }

    // Track module imports for CLI dispatcher
    const moduleImports: CliModuleImports = new Map();

    // Check for @ cli annotation on incipit
    if (!mainInfo.incipit) {
        return { moduleImports, errors };
    }

    const cliAnn = findAnnotation(mainInfo.incipit.annotations, 'cli');
    if (!cliAnn) {
        return { moduleImports, errors };
    }

    // Extract CLI metadata
    const versioAnn = findAnnotation(mainInfo.incipit.annotations, 'versio');
    const descriptioAnn = findAnnotation(mainInfo.incipit.annotations, 'descriptio');

    const cli: CliProgram = {
        name: getAnnotationString(cliAnn) ?? 'cli',
        version: versioAnn ? getAnnotationString(versioAnn) : undefined,
        description: descriptioAnn ? getAnnotationString(descriptioAnn) : undefined,
        root: createCommandNode('', ''),
    };

    // Extract commands from main file (no module prefix - these are local functions)
    extractCommands(mainInfo.functions, cli.root, [], errors);

    // Process @ imperia for mounted submodules (if we have a file path)
    if (filePath) {
        const ctx = createResolverContext(filePath);
        ctx.cache.set(filePath, mainInfo);

        const visited = new Set<string>([filePath]);
        mountSubmodules(mainInfo, cli.root, [], ctx, errors, visited, moduleImports, filePath);
    }

    return { cli, moduleImports, errors };
}
