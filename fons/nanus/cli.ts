#!/usr/bin/env bun
/**
 * NANUS - Minimal Faber Compiler
 *
 * CLI with faber-compatible interface for bootstrapping rivus.
 *
 * Usage:
 *   nanus compile <file.fab>           Compile to stdout
 *   nanus compile <file.fab> -o out.ts Compile to file
 *   echo "..." | nanus compile         Compile from stdin
 */

import { compile, lex, prepare, parse } from './index';
import { formatError } from './errors';

function showHelp() {
    console.log('nanus - Minimal Faber compiler');
    console.log('Compiles the Faber subset needed to bootstrap rivus.');
    console.log('');
    console.log('Usage:');
    console.log('  nanus <command> <file> [options]');
    console.log('');
    console.log('Commands:');
    console.log('  emit, compile <file>   Emit .fab file as TypeScript');
    console.log('  parse <file>           Parse and output AST as JSON');
    console.log('  lex <file>             Lex and output tokens as JSON');
    console.log('');
    console.log('Options:');
    console.log('  -o, --output <file>    Output file (default: stdout)');
    console.log('  -h, --help             Show this help');
    console.log('');
    console.log('Reads from stdin if no file specified (or use \'-\' explicitly).');
}

async function main() {
    const args = process.argv.slice(2);

    if (args.length === 0 || args.includes('-h') || args.includes('--help')) {
        showHelp();
        process.exit(0);
    }

    let command: string | undefined;
    let input: string | undefined;
    let output: string | undefined;
    let filename = '<stdin>';

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-o' || arg === '--output') {
            output = args[++i];
        } else if (arg === '-h' || arg === '--help') {
            showHelp();
            process.exit(0);
        } else if (arg.startsWith('-')) {
            // Ignore unknown flags for forward compatibility
        } else if (!command) {
            command = arg;
        } else if (!input) {
            input = arg;
            filename = arg;
        }
    }

    const validCommands = ['emit', 'compile', 'parse', 'lex'];
    if (!command || !validCommands.includes(command)) {
        console.error(`Unknown command: ${command ?? '(none)'}`);
        console.error('Use --help for usage.');
        process.exit(1);
    }

    let source: string;
    if (input && input !== '-') {
        source = await Bun.file(input).text();
    } else {
        // Read from stdin
        const chunks: string[] = [];
        for await (const chunk of Bun.stdin.stream()) {
            chunks.push(new TextDecoder().decode(chunk));
        }
        source = chunks.join('');
    }

    try {
        if (command === 'lex') {
            const tokens = lex(source, filename);
            const out = JSON.stringify(tokens, null, 2);
            if (output) {
                await Bun.write(output, out);
            } else {
                console.log(out);
            }
            return;
        }

        if (command === 'parse') {
            const tokens = prepare(lex(source, filename));
            const ast = parse(tokens, filename);
            const out = JSON.stringify(ast, null, 2);
            if (output) {
                await Bun.write(output, out);
            } else {
                console.log(out);
            }
            return;
        }

        // emit/compile
        const result = compile(source, { filename });

        if (!result.success) {
            console.error(result.error);
            process.exit(1);
        }

        if (output) {
            await Bun.write(output, result.output!);
        } else {
            console.log(result.output);
        }
    } catch (err) {
        console.error(formatError(err, source, filename));
        process.exit(1);
    }
}

main();
