/**
 * Isolated compiler runner for rivus tests.
 *
 * Runs the bootstrap compiler (opus/rivus) in a separate process so the
 * test harness can enforce hard timeouts even if the parser hits a CPU-bound
 * infinite loop.
 *
 * Reads Faber source from stdin, writes generated TypeScript to stdout.
 * On any error, writes a single-line message to stderr and exits non-zero.
 */

import { lexare } from '../../opus/rivus/fons/ts/lexor/index';
import { resolvere } from '../../opus/rivus/fons/ts/parser/index';
import { analyze } from '../../opus/rivus/fons/ts/semantic/index';
import { generateTs } from '../../opus/rivus/fons/ts/codegen/ts/index';

import type { Programma } from '../../opus/rivus/fons/ts/ast/radix';
import type { Sententia } from '../../opus/rivus/fons/ts/ast/sententia';

function fail(message: string): never {
    console.error(message);
    process.exit(1);
}

const source = await Bun.stdin.text();

const lexResult = lexare(source);
if (lexResult.errores.length > 0) {
    const msgs = lexResult.errores.map((e: any) => `${e.codice ?? 'L???'} ${e.textus || String(e)}`).join('; ');
    fail(`Tokenizer errors: ${msgs}`);
}

const parseResult = resolvere(lexResult.symbola);
if (parseResult.errores.length > 0) {
    const msgs = parseResult.errores.map((e: any) => `${e.codice ?? 'P???'} ${e.nuntius || String(e)}`).join('; ');
    fail(`Parse errors: ${msgs}`);
}
if (!parseResult.programma) {
    fail('Parse failed: no program');
}

// WHY: Most codegen tests are "snippet" style and may reference undefined
// variables as placeholders, so the default mode is to ignore semantic errors.
// Errata tests can opt into strict checking via an env var.
const semanticResult = analyze(parseResult.programma as Programma);
const strictSemantic = process.env.RIVUS_STRICT_SEMANTIC === '1';
if (strictSemantic && semanticResult.errores.length > 0) {
    const msgs = semanticResult.errores.map((e: any) => e.nuntius || e.textus || String(e)).join('; ');
    fail(`Semantic errors: ${msgs}`);
}

const output = generateTs((parseResult.programma as Programma).corpus as Sententia[]);
process.stdout.write(output);
