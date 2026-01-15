import { test, expect, describe, beforeAll, afterAll } from 'bun:test';
import { solum } from './solum';
import * as path from 'node:path';
import * as os from 'node:os';

describe('solum HAL', () => {
    let testDir: string;

    beforeAll(async () => {
        // Create a unique temp directory for tests
        testDir = path.join(os.tmpdir(), `solum-test-${Date.now()}`);
        await solum.creaDir(testDir);
    });

    afterAll(async () => {
        // Clean up test directory
        await solum.deleArborem(testDir);
    });

    describe('reading and writing text', () => {
        test('scribe and lege roundtrip', async () => {
            const filePath = path.join(testDir, 'text-roundtrip.txt');
            const content = 'Hello, solum!';

            await solum.scribe(filePath, content);
            const result = await solum.lege(filePath);

            expect(result).toBe(content);
        });

        test('handles unicode text', async () => {
            const filePath = path.join(testDir, 'unicode.txt');
            const content = 'Salve, mundus! Hic sunt characteres: \u{1F600} \u{4E2D}\u{6587}';

            await solum.scribe(filePath, content);
            const result = await solum.lege(filePath);

            expect(result).toBe(content);
        });
    });

    describe('reading and writing bytes', () => {
        test('scribeOctetos and legeOctetos roundtrip', async () => {
            const filePath = path.join(testDir, 'bytes-roundtrip.bin');
            const content = new Uint8Array([0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]);

            await solum.scribeOctetos(filePath, content);
            const result = await solum.legeOctetos(filePath);

            expect(result).toEqual(content);
        });
    });

    describe('appending', () => {
        test('appone adds to existing file', async () => {
            const filePath = path.join(testDir, 'append.txt');

            await solum.scribe(filePath, 'First');
            await solum.appone(filePath, 'Second');
            await solum.appone(filePath, 'Third');

            const result = await solum.lege(filePath);
            expect(result).toBe('FirstSecondThird');
        });
    });

    describe('streaming read', () => {
        test('legens iterates file chunks', async () => {
            const filePath = path.join(testDir, 'stream-read.txt');
            const content = 'Streaming content for reading';

            await solum.scribe(filePath, content);

            const chunks: Uint8Array[] = [];
            for await (const chunk of solum.legens(filePath)) {
                chunks.push(chunk);
            }

            const combined = new TextDecoder().decode(
                new Uint8Array(chunks.flatMap(c => [...c]))
            );
            expect(combined).toBe(content);
        });
    });

    describe('file info', () => {
        test('exstat returns true for existing file', async () => {
            const filePath = path.join(testDir, 'exists.txt');
            await solum.scribe(filePath, 'test');

            expect(solum.exstat(filePath)).toBe(true);
        });

        test('exstat returns false for non-existing file', () => {
            const filePath = path.join(testDir, 'does-not-exist.txt');
            expect(solum.exstat(filePath)).toBe(false);
        });

        test('estLimae returns true for files', async () => {
            const filePath = path.join(testDir, 'is-file.txt');
            await solum.scribe(filePath, 'test');

            expect(solum.estLimae(filePath)).toBe(true);
            expect(solum.estDirectorii(filePath)).toBe(false);
        });

        test('estDirectorii returns true for directories', () => {
            expect(solum.estDirectorii(testDir)).toBe(true);
            expect(solum.estLimae(testDir)).toBe(false);
        });

        test('magnitudo returns file size', async () => {
            const filePath = path.join(testDir, 'size.txt');
            const content = 'exactly 20 bytes!!!';

            await solum.scribe(filePath, content);
            const size = await solum.magnitudo(filePath);

            expect(size).toBe(19);
        });

        test('modificatum returns modification time', async () => {
            const filePath = path.join(testDir, 'mtime.txt');
            const before = Date.now();

            await solum.scribe(filePath, 'test');
            const mtime = await solum.modificatum(filePath);
            const after = Date.now();

            expect(mtime).toBeGreaterThanOrEqual(before);
            expect(mtime).toBeLessThanOrEqual(after);
        });
    });

    describe('file operations', () => {
        test('dele removes file', async () => {
            const filePath = path.join(testDir, 'to-delete.txt');
            await solum.scribe(filePath, 'delete me');

            expect(solum.exstat(filePath)).toBe(true);
            await solum.dele(filePath);
            expect(solum.exstat(filePath)).toBe(false);
        });

        test('copia copies file', async () => {
            const src = path.join(testDir, 'copy-src.txt');
            const dest = path.join(testDir, 'copy-dest.txt');
            const content = 'copy this content';

            await solum.scribe(src, content);
            await solum.copia(src, dest);

            expect(await solum.lege(dest)).toBe(content);
            expect(solum.exstat(src)).toBe(true); // source still exists
        });

        test('move renames file', async () => {
            const src = path.join(testDir, 'move-src.txt');
            const dest = path.join(testDir, 'move-dest.txt');
            const content = 'move this content';

            await solum.scribe(src, content);
            await solum.move(src, dest);

            expect(await solum.lege(dest)).toBe(content);
            expect(solum.exstat(src)).toBe(false); // source removed
        });

        test('tange creates empty file if not exists', async () => {
            const filePath = path.join(testDir, 'touched.txt');

            expect(solum.exstat(filePath)).toBe(false);
            await solum.tange(filePath);
            expect(solum.exstat(filePath)).toBe(true);
            expect(await solum.lege(filePath)).toBe('');
        });

        test('tange updates mtime for existing file', async () => {
            const filePath = path.join(testDir, 'touch-existing.txt');
            await solum.scribe(filePath, 'content');

            const mtimeBefore = await solum.modificatum(filePath);
            // Wait a bit to ensure time difference
            await new Promise(r => setTimeout(r, 10));
            await solum.tange(filePath);
            const mtimeAfter = await solum.modificatum(filePath);

            expect(mtimeAfter).toBeGreaterThanOrEqual(mtimeBefore);
        });
    });

    describe('directory operations', () => {
        test('creaDir creates nested directories', async () => {
            const nested = path.join(testDir, 'a', 'b', 'c');
            await solum.creaDir(nested);

            expect(solum.estDirectorii(nested)).toBe(true);
        });

        test('elenca lists directory contents', async () => {
            const dir = path.join(testDir, 'list-dir');
            await solum.creaDir(dir);
            await solum.scribe(path.join(dir, 'file1.txt'), 'a');
            await solum.scribe(path.join(dir, 'file2.txt'), 'b');

            const entries = await solum.elenca(dir);

            expect(entries.sort()).toEqual(['file1.txt', 'file2.txt']);
        });

        test('deleDir removes empty directory', async () => {
            const dir = path.join(testDir, 'empty-dir');
            await solum.creaDir(dir);

            expect(solum.estDirectorii(dir)).toBe(true);
            await solum.deleDir(dir);
            expect(solum.estDirectorii(dir)).toBe(false);
        });

        test('deleArborem removes directory tree', async () => {
            const dir = path.join(testDir, 'tree-to-delete');
            await solum.creaDir(path.join(dir, 'sub'));
            await solum.scribe(path.join(dir, 'file.txt'), 'a');
            await solum.scribe(path.join(dir, 'sub', 'nested.txt'), 'b');

            expect(solum.estDirectorii(dir)).toBe(true);
            await solum.deleArborem(dir);
            expect(solum.estDirectorii(dir)).toBe(false);
        });
    });

    describe('ambula (recursive walk)', () => {
        test('ambula recursively lists all files', async () => {
            const dir = path.join(testDir, 'walk-dir');
            await solum.creaDir(path.join(dir, 'sub1'));
            await solum.creaDir(path.join(dir, 'sub2'));
            await solum.scribe(path.join(dir, 'root.txt'), 'a');
            await solum.scribe(path.join(dir, 'sub1', 'one.txt'), 'b');
            await solum.scribe(path.join(dir, 'sub2', 'two.txt'), 'c');

            const files: string[] = [];
            for await (const file of solum.ambula(dir)) {
                files.push(file);
            }

            expect(files.sort()).toEqual([
                path.join(dir, 'root.txt'),
                path.join(dir, 'sub1', 'one.txt'),
                path.join(dir, 'sub2', 'two.txt'),
            ]);
        });
    });

    describe('path utilities', () => {
        test('iunge joins path segments', () => {
            expect(solum.iunge(['a', 'b', 'c'])).toBe(path.join('a', 'b', 'c'));
            expect(solum.iunge(['/root', 'sub', 'file.txt'])).toBe('/root/sub/file.txt');
        });

        test('dir extracts directory', () => {
            expect(solum.dir('/a/b/c.txt')).toBe('/a/b');
            expect(solum.dir('/a/b/')).toBe('/a');
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

        test('temp returns temp directory', () => {
            expect(solum.temp()).toBe(os.tmpdir());
            expect(solum.temp().length).toBeGreaterThan(0);
        });
    });
});
