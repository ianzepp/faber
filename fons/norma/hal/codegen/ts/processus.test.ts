import { test, expect, describe, beforeEach, afterEach } from 'bun:test';
import { processus } from './processus';

describe('processus HAL', () => {
    // =========================================================================
    // SPAWN - Attached (genera)
    // =========================================================================

    describe('genera (spawn attached)', () => {
        test('returns Subprocessus with pid', () => {
            const proc = processus.genera(['echo', 'hello']);
            expect(typeof proc.pid).toBe('number');
            expect(proc.pid).toBeGreaterThan(0);
        });

        test('returns Subprocessus with expiravit promise', async () => {
            const proc = processus.genera(['echo', 'hello']);
            expect(proc.expiravit).toBeInstanceOf(Promise);
            const exitCode = await proc.expiravit;
            expect(exitCode).toBe(0);
        });

        test('expiravit resolves to exit code', async () => {
            const proc = processus.genera(['sh', '-c', 'exit 42']);
            const exitCode = await proc.expiravit;
            expect(exitCode).toBe(42);
        });

        test('spawned process runs in specified directorium', async () => {
            const marker = `/tmp/_processus_test_cwd_${Date.now()}`;
            const proc = processus.genera(['sh', '-c', `pwd > ${marker}`], '/tmp');
            await proc.expiravit;
            const content = await Bun.file(marker).text();
            expect(content.trim()).toMatch(/^(\/tmp|\/private\/tmp)$/);
            await Bun.$`rm -f ${marker}`;
        });

        test('spawned process receives custom ambitus', async () => {
            const marker = `/tmp/_processus_test_env_${Date.now()}`;
            const proc = processus.genera(
                ['sh', '-c', `echo $TEST_VAR > ${marker}`],
                undefined,
                { TEST_VAR: 'hello_from_genera' }
            );
            await proc.expiravit;
            const content = await Bun.file(marker).text();
            expect(content.trim()).toBe('hello_from_genera');
            await Bun.$`rm -f ${marker}`;
        });
    });

    // =========================================================================
    // SPAWN - Detached (generabit)
    // =========================================================================

    describe('generabit (spawn detached)', () => {
        test('returns PID', async () => {
            const pid = await processus.generabit(['sleep', '0.01']);
            expect(typeof pid).toBe('number');
            expect(pid).toBeGreaterThan(0);
        });

        test('process runs independently', async () => {
            const marker = `/tmp/_processus_test_detached_${Date.now()}`;
            const pid = await processus.generabit(['sh', '-c', `echo detached > ${marker}`]);
            expect(pid).toBeGreaterThan(0);
            await Bun.sleep(100);
            const content = await Bun.file(marker).text();
            expect(content.trim()).toBe('detached');
            await Bun.$`rm -f ${marker}`;
        });

        test('spawned process runs in specified directorium', async () => {
            const marker = `/tmp/_processus_test_detached_cwd_${Date.now()}`;
            await processus.generabit(['sh', '-c', `pwd > ${marker}`], '/tmp');
            await Bun.sleep(100);
            const content = await Bun.file(marker).text();
            expect(content.trim()).toMatch(/^(\/tmp|\/private\/tmp)$/);
            await Bun.$`rm -f ${marker}`;
        });
    });

    // =========================================================================
    // SHELL EXECUTION - Sync (exsequi)
    // =========================================================================

    describe('exsequi (shell exec sync)', () => {
        test('runs command and returns output', () => {
            const output = processus.exsequi('echo hello');
            expect(output.trim()).toBe('hello');
        });

        test('handles commands with pipes', () => {
            const output = processus.exsequi('echo hello world | tr a-z A-Z');
            expect(output.trim()).toBe('HELLO WORLD');
        });

        test('handles commands with multiple statements', () => {
            const output = processus.exsequi('echo first && echo second');
            expect(output.trim()).toBe('first\nsecond');
        });
    });

    // =========================================================================
    // SHELL EXECUTION - Async (exsequetur)
    // =========================================================================

    describe('exsequetur (shell exec async)', () => {
        test('runs command and returns output', async () => {
            const output = await processus.exsequetur('echo hello');
            expect(output.trim()).toBe('hello');
        });

        test('handles commands with pipes', async () => {
            const output = await processus.exsequetur('echo hello world | tr a-z A-Z');
            expect(output.trim()).toBe('HELLO WORLD');
        });

        test('handles long-running commands', async () => {
            const output = await processus.exsequetur('sleep 0.05 && echo done');
            expect(output.trim()).toBe('done');
        });
    });

    // =========================================================================
    // ENVIRONMENT - Read (lege)
    // =========================================================================

    describe('lege (read env)', () => {
        test('returns environment variable value', () => {
            const path = processus.lege('PATH');
            expect(path).not.toBe(null);
            expect(typeof path).toBe('string');
        });

        test('returns null for non-existent variable', () => {
            const result = processus.lege('DEFINITELY_NOT_A_REAL_ENV_VAR_12345');
            expect(result).toBe(null);
        });
    });

    // =========================================================================
    // ENVIRONMENT - Write (scribe)
    // =========================================================================

    describe('scribe (write env)', () => {
        const testVarName = '_PROCESSUS_TEST_VAR_' + Date.now();

        afterEach(() => {
            delete Bun.env[testVarName];
        });

        test('sets environment variable', () => {
            expect(processus.lege(testVarName)).toBe(null);
            processus.scribe(testVarName, 'test-value');
            expect(processus.lege(testVarName)).toBe('test-value');
        });

        test('overwrites existing variable', () => {
            processus.scribe(testVarName, 'first');
            processus.scribe(testVarName, 'second');
            expect(processus.lege(testVarName)).toBe('second');
        });
    });

    // =========================================================================
    // PROCESS INFO - Working Directory
    // =========================================================================

    describe('directorium (get cwd)', () => {
        test('returns current working directory', () => {
            const dir = processus.directorium();
            expect(typeof dir).toBe('string');
            expect(dir.length).toBeGreaterThan(0);
            expect(dir.startsWith('/')).toBe(true);
        });

        test('matches process.cwd()', () => {
            expect(processus.directorium()).toBe(process.cwd());
        });
    });

    describe('muta (change cwd)', () => {
        let originalCwd: string;

        beforeEach(() => {
            originalCwd = process.cwd();
        });

        afterEach(() => {
            process.chdir(originalCwd);
        });

        test('changes current working directory', () => {
            processus.muta('/tmp');
            expect(processus.directorium()).toMatch(/^(\/tmp|\/private\/tmp)$/);
        });
    });

    // =========================================================================
    // PROCESS INFO - Identity
    // =========================================================================

    describe('identitas (get pid)', () => {
        test('returns process ID', () => {
            const pid = processus.identitas();
            expect(typeof pid).toBe('number');
            expect(pid).toBeGreaterThan(0);
            expect(Number.isInteger(pid)).toBe(true);
        });

        test('matches process.pid', () => {
            expect(processus.identitas()).toBe(process.pid);
        });
    });

    // =========================================================================
    // PROCESS INFO - Arguments
    // =========================================================================

    describe('argumenta (get argv)', () => {
        test('returns command line arguments as array', () => {
            const args = processus.argumenta();
            expect(Array.isArray(args)).toBe(true);
        });

        test('returns strings', () => {
            const args = processus.argumenta();
            for (const arg of args) {
                expect(typeof arg).toBe('string');
            }
        });

        test('excludes bun executable and script path', () => {
            const args = processus.argumenta();
            const fullArgv = Bun.argv;
            expect(args).toEqual(fullArgv.slice(2));
        });
    });

    // =========================================================================
    // EXIT
    // =========================================================================

    // Note: exi() cannot be easily tested as it calls process.exit()
    // which would terminate the test runner
});
