/**
 * Target language verification utilities.
 *
 * Validates that generated code is syntactically correct for each target.
 */

import { writeFileSync, unlinkSync, mkdtempSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';
import type { ExecutableTarget } from '../shared';

export interface VerifyResult {
    valid: boolean;
    error?: string;
}

/**
 * Verify TypeScript syntax using tsc --noEmit.
 */
function verifyTypeScript(code: string): VerifyResult {
    const dir = mkdtempSync(join(tmpdir(), 'faber-verify-'));
    const filePath = join(dir, 'test.ts');

    try {
        writeFileSync(filePath, code);

        const result = Bun.spawnSync(['tsc', '--noEmit', '--skipLibCheck', filePath], {
            stderr: 'pipe',
            stdout: 'pipe',
        });

        if (result.exitCode !== 0) {
            return {
                valid: false,
                error: result.stderr.toString() || result.stdout.toString(),
            };
        }

        return { valid: true };
    }
    catch (err) {
        return { valid: false, error: err instanceof Error ? err.message : String(err) };
    }
    finally {
        try { unlinkSync(filePath); } catch {}
    }
}

/**
 * Verify Python syntax using ast.parse.
 */
function verifyPython(code: string): VerifyResult {
    const result = Bun.spawnSync(['python3', '-c', `import ast; ast.parse('''${code.replace(/'/g, "\\'")}''')`], {
        stderr: 'pipe',
        stdout: 'pipe',
    });

    if (result.exitCode !== 0) {
        return {
            valid: false,
            error: result.stderr.toString(),
        };
    }

    return { valid: true };
}

/**
 * Verify Rust syntax using rustc --emit=metadata.
 */
function verifyRust(code: string): VerifyResult {
    const dir = mkdtempSync(join(tmpdir(), 'faber-verify-'));
    const filePath = join(dir, 'test.rs');

    try {
        writeFileSync(filePath, code);

        const result = Bun.spawnSync(['rustc', '--emit=metadata', '--out-dir', dir, filePath], {
            stderr: 'pipe',
            stdout: 'pipe',
        });

        if (result.exitCode !== 0) {
            return {
                valid: false,
                error: result.stderr.toString(),
            };
        }

        return { valid: true };
    }
    catch (err) {
        return { valid: false, error: err instanceof Error ? err.message : String(err) };
    }
    finally {
        try { unlinkSync(filePath); } catch {}
    }
}

/**
 * Verify C++ syntax using clang++ -fsyntax-only.
 */
function verifyCpp(code: string): VerifyResult {
    const dir = mkdtempSync(join(tmpdir(), 'faber-verify-'));
    const filePath = join(dir, 'test.cpp');

    try {
        writeFileSync(filePath, code);

        const result = Bun.spawnSync(['clang++', '-std=c++20', '-fsyntax-only', filePath], {
            stderr: 'pipe',
            stdout: 'pipe',
        });

        if (result.exitCode !== 0) {
            return {
                valid: false,
                error: result.stderr.toString(),
            };
        }

        return { valid: true };
    }
    catch (err) {
        return { valid: false, error: err instanceof Error ? err.message : String(err) };
    }
    finally {
        try { unlinkSync(filePath); } catch {}
    }
}

/**
 * Verify Zig syntax using zig ast-check.
 */
function verifyZig(code: string): VerifyResult {
    const dir = mkdtempSync(join(tmpdir(), 'faber-verify-'));
    const filePath = join(dir, 'test.zig');

    try {
        writeFileSync(filePath, code);

        const result = Bun.spawnSync(['zig', 'ast-check', filePath], {
            stderr: 'pipe',
            stdout: 'pipe',
        });

        if (result.exitCode !== 0) {
            return {
                valid: false,
                error: result.stderr.toString() || result.stdout.toString(),
            };
        }

        return { valid: true };
    }
    catch (err) {
        return { valid: false, error: err instanceof Error ? err.message : String(err) };
    }
    finally {
        try { unlinkSync(filePath); } catch {}
    }
}

/**
 * Verify generated code for a specific target.
 *
 * Note: Some targets may need wrapper code (imports, main function) to be valid.
 * This function assumes the code is already wrapped appropriately.
 */
export function verify(target: ExecutableTarget, code: string): VerifyResult {
    switch (target) {
        case 'ts':
            return verifyTypeScript(code);
        case 'py':
            return verifyPython(code);
        case 'rs':
            return verifyRust(code);
        case 'cpp':
            return verifyCpp(code);
        case 'zig':
            return verifyZig(code);
        default:
            return { valid: false, error: `Unknown target: ${target}` };
    }
}

/**
 * Check if verification tools are available for a target.
 */
export function isVerifierAvailable(target: ExecutableTarget): boolean {
    const commands: Record<ExecutableTarget, string[]> = {
        ts: ['tsc', '--version'],
        py: ['python3', '--version'],
        rs: ['rustc', '--version'],
        cpp: ['clang++', '--version'],
        zig: ['zig', 'version'],
    };

    const cmd = commands[target];
    if (!cmd) return false;

    try {
        const result = Bun.spawnSync(cmd, { stderr: 'pipe', stdout: 'pipe' });
        return result.exitCode === 0;
    }
    catch {
        return false;
    }
}

/**
 * Get available verifiers.
 */
export function getAvailableVerifiers(): ExecutableTarget[] {
    const targets: ExecutableTarget[] = ['ts', 'py', 'rs', 'cpp', 'zig'];
    return targets.filter(isVerifierAvailable);
}
