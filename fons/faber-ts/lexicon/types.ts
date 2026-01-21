/**
 * Types - Latin Grammar Type Definitions
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (type definitions)
 *
 * ARCHITECTURE
 * ============
 * This module defines TypeScript types for representing Latin grammatical
 * concepts. These types are used throughout the lexicon module to classify
 * and parse inflected Latin words.
 *
 * LATIN GRAMMAR FUNDAMENTALS:
 *
 * NOUNS decline by:
 * - Case: Grammatical function (nominative, accusative, genitive, dative, ablative)
 * - Number: Singular or plural
 * - Gender: Masculine, feminine, or neuter (inherent to the noun)
 * - Declension: Pattern group (1st through 5th)
 *
 * VERBS conjugate by:
 * - Tense: When the action occurs (present, future) or mood (imperative)
 * - Person: Who performs (1st = I/we, 2nd = you, 3rd = he/she/it/they)
 * - Number: Singular or plural
 * - Conjugation: Pattern group (1st through 4th)
 *
 * WHY THESE TYPES:
 * - NounEntry/VerbEntry: Dictionary entries (stems with grammatical metadata)
 * - ParsedNoun/ParsedVerb: Analysis results (stem + inflectional information)
 *
 * These types enable the lexicon to map inflected forms back to their stems
 * and extract grammatical meaning, which the parser uses for semantic analysis.
 *
 * @module lexicon/types
 */

// =============================================================================
// GRAMMATICAL PRIMITIVES
// =============================================================================

// ---------------------------------------------------------------------------
// Noun Grammar
// ---------------------------------------------------------------------------

/**
 * Latin grammatical cases.
 *
 * WHY: Cases indicate a noun's syntactic role in the sentence.
 * - nominative: Subject ("The USER sends")
 * - accusative: Direct object ("Send THE MESSAGE")
 * - genitive: Possession ("The user'S data")
 * - dative: Indirect object ("Send TO THE USER")
 * - ablative: Means/instrument ("Send BY MEANS OF the function")
 */
export type Case = 'nominative' | 'accusative' | 'genitive' | 'dative' | 'ablative';

/**
 * Grammatical number (singular or plural).
 */
export type Number = 'singular' | 'plural';

/**
 * Latin grammatical genders.
 *
 * WHY: Gender is inherent to each noun and affects which endings it takes.
 *      Not semantic (like English) but grammatical (like French/German).
 */
export type Gender = 'masculine' | 'feminine' | 'neuter';

/**
 * Latin declensions (1st through 5th).
 *
 * WHY: Declension determines the pattern of case endings a noun follows.
 *      Like conjugation for verbs, it's a classification of inflection patterns.
 */
export type Declension = 1 | 2 | 3 | 4 | 5;

// ---------------------------------------------------------------------------
// Verb Grammar
// ---------------------------------------------------------------------------

/**
 * Latin conjugations (1st through 4th).
 *
 * WHY: Conjugation determines the pattern of person/number/tense endings.
 */
export type Conjugation = 1 | 2 | 3 | 4;

/**
 * Verb tenses and moods supported.
 *
 * WHY: Present = synchronous action, Future = asynchronous (returns Promise),
 *      Imperative = command form (used for statements/expressions).
 */
export type Tense = 'present' | 'imperative' | 'future';

/**
 * Grammatical person (1st, 2nd, 3rd).
 *
 * WHY: Indicates who performs the action.
 * - 1st person: I/we (ego, nos)
 * - 2nd person: you (tu, vos)
 * - 3rd person: he/she/it/they (is/ea/id, ei/eae/ea)
 */
export type Person = 1 | 2 | 3;

// =============================================================================
// LEXICON ENTRY TYPES
// =============================================================================

/**
 * Dictionary entry for a Latin noun.
 *
 * DESIGN: Stores stem (without endings) plus grammatical classification.
 *         Meaning is for documentation/error messages.
 */
export interface NounEntry {
    stem: string;
    declension: Declension;
    gender: Gender;
    meaning: string;
}

/**
 * Dictionary entry for a Latin verb.
 *
 * DESIGN: Stores stem (without endings) plus conjugation pattern.
 */
export interface VerbEntry {
    stem: string;
    conjugation: Conjugation;
    meaning: string;
}

// =============================================================================
// PARSED RESULT TYPES
// =============================================================================

/**
 * Result of parsing an inflected Latin noun.
 *
 * DESIGN: Combines the noun's dictionary information (stem, declension, gender)
 *         with the inflectional information extracted from the ending (case, number).
 */
export interface ParsedNoun {
    stem: string;
    declension: Declension;
    gender: Gender;
    case: Case;
    number: Number;
}

/**
 * Result of parsing an inflected Latin verb.
 *
 * DESIGN: Combines stem and conjugation with extracted tense/person/number.
 *         Person and number are optional for imperative forms.
 *
 * WHY: async field maps Latin future tense to JavaScript async functions.
 */
export interface ParsedVerb {
    stem: string;
    conjugation: Conjugation;
    tense: Tense;
    person?: Person;
    number?: Number;
    async: boolean;
}
