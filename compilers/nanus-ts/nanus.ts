#!/usr/bin/env bun
/**
 * NANUS - Minimal Faber Compiler
 *
 * Pure stdin/stdout microcompiler for bootstrapping rivus.
 *
 * Usage:
 *   echo 'scribe "hello"' | nanus-ts emit
 *   cat file.fab | nanus-ts emit -t fab
 */

import { lex, prepare, parse, emit, emitFaber } from './index';
import { formatError } from './errors';

type Target = 'ts' | 'fab';

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
        console.log('Usage: <source> | nanus-ts <command> [options]');
        console.log('');
        console.log('Commands:');
        console.log('  emit     Compile to target language');
        console.log('  parse    Output AST as JSON');
        console.log('  lex      Output tokens as JSON');
        console.log('');
        console.log('Options:');
        console.log('  -t <target>            Output target: ts, fab (default: ts)');
        console.log('  --stdin-filename <f>   Filename for error messages (default: <stdin>)');
        process.exit(0);
    }

    const validCommands = ['emit', 'parse', 'lex'];
    if (!validCommands.includes(command)) {
        console.error(`Unknown command: ${command}`);
        process.exit(1);
    }

    // Parse flags
    let target: Target = 'ts';
    let filename = '<stdin>';
    for (let i = 1; i < args.length; i++) {
        if (args[i] === '-t' && i + 1 < args.length) {
            const t = args[i + 1];
            if (t !== 'ts' && t !== 'fab') {
                console.error(`Unknown target: ${t}. Valid: ts, fab`);
                process.exit(1);
            }
            target = t as Target;
            i++;
        } else if (args[i] === '--stdin-filename' && i + 1 < args.length) {
            filename = args[i + 1]!;
            i++;
        }
    }

    const source = await readStdin();

    try {
        if (command === 'lex') {
            const tokens = lex(source, filename);
            console.log(JSON.stringify(tokens, null, 2));
            return;
        }

        if (command === 'parse') {
            const tokens = prepare(lex(source, filename));
            const ast = parse(tokens, filename);
            console.log(JSON.stringify(ast, null, 2));
            return;
        }

        // emit
        const tokens = prepare(lex(source, filename));
        const ast = parse(tokens, filename);
        const output = target === 'fab' ? emitFaber(ast) : emit(ast, { sourceFile: filename });
        console.log(output);
    } catch (err) {
        console.error(formatError(err, source, filename));
        process.exit(1);
    }
}

main();
