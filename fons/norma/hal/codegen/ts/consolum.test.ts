import { test, expect, describe, mock, spyOn, beforeEach, afterEach } from 'bun:test';
import { consolum } from './consolum';
import * as fs from 'node:fs';
import * as tty from 'node:tty';

describe('consolum HAL', () => {
    describe('stdout functions', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('fundeTextum writes to stdout (fd 1)', () => {
            consolum.fundeTextum('hello');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, 'hello');
        });

        test('fundeLineam writes text with newline to stdout', () => {
            consolum.fundeLineam('hello');
            expect(writeSyncSpy).toHaveBeenCalledWith(1, 'hello\n');
        });

        test('fundeOctetos writes bytes to stdout', () => {
            const data = new Uint8Array([72, 105]);
            consolum.fundeOctetos(data);
            expect(writeSyncSpy).toHaveBeenCalledWith(1, data);
        });
    });

    describe('stderr functions', () => {
        let writeSyncSpy: ReturnType<typeof spyOn>;

        beforeEach(() => {
            writeSyncSpy = spyOn(fs, 'writeSync').mockImplementation(() => 0);
        });

        afterEach(() => {
            writeSyncSpy.mockRestore();
        });

        test('errorTextum writes to stderr (fd 2)', () => {
            consolum.errorTextum('error!');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'error!');
        });

        test('errorLineam writes text with newline to stderr', () => {
            consolum.errorLineam('error!');
            expect(writeSyncSpy).toHaveBeenCalledWith(2, 'error!\n');
        });

        test('errorOctetos writes bytes to stderr', () => {
            const data = new Uint8Array([69, 114, 114]);
            consolum.errorOctetos(data);
            expect(writeSyncSpy).toHaveBeenCalledWith(2, data);
        });
    });

    describe('TTY detection', () => {
        test('estTerminale returns a boolean', () => {
            const result = consolum.estTerminale();
            expect(typeof result).toBe('boolean');
        });

        test('estTerminaleOutput returns a boolean', () => {
            const result = consolum.estTerminaleOutput();
            expect(typeof result).toBe('boolean');
        });

        // In a test environment, stdin/stdout are typically not TTYs
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

    // Note: stdin tests (hauriOctetos, hauriLineam) are difficult to test
    // without complex mocking of readSync and piping. In practice, these
    // functions would be tested via integration tests or manual verification.
    describe('stdin functions', () => {
        test('hauriOctetos returns Uint8Array', () => {
            // Mock readSync to return 0 bytes (EOF)
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = consolum.hauriOctetos(10);
            expect(result).toBeInstanceOf(Uint8Array);
            expect(result.length).toBe(0); // EOF returns empty

            readSyncSpy.mockRestore();
        });

        test('hauriLineam returns string', () => {
            // Mock readSync to simulate EOF immediately
            const readSyncSpy = spyOn(fs, 'readSync').mockReturnValue(0);

            const result = consolum.hauriLineam();
            expect(typeof result).toBe('string');

            readSyncSpy.mockRestore();
        });
    });
});
