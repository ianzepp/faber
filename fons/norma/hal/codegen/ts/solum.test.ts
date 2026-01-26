import { test, expect, describe, beforeAll, afterAll } from 'bun:test';
import { solum } from './solum';
import * as path from 'node:path';
import * as os from 'node:os';

describe('solum HAL', () => {
    let testDir: string;

    beforeAll(async () => {
        // Create a unique temp directory for tests
        testDir = path.join(os.tmpdir(), `solum-test-${Date.now()}`);
        await solum.creabit(testDir);
    });

    afterAll(async () => {
        // Clean up test directory
        await solum.amputabit(testDir);
    });

    // =========================================================================
    // READING - Text (lege/leget)
    // =========================================================================

    describe('reading text (lege/leget)', () => {
        test('scribet and leget roundtrip (async)', async () => {
            const filePath = path.join(testDir, 'text-roundtrip-async.txt');
            const content = 'Hello, solum!';

            await solum.scribet(filePath, content);
            const result = await solum.leget(filePath);

            expect(result).toBe(content);
        });

        test('scribe and lege roundtrip (sync)', () => {
            const filePath = path.join(testDir, 'text-roundtrip-sync.txt');
            const content = 'Hello, solum sync!';

            solum.scribe(filePath, content);
            const result = solum.lege(filePath);

            expect(result).toBe(content);
        });

        test('handles unicode text', async () => {
            const filePath = path.join(testDir, 'unicode.txt');
            const content = 'Salve, mundus! Hic sunt characteres: \u{1F600} \u{4E2D}\u{6587}';

            await solum.scribet(filePath, content);
            const result = await solum.leget(filePath);

            expect(result).toBe(content);
        });
    });

    // =========================================================================
    // READING - Bytes (hauri/hauriet)
    // =========================================================================

    describe('reading bytes (hauri/hauriet)', () => {
        test('fundet and hauriet roundtrip (async)', async () => {
            const filePath = path.join(testDir, 'bytes-roundtrip-async.bin');
            const content = new Uint8Array([0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]);

            await solum.fundet(filePath, content);
            const result = await solum.hauriet(filePath);

            expect(result).toEqual(content);
        });

        test('funde and hauri roundtrip (sync)', () => {
            const filePath = path.join(testDir, 'bytes-roundtrip-sync.bin');
            const content = new Uint8Array([0xDE, 0xAD, 0xBE, 0xEF]);

            solum.funde(filePath, content);
            const result = solum.hauri(filePath);

            expect(result).toEqual(content);
        });
    });

    // =========================================================================
    // READING - Lines (carpe/carpiet)
    // =========================================================================

    describe('reading lines (carpe/carpiet)', () => {
        test('carpiet plucks lines (async)', async () => {
            const filePath = path.join(testDir, 'lines-async.txt');
            const content = 'line one\nline two\nline three';

            await solum.scribet(filePath, content);
            const lines = await solum.carpiet(filePath);

            expect(lines).toEqual(['line one', 'line two', 'line three']);
        });

        test('carpe plucks lines (sync)', () => {
            const filePath = path.join(testDir, 'lines-sync.txt');
            const content = 'alpha\nbeta\ngamma';

            solum.scribe(filePath, content);
            const lines = solum.carpe(filePath);

            expect(lines).toEqual(['alpha', 'beta', 'gamma']);
        });

        test('handles CRLF line endings', async () => {
            const filePath = path.join(testDir, 'lines-crlf.txt');
            const content = 'one\r\ntwo\r\nthree';

            await solum.scribet(filePath, content);
            const lines = await solum.carpiet(filePath);

            expect(lines).toEqual(['one', 'two', 'three']);
        });
    });

    // =========================================================================
    // WRITING - Append (appone/apponet)
    // =========================================================================

    describe('appending (appone/apponet)', () => {
        test('apponet adds to existing file (async)', async () => {
            const filePath = path.join(testDir, 'append-async.txt');

            await solum.scribet(filePath, 'First');
            await solum.apponet(filePath, 'Second');
            await solum.apponet(filePath, 'Third');

            const result = await solum.leget(filePath);
            expect(result).toBe('FirstSecondThird');
        });

        test('appone adds to existing file (sync)', () => {
            const filePath = path.join(testDir, 'append-sync.txt');

            solum.scribe(filePath, 'A');
            solum.appone(filePath, 'B');
            solum.appone(filePath, 'C');

            const result = solum.lege(filePath);
            expect(result).toBe('ABC');
        });
    });

    // =========================================================================
    // FILE INFO - Existence (exstat)
    // =========================================================================

    describe('existence (exstat)', () => {
        test('exstat returns true for existing file', async () => {
            const filePath = path.join(testDir, 'exists.txt');
            await solum.scribet(filePath, 'test');

            expect(solum.exstat(filePath)).toBe(true);
        });

        test('exstat returns false for non-existing file', () => {
            const filePath = path.join(testDir, 'does-not-exist.txt');
            expect(solum.exstat(filePath)).toBe(false);
        });

        test('exstat returns true for directory', () => {
            expect(solum.exstat(testDir)).toBe(true);
        });
    });

    // =========================================================================
    // FILE INFO - Details (describe/describet)
    // =========================================================================

    describe('file details (describe/describet)', () => {
        test('describet returns file status (async)', async () => {
            const filePath = path.join(testDir, 'status-async.txt');
            const content = 'some content here';
            await solum.scribet(filePath, content);

            const status = await solum.describet(filePath);

            expect(status.magnitudo).toBe(content.length);
            expect(status.estDirectorii).toBe(false);
            expect(status.estVinculum).toBe(false);
            expect(status.modificatum).toBeGreaterThan(0);
        });

        test('describe returns file status (sync)', () => {
            const filePath = path.join(testDir, 'status-sync.txt');
            const content = 'test content';
            solum.scribe(filePath, content);

            const status = solum.describe(filePath);

            expect(status.magnitudo).toBe(content.length);
            expect(status.estDirectorii).toBe(false);
        });

        test('describet identifies directory', async () => {
            const status = await solum.describet(testDir);

            expect(status.estDirectorii).toBe(true);
            expect(status.estVinculum).toBe(false);
        });
    });

    // =========================================================================
    // FILE INFO - Symlinks (sequere/sequetur)
    // =========================================================================

    describe('symlinks (sequere/sequetur)', () => {
        test('sequetur follows symlink (async)', async () => {
            const target = path.join(testDir, 'link-target.txt');
            const link = path.join(testDir, 'test-link');

            await solum.scribet(target, 'target content');
            const fsSync = await import('node:fs');
            fsSync.symlinkSync(target, link);

            const resolved = await solum.sequetur(link);
            expect(resolved).toBe(target);

            // Cleanup
            fsSync.unlinkSync(link);
        });

        test('sequere follows symlink (sync)', async () => {
            const target = path.join(testDir, 'link-target-sync.txt');
            const link = path.join(testDir, 'test-link-sync');

            solum.scribe(target, 'target content');
            const fsSync = await import('node:fs');
            fsSync.symlinkSync(target, link);

            const resolved = solum.sequere(link);
            expect(resolved).toBe(target);

            // Cleanup
            fsSync.unlinkSync(link);
        });
    });

    // =========================================================================
    // FILE OPERATIONS - Delete (dele/delet)
    // =========================================================================

    describe('delete file (dele/delet)', () => {
        test('delet removes file (async)', async () => {
            const filePath = path.join(testDir, 'to-delete-async.txt');
            await solum.scribet(filePath, 'delete me');

            expect(solum.exstat(filePath)).toBe(true);
            await solum.delet(filePath);
            expect(solum.exstat(filePath)).toBe(false);
        });

        test('dele removes file (sync)', () => {
            const filePath = path.join(testDir, 'to-delete-sync.txt');
            solum.scribe(filePath, 'delete me');

            expect(solum.exstat(filePath)).toBe(true);
            solum.dele(filePath);
            expect(solum.exstat(filePath)).toBe(false);
        });
    });

    // =========================================================================
    // FILE OPERATIONS - Copy (exscribe/exscribet)
    // =========================================================================

    describe('copy file (exscribe/exscribet)', () => {
        test('exscribet copies file (async)', async () => {
            const fons = path.join(testDir, 'copy-src-async.txt');
            const destinatio = path.join(testDir, 'copy-dest-async.txt');
            const content = 'copy this content';

            await solum.scribet(fons, content);
            await solum.exscribet(fons, destinatio);

            expect(await solum.leget(destinatio)).toBe(content);
            expect(solum.exstat(fons)).toBe(true); // source still exists
        });

        test('exscribe copies file (sync)', () => {
            const fons = path.join(testDir, 'copy-src-sync.txt');
            const destinatio = path.join(testDir, 'copy-dest-sync.txt');
            const content = 'sync copy';

            solum.scribe(fons, content);
            solum.exscribe(fons, destinatio);

            expect(solum.lege(destinatio)).toBe(content);
            expect(solum.exstat(fons)).toBe(true);
        });
    });

    // =========================================================================
    // FILE OPERATIONS - Move (move/movet)
    // =========================================================================

    describe('move file (move/movet)', () => {
        test('movet renames file (async)', async () => {
            const fons = path.join(testDir, 'move-src-async.txt');
            const destinatio = path.join(testDir, 'move-dest-async.txt');
            const content = 'move this content';

            await solum.scribet(fons, content);
            await solum.movet(fons, destinatio);

            expect(await solum.leget(destinatio)).toBe(content);
            expect(solum.exstat(fons)).toBe(false); // source removed
        });

        test('move renames file (sync)', () => {
            const fons = path.join(testDir, 'move-src-sync.txt');
            const destinatio = path.join(testDir, 'move-dest-sync.txt');
            const content = 'sync move';

            solum.scribe(fons, content);
            solum.move(fons, destinatio);

            expect(solum.lege(destinatio)).toBe(content);
            expect(solum.exstat(fons)).toBe(false);
        });
    });

    // =========================================================================
    // FILE OPERATIONS - Touch (tange/tanget)
    // =========================================================================

    describe('touch file (tange/tanget)', () => {
        test('tanget creates empty file if not exists (async)', async () => {
            const filePath = path.join(testDir, 'touched-async.txt');

            expect(solum.exstat(filePath)).toBe(false);
            await solum.tanget(filePath);
            expect(solum.exstat(filePath)).toBe(true);
            expect(await solum.leget(filePath)).toBe('');
        });

        test('tange creates empty file if not exists (sync)', () => {
            const filePath = path.join(testDir, 'touched-sync.txt');

            expect(solum.exstat(filePath)).toBe(false);
            solum.tange(filePath);
            expect(solum.exstat(filePath)).toBe(true);
            expect(solum.lege(filePath)).toBe('');
        });

        test('tanget updates mtime for existing file', async () => {
            const filePath = path.join(testDir, 'touch-existing.txt');
            await solum.scribet(filePath, 'content');

            const mtimeBefore = (await solum.describet(filePath)).modificatum;
            await new Promise(r => setTimeout(r, 10));
            await solum.tanget(filePath);
            const mtimeAfter = (await solum.describet(filePath)).modificatum;

            expect(mtimeAfter).toBeGreaterThanOrEqual(mtimeBefore);
        });
    });

    // =========================================================================
    // DIRECTORY OPERATIONS - Create (crea/creabit)
    // =========================================================================

    describe('create directory (crea/creabit)', () => {
        test('creabit creates nested directories (async)', async () => {
            const nested = path.join(testDir, 'async-a', 'b', 'c');
            await solum.creabit(nested);

            const status = await solum.describet(nested);
            expect(status.estDirectorii).toBe(true);
        });

        test('crea creates nested directories (sync)', () => {
            const nested = path.join(testDir, 'sync-a', 'b', 'c');
            solum.crea(nested);

            const status = solum.describe(nested);
            expect(status.estDirectorii).toBe(true);
        });
    });

    // =========================================================================
    // DIRECTORY OPERATIONS - List (enumera/enumerabit)
    // =========================================================================

    describe('list directory (enumera/enumerabit)', () => {
        test('enumerabit lists directory contents (async)', async () => {
            const dir = path.join(testDir, 'list-dir-async');
            await solum.creabit(dir);
            await solum.scribet(path.join(dir, 'file1.txt'), 'a');
            await solum.scribet(path.join(dir, 'file2.txt'), 'b');

            const entries = await solum.enumerabit(dir);

            expect(entries.sort()).toEqual(['file1.txt', 'file2.txt']);
        });

        test('enumera lists directory contents (sync)', () => {
            const dir = path.join(testDir, 'list-dir-sync');
            solum.crea(dir);
            solum.scribe(path.join(dir, 'alpha.txt'), 'a');
            solum.scribe(path.join(dir, 'beta.txt'), 'b');

            const entries = solum.enumera(dir);

            expect(entries.sort()).toEqual(['alpha.txt', 'beta.txt']);
        });
    });

    // =========================================================================
    // DIRECTORY OPERATIONS - Prune (amputa/amputabit)
    // =========================================================================

    describe('prune directory (amputa/amputabit)', () => {
        test('amputabit removes directory tree (async)', async () => {
            const dir = path.join(testDir, 'prune-async');
            await solum.creabit(path.join(dir, 'sub'));
            await solum.scribet(path.join(dir, 'file.txt'), 'a');
            await solum.scribet(path.join(dir, 'sub', 'nested.txt'), 'b');

            expect(solum.exstat(dir)).toBe(true);
            await solum.amputabit(dir);
            expect(solum.exstat(dir)).toBe(false);
        });

        test('amputa removes directory tree (sync)', () => {
            const dir = path.join(testDir, 'prune-sync');
            solum.crea(path.join(dir, 'sub'));
            solum.scribe(path.join(dir, 'file.txt'), 'a');
            solum.scribe(path.join(dir, 'sub', 'nested.txt'), 'b');

            expect(solum.exstat(dir)).toBe(true);
            solum.amputa(dir);
            expect(solum.exstat(dir)).toBe(false);
        });

        test('amputa removes empty directory', () => {
            const dir = path.join(testDir, 'empty-prune');
            solum.crea(dir);

            expect(solum.exstat(dir)).toBe(true);
            solum.amputa(dir);
            expect(solum.exstat(dir)).toBe(false);
        });
    });

    // =========================================================================
    // PATH UTILITIES
    // =========================================================================

    describe('path utilities', () => {
        test('iunge joins path segments', () => {
            expect(solum.iunge(['a', 'b', 'c'])).toBe(path.join('a', 'b', 'c'));
            expect(solum.iunge(['/root', 'sub', 'file.txt'])).toBe('/root/sub/file.txt');
        });

        test('directorium extracts directory', () => {
            expect(solum.directorium('/a/b/c.txt')).toBe('/a/b');
            expect(solum.directorium('/a/b/')).toBe('/a');
        });

        test('basis extracts filename', () => {
            expect(solum.basis('/a/b/c.txt')).toBe('c.txt');
            expect(solum.basis('/a/b/')).toBe('b');
        });

        test('extensio extracts extension', () => {
            expect(solum.extensio('/a/b/c.txt')).toBe('.txt');
            expect(solum.extensio('/a/b/c.tar.gz')).toBe('.gz');
            expect(solum.extensio('/a/b/c')).toBe('');
        });

        test('absolve resolves to absolute path', () => {
            const resolved = solum.absolve('relative/path');
            expect(path.isAbsolute(resolved)).toBe(true);
        });

        test('domus returns home directory', () => {
            expect(solum.domus()).toBe(os.homedir());
            expect(solum.domus().length).toBeGreaterThan(0);
        });

        test('temporarium returns temp directory', () => {
            expect(solum.temporarium()).toBe(os.tmpdir());
            expect(solum.temporarium().length).toBeGreaterThan(0);
        });
    });
});
