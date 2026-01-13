/**
 * TypeScript verification utilities.
 *
 * Validates that generated code is syntactically correct.
 */

import { writeFileSync, unlinkSync, mkdtempSync } from 'fs';
import { join } from 'path';
import { tmpdir } from 'os';

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
 * Verify generated code (TypeScript only).
 */
export function verify(target: 'ts', code: string): VerifyResult {
    return verifyTypeScript(code);
}

/**
 * Check if TypeScript verification is available.
 */
export function isVerifierAvailable(): boolean {
    try {
        const result = Bun.spawnSync(['tsc', '--version'], { stderr: 'pipe', stdout: 'pipe' });
        return result.exitCode === 0;
    }
    catch {
        return false;
    }
}

/**
 * Get available verifiers.
 */
export function getAvailableVerifiers(): string[] {
    return isVerifierAvailable() ? ['ts'] : [];
}
