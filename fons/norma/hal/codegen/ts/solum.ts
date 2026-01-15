/**
 * solum.ts - File System Implementation
 *
 * Native TypeScript implementation of the HAL solum (filesystem) interface.
 * Uses Bun file APIs and Node.js fs/path/os modules.
 */

import * as fs from 'node:fs/promises';
import * as fsSync from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';

export const solum = {
    // =========================================================================
    // READING
    // =========================================================================

    async lege(filePath: string): Promise<string> {
        return Bun.file(filePath).text();
    },

    async legeOctetos(filePath: string): Promise<Uint8Array> {
        return new Uint8Array(await Bun.file(filePath).arrayBuffer());
    },

    async *legens(filePath: string): AsyncIterable<Uint8Array> {
        const stream = Bun.file(filePath).stream();
        for await (const chunk of stream) {
            yield chunk;
        }
    },

    // =========================================================================
    // WRITING
    // =========================================================================

    async scribe(filePath: string, data: string): Promise<void> {
        await Bun.write(filePath, data);
    },

    async scribeOctetos(filePath: string, data: Uint8Array): Promise<void> {
        await Bun.write(filePath, data);
    },

    async appone(filePath: string, data: string): Promise<void> {
        await fs.appendFile(filePath, data);
    },

    async *scribens(filePath: string): AsyncGenerator<void, void, Uint8Array> {
        const handle = await fs.open(filePath, 'w');
        try {
            while (true) {
                const chunk = yield;
                if (chunk === undefined) break;
                await handle.write(chunk);
            }
        }
        finally {
            await handle.close();
        }
    },

    // =========================================================================
    // FILE INFO
    // =========================================================================

    exstat(filePath: string): boolean {
        try {
            fsSync.accessSync(filePath);
            return true;
        }
        catch {
            return false;
        }
    },

    estLimae(filePath: string): boolean {
        try {
            return fsSync.statSync(filePath).isFile();
        }
        catch {
            return false;
        }
    },

    estDirectorii(filePath: string): boolean {
        try {
            return fsSync.statSync(filePath).isDirectory();
        }
        catch {
            return false;
        }
    },

    async magnitudo(filePath: string): Promise<number> {
        const stats = await fs.stat(filePath);
        return stats.size;
    },

    async modificatum(filePath: string): Promise<number> {
        const stats = await fs.stat(filePath);
        return stats.mtimeMs;
    },

    // =========================================================================
    // FILE OPERATIONS
    // =========================================================================

    async dele(filePath: string): Promise<void> {
        await fs.unlink(filePath);
    },

    async copia(src: string, dest: string): Promise<void> {
        await fs.copyFile(src, dest);
    },

    async move(src: string, dest: string): Promise<void> {
        await fs.rename(src, dest);
    },

    async tange(filePath: string): Promise<void> {
        const now = new Date();
        try {
            await fs.utimes(filePath, now, now);
        }
        catch {
            // File doesn't exist, create it
            await Bun.write(filePath, '');
        }
    },

    // =========================================================================
    // DIRECTORY OPERATIONS
    // =========================================================================

    async creaDir(dirPath: string): Promise<void> {
        await fs.mkdir(dirPath, { recursive: true });
    },

    async elenca(dirPath: string): Promise<string[]> {
        return fs.readdir(dirPath);
    },

    async *ambula(dirPath: string): AsyncIterable<string> {
        const entries = await fs.readdir(dirPath, { withFileTypes: true });
        for (const entry of entries) {
            const fullPath = path.join(dirPath, entry.name);
            if (entry.isDirectory()) {
                yield* solum.ambula(fullPath);
            }
            else {
                yield fullPath;
            }
        }
    },

    async deleDir(dirPath: string): Promise<void> {
        await fs.rmdir(dirPath);
    },

    async deleArborem(dirPath: string): Promise<void> {
        await fs.rm(dirPath, { recursive: true, force: true });
    },

    // =========================================================================
    // PATH UTILITIES
    // =========================================================================

    iunge(parts: string[]): string {
        return path.join(...parts);
    },

    dir(filePath: string): string {
        return path.dirname(filePath);
    },

    basis(filePath: string): string {
        return path.basename(filePath);
    },

    extensio(filePath: string): string {
        return path.extname(filePath);
    },

    absolve(filePath: string): string {
        return path.resolve(filePath);
    },

    domus(): string {
        return os.homedir();
    },

    temp(): string {
        return os.tmpdir();
    },
};
