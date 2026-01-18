#!/usr/bin/env bun
/**
 * NANUS - Minimal Faber Compiler
 *
 * CLI entry point for testing and development.
 *
 * Usage:
 *   nanus <file.fab>           Compile to stdout
 *   nanus <file.fab> -o out.ts Compile to file
 *   echo "..." | nanus         Compile from stdin
 */

import { compile } from './index';

async function main() {
    const args = process.argv.slice(2);

    let input: string | undefined;
    let output: string | undefined;
    let filename = '<stdin>';

    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        if (arg === '-o' || arg === '--output') {
            output = args[++i];
        } else if (arg === '-h' || arg === '--help') {
            console.log('Usage: nanus [options] [file.fab]');
            console.log('');
            console.log('Options:');
            console.log('  -o, --output <file>  Output file (default: stdout)');
            console.log('  -h, --help           Show this help');
            process.exit(0);
        } else if (!arg.startsWith('-')) {
            input = arg;
            filename = arg;
        }
    }

    let source: string;
    if (input) {
        source = await Bun.file(input).text();
    } else {
        // Read from stdin
        const chunks: string[] = [];
        for await (const chunk of Bun.stdin.stream()) {
            chunks.push(new TextDecoder().decode(chunk));
        }
        source = chunks.join('');
    }

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
}

main();
