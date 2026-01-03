/**
 * Isolated compiler runner for rivus tests.
 *
 * Runs the bootstrap compiler (opus/bootstrap) in a separate process so the
 * test harness can enforce hard timeouts even if the parser hits a CPU-bound
 * infinite loop.
 *
 * Reads Faber source from stdin, writes generated TypeScript to stdout.
 * On any error, writes a single-line message to stderr and exits non-zero.
 */

import { lexare } from '../../opus/bootstrap/lexor/index';
import { resolvere } from '../../opus/bootstrap/parser/index';
import { analyze } from '../../opus/bootstrap/semantic/index';
import { generateTs } from '../../opus/bootstrap/codegen/ts/index';

import type { Programma } from '../../opus/bootstrap/ast/radix';
import type { Sententia } from '../../opus/bootstrap/ast/sententia';

function fail(message: string): never {
    console.error(message);
    process.exit(1);
}

const source = await Bun.stdin.text();

const lexResult = lexare(source);
if (lexResult.errores.length > 0) {
    const msgs = lexResult.errores.map((e: any) => e.textus || String(e)).join('; ');
    fail(`Lexor errors: ${msgs}`);
}

const parseResult = resolvere(lexResult.symbola);
if (parseResult.errores.length > 0) {
    const msgs = parseResult.errores.map((e: any) => e.nuntius || String(e)).join('; ');
    fail(`Parser errors: ${msgs}`);
}
if (!parseResult.programma) {
    fail('Parse failed: no program');
}

// WHY: Lenient mode - run semantic analysis but ignore errors.
// Snippet tests often have undefined variables as placeholders.
analyze(parseResult.programma as Programma);

const output = generateTs((parseResult.programma as Programma).corpus as Sententia[]);
process.stdout.write(output);
