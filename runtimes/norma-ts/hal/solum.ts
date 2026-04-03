/**
 * solum.ts - File System Implementation
 *
 * Native TypeScript implementation of the HAL solum (filesystem) interface.
 * Uses Bun file APIs and Node.js fs/path/os modules.
 *
 * Verb conjugation encodes sync/async:
 *   - Imperative (-a, -e, -i): synchronous
 *   - Future indicative (-et, -ebit): asynchronous (returns Promise)
 */

import * as fs from 'node:fs/promises';
import * as fsSync from 'node:fs';
import * as path from 'node:path';
import * as os from 'node:os';

/** Full file status returned by describe/describet */
export interface SolumStatus {
    modus: number;        // permission bits (e.g., 0o755)
    nexus: number;        // hard link count
    possessor: number;    // owner uid
    grex: number;         // group gid
    magnitudo: number;    // size in bytes
    modificatum: number;  // mtime (ms since epoch)
    estDirectorii: boolean;
    estVinculum: boolean; // is symlink
}

export const solum = {
    // =========================================================================
    // READING - Text
    // =========================================================================
    // Verb: lege/leget from "legere" (to read, gather)

    /** Read entire file as text (sync) */
    lege(via: string): string {
        return fsSync.readFileSync(via, 'utf-8');
    },

    /** Read entire file as text (async) */
    async leget(via: string): Promise<string> {
        return Bun.file(via).text();
    },

    // =========================================================================
    // READING - Bytes
    // =========================================================================
    // Verb: hauri/hauriet from "haurire" (to draw up, draw water)

    /** Draw entire file as bytes (sync) */
    hauri(via: string): Uint8Array {
        return new Uint8Array(fsSync.readFileSync(via));
    },

    /** Draw entire file as bytes (async) */
    async hauriet(via: string): Promise<Uint8Array> {
        return new Uint8Array(await Bun.file(via).arrayBuffer());
    },

    // =========================================================================
    // READING - Lines
    // =========================================================================
    // Verb: carpe/carpiet from "carpere" (to pluck, pick, harvest)

    /** Pluck lines from file (sync) */
    carpe(via: string): string[] {
        const content = fsSync.readFileSync(via, 'utf-8');
        return content.split(/\r?\n/);
    },

    /** Pluck lines from file (async) */
    async carpiet(via: string): Promise<string[]> {
        const content = await Bun.file(via).text();
        return content.split(/\r?\n/);
    },

    // =========================================================================
    // WRITING - Text
    // =========================================================================
    // Verb: scribe/scribet from "scribere" (to write)

    /** Write text to file, overwrites existing (sync) */
    scribe(via: string, data: string): void {
        fsSync.writeFileSync(via, data, 'utf-8');
    },

    /** Write text to file, overwrites existing (async) */
    async scribet(via: string, data: string): Promise<void> {
        await Bun.write(via, data);
    },

    // =========================================================================
    // WRITING - Bytes
    // =========================================================================
    // Verb: funde/fundet from "fundere" (to pour, pour out)

    /** Pour bytes to file, overwrites existing (sync) */
    funde(via: string, data: Uint8Array): void {
        fsSync.writeFileSync(via, data);
    },

    /** Pour bytes to file, overwrites existing (async) */
    async fundet(via: string, data: Uint8Array): Promise<void> {
        await Bun.write(via, data);
    },

    // =========================================================================
    // WRITING - Append
    // =========================================================================
    // Verb: appone/apponet from "apponere" (to place near, add to)

    /** Append text to file (sync) */
    appone(via: string, data: string): void {
        fsSync.appendFileSync(via, data);
    },

    /** Append text to file (async) */
    async apponet(via: string, data: string): Promise<void> {
        await fs.appendFile(via, data);
    },

    // =========================================================================
    // FILE INFO - Existence
    // =========================================================================
    // Verb: exstat/exstabit from "exstare" (to stand out, exist)

    /** Check if path exists (sync) */
    exstat(via: string): boolean {
        try {
            fsSync.accessSync(via);
            return true;
        }
        catch {
            return false;
        }
    },

    /** Check if path exists (async) */
    async exstabit(via: string): Promise<boolean> {
        try {
            await fs.access(via);
            return true;
        }
        catch {
            return false;
        }
    },

    // =========================================================================
    // FILE INFO - Details
    // =========================================================================
    // Verb: describe/describet from "describere" (to describe, delineate)

    /** Get file details (sync) */
    describe(via: string): SolumStatus {
        const stats = fsSync.lstatSync(via);
        return {
            modus: stats.mode & 0o7777,
            nexus: stats.nlink,
            possessor: stats.uid,
            grex: stats.gid,
            magnitudo: stats.size,
            modificatum: stats.mtimeMs,
            estDirectorii: stats.isDirectory(),
            estVinculum: stats.isSymbolicLink(),
        };
    },

    /** Get file details (async) */
    async describet(via: string): Promise<SolumStatus> {
        const stats = await fs.lstat(via);
        return {
            modus: stats.mode & 0o7777,
            nexus: stats.nlink,
            possessor: stats.uid,
            grex: stats.gid,
            magnitudo: stats.size,
            modificatum: stats.mtimeMs,
            estDirectorii: stats.isDirectory(),
            estVinculum: stats.isSymbolicLink(),
        };
    },

    // =========================================================================
    // FILE INFO - Symlinks
    // =========================================================================
    // Verb: sequere/sequetur from "sequi" (to follow)

    /** Follow symlink to get target path (sync) */
    sequere(via: string): string {
        return fsSync.readlinkSync(via);
    },

    /** Follow symlink to get target path (async) */
    async sequetur(via: string): Promise<string> {
        return fs.readlink(via);
    },

    // =========================================================================
    // FILE OPERATIONS - Delete
    // =========================================================================
    // Verb: dele/delet from "delere" (to destroy, delete)

    /** Delete file (sync) */
    dele(via: string): void {
        fsSync.unlinkSync(via);
    },

    /** Delete file (async) */
    async delet(via: string): Promise<void> {
        await fs.unlink(via);
    },

    // =========================================================================
    // FILE OPERATIONS - Copy
    // =========================================================================
    // Verb: exscribe/exscribet from "exscribere" (to copy out, transcribe)

    /** Copy file (sync) */
    exscribe(fons: string, destinatio: string): void {
        fsSync.copyFileSync(fons, destinatio);
    },

    /** Copy file (async) */
    async exscribet(fons: string, destinatio: string): Promise<void> {
        await fs.copyFile(fons, destinatio);
    },

    // =========================================================================
    // FILE OPERATIONS - Rename/Move
    // =========================================================================
    // Verb: renomina/renominabit from "renominare" (to rename)

    /** Rename or move file (sync) */
    renomina(fons: string, destinatio: string): void {
        fsSync.renameSync(fons, destinatio);
    },

    /** Rename or move file (async) */
    async renominabit(fons: string, destinatio: string): Promise<void> {
        await fs.rename(fons, destinatio);
    },

    // =========================================================================
    // FILE OPERATIONS - Touch
    // =========================================================================
    // Verb: tange/tanget from "tangere" (to touch)

    /** Touch file - create or update mtime (sync) */
    tange(via: string): void {
        const now = new Date();
        try {
            fsSync.utimesSync(via, now, now);
        }
        catch {
            fsSync.writeFileSync(via, '');
        }
    },

    /** Touch file - create or update mtime (async) */
    async tanget(via: string): Promise<void> {
        const now = new Date();
        try {
            await fs.utimes(via, now, now);
        }
        catch {
            await Bun.write(via, '');
        }
    },

    // =========================================================================
    // DIRECTORY OPERATIONS - Create
    // =========================================================================
    // Verb: crea/creabit from "creare" (to create, bring forth)

    /** Create directory, recursive (sync) */
    crea(via: string): void {
        fsSync.mkdirSync(via, { recursive: true });
    },

    /** Create directory, recursive (async) */
    async creabit(via: string): Promise<void> {
        await fs.mkdir(via, { recursive: true });
    },

    // =========================================================================
    // DIRECTORY OPERATIONS - List
    // =========================================================================
    // Verb: enumera/enumerabit from "enumerare" (to count out, enumerate)

    /** List directory contents (sync) */
    enumera(via: string): string[] {
        return fsSync.readdirSync(via);
    },

    /** List directory contents (async) */
    async enumerabit(via: string): Promise<string[]> {
        return fs.readdir(via);
    },

    // =========================================================================
    // DIRECTORY OPERATIONS - Prune/Remove
    // =========================================================================
    // Verb: amputa/amputabit from "amputare" (to cut off, prune)

    /** Prune directory tree, recursive (sync) */
    amputa(via: string): void {
        fsSync.rmSync(via, { recursive: true, force: true });
    },

    /** Prune directory tree, recursive (async) */
    async amputabit(via: string): Promise<void> {
        await fs.rm(via, { recursive: true, force: true });
    },

    // =========================================================================
    // PATH UTILITIES
    // =========================================================================
    // Pure functions on path strings, not filesystem I/O. Sync only.

    /** Join path segments */
    iunge(partes: string[]): string {
        return path.join(...partes);
    },

    /** Get directory part of path */
    directorium(via: string): string {
        return path.dirname(via);
    },

    /** Get filename part of path */
    basis(via: string): string {
        return path.basename(via);
    },

    /** Get file extension (includes dot) */
    extensio(via: string): string {
        return path.extname(via);
    },

    /** Resolve to absolute path */
    absolve(via: string): string {
        return path.resolve(via);
    },

    /** Get user's home directory */
    domus(): string {
        return os.homedir();
    },

    /** Get system temp directory */
    temporarium(): string {
        return os.tmpdir();
    },
};
