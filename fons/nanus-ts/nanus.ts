#!/usr/bin/env bun
/**
 * NANUS - Minimal Faber Compiler
 *
 * Pure stdin/stdout microcompiler for bootstrapping rivus.
 *
 * Usage:
 *   echo 'scribe "hello"' | nanus-ts emit
 *   cat file.fab | nanus-ts emit
 */

import { compile, lex, prepare, parse } from './index';
import { formatError } from './errors';

async function readStdin(): Promise<string> {
    const chunks: string[] = [];
    for await (const chunk of Bun.stdin.stream()) {
        chunks.push(new TextDecoder().decode(chunk));
    }
    return chunks.join('');
}

async function main() {
    const args = process.argv.slice(2);
    const command = args[0];

    if (!command || command === '-h' || command === '--help') {
        console.log('nanus-ts - Minimal Faber compiler (stdin/stdout)');
        console.log('');
        console.log('Usage: <source> | nanus-ts <command>');
        console.log('');
        console.log('Commands:');
        console.log('  emit     Compile to TypeScript');
        console.log('  parse    Output AST as JSON');
        console.log('  lex      Output tokens as JSON');
        process.exit(0);
    }

    const validCommands = ['emit', 'parse', 'lex'];
    if (!validCommands.includes(command)) {
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }

    const source = await readStdin();

    try {
        if (command === 'lex') {
            const tokens = lex(source, '<stdin>');
            console.log(JSON.stringify(tokens, null, 2));
            return;
        }

        if (command === 'parse') {
            const tokens = prepare(lex(source, '<stdin>'));
            const ast = parse(tokens, '<stdin>');
            console.log(JSON.stringify(ast, null, 2));
            return;
        }

        // emit
        const result = compile(source);
        if (!result.success) {
            console.error(result.error);
            process.exit(1);
        }
        console.log(result.output);
    } catch (err) {
        console.error(formatError(err, source, '<stdin>'));
        process.exit(1);
    }
}

main();
