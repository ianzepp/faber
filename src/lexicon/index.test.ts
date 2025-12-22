import { test, expect, describe } from 'bun:test';
import {
    parseNoun,
    parseVerb,
    parseType,
    isKeyword,
    getKeyword,
    isBuiltinType,
    isLexiconError,
} from './index';

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
                expect(results[0].case).toBe('accusative');
                expect(results[0].number).toBe('singular');
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
                expect(results[0].case).toBe('accusative');
                expect(results[0].number).toBe('plural');
            }
        });

        test('genitive plural: nuntiorum', () => {
            const results = parseNoun('nuntiorum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].case).toBe('genitive');
                expect(results[0].number).toBe('plural');
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
                expect(results[0].stem).toBe('mitt');
                expect(results[0].tense).toBe('imperative');
                expect(results[0].async).toBe(false);
            }
        });

        test('present 3rd person: mittit (sync)', () => {
            const results = parseVerb('mittit');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].tense).toBe('present');
                expect(results[0].person).toBe(3);
                expect(results[0].async).toBe(false);
            }
        });

        test('future 3rd person: mittet (async)', () => {
            const results = parseVerb('mittet');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].tense).toBe('future');
                expect(results[0].person).toBe(3);
                expect(results[0].async).toBe(true);
            }
        });
    });

    describe('1st conjugation: creare (create)', () => {
        test('imperative: crea (sync)', () => {
            const results = parseVerb('crea');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].stem).toBe('cre');
                expect(results[0].tense).toBe('imperative');
                expect(results[0].async).toBe(false);
            }
        });

        test('present 3rd person: creat (sync)', () => {
            const results = parseVerb('creat');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].tense).toBe('present');
                expect(results[0].async).toBe(false);
            }
        });

        test('future 3rd person: creabit (async)', () => {
            const results = parseVerb('creabit');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].tense).toBe('future');
                expect(results[0].async).toBe(true);
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
        expect(isKeyword('aliter')).toBe(true);
        expect(isKeyword('dum')).toBe(true);
        expect(isKeyword('redde')).toBe(true);
    });

    test('recognizes declaration keywords', () => {
        expect(isKeyword('esto')).toBe(true);
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
        expect(getKeyword('esto')?.meaning).toBe('let');
        expect(getKeyword('fixum')?.meaning).toBe('const');
        expect(getKeyword('verum')?.meaning).toBe('true');
        expect(getKeyword('futura')?.meaning).toBe('async');
    });

    test('getKeyword is case insensitive', () => {
        expect(getKeyword('SI')?.meaning).toBe('if');
        expect(getKeyword('Fixum')?.meaning).toBe('const');
    });
});

describe('parseType', () => {
    describe('primitives', () => {
        test('Textus (4th declension)', () => {
            const results = parseType('Textus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('string');
                expect(results[0].category).toBe('primitive');
            }
        });

        test('Numerus (2nd declension masculine)', () => {
            const results = parseType('Numerus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('number');
            }
        });

        test('Numerum (accusative)', () => {
            const results = parseType('Numerum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].case).toBe('accusative');
                expect(results[0].jsType).toBe('number');
            }
        });
    });

    describe('collections', () => {
        test('Lista (1st declension feminine)', () => {
            const results = parseType('Lista');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Array');
                expect(results[0].category).toBe('collection');
                expect(results[0].generic).toBe(true);
            }
        });

        test('Listam (accusative)', () => {
            const results = parseType('Listam');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].case).toBe('accusative');
            }
        });

        test('Tabula (Map)', () => {
            const results = parseType('Tabula');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Map');
            }
        });

        test('Copia (Set)', () => {
            const results = parseType('Copia');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Set');
            }
        });
    });

    describe('structural', () => {
        test('Promissum (2nd declension neuter)', () => {
            const results = parseType('Promissum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Promise');
                expect(results[0].generic).toBe(true);
            }
        });

        test('Erratum (Error)', () => {
            const results = parseType('Erratum');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Error');
            }
        });

        test('Cursor (3rd declension, no ending)', () => {
            const results = parseType('Cursor');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Iterator');
                expect(results[0].case).toBe('nominative');
            }
        });

        test('Tempus (3rd declension neuter, alternate nominative)', () => {
            const results = parseType('Tempus');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Date');
                expect(results[0].case).toBe('nominative');
                expect(results[0].stem).toBe('tempor');
            }
        });

        test('Temporis (3rd declension neuter, genitive)', () => {
            const results = parseType('Temporis');

            expect(Array.isArray(results)).toBe(true);
            if (Array.isArray(results)) {
                expect(results[0].jsType).toBe('Date');
                expect(results[0].case).toBe('genitive');
            }
        });
    });

    describe('builtin type helpers', () => {
        test('isBuiltinType recognizes type stems', () => {
            expect(isBuiltinType('Text')).toBe(true);
            expect(isBuiltinType('Numer')).toBe(true);
            expect(isBuiltinType('List')).toBe(true);
            expect(isBuiltinType('asdfgh')).toBe(false);
        });
    });
});
