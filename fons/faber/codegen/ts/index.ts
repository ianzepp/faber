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

    // Pre-pass: detect CLI mode and collect command metadata
    // Pass filePath for module resolution (@ imperia ex module)
    const cliResult = detectCliProgram(program, options.filePath);

    // Fail hard on CLI detection errors
    if (cliResult.errors.length > 0) {
        throw new Error(`CLI detection errors:\n  ${cliResult.errors.join('\n  ')}`);
    }

    g.cli = cliResult.cli;
    g.cliModuleImports = cliResult.moduleImports;

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

    // Third: generate CLI module imports (must be at top of file for valid ESM)
    // WHY: CLI imports are hoisted here instead of inside incipit to ensure
    // they appear before any non-import statements
    let cliImports = '';
    if (g.cliModuleImports.size > 0) {
        const importLines: string[] = [];
        for (const [alias, path] of g.cliModuleImports) {
            importLines.push(`import * as ${alias} from "${path}";`);
        }
        cliImports = importLines.join('\n') + '\n\n';
    }

    return preamble + cliImports + body;
}
