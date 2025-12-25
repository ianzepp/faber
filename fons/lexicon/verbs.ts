/**
 * Verbs - Latin Verb Conjugations and User Vocabulary
 *
 * COMPILER PHASE
 * ==============
 * Lexical analysis (morphological data)
 *
 * ARCHITECTURE
 * ============
 * This module defines the Latin verb conjugation system and provides vocabulary
 * for user-defined verbs. Latin verbs conjugate (change form) to indicate tense,
 * person, number, and mood. The ending pattern depends on the verb's conjugation
 * class (1st through 4th).
 *
 * LATIN VERB SYSTEM:
 * - Tense/Mood: When/how the action occurs
 *   * Present: Action happening now (synchronous)
 *   * Future: Action will happen (asynchronous, returns Promise)
 *   * Imperative: Command (do this!)
 * - Person: Who performs (1st = I/we, 2nd = you, 3rd = he/they)
 * - Number: Singular or plural
 *
 * CONJUGATION PATTERNS:
 * - 1st: -are verbs (creare, portare) - transformations
 * - 2nd: -ēre verbs (videre, habere) - stateful operations
 * - 3rd: -ere verbs (mittere, legere) - transformations
 * - 4th: -ire verbs (aperire, audire) - IO/side effects
 *
 * MAPPING TO ASYNC/AWAIT:
 * Latin's future tense naturally expresses asynchronous operations. A future
 * tense verb (e.g., "mittet" = "will send") maps to async functions that return
 * Promises. Present tense = synchronous, Future = async.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Conjugation number
 * OUTPUT: EndingMap mapping endings to tense/person/number/async
 * ERRORS: Returns null for unrecognized verbs
 *
 * INVARIANTS
 * ==========
 * INV-1: All endings are lowercase
 * INV-2: Future tense always has async: true
 * INV-3: Present tense always has async: false
 * INV-4: Imperative forms may lack person/number (2nd person implied)
 *
 * @module lexicon/verbs
 */

import type { VerbEntry, Tense, Person, Number } from './types';

// =============================================================================
// USER VOCABULARY
// =============================================================================

/**
 * Common verbs available for use in user code.
 *
 * WHY: These are example verbs for documentation and testing. Users can define
 *      their own verbs; this is just starter vocabulary.
 *
 * DESIGN: Stems stored without endings (e.g., "mitt" not "mittere") to match
 *         the parsing algorithm which strips endings.
 */
export const verbs: VerbEntry[] = [
    // 1st conjugation (-are)
    // WHY: Creation operations
    { stem: 'cre', conjugation: 1, meaning: 'create' },

    // 2nd conjugation (-ēre)
    // WHY: Output and observation (like Unix echo/debug)
    { stem: 'vid', conjugation: 2, meaning: 'see/debug' },
    { stem: 'mon', conjugation: 2, meaning: 'warn' },
    { stem: 'respond', conjugation: 2, meaning: 'respond/yield' },

    // 3rd conjugation (-ere)
    // WHY: Data transformation (like Unix cat/send)
    { stem: 'mitt', conjugation: 3, meaning: 'send' },
    { stem: 'leg', conjugation: 3, meaning: 'read' },
    { stem: 'scrib', conjugation: 3, meaning: 'write' },

    // 4th conjugation (-ire)
    // WHY: IO and side-effects (like Unix open/close/sleep)
    { stem: 'aper', conjugation: 4, meaning: 'open' },
    { stem: 'fin', conjugation: 4, meaning: 'finish/close' },
    { stem: 'aud', conjugation: 4, meaning: 'hear/listen' },
    { stem: 'ven', conjugation: 4, meaning: 'come/fetch' },
    { stem: 'inven', conjugation: 4, meaning: 'find/grep' },
    { stem: 'sc', conjugation: 4, meaning: 'know/type-check' },
    { stem: 'dorm', conjugation: 4, meaning: 'sleep' },
    { stem: 'ex', conjugation: 4, meaning: 'exit' },
];

// =============================================================================
// CONJUGATION TABLES
// =============================================================================

// ---------------------------------------------------------------------------
// 3rd Conjugation
// ---------------------------------------------------------------------------

/**
 * 3rd conjugation endings (present/imperative/future).
 *
 * EXAMPLES: mittere (to send), legere (to read), scribere (to write)
 *
 * WHY: Third conjugation is one of the most common in Latin. It has a short
 *      stem vowel (unlike 2nd conjugation's long -ē-) and uses -i- as a
 *      connecting vowel in some forms.
 */
export const conjugation3Endings: Record<string, { tense: Tense; person?: Person; number?: Number; async: boolean }[]> = {
    // WHY: Imperative is command form (2nd person only), always synchronous
    e: [{ tense: 'imperative', person: 2, number: 'singular', async: false }],
    ite: [{ tense: 'imperative', person: 2, number: 'plural', async: false }],

    // WHY: Present tense describes current action, maps to synchronous functions
    o: [{ tense: 'present', person: 1, number: 'singular', async: false }],
    is: [{ tense: 'present', person: 2, number: 'singular', async: false }],
    it: [{ tense: 'present', person: 3, number: 'singular', async: false }],
    imus: [{ tense: 'present', person: 1, number: 'plural', async: false }],
    itis: [{ tense: 'present', person: 2, number: 'plural', async: false }],
    unt: [{ tense: 'present', person: 3, number: 'plural', async: false }],

    // WHY: Future tense describes future action, maps to async functions (Promise)
    //      Example: "mittet" (will send) -> async function returning Promise
    am: [{ tense: 'future', person: 1, number: 'singular', async: true }],
    es: [{ tense: 'future', person: 2, number: 'singular', async: true }],
    et: [{ tense: 'future', person: 3, number: 'singular', async: true }],
    emus: [{ tense: 'future', person: 1, number: 'plural', async: true }],
    etis: [{ tense: 'future', person: 2, number: 'plural', async: true }],
    ent: [{ tense: 'future', person: 3, number: 'plural', async: true }],
};

// ---------------------------------------------------------------------------
// 1st Conjugation
// ---------------------------------------------------------------------------

/**
 * 1st conjugation endings (present/imperative/future).
 *
 * EXAMPLES: creare (to create), portare (to carry)
 *
 * WHY: First conjugation is the largest and most regular conjugation class.
 *      Characterized by the -a- theme vowel in most forms.
 */
export const conjugation1Endings: Record<string, { tense: Tense; person?: Person; number?: Number; async: boolean }[]> = {
    // WHY: Imperative singular is just the stem + -a
    a: [{ tense: 'imperative', person: 2, number: 'singular', async: false }],
    ate: [{ tense: 'imperative', person: 2, number: 'plural', async: false }],

    // WHY: Present tense uses -a- theme vowel characteristic of 1st conjugation
    o: [{ tense: 'present', person: 1, number: 'singular', async: false }],
    as: [{ tense: 'present', person: 2, number: 'singular', async: false }],
    at: [{ tense: 'present', person: 3, number: 'singular', async: false }],
    amus: [{ tense: 'present', person: 1, number: 'plural', async: false }],
    atis: [{ tense: 'present', person: 2, number: 'plural', async: false }],
    ant: [{ tense: 'present', person: 3, number: 'plural', async: false }],

    // WHY: Future tense in 1st conjugation uses -ab- + personal endings
    //      Different from 3rd conjugation which uses -e- + personal endings
    abo: [{ tense: 'future', person: 1, number: 'singular', async: true }],
    abis: [{ tense: 'future', person: 2, number: 'singular', async: true }],
    abit: [{ tense: 'future', person: 3, number: 'singular', async: true }],
    abimus: [{ tense: 'future', person: 1, number: 'plural', async: true }],
    abitis: [{ tense: 'future', person: 2, number: 'plural', async: true }],
    abunt: [{ tense: 'future', person: 3, number: 'plural', async: true }],
};

// ---------------------------------------------------------------------------
// 2nd Conjugation
// ---------------------------------------------------------------------------

/**
 * 2nd conjugation endings (present/imperative/future).
 *
 * EXAMPLES: videre (to see), habere (to have), tenere (to hold)
 *
 * WHY: Second conjugation has a long -ē- theme vowel. Used for stateful
 *      operations: checking (.has), holding (loop variables), yielding.
 *
 * SEMANTIC MAPPING:
 * - videre: console.debug (developer output)
 * - habere: .has() / .includes() (containment checks)
 * - tenere: loop variable binding
 * - monere: console.warn
 * - respondere: yield (generators/async iterators)
 */
export const conjugation2Endings: Record<string, { tense: Tense; person?: Person; number?: Number; async: boolean }[]> = {
    // WHY: Imperative uses -e (like 3rd conj) but from -ē- stem
    e: [{ tense: 'imperative', person: 2, number: 'singular', async: false }],
    ete: [{ tense: 'imperative', person: 2, number: 'plural', async: false }],

    // WHY: Present tense uses -e- theme vowel characteristic of 2nd conjugation
    eo: [{ tense: 'present', person: 1, number: 'singular', async: false }],
    es: [{ tense: 'present', person: 2, number: 'singular', async: false }],
    et: [{ tense: 'present', person: 3, number: 'singular', async: false }],
    emus: [{ tense: 'present', person: 1, number: 'plural', async: false }],
    etis: [{ tense: 'present', person: 2, number: 'plural', async: false }],
    ent: [{ tense: 'present', person: 3, number: 'plural', async: false }],

    // WHY: Future tense in 2nd conjugation uses -eb- + personal endings
    //      Same pattern as 1st conjugation but with -e- instead of -a-
    ebo: [{ tense: 'future', person: 1, number: 'singular', async: true }],
    ebis: [{ tense: 'future', person: 2, number: 'singular', async: true }],
    ebit: [{ tense: 'future', person: 3, number: 'singular', async: true }],
    ebimus: [{ tense: 'future', person: 1, number: 'plural', async: true }],
    ebitis: [{ tense: 'future', person: 2, number: 'plural', async: true }],
    ebunt: [{ tense: 'future', person: 3, number: 'plural', async: true }],
};

// ---------------------------------------------------------------------------
// 4th Conjugation
// ---------------------------------------------------------------------------

/**
 * 4th conjugation endings (present/imperative/future).
 *
 * EXAMPLES: aperire (to open), finire (to finish), audire (to hear)
 *
 * WHY: Fourth conjugation has a long -ī- theme vowel. Used for IO and
 *      side-effect operations: opening/closing, listening, finding.
 *
 * SEMANTIC MAPPING:
 * - aperire: open file/stream/connection
 * - finire: close/dispose resources
 * - audire: listen for events, receive input
 * - venire: incoming data, fetch arrival
 * - invenire: .find() / search operations
 * - scire: type guards, instanceof checks
 */
export const conjugation4Endings: Record<string, { tense: Tense; person?: Person; number?: Number; async: boolean }[]> = {
    // WHY: Imperative uses -i (singular), -ite (plural)
    i: [{ tense: 'imperative', person: 2, number: 'singular', async: false }],
    ite: [{ tense: 'imperative', person: 2, number: 'plural', async: false }],

    // WHY: Present tense uses -i- theme vowel, but 1st person has -io
    io: [{ tense: 'present', person: 1, number: 'singular', async: false }],
    is: [{ tense: 'present', person: 2, number: 'singular', async: false }],
    it: [{ tense: 'present', person: 3, number: 'singular', async: false }],
    imus: [{ tense: 'present', person: 1, number: 'plural', async: false }],
    itis: [{ tense: 'present', person: 2, number: 'plural', async: false }],
    iunt: [{ tense: 'present', person: 3, number: 'plural', async: false }],

    // WHY: Future tense in 4th conjugation uses -ie- + personal endings
    iam: [{ tense: 'future', person: 1, number: 'singular', async: true }],
    ies: [{ tense: 'future', person: 2, number: 'singular', async: true }],
    iet: [{ tense: 'future', person: 3, number: 'singular', async: true }],
    iemus: [{ tense: 'future', person: 1, number: 'plural', async: true }],
    ietis: [{ tense: 'future', person: 2, number: 'plural', async: true }],
    ient: [{ tense: 'future', person: 3, number: 'plural', async: true }],
};
