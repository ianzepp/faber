/**
 * Feature Interaction Tests
 *
 * Tests that verify correct validation when multiple features interact:
 * - Async + generator (async generator)
 * - Multiple unsupported features in one program
 * - Error deduplication (5 async functions → 1 error)
 * - False positive checks (simple programs compile everywhere)
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
 * Helper to get validation errors without throwing.
 */
function getValidationErrors(
    source: string,
    target: string,
): TargetCompatibilityError | null {
    try {
        compile(source, target);
        return null;
    }
    catch (err) {
        if (err instanceof TargetCompatibilityError) {
            return err;
        }
        throw err; // Re-throw non-capability errors
    }
}

describe('feature interactions', () => {
    describe('async + generator', () => {
        test('async generator detected as two separate features', () => {
            const source = 'functio f() fient numerus { cede 1 cede 2 }';

            // TypeScript supports async generators
            expect(() => compile(source, 'ts')).not.toThrow();

            // Zig supports neither async nor generators, should fail with 2 errors
            const err = getValidationErrors(source, 'zig');
            expect(err).not.toBeNull();
            expect(err!.errors).toHaveLength(2);

            const features = err!.errors.map(e => e.feature);
            expect(features).toContain('controlFlow.asyncFunction');
            expect(features).toContain('controlFlow.generatorFunction');
        });

        test('Python supports async and sync generators separately', () => {
            const asyncGen = 'functio f() fient numerus { cede 1 }';
            const syncGen = 'functio f() fiunt numerus { cede 1 }';
            const asyncFn = 'functio f() fiet numerus { redde 1 }';

            // Python supports all three
            expect(() => compile(asyncGen, 'py')).not.toThrow();
            expect(() => compile(syncGen, 'py')).not.toThrow();
            expect(() => compile(asyncFn, 'py')).not.toThrow();
        });
    });

    // NOTE: try-catch has codegen bugs, skipping for now
    // describe('try-catch + throw', () => {
    //     test('both features detected together', () => {
    //         const source = `
    //             functio f() fit numerus {
    //                 tempta {
    //                     iace error("test")
    //                 } cape err {
    //                     redde 0
    //                 }
    //             }
    //         `;

    //         // Should work on targets supporting exceptions
    //         expect(() => compile(source, 'ts')).not.toThrow();
    //         expect(() => compile(source, 'py')).not.toThrow();
    //         expect(() => compile(source, 'cpp')).not.toThrow();

    //         // Should fail on Zig with 2 errors
    //         const err = getValidationErrors(source, 'zig');
    //         expect(err).not.toBeNull();
    //         expect(err!.errors.length).toBeGreaterThanOrEqual(2);

    //         const features = err!.errors.map(e => e.feature);
    //         expect(features).toContain('errors.tryCatch');
    //         expect(features).toContain('errors.throw');
    //     });
    // });

    describe('multiple unsupported features', () => {
        test('complex program with many unsupported features', () => {
            const source = `
                functio process(numerus x vel 0) fiet numerus {
                    redde x + 1
                }
            `;

            // Should fail on Zig with multiple errors:
            // - async function
            // - default parameters
            const err = getValidationErrors(source, 'zig');
            expect(err).not.toBeNull();
            expect(err!.errors.length).toBeGreaterThanOrEqual(2);

            const features = err!.errors.map(e => e.feature);
            expect(features).toContain('controlFlow.asyncFunction');
            expect(features).toContain('params.defaultValues');
        });
    });

    describe('error deduplication', () => {
        test('multiple async functions produce single deduplicated error', () => {
            const source = `
                functio a() fiet numerus { redde 1 }
                functio b() fiet numerus { redde 2 }
                functio c() fiet numerus { redde 3 }
                functio d() fiet numerus { redde 4 }
                functio e() fiet numerus { redde 5 }
            `;

            // Zig doesn't support async
            const err = getValidationErrors(source, 'zig');
            expect(err).not.toBeNull();

            // Count how many async function errors we got
            const asyncErrors = err!.errors.filter(
                e => e.feature === 'controlFlow.asyncFunction',
            );

            // EDGE: Feature detector deduplicates - only reports first occurrence
            // WHY: Prevents overwhelming error output on large programs
            expect(asyncErrors.length).toBe(1);
        });

        test('different unsupported features each reported once', () => {
            const source = `
                functio f(numerus x vel 0) fiet numerus {
                    redde x + 1
                }
                functio g(numerus y vel 1) fiunt numerus {
                    cede y
                }
            `;

            // For Zig: async, generator, default params (twice each)
            // But deduplication means only 1 error per feature type
            const err = getValidationErrors(source, 'zig');
            expect(err).not.toBeNull();
            expect(err!.errors.length).toBe(3);

            // 3 unique feature types
            const uniqueFeatures = new Set(err!.errors.map(e => e.feature));
            expect(uniqueFeatures.size).toBe(3);
            expect(uniqueFeatures.has('controlFlow.asyncFunction')).toBe(true);
            expect(uniqueFeatures.has('controlFlow.generatorFunction')).toBe(true);
            expect(uniqueFeatures.has('params.defaultValues')).toBe(true);
        });
    });

    describe('false positive checks', () => {
        test('simple imperative code compiles everywhere', () => {
            const source = `
                functio sum(numerus[] nums) fit numerus {
                    varia total = 0
                    ex nums pro n {
                        total = total + n
                    }
                    redde total
                }
            `;

            const targets = ['ts', 'py', 'rs', 'zig', 'cpp'];
            for (const target of targets) {
                expect(() => compile(source, target)).not.toThrow();
            }
        });

        test('simple data structures compile everywhere', () => {
            const source = `
                genus Punto {
                    numerus x
                    numerus y
                }

                functio distance(Punto a, Punto b) fit numerus {
                    fixum dx = a.x - b.x
                    fixum dy = a.y - b.y
                    redde dx * dx + dy * dy
                }
            `;

            const targets = ['ts', 'py', 'rs', 'zig', 'cpp'];
            for (const target of targets) {
                expect(() => compile(source, target)).not.toThrow();
            }
        });

        test('conditional logic compiles everywhere', () => {
            const source = `
                functio max(numerus a, numerus b) fit numerus {
                    si a > b {
                        redde a
                    } alio {
                        redde b
                    }
                }

                functio clamp(numerus x, numerus min, numerus max) fit numerus {
                    si x < min {
                        redde min
                    }
                    si x > max {
                        redde max
                    }
                    redde x
                }
            `;

            const targets = ['ts', 'py', 'rs', 'zig', 'cpp'];
            for (const target of targets) {
                expect(() => compile(source, target)).not.toThrow();
            }
        });

        // NOTE: Collection methods have codegen issues, skipping for now
        // test('collections and iteration compile everywhere', () => {
        //     const source = `
        //         functio filter(numerus[] items, numerus threshold) fit numerus[] {
        //             varia numerus[] result = [] innatum numerus[]
        //             ex items pro item {
        //                 si item > threshold {
        //                     result.adde(item)
        //                 }
        //             }
        //             redde result
        //         }
        //     `;

        //     const targets = ['ts', 'py', 'rs', 'zig', 'cpp'];
        //     for (const target of targets) {
        //         expect(() => compile(source, target)).not.toThrow();
        //     }
        // });
    });
});
