/**
 * consolum.ts - Console Device Implementation (Bun/Node)
 *
 * Native TypeScript implementation of the HAL console interface.
 */

import { readSync, writeSync } from 'node:fs';
import { isatty } from 'node:tty';

export const consolum = {
    // STDIN
    hauriOctetos(magnitudo: number): Uint8Array {
        const buffer = new Uint8Array(magnitudo);
        const bytesRead = readSync(0, buffer);
        return buffer.slice(0, bytesRead);
    },

    hauriLineam(): string {
        const decoder = new TextDecoder();
        const chunks: number[] = [];
        const buffer = new Uint8Array(1);

        while (true) {
            const bytesRead = readSync(0, buffer);
            if (bytesRead === 0 || buffer[0] === 10) break;
            if (buffer[0] !== 13) chunks.push(buffer[0]!);
        }

        return decoder.decode(new Uint8Array(chunks));
    },

    async hauriOmnia(): Promise<string> {
        return Bun.stdin.text();
    },

    // STDOUT
    fundeOctetos(data: Uint8Array): void {
        writeSync(1, data);
    },

    fundeTextum(msg: string): void {
        writeSync(1, msg);
    },

    fundeLineam(msg: string): void {
        writeSync(1, msg + '\n');
    },

    // STDERR
    errorOctetos(data: Uint8Array): void {
        writeSync(2, data);
    },

    errorTextum(msg: string): void {
        writeSync(2, msg);
    },

    errorLineam(msg: string): void {
        writeSync(2, msg + '\n');
    },

    // TTY
    estTerminale(): boolean {
        return isatty(0);
    },

    estTerminaleOutput(): boolean {
        return isatty(1);
    },
};
