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

import type { Program, IncipitStatement, FunctioDeclaration, Annotation } from '../../parser/ast';
import type { CodegenOptions } from '../types';
import { TsGenerator, type CliProgram, type CliCommand } from './generator';
import { genPreamble } from './preamble';

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

/**
 * Scan program for CLI annotations and build CLI metadata.
 *
 * Looks for:
 * - @ cli on incipit -> marks file as CLI program
 * - @ versio on incipit -> program version
 * - @ descriptio on incipit -> program description
 * - @ imperium on functions -> subcommands
 * - @ alias on functions -> command aliases
 */
function detectCliProgram(program: Program): CliProgram | undefined {
    let cliIncipit: IncipitStatement | undefined;

    // Find incipit with @ cli annotation
    for (const stmt of program.body) {
        if (stmt.type === 'IncipitStatement') {
            const incipit = stmt as IncipitStatement;
            if (findAnnotation(incipit.annotations, 'cli')) {
                cliIncipit = incipit;
                break;
            }
        }
    }

    if (!cliIncipit) return undefined;

    // Extract CLI metadata from incipit annotations
    const cliAnn = findAnnotation(cliIncipit.annotations, 'cli')!;
    const versioAnn = findAnnotation(cliIncipit.annotations, 'versio');
    const descriptioAnn = findAnnotation(cliIncipit.annotations, 'descriptio');

    const cli: CliProgram = {
        name: getAnnotationString(cliAnn) ?? 'cli',
        version: versioAnn ? getAnnotationString(versioAnn) : undefined,
        description: descriptioAnn ? getAnnotationString(descriptioAnn) : undefined,
        commands: [],
    };

    // Collect @ imperium functions
    for (const stmt of program.body) {
        if (stmt.type === 'FunctioDeclaration') {
            const fn = stmt as FunctioDeclaration;
            const imperiumAnn = findAnnotation(fn.annotations, 'imperium');
            if (imperiumAnn) {
                const aliasAnn = findAnnotation(fn.annotations, 'alias');
                const cmd: CliCommand = {
                    name: getAnnotationString(imperiumAnn) ?? fn.name.name,
                    alias: aliasAnn ? getAnnotationString(aliasAnn) : undefined,
                    functionName: fn.name.name,
                    params: fn.params.map(p => ({
                        name: p.name.name,
                        type: p.typeAnnotation?.name ?? 'textus',
                        optional: p.optional === true,
                        shortFlag: p.alias?.name,
                        defaultValue: p.defaultValue?.type === 'Literal'
                            ? String(p.defaultValue.value)
                            : undefined,
                    })),
                };
                cli.commands.push(cmd);
            }
        }
    }

    return cli;
}

/**
 * Generate TypeScript source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> TypeScript source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent, semicolons)
 * @returns TypeScript source code
 */
export function generateTs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 2 spaces is TypeScript convention
    const indent = options.indent ?? '  ';
    // WHY: Semicolons are recommended in TypeScript style guides
    const semi = options.semicolons ?? true;

    const g = new TsGenerator(indent, semi);

    // Pre-pass: detect CLI mode and collect command metadata
    g.cli = detectCliProgram(program);

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

    return preamble + body;
}
