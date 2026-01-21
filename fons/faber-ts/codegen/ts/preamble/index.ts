/**
 * TypeScript Preamble Generator
 *
 * Assembles preamble snippets based on features used.
 */

import type { RequiredFeatures } from '../../types';

const PANIC = `class Panic extends Error {
  name = "Panic";
}`;

const DECIMAL = `import type Decimal from 'decimal.js';`;

const FS = `import * as fs from 'fs';`;

const NODE_PATH = `import * as path from 'path';`;

const FLUMINA = `type Responsum<T = unknown> =
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

function asFit<T>(gen: () => Generator<Responsum<T>>): T {
  for (const resp of gen()) {
    if (resp.op === 'bene') return resp.data;
    if (resp.op === 'error') throw new Error(\`\${resp.code}: \${resp.message}\`);
  }
  throw new Error('EPROTO: No terminal response');
}

function* asFiunt<T>(gen: Generator<Responsum<T>>): Generator<T> {
  for (const resp of gen) {
    if (resp.op === 'res') yield resp.data;
    else if (resp.op === 'error') throw new Error(\`\${resp.code}: \${resp.message}\`);
    else if (resp.op === 'factum') return;
    else if (resp.op === 'bene') { yield resp.data; return; }
  }
}

async function asFiet<T>(gen: () => AsyncGenerator<Responsum<T>>): Promise<T> {
  for await (const resp of gen()) {
    if (resp.op === 'bene') return resp.data;
    if (resp.op === 'error') throw new Error(\`\${resp.code}: \${resp.message}\`);
  }
  throw new Error('EPROTO: No terminal response');
}

async function* asFient<T>(gen: AsyncGenerator<Responsum<T>>): AsyncGenerator<T> {
  for await (const resp of gen) {
    if (resp.op === 'res') yield resp.data;
    else if (resp.op === 'error') throw new Error(\`\${resp.code}: \${resp.message}\`);
    else if (resp.op === 'factum') return;
    else if (resp.op === 'bene') { yield resp.data; return; }
  }
}`;

/**
 * Generate preamble based on features used.
 *
 * @param features - Feature flags set during codegen traversal
 * @returns Preamble string (empty if no features need setup)
 */
export function genPreamble(features: RequiredFeatures): string {
    const imports: string[] = [];
    const definitions: string[] = [];

    if (features.decimal) {
        imports.push(DECIMAL);
    }

    if (features.fs) {
        imports.push(FS);
    }

    if (features.nodePath) {
        imports.push(NODE_PATH);
    }

    if (features.panic) {
        definitions.push(PANIC);
    }

    if (features.flumina) {
        definitions.push(FLUMINA);
    }

    const lines = [...imports, ...definitions];
    return lines.length > 0 ? lines.join('\n') + '\n\n' : '';
}
