import { test, expect, describe, spyOn, beforeEach, afterEach } from 'bun:test';
import { consolum } from './consolum';
import * as fs from 'node:fs';
import * as tty from 'node:tty';

describe('consolum HAL', () => {
    // =========================================================================
    // STDOUT - Bytes (funde/fundet)
    // =========================================================================

    describe('funde/fundet (stdout bytes)', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('funde writes bytes to stdout (fd 1)', () => {
            const data = new Uint8Array([72, 105]);
            consolum.funde(data);
            expect(writeSyncSpy).toHaveBeenCalledWith(1, data);
        });

        test('fundet writes bytes to stdout (async)', async () => {
            const data = new Uint8Array([72, 105]);
            await consolum.fundet(data);
            expect(writeSyncSpy).toHaveBeenCalledWith(1, data);
        });
    });

    // =========================================================================
    // STDOUT - Text with Newline (scribe/scribet)
    // =========================================================================

    describe('scribe/scribet (stdout line)', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('scribe writes text with newline to stdout', () => {
            consolum.scribe('hello');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, 'hello\n');
        });

        test('scribet writes text with newline to stdout (async)', async () => {
            await consolum.scribet('hello');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, 'hello\n');
        });
    });

    // =========================================================================
    // STDOUT - Text without Newline (dic/dicet)
    // =========================================================================

    describe('dic/dicet (stdout partial)', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('dic writes text without newline to stdout', () => {
            consolum.dic('.');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, '.');
        });

        test('dicet writes text without newline to stdout (async)', async () => {
            await consolum.dicet('.');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, '.');
        });
    });

    // =========================================================================
    // STDERR - Warning (mone/monet)
    // =========================================================================

    describe('mone/monet (stderr warning)', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('mone writes text with newline to stderr (fd 2)', () => {
            consolum.mone('warning!');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'warning!\n');
        });

        test('monet writes text with newline to stderr (async)', async () => {
            await consolum.monet('warning!');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'warning!\n');
        });
    });

    // =========================================================================
    // DEBUG (vide/videbit)
    // =========================================================================

    describe('vide/videbit (debug output)', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('vide writes debug text with newline to stderr', () => {
            consolum.vide('debug info');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'debug info\n');
        });

        test('videbit writes debug text with newline to stderr (async)', async () => {
            await consolum.videbit('debug info');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'debug info\n');
        });
    });

    // =========================================================================
    // STDIN - Bytes (hauri/hauriet)
    // =========================================================================

    describe('hauri/hauriet (stdin bytes)', () => {
        test('hauri returns Uint8Array', () => {
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = consolum.hauri(10);
            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBe(0); // EOF returns empty

            readSyncSpy.mockRestore();
        });

        test('hauriet returns Uint8Array (async)', async () => {
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = await consolum.hauriet(10);
            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBe(0);

            readSyncSpy.mockRestore();
        });
    });

    // =========================================================================
    // STDIN - Text (lege/leget)
    // =========================================================================

    describe('lege/leget (stdin line)', () => {
        test('lege returns string', () => {
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = consolum.lege();
            expect(typeof result).toBe('string');

            readSyncSpy.mockRestore();
        });

        test('leget returns string (async)', async () => {
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = await consolum.leget();
            expect(typeof result).toBe('string');

            readSyncSpy.mockRestore();
        });
    });

    // =========================================================================
    // TTY Detection
    // =========================================================================

    describe('TTY detection', () => {
        test('estTerminale returns a boolean', () => {
            const result = consolum.estTerminale();
            expect(typeof result).toBe('boolean');
        });

        test('estTerminaleOutput returns a boolean', () => {
            const result = consolum.estTerminaleOutput();
            expect(typeof result).toBe('boolean');
        });

        test('estTerminale checks stdin (fd 0)', () => {
            const isattySpy = spyOn(tty, 'isatty');

            isattySpy.mockReturnValue(true);
            expect(consolum.estTerminale()).toBe(true);

            isattySpy.mockReturnValue(false);
            expect(consolum.estTerminale()).toBe(false);

            isattySpy.mockRestore();
        });

        test('estTerminaleOutput checks stdout (fd 1)', () => {
            const isattySpy = spyOn(tty, 'isatty');

            isattySpy.mockReturnValue(true);
            expect(consolum.estTerminaleOutput()).toBe(true);

            isattySpy.mockReturnValue(false);
            expect(consolum.estTerminaleOutput()).toBe(false);

            isattySpy.mockRestore();
        });
    });
});
