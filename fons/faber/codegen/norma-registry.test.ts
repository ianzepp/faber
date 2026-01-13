import { test, expect, describe } from 'bun:test';
import { validateMorphology, getNormaTranslation, getNormaRadixForms, getReceiverOwnership } from './norma-registry';

describe('norma-registry', () => {
    describe('getNormaTranslation', () => {
        test('lista.adde -> ts push', () => {
            const result = getNormaTranslation('ts', 'lista', 'adde');

            expect(result).toBeDefined();
            expect(result!.method).toBe('push');
        });

        test('target parameter is ignored (TS-only)', () => {
            // WHY: Registry is now TS-only; target param kept for API compat
            const result = getNormaTranslation('py', 'lista', 'adde');

            expect(result).toBeDefined();
            expect(result!.method).toBe('push'); // Returns TS translation
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

            test('selecta is valid (perfectum declared, stem select)', () => {
                // WHY: Regression test for issue #5. The method 'selecta' with stem 'select'
                // was being misclassified as imperativus (suffix 'a') instead of perfectum
                // (suffix 'ta'). The greedy parser correctly identifies '-ta' as perfectum.
                const result = validateMorphology('tabula', 'selecta');

                expect(result.valid).toBe(true);
                expect(result.error).toBeUndefined();
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

    describe('getReceiverOwnership', () => {
        describe('mutating methods (imperativus) return "in"', () => {
            test('lista.adde -> in (mutates)', () => {
                expect(getReceiverOwnership('lista', 'adde')).toBe('in');
            });

            test('lista.filtra -> in (mutates)', () => {
                expect(getReceiverOwnership('lista', 'filtra')).toBe('in');
            });

            test('lista.ordina -> in (mutates)', () => {
                expect(getReceiverOwnership('lista', 'ordina')).toBe('in');
            });
        });

        describe('non-mutating methods (perfectum) return "de"', () => {
            test('lista.addita -> de (returns new)', () => {
                expect(getReceiverOwnership('lista', 'addita')).toBe('de');
            });

            test('lista.filtrata -> de (returns new)', () => {
                expect(getReceiverOwnership('lista', 'filtrata')).toBe('de');
            });

            test('lista.ordinata -> de (returns new)', () => {
                expect(getReceiverOwnership('lista', 'ordinata')).toBe('de');
            });

            test('lista.inversa -> de (returns new)', () => {
                expect(getReceiverOwnership('lista', 'inversa')).toBe('de');
            });
        });

        describe('unknown methods return undefined', () => {
            test('unknown collection -> undefined', () => {
                expect(getReceiverOwnership('unknown', 'adde')).toBeUndefined();
            });

            test('unknown method -> undefined', () => {
                expect(getReceiverOwnership('lista', 'unknownMethod')).toBeUndefined();
            });
        });
    });
});
