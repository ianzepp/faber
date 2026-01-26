/**
 * consolum.ts - Console Device Implementation
 *
 * Native TypeScript implementation of the HAL console interface.
 * Uses Node.js fs for sync operations, Bun APIs for async.
 *
 * Verb conjugation encodes sync/async:
 *   - Imperative (-a, -e, -i): synchronous
 *   - Future indicative (-et, -ebit): asynchronous
 *
 * Aligns with language keywords: scribe (info), mone (warn), vide (debug)
 */

import { readSync, writeSync } from 'node:fs';
import { isatty } from 'node:tty';

export const consolum = {
    // =========================================================================
    // STDIN - Bytes
    // =========================================================================
    // Verb: hauri/hauriet from "haurire" (to draw up)

    /** Draw bytes from stdin (sync) */
    hauri(magnitudo: number): Uint8Array {
        const buffer = new Uint8Array(magnitudo);
        const bytesRead = readSync(0, buffer);
        return buffer.slice(0, bytesRead);
    },

    /** Draw bytes from stdin (async) */
    async hauriet(magnitudo: number): Promise<Uint8Array> {
        // Async version - for now, wraps sync
        // Future: could use async stdin stream
        const buffer = new Uint8Array(magnitudo);
        const bytesRead = readSync(0, buffer);
        return buffer.slice(0, bytesRead);
    },

    // =========================================================================
    // STDIN - Text
    // =========================================================================
    // Verb: lege/leget from "legere" (to read)

    /** Read line from stdin (sync, blocks until newline) */
    lege(): string {
        const decoder = new TextDecoder();
        const chunks: number[] = [];
        const buffer = new Uint8Array(1);

        while (true) {
            const bytesRead = readSync(0, buffer);
            if (bytesRead === 0 || buffer[0] === 10) break; // EOF or newline
            if (buffer[0] !== 13) chunks.push(buffer[0]!);   // skip CR
        }

        return decoder.decode(new Uint8Array(chunks));
    },

    /** Read line from stdin (async) */
    async leget(): Promise<string> {
        // Async version - for now, wraps sync
        // Future: could use readline or async stdin
        return consolum.lege();
    },

    // =========================================================================
    // STDOUT - Bytes
    // =========================================================================
    // Verb: funde/fundet from "fundere" (to pour)

    /** Pour bytes to stdout (sync) */
    funde(data: Uint8Array): void {
        writeSync(1, data);
    },

    /** Pour bytes to stdout (async) */
    async fundet(data: Uint8Array): Promise<void> {
        writeSync(1, data);
    },

    // =========================================================================
    // STDOUT - Text with Newline
    // =========================================================================
    // Verb: scribe/scribet from "scribere" (to write)

    /** Write line to stdout with newline (sync) */
    scribe(msg: string): void {
        writeSync(1, msg + '\n');
    },

    /** Write line to stdout with newline (async) */
    async scribet(msg: string): Promise<void> {
        writeSync(1, msg + '\n');
    },

    // =========================================================================
    // STDOUT - Text without Newline
    // =========================================================================
    // Verb: dic/dicet from "dicere" (to say)

    /** Say text to stdout without newline (sync) */
    dic(msg: string): void {
        writeSync(1, msg);
    },

    /** Say text to stdout without newline (async) */
    async dicet(msg: string): Promise<void> {
        writeSync(1, msg);
    },

    // =========================================================================
    // STDERR - Warning/Error Output
    // =========================================================================
    // Verb: mone/monet from "monere" (to warn)

    /** Warn line to stderr with newline (sync) */
    mone(msg: string): void {
        writeSync(2, msg + '\n');
    },

    /** Warn line to stderr with newline (async) */
    async monet(msg: string): Promise<void> {
        writeSync(2, msg + '\n');
    },

    // =========================================================================
    // DEBUG Output
    // =========================================================================
    // Verb: vide/videbit from "videre" (to see)

    /** Debug line with newline (sync) */
    vide(msg: string): void {
        writeSync(2, msg + '\n');
    },

    /** Debug line with newline (async) */
    async videbit(msg: string): Promise<void> {
        writeSync(2, msg + '\n');
    },

    // =========================================================================
    // TTY Detection
    // =========================================================================

    /** Is stdin connected to a terminal? */
    estTerminale(): boolean {
        return isatty(0);
    },

    /** Is stdout connected to a terminal? */
    estTerminaleOutput(): boolean {
        return isatty(1);
    },
};
