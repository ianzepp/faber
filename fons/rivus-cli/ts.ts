/**
 * rivus.ts - Native TypeScript CLI shim for rivus compiler
 *
 * This is a minimal CLI wrapper that provides I/O for the rivus compiler library.
 * The rivus library itself is pure (no I/O), compiled from Faber source.
 *
 * During build, this file is copied to opus/nanus-ts/fons/cli.ts and bundled
 * as the entry point for the rivus executable.
 *
 * Usage:
 *   cat hello.fab | rivus lex              # Tokenize
 *   cat hello.fab | rivus parse            # Parse to AST
 *   cat hello.fab | rivus emit             # Compile to TypeScript
 *   cat hello.fab | rivus emit -t go       # Compile to Go
 */

import { readFileSync } from 'node:fs';
import { lexare } from './lexor/index';
import { resolvere } from './parser/index';
import { analyze } from './semantic/index';
import { generate } from './codegen/index';

const args = process.argv.slice(2);

if (args.length === 0) {
    console.log('Usage: <source> | rivus <command> [options]');
    console.log('Commands: lex, parse, emit');
    console.log('Options:');
    console.log('  -t, --target <lang>      Target language: ts (default), go');
    console.log('  --stdin-filename <name>  Filename for error messages');
    process.exit(0);
}

const command = args[0];

// Parse options
let stdinFilename = '<stdin>';
let target = 'ts';

for (let i = 1; i < args.length; i++) {
    const arg = args[i];
    if (arg === '--stdin-filename' && args[i + 1]) {
        stdinFilename = args[++i];
    } else if ((arg === '-t' || arg === '--target') && args[i + 1]) {
        target = args[++i];
    }
}

// Read source from stdin
const source = readFileSync(0, 'utf-8');

// Command: lex
if (command === 'lex') {
    const result = lexare(source);
    console.log(JSON.stringify(result.symbola, null, 2));
    process.exit(0);
}

// Command: parse
if (command === 'parse') {
    const lexResult = lexare(source);
    const parseResult = resolvere(lexResult.symbola);
    console.log(JSON.stringify(parseResult.programma, null, 2));
    process.exit(0);
}

// Command: emit
if (command === 'emit') {
    const lexResult = lexare(source);

    if (lexResult.errores.length > 0) {
        for (const err of lexResult.errores) {
            console.error(`${stdinFilename}:${err.locus.linea}:${err.locus.columna}: ${err.textus}`);
        }
        process.exit(1);
    }

    const parseResult = resolvere(lexResult.symbola);

    if (parseResult.errores.length > 0) {
        for (const err of parseResult.errores) {
            console.error(`${stdinFilename}:${err.locus.linea}:${err.locus.columna}: ${err.nuntius}`);
        }
        process.exit(1);
    }

    if (!parseResult.programma) {
        console.error('Parse failed');
        process.exit(1);
    }

    const semResult = analyze(parseResult.programma, null, stdinFilename);

    if (semResult.errores.length > 0) {
        for (const err of semResult.errores) {
            console.error(`${stdinFilename}:${err.locus.linea}:${err.locus.columna}: ${err.nuntius}`);
        }
    }

    const output = generate(parseResult.programma.corpus, target);
    console.log(output);
    process.exit(0);
}

console.error(`Unknown command: ${command}`);
console.error('Commands: lex, parse, emit');
process.exit(1);
