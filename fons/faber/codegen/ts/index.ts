/**
 * TypeScript Code Generator - Emit JavaScript with type annotations
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into TypeScript source code.
 * It preserves JavaScript runtime semantics while adding TypeScript type
 * annotations derived from Latin type declarations.
 *
 * The generator uses a recursive descent pattern that mirrors the AST structure.
 * Each AST node type has a corresponding gen* function that produces a string
 * fragment. These fragments are composed bottom-up to build the complete output.
 *
 * Indentation is managed via a depth counter that tracks nesting level. The
 * ind() helper function generates the appropriate indentation string for the
 * current depth.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid TypeScript source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * TypeScript preserves JavaScript semantics:
 * - Dynamic typing with optional annotations
 * - Prototype-based objects
 * - Async/await for concurrency
 * - Exception-based error handling
 * - Nullable types via union with null
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid TypeScript
 * INV-2: All Latin type names are mapped to TypeScript equivalents
 * INV-3: Indentation depth is correctly maintained (incremented/decremented)
 */

import type { Program } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { TsGenerator } from './generator';
import { genPreamble } from './preamble';
import { detectCliProgram } from '../cli/detector';
import { join, dirname, relative, resolve } from 'node:path';

/**
 * Generate TypeScript source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> TypeScript source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent, semicolons, filePath)
 * @returns TypeScript source code
 */
export function generateTs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 2 spaces is TypeScript convention
    const indent = options.indent ?? '  ';
    // WHY: Semicolons are recommended in TypeScript style guides
    const semi = options.semicolons ?? true;

    const g = new TsGenerator(indent, semi);
    g.sourceFilePath = options.filePath;
    g.keepRelativeImports = options.keepRelativeImports ?? false;

    // Pre-pass: detect CLI mode and collect command metadata
    // Pass filePath for module resolution (@ imperia ex module)
    const cliResult = detectCliProgram(program, options.filePath);

    // Fail hard on CLI detection errors
    if (cliResult.errors.length > 0) {
        throw new Error(`CLI detection errors:\n  ${cliResult.errors.join('\n  ')}`);
    }

    g.cli = cliResult.cli;
    g.cliModuleImports = cliResult.moduleImports;

    // Store subsidia imports from semantic analysis
    if (options.subsidiaImports) {
        g.subsidiaImports = options.subsidiaImports;
    }

    // First pass: generate body (this populates features)
    const body = program.body.map(stmt => g.genStatement(stmt)).join('\n');

    // Fail hard on codegen errors (no fallback guessing).
    if (g.codegenErrors.length > 0) {
        const lines = g.codegenErrors.map(err => {
            const loc = err.position ? `${err.position.line}:${err.position.column} ` : '';
            return `  ${loc}${err.message}`;
        });
        throw new Error(`TypeScript codegen errors:\n${lines.join('\n')}`);
    }

    // Second: prepend preamble based on detected features
    const preamble = genPreamble(g.features);

    // Third: generate HAL imports (must be at top for valid ESM)
    // WHY: HAL pactums with @subsidia need native implementations imported
    let halImports = '';
    if (g.halImports.size > 0) {
        const importLines: string[] = [];
        for (const [name, relativePath] of g.halImports) {
            // Resolve path relative to source file
            // WHY: @subsidia paths are relative to declaring .fab file
            const importPath = resolveHalImportPath(relativePath, options.filePath);
            importLines.push(`import { ${name} } from "${importPath}";`);
        }
        halImports = importLines.join('\n') + '\n\n';
    }

    // Fourth: generate CLI module imports (must be at top of file for valid ESM)
    // WHY: CLI imports are hoisted here instead of inside incipit to ensure
    // they appear before any non-import statements
    let cliImports = '';
    if (g.cliModuleImports.size > 0) {
        const importLines: string[] = [];
        for (const [alias, path] of g.cliModuleImports) {
            // WHY: Relative imports need absolutizing for temp file execution (faber run)
            // But for build output where directory structure is preserved, keep relative imports.
            let resolvedPath = path;
            if (g.sourceFilePath && !g.keepRelativeImports && (path.startsWith('./') || path.startsWith('../'))) {
                resolvedPath = resolve(dirname(g.sourceFilePath), path);
            }
            importLines.push(`import * as ${alias} from "${resolvedPath}";`);
        }
        cliImports = importLines.join('\n') + '\n\n';
    }

    return preamble + halImports + cliImports + body;
}

/**
 * Resolve HAL import path relative to output file.
 *
 * WHY: @subsidia paths are relative to the declaring .fab file.
 *      We need to convert them to paths relative to the output .ts file.
 *
 * @param relativePath - Path from @subsidia annotation (e.g., "codegen/ts/consolum.ts")
 * @param sourceFilePath - Original .fab file path (optional)
 * @returns Resolved import path
 */
function resolveHalImportPath(relativePath: string, sourceFilePath?: string): string {
    // For now, use path as-is if no source file context
    // WHY: When compiling from stdin or without file context, paths are passed through
    if (!sourceFilePath) {
        return relativePath;
    }

    // Resolve path relative to source file's directory
    // Example: fons/norma/hal/consolum.fab -> fons/norma/hal/codegen/ts/consolum.ts
    const sourceDir = dirname(sourceFilePath);
    const absolutePath = join(sourceDir, relativePath);

    // Make import path relative to current directory (for TypeScript imports)
    // WHY: TypeScript imports should be relative to project root or use absolute paths
    const importPath = relative(process.cwd(), absolutePath);

    // Ensure path starts with ./ or ../ for relative imports
    if (!importPath.startsWith('.') && !importPath.startsWith('/')) {
        return './' + importPath;
    }

    // Remove .ts extension if present (TypeScript imports don't need it)
    return importPath.replace(/\.ts$/, '');
}
