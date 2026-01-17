import { test, expect, describe, beforeEach, afterEach } from 'bun:test';
import { processus } from './processus';

describe('processus HAL', () => {
    describe('exsequi', () => {
        test('runs command and returns output', () => {
            const output = processus.exsequi('echo hello');
            expect(output.trim()).toBe('hello');
        });

        test('handles commands with multiple words', () => {
            const output = processus.exsequi('echo hello world');
            expect(output.trim()).toBe('hello world');
        });
    });

    describe('exsequiCodem', () => {
        test('returns exit code 0 for successful command', () => {
            const code = processus.exsequiCodem('true');
            expect(code).toBe(0);
        });

        test('returns non-zero exit code for failed command', () => {
            const code = processus.exsequiCodem('false');
            expect(code).toBe(1);
        });
    });

    describe('genera', () => {
        test('spawns process and returns PID', () => {
            const pid = processus.genera(['sleep', '0.1']);
            expect(typeof pid).toBe('number');
            expect(pid).toBeGreaterThan(0);
            expect(Number.isInteger(pid)).toBe(true);
        });

        test('spawned process runs in specified cwd', async () => {
            const marker = `/tmp/_processus_test_cwd_${Date.now()}`;
            // pwd writes cwd to file
            processus.genera(['sh', '-c', `pwd > ${marker}`], '/tmp');
            await Bun.sleep(100);
            const content = await Bun.file(marker).text();
            // macOS resolves /tmp to /private/tmp
            expect(content.trim()).toMatch(/^(\/tmp|\/private\/tmp)$/);
            await Bun.$`rm -f ${marker}`;
        });

        test('spawned process receives custom env', async () => {
            const marker = `/tmp/_processus_test_env_${Date.now()}`;
            processus.genera(
                ['sh', '-c', `echo $TEST_VAR > ${marker}`],
                undefined,
                { TEST_VAR: 'hello_from_genera' }
            );
            await Bun.sleep(100);
            const content = await Bun.file(marker).text();
            expect(content.trim()).toBe('hello_from_genera');
            await Bun.$`rm -f ${marker}`;
        });
    });

    describe('env', () => {
        test('returns environment variable value', () => {
            // PATH should always be set
            const path = processus.env('PATH');
            expect(path).not.toBe(null);
            expect(typeof path).toBe('string');
        });

        test('returns null for non-existent variable', () => {
            const result = processus.env('DEFINITELY_NOT_A_REAL_ENV_VAR_12345');
            expect(result).toBe(null);
        });
    });

    describe('envVel', () => {
        test('returns environment variable value when set', () => {
            const path = processus.envVel('PATH', 'default');
            expect(path).not.toBe('default');
        });

        test('returns default when variable not set', () => {
            const result = processus.envVel('DEFINITELY_NOT_A_REAL_ENV_VAR_12345', 'my-default');
            expect(result).toBe('my-default');
        });
    });

    describe('poneEnv', () => {
        const testVarName = '_PROCESSUS_TEST_VAR_' + Date.now();

        afterEach(() => {
            // Clean up
            delete Bun.env[testVarName];
        });

        test('sets environment variable', () => {
            expect(processus.env(testVarName)).toBe(null);

            processus.poneEnv(testVarName, 'test-value');

            expect(processus.env(testVarName)).toBe('test-value');
        });

        test('overwrites existing variable', () => {
            processus.poneEnv(testVarName, 'first');
            processus.poneEnv(testVarName, 'second');

            expect(processus.env(testVarName)).toBe('second');
        });
    });

    describe('cwd', () => {
        test('returns current working directory', () => {
            const cwd = processus.cwd();
            expect(typeof cwd).toBe('string');
            expect(cwd.length).toBeGreaterThan(0);
            expect(cwd.startsWith('/')).toBe(true); // Absolute path
        });

        test('matches process.cwd()', () => {
            expect(processus.cwd()).toBe(process.cwd());
        });
    });

    describe('pid', () => {
        test('returns process ID', () => {
            const pid = processus.pid();
            expect(typeof pid).toBe('number');
            expect(pid).toBeGreaterThan(0);
            expect(Number.isInteger(pid)).toBe(true);
        });

        test('matches process.pid', () => {
            expect(processus.pid()).toBe(process.pid);
        });
    });

    describe('argv', () => {
        test('returns command line arguments as array', () => {
            const argv = processus.argv();
            expect(Array.isArray(argv)).toBe(true);
        });

        test('returns strings', () => {
            const argv = processus.argv();
            for (const arg of argv) {
                expect(typeof arg).toBe('string');
            }
        });

        // Note: The actual contents depend on how the test is run,
        // but we can verify it excludes the first two Bun.argv elements
        test('excludes bun executable and script path', () => {
            const argv = processus.argv();
            const fullArgv = Bun.argv;
            // processus.argv() should be Bun.argv.slice(2)
            expect(argv).toEqual(fullArgv.slice(2));
        });
    });

    describe('chdir', () => {
        let originalCwd: string;

        beforeEach(() => {
            originalCwd = process.cwd();
        });

        afterEach(() => {
            process.chdir(originalCwd);
        });

        test('changes current working directory', () => {
            processus.chdir('/tmp');
            // macOS resolves /tmp to /private/tmp
            expect(processus.cwd()).toMatch(/^(\/tmp|\/private\/tmp)$/);
        });
    });

    // Note: exi() cannot be easily tested as it calls process.exit()
    // which would terminate the test runner
});
