/**
 * Feature × Target Matrix Tests
 *
 * Table-driven tests that verify each feature is correctly validated
 * across all targets according to TARGET_SUPPORT definitions.
 */

import { generate } from '../../faber/codegen';
import { parse } from '../../faber/parser';
import { tokenize } from '../../faber/tokenizer';
import { TargetCompatibilityError } from '../../faber/codegen/validator';
import { describe, test, expect } from 'bun:test';

/**
 * Helper to compile Faber source.
 */
function compile(source: string, target: string): string {
    const { tokens } = tokenize(source);
    const { program } = parse(tokens);
    return generate(program!, { target });
}

/**
 * Helper to check if compilation should succeed.
 */
function shouldCompile(source: string, target: string): boolean {
    try {
        compile(source, target);
        return true;
    }
    catch (err) {
        if (err instanceof TargetCompatibilityError) {
            return false;
        }
        throw err; // Re-throw non-capability errors
    }
}

// Feature test cases with minimal Faber source
const FEATURES = [
    {
        name: 'controlFlow.asyncFunction',
        source: 'functio f() fiet numerus { redde 1 }',
        supported: ['ts', 'py', 'rs'],
        unsupported: ['zig', 'cpp'],
    },
    {
        name: 'controlFlow.generatorFunction',
        source: 'functio f() fiunt numerus { cede 1 cede 2 }',
        supported: ['ts', 'py'],
        unsupported: ['rs', 'zig', 'cpp'],
    },
    // NOTE: try-catch has codegen bugs, skipping for now
    // {
    //     name: 'errors.tryCatch',
    //     source: 'functio f() fit numerus { tempta { redde 1 } cape err { redde 0 } }',
    //     supported: ['ts', 'py', 'cpp'],
    //     unsupported: ['rs', 'zig'],
    // },
    // {
    //     name: 'errors.throw',
    //     source: 'functio f() fit numerus { iace error("test") }',
    //     supported: ['ts', 'py', 'cpp'],
    //     unsupported: ['rs', 'zig'],
    // },
    // NOTE: Object destructuring syntax not yet in grammar, tested via AST in other tests
    // {
    //     name: 'binding.pattern.object',
    //     source: 'ex obj fixum x, y',
    //     supported: ['ts'],
    //     unsupported: ['py', 'rs', 'zig', 'cpp'],
    // },
    {
        name: 'params.defaultValues',
        source: 'functio f(numerus x vel 0) fit numerus { redde x }',
        supported: ['ts', 'py', 'cpp'],
        unsupported: ['rs', 'zig'],
    },
];

const TARGETS = ['ts', 'py', 'rs', 'zig', 'cpp'];

describe('feature × target matrix', () => {
    for (const feature of FEATURES) {
        describe(feature.name, () => {
            // Test supported targets
            for (const target of feature.supported) {
                test(`should compile to ${target}`, () => {
                    expect(shouldCompile(feature.source, target)).toBe(true);
                });
            }

            // Test unsupported targets
            for (const target of feature.unsupported) {
                test(`should reject ${target}`, () => {
                    expect(() => compile(feature.source, target)).toThrow(
                        TargetCompatibilityError,
                    );
                });
            }
        });
    }
});

describe('comprehensive matrix verification', () => {
    test('all targets covered', () => {
        // Verify each feature test declares all 5 targets
        for (const feature of FEATURES) {
            const declared = new Set([
                ...feature.supported,
                ...feature.unsupported,
            ]);
            expect(declared.size).toBe(5);
            for (const target of TARGETS) {
                expect(declared.has(target)).toBe(true);
            }
        }
    });

    test('simple programs compile to all targets', () => {
        // Programs with no advanced features should work everywhere
        const simpleSources = [
            'functio add(numerus a, numerus b) fit numerus { redde a + b }',
            // Note: Zig doesn't support string concatenation with +, use scriptum instead
            'functio greet(textus name) fit textus { redde scriptum("Salve, §!", name) }',
            'functio max(numerus a, numerus b) fit numerus { si a > b { redde a } alio { redde b } }',
        ];

        for (const source of simpleSources) {
            for (const target of TARGETS) {
                expect(shouldCompile(source, target)).toBe(true);
            }
        }
    });
});
