/**
 * Lexicon - Latin Morphological Parser
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (morphological parsing)
 *
 * ARCHITECTURE
 * ============
 * This module implements Latin morphological parsing for the Faber Romanus compiler.
 * It analyzes inflected Latin words to determine their grammatical properties based
 * on traditional Latin grammar rules: declensions for nouns/types and conjugations
 * for verbs.
 *
 * Latin is a highly inflected language where word endings encode grammatical meaning.
 * Unlike English where word order is primary, Latin uses endings to indicate a word's
 * role in a sentence. For example, "numerus" (nominative) vs "numerum" (accusative)
 * vs "numeri" (genitive) - same stem, different grammatical cases.
 *
 * The lexicon provides three main parsing functions:
 * - parseNoun: Identifies case/number from noun endings (user vocabulary)
 * - parseType: Identifies case/number from type names (built-in types)
 * - parseVerb: Identifies tense/person/number from verb endings
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Inflected Latin word (string)
 * OUTPUT: Array of possible grammatical interpretations, or LexiconError
 * ERRORS: Returns LexiconError with kind indicating why parsing failed:
 *         - unknown_stem: No vocabulary entry matches the word
 *         - invalid_ending: Stem matched but ending not in declension/conjugation
 *
 * INVARIANTS
 * ==========
 * INV-1: All stems are stored without endings (e.g., "numer" not "numerus")
 * INV-2: Ending tables are complete for each declension/conjugation
 * INV-3: Multiple interpretations are returned when forms are ambiguous
 * INV-4: Case-sensitive matching for types (TitleCase), case-insensitive for others
 *
 * @module lexicon
 */

import type { ParsedNoun, ParsedVerb, Number, Tense } from './types';
import { nouns, getEndingsForDeclension } from './nouns';
import {
    verbs,
    conjugation1Endings,
    conjugation2Endings,
    conjugation3Endings,
    conjugation4Endings,
} from './verbs';
import { builtinTypes } from './types-builtin';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Error returned when lexicon parsing fails.
 *
 * WHY: Instead of returning null, we return what we learned during parsing.
 *      This enables better error messages for users.
 */
export interface LexiconError {
    error: 'unknown_stem' | 'invalid_ending';
    word: string;
    stem?: string; // Present if we matched a stem but ending failed
    ending?: string; // The ending that didn't match
    suggestion?: string; // "Did you mean?" suggestion for near-misses
}

/**
 * Check if a parse result is an error.
 */
export function isLexiconError(result: unknown): result is LexiconError {
    return result !== null && typeof result === 'object' && 'error' in result;
}

// =============================================================================
// STRING SIMILARITY FOR "DID YOU MEAN?" SUGGESTIONS
// =============================================================================

/**
 * Compute Levenshtein edit distance between two strings.
 *
 * WHY: Edit distance measures how many single-character edits (insertions,
 *      deletions, substitutions) are needed to transform one string into another.
 *      Lower distance = more similar.
 *
 * PERF: Uses Wagner-Fischer algorithm with O(m*n) time and O(min(m,n)) space.
 */
function levenshtein(a: string, b: string): number {
    if (a.length === 0) {
        return b.length;
    }

    if (b.length === 0) {
        return a.length;
    }

    // WHY: Ensure a is the shorter string for O(min(m,n)) space
    if (a.length > b.length) {
        [a, b] = [b, a];
    }

    let prev = Array.from({ length: a.length + 1 }, (_, i) => i);
    let curr = new Array(a.length + 1);

    for (let j = 1; j <= b.length; j++) {
        curr[0] = j;

        for (let i = 1; i <= a.length; i++) {
            const cost = a[i - 1] === b[j - 1] ? 0 : 1;

            curr[i] = Math.min(
                prev[i] + 1, // deletion
                curr[i - 1] + 1, // insertion
                prev[i - 1] + cost, // substitution
            );
        }

        [prev, curr] = [curr, prev];
    }

    return prev[a.length];
}

/**
 * Find the closest match in a vocabulary list.
 *
 * @param word - The unknown word to find suggestions for
 * @param candidates - List of known stems to compare against
 * @param maxDistance - Maximum edit distance to consider (default 3)
 * @returns Closest matching stem, or undefined if none close enough
 */
function findClosestMatch(word: string, candidates: string[], maxDistance = 3): string | undefined {
    const lowerWord = word.toLowerCase();
    let bestMatch: string | undefined;
    let bestDistance = Infinity;

    for (const candidate of candidates) {
        const distance = levenshtein(lowerWord, candidate.toLowerCase());

        if (distance < bestDistance && distance <= maxDistance) {
            bestDistance = distance;
            bestMatch = candidate;
        }
    }

    return bestMatch;
}

/**
 * Parsed type with JavaScript/TypeScript target information.
 *
 * DESIGN: Extends ParsedNoun because Latin type names follow noun declension
 *         patterns. jsType maps to target language primitives.
 */
export interface ParsedType extends ParsedNoun {
    jsType: string;
    category: 'primitive' | 'collection' | 'structural' | 'iteration';
    generic?: boolean;
}

// =============================================================================
// PARSING FUNCTIONS
// =============================================================================

// ---------------------------------------------------------------------------
// Noun Parsing
// ---------------------------------------------------------------------------

/**
 * Parse an inflected Latin noun to determine its grammatical properties.
 *
 * ALGORITHM:
 *   1. Normalize to lowercase (Latin nouns are case-insensitive in source)
 *   2. Try each known noun stem
 *   3. Extract ending by removing stem
 *   4. Lookup ending in appropriate declension table
 *   5. Return all matching interpretations (may be multiple)
 *
 * WHY: Returns array rather than single result because Latin has homographs.
 *      For example, "lista" could be nominative singular OR ablative singular.
 *      The parser must handle this ambiguity.
 *
 * @param word - The inflected Latin noun (e.g., "numerum", "usuarii")
 * @returns Array of possible interpretations, or LexiconError
 */
export function parseNoun(word: string): ParsedNoun[] | LexiconError {
    // WHY: Latin nouns in user code are case-insensitive (unlike types which are TitleCase)
    const lowerWord = word.toLowerCase();

    // Track best match for error reporting
    let bestMatch: { stem: string; ending: string } | null = null;

    for (const noun of nouns) {
        if (!lowerWord.startsWith(noun.stem)) {
            continue;
        }

        const ending = lowerWord.slice(noun.stem.length);
        const endingsTable = getEndingsForDeclension(noun.declension, noun.gender);

        if (!endingsTable) {
            continue;
        }

        // Remember we got this far (stem matched)
        bestMatch = { stem: noun.stem, ending };

        const matches = endingsTable[ending];

        if (matches) {
            return matches.map(m => ({
                stem: noun.stem,
                declension: noun.declension,
                gender: noun.gender,
                case: m.case,
                number: m.number,
            }));
        }
    }

    // Return what we learned, with suggestion if available
    if (bestMatch) {
        return { error: 'invalid_ending', word, stem: bestMatch.stem, ending: bestMatch.ending };
    }

    const suggestion = findClosestMatch(
        word,
        nouns.map(n => n.stem),
    );

    return { error: 'unknown_stem', word, suggestion };
}

// ---------------------------------------------------------------------------
// Type Parsing
// ---------------------------------------------------------------------------

/**
 * Parse an inflected Latin type name to determine its grammatical properties.
 *
 * ALGORITHM:
 *   1. Check for alternate nominative forms (e.g., Tempus for Tempor-)
 *   2. Preserve original case for stem matching (types are TitleCase)
 *   3. Try each built-in type stem
 *   4. Extract ending (convert to lowercase for ending lookup)
 *   5. Lookup ending in appropriate declension table
 *   6. Return all matching interpretations
 *
 * WHY: Types use TitleCase (Textus, Numerus) to distinguish from user nouns.
 *      This follows TypeScript convention where types start with uppercase.
 *
 * EDGE: 3rd declension types have special nominative handling:
 *       - Regular: stem equals nominative (Cursor)
 *       - Neuter: nominative differs from stem (Tempus vs Tempor-)
 *
 * @param word - The inflected type name (e.g., "Textus", "Cursorem", "Tempus")
 * @returns Array of possible interpretations with target type info, or LexiconError
 */
export function parseType(word: string): ParsedType[] | LexiconError {
    // Track best match for error reporting
    let bestMatch: { stem: string; ending: string } | null = null;

    // WHY: Normalize to lowercase for case-insensitive matching
    //      (Latin had no case distinction, so textus/Textus/TEXTUS all work)
    const lowerWord = word.toLowerCase();

    for (const typeEntry of builtinTypes) {
        // EDGE: Check alternate nominative forms first (e.g., tempus for tempor-)
        //       3rd declension neuters have nominatives that differ from stems
        if (typeEntry.nominative && lowerWord === typeEntry.nominative) {
            return [
                {
                    stem: typeEntry.stem,
                    declension: typeEntry.declension,
                    gender: typeEntry.gender,
                    case: 'nominative',
                    number: 'singular',
                    jsType: typeEntry.jsType,
                    category: typeEntry.category,
                    generic: typeEntry.generic,
                },
            ];
        }

        if (!lowerWord.startsWith(typeEntry.stem)) {
            continue;
        }

        // WHY: Both word and stem are now lowercase
        const ending = lowerWord.slice(typeEntry.stem.length);
        const endingsTable = getEndingsForDeclension(typeEntry.declension, typeEntry.gender);

        if (!endingsTable) {
            continue;
        }

        // Remember we got this far (stem matched)
        bestMatch = { stem: typeEntry.stem, ending };

        // EDGE: 3rd declension nominative singular has no ending
        //       Example: "Cursor" (not "Cursorus")
        if (ending === '' && typeEntry.declension === 3) {
            return [
                {
                    stem: typeEntry.stem,
                    declension: typeEntry.declension,
                    gender: typeEntry.gender,
                    case: 'nominative',
                    number: 'singular',
                    jsType: typeEntry.jsType,
                    category: typeEntry.category,
                    generic: typeEntry.generic,
                },
            ];
        }

        const matches = endingsTable[ending];

        if (matches) {
            return matches.map(m => ({
                stem: typeEntry.stem,
                declension: typeEntry.declension,
                gender: typeEntry.gender,
                case: m.case,
                number: m.number,
                jsType: typeEntry.jsType,
                category: typeEntry.category,
                generic: typeEntry.generic,
            }));
        }
    }

    // Return what we learned, with suggestion if available
    if (bestMatch) {
        return { error: 'invalid_ending', word, stem: bestMatch.stem, ending: bestMatch.ending };
    }

    const suggestion = findClosestMatch(
        word,
        builtinTypes.map(t => t.stem),
    );

    return { error: 'unknown_stem', word, suggestion };
}

// ---------------------------------------------------------------------------
// Verb Parsing
// ---------------------------------------------------------------------------

/**
 * Parse an inflected Latin verb to determine its grammatical properties.
 *
 * ALGORITHM:
 *   1. Normalize to lowercase
 *   2. Try each known verb stem
 *   3. Extract ending by removing stem
 *   4. Select conjugation table (1st or 3rd conjugation)
 *   5. Lookup ending to get tense/person/number
 *   6. Return all matching interpretations
 *
 * WHY: Latin verbs encode tense, person, number, and mood in their endings.
 *      Present tense = synchronous, Future tense = asynchronous (returns Promise).
 *      This mapping allows Latin grammar to express async/await naturally.
 *
 * GRAMMAR:
 *   1st conjugation: stem + {-o, -as, -at, -amus, -atis, -ant} (present)
 *                    stem + {-abo, -abis, -abit, ...} (future)
 *   3rd conjugation: stem + {-o, -is, -it, -imus, -itis, -unt} (present)
 *                    stem + {-am, -es, -et, ...} (future)
 *
 * @param word - The inflected Latin verb (e.g., "mittit", "legam")
 * @returns Array of possible interpretations, or LexiconError
 */
export function parseVerb(word: string): ParsedVerb[] | LexiconError {
    const lowerWord = word.toLowerCase();

    // Track best match for error reporting
    let bestMatch: { stem: string; ending: string } | null = null;

    for (const verb of verbs) {
        if (!lowerWord.startsWith(verb.stem)) {
            continue;
        }

        const ending = lowerWord.slice(verb.stem.length);

        let endingsTable: typeof conjugation1Endings | null = null;

        if (verb.conjugation === 1) {
            endingsTable = conjugation1Endings;
        } else if (verb.conjugation === 2) {
            endingsTable = conjugation2Endings;
        } else if (verb.conjugation === 3) {
            endingsTable = conjugation3Endings;
        } else if (verb.conjugation === 4) {
            endingsTable = conjugation4Endings;
        }

        if (!endingsTable) {
            continue;
        }

        // Remember we got this far (stem matched)
        bestMatch = { stem: verb.stem, ending };

        const matches = endingsTable[ending];

        if (matches) {
            return matches.map(m => ({
                stem: verb.stem,
                conjugation: verb.conjugation,
                tense: m.tense as Tense,
                person: m.person,
                number: m.number as Number | undefined,
                // WHY: Future tense verbs are async in our mapping
                async: m.async,
            }));
        }
    }

    // Return what we learned, with suggestion if available
    if (bestMatch) {
        return { error: 'invalid_ending', word, stem: bestMatch.stem, ending: bestMatch.ending };
    }

    const suggestion = findClosestMatch(
        word,
        verbs.map(v => v.stem),
    );

    return { error: 'unknown_stem', word, suggestion };
}

// =============================================================================
// RE-EXPORTS
// =============================================================================

export * from './types';
export { isKeyword, getKeyword, keywords } from './keywords';
export {
    isBuiltinType,
    getBuiltinType,
    builtinTypes,
    typeModifiers,
    isTypeModifier,
} from './types-builtin';
export type { TypeEntry, TypeModifier } from './types-builtin';
