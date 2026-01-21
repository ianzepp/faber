/**
 * Nouns - Latin Noun Declensions and User Vocabulary
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (morphological data)
 *
 * ARCHITECTURE
 * ============
 * This module defines the Latin noun declension system and provides vocabulary
 * for user-defined nouns. Latin nouns decline (change form) based on their
 * grammatical function in a sentence, indicated by case and number.
 *
 * LATIN CASE SYSTEM:
 * - Nominative: Subject of the sentence ("The user sends a message")
 * - Accusative: Direct object ("User sends THE MESSAGE")
 * - Genitive: Possession ("The message'S content")
 * - Dative: Indirect object ("Send TO THE USER")
 * - Ablative: Instrument/means ("Send WITH THE DATA")
 *
 * Each noun belongs to a declension (1st through 5th) which determines its
 * ending patterns. Gender (masculine, feminine, neuter) also affects endings.
 * For example, 1st declension nouns are typically feminine and end in -a.
 *
 * DECLENSION PATTERNS:
 * - 1st: Feminine nouns ending in -a (lista, copia)
 * - 2nd: Masculine -us or neuter -um (numerus, datum)
 * - 3rd: Various forms, consonant stems (functio, cursor)
 * - 4th: -us masculine (textus, fluxus)
 * - 5th: -es feminine (not yet implemented)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Declension number and gender
 * OUTPUT: EndingMap mapping endings to case/number combinations
 * ERRORS: Returns null for unsupported declension/gender combinations
 *
 * INVARIANTS
 * ==========
 * INV-1: Each ending may map to multiple case/number pairs (homographs)
 * INV-2: All endings are lowercase (stems handle TitleCase for types)
 * INV-3: Ending tables are complete for the declension they represent
 * INV-4: Empty string endings are valid (e.g., 3rd declension nominative)
 *
 * @module lexicon/nouns
 */

import type { NounEntry, Declension, Gender, Case, Number } from './types';

// =============================================================================
// TYPES
// =============================================================================

/**
 * Map from noun ending to possible case/number interpretations.
 *
 * WHY: Array values handle homographs (same ending, multiple meanings).
 *      Example: "lista" could be nominative or ablative singular.
 */
type EndingMap = Record<string, { case: Case; number: Number }[]>;

// =============================================================================
// DECLENSION TABLES
// =============================================================================

// ---------------------------------------------------------------------------
// 1st Declension (Feminine -a)
// ---------------------------------------------------------------------------

/**
 * 1st declension endings (feminine, -a stems).
 *
 * EXAMPLES: lista (list), tabula (table), copia (set/collection)
 *
 * WHY: First declension is typically feminine and follows the -a pattern.
 *      Used for many collection types in Faber Romanus.
 */
export const declension1Endings: EndingMap = {
    // WHY: -a is homograph (nominative subject OR ablative instrument)
    a: [
        { case: 'nominative', number: 'singular' },
        { case: 'ablative', number: 'singular' },
    ],
    am: [{ case: 'accusative', number: 'singular' }],
    // WHY: -ae appears in three different contexts (genitive/dative sing, nominative pl)
    ae: [
        { case: 'genitive', number: 'singular' },
        { case: 'dative', number: 'singular' },
        { case: 'nominative', number: 'plural' },
    ],
    as: [{ case: 'accusative', number: 'plural' }],
    arum: [{ case: 'genitive', number: 'plural' }],
    // WHY: -is is shared between dative and ablative plural (syncretism)
    is: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

// ---------------------------------------------------------------------------
// 2nd Declension Masculine (-us)
// ---------------------------------------------------------------------------

/**
 * 2nd declension masculine endings (-us stems).
 *
 * EXAMPLES: numerus (number), Usuarius (user)
 *
 * WHY: Second declension masculine is the most common pattern for Latin nouns.
 *      Many technical terms use this declension.
 */
export const declension2MascEndings: EndingMap = {
    us: [{ case: 'nominative', number: 'singular' }],
    um: [{ case: 'accusative', number: 'singular' }],
    // WHY: -i is homograph (genitive singular "of the number" OR nominative plural "numbers")
    i: [
        { case: 'genitive', number: 'singular' },
        { case: 'nominative', number: 'plural' },
    ],
    // WHY: -o serves dual purpose (dative "to X" and ablative "by means of X")
    o: [
        { case: 'dative', number: 'singular' },
        { case: 'ablative', number: 'singular' },
    ],
    os: [{ case: 'accusative', number: 'plural' }],
    orum: [{ case: 'genitive', number: 'plural' }],
    is: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

// ---------------------------------------------------------------------------
// 2nd Declension Neuter (-um)
// ---------------------------------------------------------------------------

/**
 * 2nd declension neuter endings (-um stems).
 *
 * EXAMPLES: promissum (promise), erratum (error), datum (data)
 *
 * WHY: Neuter nouns follow the rule "nominative equals accusative" - the subject
 *      and direct object forms are identical. This is a key feature of Latin neuters.
 */
export const declension2NeutEndings: EndingMap = {
    // WHY: Neuter rule - nominative and accusative are always identical
    um: [
        { case: 'nominative', number: 'singular' },
        { case: 'accusative', number: 'singular' },
    ],
    i: [{ case: 'genitive', number: 'singular' }],
    o: [
        { case: 'dative', number: 'singular' },
        { case: 'ablative', number: 'singular' },
    ],
    // WHY: Plural also follows neuter rule - nominative/accusative identical
    a: [
        { case: 'nominative', number: 'plural' },
        { case: 'accusative', number: 'plural' },
    ],
    orum: [{ case: 'genitive', number: 'plural' }],
    is: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

// ---------------------------------------------------------------------------
// 3rd Declension (Consonant stems)
// ---------------------------------------------------------------------------

/**
 * 3rd declension endings (consonant stems, masculine/feminine).
 *
 * EXAMPLES: cursor (cursor/iterator)
 *
 * WHY: Third declension is the most irregular and complex. Nominative singular
 *      varies greatly (no predictable ending), but other cases follow patterns.
 *      Many Greek loan words use this declension.
 *
 * EDGE: Nominative singular must be handled separately in parsing code (no ending).
 */
export const declension3Endings: EndingMap = {
    // EDGE: Nominative singular has no standard ending - handled in parsing code
    em: [{ case: 'accusative', number: 'singular' }],
    is: [{ case: 'genitive', number: 'singular' }],
    i: [{ case: 'dative', number: 'singular' }],
    e: [{ case: 'ablative', number: 'singular' }],
    // WHY: Unlike other declensions, 3rd has nominative = accusative for plural
    es: [
        { case: 'nominative', number: 'plural' },
        { case: 'accusative', number: 'plural' },
    ],
    um: [{ case: 'genitive', number: 'plural' }],
    ibus: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

/**
 * 3rd declension neuter endings (consonant stems).
 *
 * EXAMPLES: Tempus (time), Corpus (body), Opus (work)
 *
 * WHY: Neuter 3rd declension follows the neuter rule (nom = acc) and has
 *      -a for nominative/accusative plural instead of -es.
 *
 * EDGE: Nominative singular has alternate form (tempus, not tempor) - handled
 *       via nominative field in TypeEntry. Accusative = nominative for neuters.
 */
export const declension3NeutEndings: EndingMap = {
    // EDGE: Nominative/accusative singular use alternate form (e.g., Tempus)
    //       Handled via nominative field in TypeEntry
    is: [{ case: 'genitive', number: 'singular' }],
    i: [{ case: 'dative', number: 'singular' }],
    e: [{ case: 'ablative', number: 'singular' }],
    // WHY: Neuter plural uses -a instead of -es
    a: [
        { case: 'nominative', number: 'plural' },
        { case: 'accusative', number: 'plural' },
    ],
    um: [{ case: 'genitive', number: 'plural' }],
    ibus: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

// ---------------------------------------------------------------------------
// 4th Declension Masculine (-us)
// ---------------------------------------------------------------------------

/**
 * 4th declension masculine endings (-us stems, but different from 2nd declension).
 *
 * EXAMPLES: textus (text/string), fluxus (flow/stream)
 *
 * WHY: Fourth declension also ends in -us (like 2nd masculine), but with different
 *      case endings. Genitive singular is -us (not -i like 2nd declension).
 *      Less common but used for abstract nouns and some technical terms.
 */
export const declension4Endings: EndingMap = {
    // WHY: 4th declension has nominative = genitive (both -us), unlike other declensions
    us: [
        { case: 'nominative', number: 'singular' },
        { case: 'genitive', number: 'singular' },
    ],
    um: [{ case: 'accusative', number: 'singular' }],
    ui: [{ case: 'dative', number: 'singular' }],
    u: [{ case: 'ablative', number: 'singular' }],
    uum: [{ case: 'genitive', number: 'plural' }],
    ibus: [
        { case: 'dative', number: 'plural' },
        { case: 'ablative', number: 'plural' },
    ],
};

// =============================================================================
// DECLENSION LOOKUP
// =============================================================================

/**
 * Get the appropriate ending table for a declension/gender combination.
 *
 * WHY: Gender affects endings in 2nd and 3rd declension (masculine vs neuter have
 *      different patterns). 1st and 4th declensions are gender-neutral in their endings.
 *
 * @param declension - The declension number (1-5)
 * @param gender - The grammatical gender
 * @returns EndingMap for the declension, or null if unsupported
 */
export function getEndingsForDeclension(declension: Declension, gender: Gender): EndingMap | null {
    if (declension === 1) {
        return declension1Endings;
    }

    // WHY: 2nd declension splits by gender (masculine -us vs neuter -um)
    if (declension === 2 && gender === 'masculine') {
        return declension2MascEndings;
    }

    if (declension === 2 && gender === 'neuter') {
        return declension2NeutEndings;
    }

    // WHY: 3rd declension neuters have -a plurals instead of -es
    if (declension === 3 && gender === 'neuter') {
        return declension3NeutEndings;
    }

    if (declension === 3) {
        return declension3Endings;
    }

    if (declension === 4) {
        return declension4Endings;
    }

    // EDGE: 5th declension not yet implemented
    return null;
}

// =============================================================================
// USER VOCABULARY
// =============================================================================

/**
 * Common nouns available for use in user code.
 *
 * WHY: These are example nouns for documentation and testing. Users can define
 *      their own nouns; this is just starter vocabulary.
 *
 * DESIGN: Stems stored without endings (e.g., "numer" not "numerus") to match
 *         the parsing algorithm which strips endings.
 */
export const nouns: NounEntry[] = [
    { stem: 'nunti', declension: 2, gender: 'masculine', meaning: 'message' },
    { stem: 'numer', declension: 2, gender: 'masculine', meaning: 'number' },
    { stem: 'usuar', declension: 2, gender: 'masculine', meaning: 'user' },
    { stem: 'dat', declension: 2, gender: 'neuter', meaning: 'data' },
];
