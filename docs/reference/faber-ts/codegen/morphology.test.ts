import { test, expect, describe } from 'bun:test';
import { parseMethodum, formFromFlags, type MorphologyForm } from './morphology';

describe('morphology', () => {
    describe('parseMethodum', () => {
        describe('imperativus (-a/-e/-i)', () => {
            test('adde -> stem=add, form=imperativus', () => {
                const result = parseMethodum('adde');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('add');
                expect(result!.form).toBe('imperativus');
                expect(result!.flags.mutare).toBe(true);
                expect(result!.flags.async).toBe(false);
                expect(result!.flags.reddeNovum).toBe(false);
            });

            test('filtra -> stem=filtr, form=imperativus', () => {
                const result = parseMethodum('filtra');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('filtr');
                expect(result!.form).toBe('imperativus');
            });

            test('ordina -> stem=ordin, form=imperativus', () => {
                const result = parseMethodum('ordina');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('ordin');
                expect(result!.form).toBe('imperativus');
            });

            test('pone -> stem=pon, form=imperativus', () => {
                const result = parseMethodum('pone');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('pon');
                expect(result!.form).toBe('imperativus');
            });
        });

        describe('perfectum (-ata/-ita)', () => {
            test('filtrata -> stem=filtr, form=perfectum', () => {
                const result = parseMethodum('filtrata');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('filtr');
                expect(result!.form).toBe('perfectum');
                expect(result!.flags.mutare).toBe(false);
                expect(result!.flags.reddeNovum).toBe(true);
                expect(result!.flags.allocatio).toBe(true);
            });

            test('ordinata -> stem=ordin, form=perfectum', () => {
                const result = parseMethodum('ordinata');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('ordin');
                expect(result!.form).toBe('perfectum');
            });

            test('addita -> stem=add, form=perfectum', () => {
                const result = parseMethodum('addita');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('add');
                expect(result!.form).toBe('perfectum');
            });

            test('mappata -> stem=mapp, form=perfectum', () => {
                const result = parseMethodum('mappata');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('mapp');
                expect(result!.form).toBe('perfectum');
            });
        });

        describe('perfectum (-ta/-sa)', () => {
            test('inversa -> stem=inver, form=perfectum', () => {
                const result = parseMethodum('inversa');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('inver');
                expect(result!.form).toBe('perfectum');
            });
        });

        describe('futurum_activum (-atura/-itura)', () => {
            test('filtratura -> stem=filtr, form=futurum_activum', () => {
                const result = parseMethodum('filtratura');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('filtr');
                expect(result!.form).toBe('futurum_activum');
                expect(result!.flags.async).toBe(true);
                expect(result!.flags.reddeNovum).toBe(true);
            });

            test('additura -> stem=add, form=futurum_activum', () => {
                const result = parseMethodum('additura');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('add');
                expect(result!.form).toBe('futurum_activum');
            });
        });

        describe('futurum_indicativum (-abit/-ebit/-iet)', () => {
            test('filtrabit -> stem=filtr, form=futurum_indicativum', () => {
                const result = parseMethodum('filtrabit');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('filtr');
                expect(result!.form).toBe('futurum_indicativum');
                expect(result!.flags.mutare).toBe(true);
                expect(result!.flags.async).toBe(true);
            });

            test('leget -> stem=leg, form=futurum_indicativum', () => {
                const result = parseMethodum('legiet');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('leg');
                expect(result!.form).toBe('futurum_indicativum');
            });
        });

        describe('participium_praesens (-ans/-ens)', () => {
            test('filtrans -> stem=filtr, form=participium_praesens', () => {
                const result = parseMethodum('filtrans');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('filtr');
                expect(result!.form).toBe('participium_praesens');
            });

            test('legens -> stem=leg, form=participium_praesens', () => {
                const result = parseMethodum('legens');

                expect(result).toBeDefined();
                expect(result!.stem).toBe('leg');
                expect(result!.form).toBe('participium_praesens');
            });
        });

        describe('edge cases', () => {
            test('single char returns undefined', () => {
                expect(parseMethodum('a')).toBeUndefined();
            });

            test('empty string returns undefined', () => {
                expect(parseMethodum('')).toBeUndefined();
            });

            test('no recognized suffix returns undefined', () => {
                expect(parseMethodum('test')).toBeUndefined();
            });
        });
    });

    describe('formFromFlags', () => {
        test('imperativus flags -> imperativus', () => {
            const form = formFromFlags({
                mutare: true,
                async: false,
                reddeNovum: false,
                allocatio: false,
            });
            expect(form).toBe('imperativus');
        });

        test('perfectum flags -> perfectum', () => {
            const form = formFromFlags({
                mutare: false,
                async: false,
                reddeNovum: true,
                allocatio: true,
            });
            expect(form).toBe('perfectum');
        });

        test('futurum_indicativum flags -> futurum_indicativum', () => {
            const form = formFromFlags({
                mutare: true,
                async: true,
                reddeNovum: false,
                allocatio: false,
            });
            expect(form).toBe('futurum_indicativum');
        });

        test('futurum_activum flags -> futurum_activum', () => {
            const form = formFromFlags({
                mutare: false,
                async: true,
                reddeNovum: true,
                allocatio: true,
            });
            expect(form).toBe('futurum_activum');
        });

        test('unknown combination -> ignotum', () => {
            const form = formFromFlags({
                mutare: true,
                async: true,
                reddeNovum: true,
                allocatio: true,
            });
            expect(form).toBe('ignotum');
        });
    });
});
