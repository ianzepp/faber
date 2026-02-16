import { test, expect, describe } from 'bun:test';
import { parseNoun, parseVerb, parseType, isKeyword, getKeyword, isBuiltinType, isLexiconError } from './index';

describe('parseNoun', () => {
    describe('2nd declension masculine', () => {
        test('nominative singular: nuntius', () => {
            const results = parseNoun('nuntius');

            expect(isLexiconError(results)).toBe(false);
            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results).toHaveLength(1);
                expect(results[0]).toEqual({
                    stem: 'nunti',
                    declension: 2,
                    gender: 'masculine',
                    case: 'nominative',
                    number: 'singular',
                });
            }
        });

        test('accusative singular: nuntium', () => {
            const results = parseNoun('nuntium');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results).toHaveLength(1);
                expect(results[0]!.case).toBe('accusative');
                expect(results[0]!.number).toBe('singular');
            }
        });

        test('genitive singular: nuntii', () => {
            const results = parseNoun('nuntii');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                // Could be genitive singular OR nominative plural
                expect(results.length).toBeGreaterThanOrEqual(1);
                expect(results.some(r => r.case === 'genitive')).toBe(true);
            }
        });

        test('dative/ablative singular: nuntio (ambiguous)', () => {
            const results = parseNoun('nuntio');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results).toHaveLength(2);
                expect(results.map(r => r.case)).toContain('dative');
                expect(results.map(r => r.case)).toContain('ablative');
            }
        });

        test('accusative plural: nuntios', () => {
            const results = parseNoun('nuntios');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.case).toBe('accusative');
                expect(results[0]!.number).toBe('plural');
            }
        });

        test('genitive plural: nuntiorum', () => {
            const results = parseNoun('nuntiorum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.case).toBe('genitive');
                expect(results[0]!.number).toBe('plural');
            }
        });
    });

    describe('error handling', () => {
        test('returns unknown_stem error for unknown word', () => {
            const result = parseNoun('asdfgh');

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('unknown_stem');
                expect(result.word).toBe('asdfgh');
            }
        });

        test('returns invalid_ending error for known stem with bad ending', () => {
            const result = parseNoun('nuntixx');

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('invalid_ending');
                expect(result.word).toBe('nuntixx');
                expect(result.stem).toBe('nunti');
                expect(result.ending).toBe('xx');
            }
        });

        test('suggests similar stem for unknown word', () => {
            const result = parseNoun('numer'); // typo/incomplete, close to 'numer'

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('invalid_ending');
                // No suggestion needed for invalid_ending - we already know the stem
            }

            const result2 = parseNoun('numer'); // This is actually valid with empty ending... let's use different word

            const result3 = parseNoun('nunts'); // close to nunti stem

            expect(isLexiconError(result3)).toBe(true);
            if (isLexiconError(result3)) {
                expect(result3.error).toBe('unknown_stem');
                expect(result3.suggestion).toBe('nunti');
            }
        });
    });
});

describe('parseVerb', () => {
    describe('3rd conjugation: mittere (send)', () => {
        test('imperative: mitte (sync)', () => {
            const results = parseVerb('mitte');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.stem).toBe('mitt');
                expect(results[0]!.tense).toBe('imperative');
                expect(results[0]!.async).toBe(false);
            }
        });

        test('present 3rd person: mittit (sync)', () => {
            const results = parseVerb('mittit');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.tense).toBe('present');
                expect(results[0]!.person).toBe(3);
                expect(results[0]!.async).toBe(false);
            }
        });

        test('future 3rd person: mittet (async)', () => {
            const results = parseVerb('mittet');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.tense).toBe('future');
                expect(results[0]!.person).toBe(3);
                expect(results[0]!.async).toBe(true);
            }
        });
    });

    describe('1st conjugation: creare (create)', () => {
        test('imperative: crea (sync)', () => {
            const results = parseVerb('crea');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.stem).toBe('cre');
                expect(results[0]!.tense).toBe('imperative');
                expect(results[0]!.async).toBe(false);
            }
        });

        test('present 3rd person: creat (sync)', () => {
            const results = parseVerb('creat');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.tense).toBe('present');
                expect(results[0]!.async).toBe(false);
            }
        });

        test('future 3rd person: creabit (async)', () => {
            const results = parseVerb('creabit');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.tense).toBe('future');
                expect(results[0]!.async).toBe(true);
            }
        });
    });

    describe('error handling', () => {
        test('returns unknown_stem error for unknown verb', () => {
            const result = parseVerb('asdfgh');

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('unknown_stem');
                expect(result.word).toBe('asdfgh');
            }
        });

        test('returns invalid_ending error for known stem with bad ending', () => {
            const result = parseVerb('mittxx');

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('invalid_ending');
                expect(result.stem).toBe('mitt');
                expect(result.ending).toBe('xx');
            }
        });

        test('suggests similar stem for unknown verb', () => {
            const result = parseVerb('mett'); // typo, close to mitt

            expect(isLexiconError(result)).toBe(true);
            if (isLexiconError(result)) {
                expect(result.error).toBe('unknown_stem');
                expect(result.suggestion).toBe('mitt');
            }
        });
    });
});

describe('keywords', () => {
    test('recognizes control flow keywords', () => {
        expect(isKeyword('si')).toBe(true);
        expect(isKeyword('secus')).toBe(true);
        expect(isKeyword('dum')).toBe(true);
        expect(isKeyword('redde')).toBe(true);
    });

    test('recognizes declaration keywords', () => {
        expect(isKeyword('varia')).toBe(true);
        expect(isKeyword('fixum')).toBe(true);
        expect(isKeyword('functio')).toBe(true);
    });

    test('recognizes values', () => {
        expect(isKeyword('verum')).toBe(true);
        expect(isKeyword('falsum')).toBe(true);
        expect(isKeyword('nihil')).toBe(true);
    });

    test('returns false for non-keywords', () => {
        expect(isKeyword('nuntius')).toBe(false);
        expect(isKeyword('asdfgh')).toBe(false);
    });

    test('getKeyword returns meaning', () => {
        expect(getKeyword('si')?.meaning).toBe('if');
        expect(getKeyword('varia')?.meaning).toBe('let');
        expect(getKeyword('fixum')?.meaning).toBe('const');
        expect(getKeyword('verum')?.meaning).toBe('true');
        expect(getKeyword('futura')?.meaning).toBe('async');
    });

    test('getKeyword is case sensitive', () => {
        // Keywords must be lowercase
        expect(getKeyword('si')?.meaning).toBe('if');
        expect(getKeyword('fixum')?.meaning).toBe('const');
        // Uppercase variants are not keywords
        expect(getKeyword('SI')).toBe(undefined);
        expect(getKeyword('Fixum')).toBe(undefined);
    });
});

describe('parseType', () => {
    describe('primitives', () => {
        test('textus (4th declension)', () => {
            const results = parseType('textus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('text/string');
                expect(results[0]!.category).toBe('primitive');
            }
        });

        test('numerus (2nd declension masculine)', () => {
            const results = parseType('numerus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('number');
            }
        });

        test('numerum (accusative)', () => {
            const results = parseType('numerum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.case).toBe('accusative');
                expect(results[0]!.meaning).toBe('number');
            }
        });
    });

    describe('collections', () => {
        test('lista (1st declension feminine)', () => {
            const results = parseType('lista');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('list/array');
                expect(results[0]!.category).toBe('collection');
                expect(results[0]!.generic).toBe(true);
            }
        });

        test('listam (accusative)', () => {
            const results = parseType('listam');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.case).toBe('accusative');
            }
        });

        test('tabula (Map)', () => {
            const results = parseType('tabula');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('table/map');
            }
        });

        test('copia (Set)', () => {
            const results = parseType('copia');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('set/collection');
            }
        });
    });

    describe('structural', () => {
        test('promissum (2nd declension neuter)', () => {
            const results = parseType('promissum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('promise');
                expect(results[0]!.generic).toBe(true);
            }
        });

        test('erratum (Error)', () => {
            const results = parseType('erratum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('error');
            }
        });

        test('cursor (3rd declension, no ending)', () => {
            const results = parseType('cursor');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('cursor/iterator');
                expect(results[0]!.case).toBe('nominative');
            }
        });

        test('tempus (3rd declension neuter, alternate nominative)', () => {
            const results = parseType('tempus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('time/date');
                expect(results[0]!.case).toBe('nominative');
                expect(results[0]!.stem).toBe('tempor');
            }
        });

        test('temporis (3rd declension neuter, genitive)', () => {
            const results = parseType('temporis');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0]!.meaning).toBe('time/date');
                expect(results[0]!.case).toBe('genitive');
            }
        });
    });

    describe('builtin type helpers', () => {
        test('isBuiltinType recognizes type stems (case-sensitive)', () => {
            // Stems are lowercase
            expect(isBuiltinType('text')).toBe(true);
            expect(isBuiltinType('numer')).toBe(true);
            expect(isBuiltinType('list')).toBe(true);
            // Uppercase variants don't match
            expect(isBuiltinType('Text')).toBe(false);
            expect(isBuiltinType('Numer')).toBe(false);
            // Unknown stems
            expect(isBuiltinType('asdfgh')).toBe(false);
        });
    });
});
