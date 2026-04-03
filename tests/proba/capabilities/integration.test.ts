/**
 * Integration Tests - Target capability validation in compilation pipeline
 *
 * Tests that validation errors are thrown during compilation when using
 * unsupported features for specific targets.
 */

import { generate } from '../../faber/codegen';
import { parse } from '../../faber/parser';
import { tokenize } from '../../faber/tokenizer';
import { TargetCompatibilityError } from '../../faber/codegen/validator';
import { describe, test, expect } from 'bun:test';

/**
 * Helper to compile Faber source.
 * WHY: Encapsulates the three-phase pipeline for cleaner test code.
 */
function compile(source: string, target: string): string {
    const { tokens } = tokenize(source);
    const { program } = parse(tokens);
    return generate(program!, { target });
}

describe('codegen integration', () => {
    test('async function compiles to TypeScript', () => {
        const source = 'functio f() fiet numerus { redde 1 }';
        expect(() => compile(source, 'ts')).not.toThrow();
    });

    test('async function fails for Zig', () => {
        const source = 'functio f() fiet numerus { redde 1 }';
        expect(() => compile(source, 'zig')).toThrow(TargetCompatibilityError);
    });

    test('generator function compiles to TypeScript', () => {
        const source = 'functio f() fiunt numerus { cede 1 cede 2 }';
        expect(() => compile(source, 'ts')).not.toThrow();
    });

    test('generator function fails for Rust', () => {
        const source = 'functio f() fiunt numerus { cede 1 cede 2 }';
        expect(() => compile(source, 'rs')).toThrow(TargetCompatibilityError);
    });

    test('compatible code compiles to all targets', () => {
        // Simple function with no advanced features
        const source = 'functio add(numerus a, numerus b) fit numerus { redde a + b }';
        for (const target of ['ts', 'py', 'rs', 'zig', 'cpp']) {
            expect(() => compile(source, target)).not.toThrow();
        }
    });
});
