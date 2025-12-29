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

    // First pass: generate body (this populates features)
    const body = program.body.map(stmt => g.genStatement(stmt)).join('\n');

    // Second: prepend preamble based on detected features
    const preamble = genPreamble(g);

    return preamble + body;
}

/**
 * Generate preamble based on features used.
 *
 * WHY: Only emit setup code for features actually used in the program.
 */
function genPreamble(g: TsGenerator): string {
    const imports: string[] = [];
    const definitions: string[] = [];

    if (g.features.decimal) {
        imports.push("import type Decimal from 'decimal.js';");
    }

    if (g.features.panic) {
        definitions.push('class Panic extends Error { name = "Panic"; }');
    }

    // WHY: Flumina (streams-first) requires Responsum type, respond helpers, and drain()
    if (g.features.flumina) {
        definitions.push(`type Responsum<T = unknown> =
  | { op: 'bene'; data: T }
  | { op: 'error'; code: string; message: string }
  | { op: 'factum' }
  | { op: 'res'; data: T };

const respond = {
  ok: <T>(data: T): Responsum<T> => ({ op: 'bene', data }),
  error: (code: string, message: string): Responsum<never> => ({ op: 'error', code, message }),
  done: (): Responsum<never> => ({ op: 'factum' }),
  item: <T>(data: T): Responsum<T> => ({ op: 'res', data }),
};

function drain<T>(gen: () => Generator<Responsum<T>>): T {
  for (const resp of gen()) {
    if (resp.op === 'bene') return resp.data;
    if (resp.op === 'error') throw new Error(\`\${resp.code}: \${resp.message}\`);
  }
  throw new Error('EPROTO: No terminal response');
}`);
    }

    const lines = [...imports, ...definitions];
    return lines.length > 0 ? lines.join('\n') + '\n\n' : '';
}
