import { test, expect, describe } from 'bun:test';
import { validateMorphology, getNormaTranslation, getNormaRadixForms } from './norma-registry';

describe('norma-registry', () => {
    describe('getNormaTranslation', () => {
        test('lista.adde -> ts push', () => {
            const result = getNormaTranslation('ts', 'lista', 'adde');

            expect(result).toBeDefined();
            expect(result!.method).toBe('push');
        });

        test('lista.adde -> py append', () => {
            const result = getNormaTranslation('py', 'lista', 'adde');

            expect(result).toBeDefined();
            expect(result!.method).toBe('append');
        });

        test('lista.filtrata -> ts filter', () => {
            const result = getNormaTranslation('ts', 'lista', 'filtrata');

            expect(result).toBeDefined();
            expect(result!.method).toBe('filter');
        });

        test('lista.addita -> ts template', () => {
            const result = getNormaTranslation('ts', 'lista', 'addita');

            expect(result).toBeDefined();
            expect(result!.template).toBeDefined();
        });

        test('unknown collection returns undefined', () => {
            const result = getNormaTranslation('ts', 'unknown', 'adde');

            expect(result).toBeUndefined();
        });

        test('unknown method returns undefined', () => {
            const result = getNormaTranslation('ts', 'lista', 'unknown');

            expect(result).toBeUndefined();
        });
    });

    describe('getNormaRadixForms', () => {
        test('lista.adde has radixForms', () => {
            const forms = getNormaRadixForms('lista', 'adde');

            expect(forms).toBeDefined();
            expect(forms![0]).toBe('add');
            expect(forms).toContain('imperativus');
            expect(forms).toContain('perfectum');
        });

        test('unknown collection returns undefined', () => {
            const forms = getNormaRadixForms('unknown', 'adde');

            expect(forms).toBeUndefined();
        });
    });

    describe('validateMorphology', () => {
        describe('valid method calls', () => {
            test('adde is valid (imperativus declared)', () => {
                const result = validateMorphology('lista', 'adde');

                expect(result.valid).toBe(true);
                expect(result.error).toBeUndefined();
            });

            test('addita is valid (perfectum declared)', () => {
                const result = validateMorphology('lista', 'addita');

                expect(result.valid).toBe(true);
            });

            test('filtrata is valid (perfectum declared)', () => {
                const result = validateMorphology('lista', 'filtrata');

                expect(result.valid).toBe(true);
            });
        });

        describe('invalid method calls', () => {
            test('additura is invalid (futurum_activum not declared for add)', () => {
                const result = validateMorphology('lista', 'additura');

                expect(result.valid).toBe(false);
                expect(result.error).toContain('futurum_activum');
                expect(result.error).toContain('not declared');
                expect(result.stem).toBe('add');
                expect(result.form).toBe('futurum_activum');
            });

            test('addabit is invalid (futurum_indicativum not declared for add)', () => {
                const result = validateMorphology('lista', 'addabit');

                expect(result.valid).toBe(false);
                expect(result.error).toContain('futurum_indicativum');
                expect(result.stem).toBe('add');
            });
        });

        describe('non-stdlib collections', () => {
            test('unknown collection passes validation', () => {
                const result = validateMorphology('unknown', 'anything');

                expect(result.valid).toBe(true);
            });
        });

        describe('methods without radix', () => {
            test('method without radix passes validation', () => {
                // longitudo likely doesn't have @ radix defined
                const result = validateMorphology('lista', 'longitudo');

                expect(result.valid).toBe(true);
            });
        });

        describe('unknown methods', () => {
            test('unknown method on known collection passes through', () => {
                // WHY: Could be a user extension method
                const result = validateMorphology('lista', 'customMethod');

                expect(result.valid).toBe(true);
            });
        });
    });
});
