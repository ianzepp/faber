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
 * - 1st: -are verbs (creare, portare) - most regular
 * - 2nd: -ēre verbs (not yet implemented)
 * - 3rd: -ere verbs (mittere, legere) - common but irregular
 * - 4th: -ire verbs (not yet implemented)
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
 * ERRORS: Returns null for unsupported conjugations (2nd, 4th not implemented)
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

import type { VerbEntry } from "./types"

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
  { stem: "mitt", conjugation: 3, meaning: "send" },
  { stem: "leg", conjugation: 3, meaning: "read" },
  { stem: "scrib", conjugation: 3, meaning: "write" },
  { stem: "cre", conjugation: 1, meaning: "create" },
  { stem: "port", conjugation: 1, meaning: "carry" },
]

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
export const conjugation3Endings: Record<string, { tense: string; person?: number; number?: string; async: boolean }[]> = {
  // WHY: Imperative is command form (2nd person only), always synchronous
  "e": [{ tense: "imperative", person: 2, number: "singular", async: false }],
  "ite": [{ tense: "imperative", person: 2, number: "plural", async: false }],

  // WHY: Present tense describes current action, maps to synchronous functions
  "o": [{ tense: "present", person: 1, number: "singular", async: false }],
  "is": [{ tense: "present", person: 2, number: "singular", async: false }],
  "it": [{ tense: "present", person: 3, number: "singular", async: false }],
  "imus": [{ tense: "present", person: 1, number: "plural", async: false }],
  "itis": [{ tense: "present", person: 2, number: "plural", async: false }],
  "unt": [{ tense: "present", person: 3, number: "plural", async: false }],

  // WHY: Future tense describes future action, maps to async functions (Promise)
  //      Example: "mittet" (will send) -> async function returning Promise
  "am": [{ tense: "future", person: 1, number: "singular", async: true }],
  "es": [{ tense: "future", person: 2, number: "singular", async: true }],
  "et": [{ tense: "future", person: 3, number: "singular", async: true }],
  "emus": [{ tense: "future", person: 1, number: "plural", async: true }],
  "etis": [{ tense: "future", person: 2, number: "plural", async: true }],
  "ent": [{ tense: "future", person: 3, number: "plural", async: true }],
}

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
export const conjugation1Endings: Record<string, { tense: string; person?: number; number?: string; async: boolean }[]> = {
  // WHY: Imperative singular is just the stem + -a
  "a": [{ tense: "imperative", person: 2, number: "singular", async: false }],
  "ate": [{ tense: "imperative", person: 2, number: "plural", async: false }],

  // WHY: Present tense uses -a- theme vowel characteristic of 1st conjugation
  "o": [{ tense: "present", person: 1, number: "singular", async: false }],
  "as": [{ tense: "present", person: 2, number: "singular", async: false }],
  "at": [{ tense: "present", person: 3, number: "singular", async: false }],
  "amus": [{ tense: "present", person: 1, number: "plural", async: false }],
  "atis": [{ tense: "present", person: 2, number: "plural", async: false }],
  "ant": [{ tense: "present", person: 3, number: "plural", async: false }],

  // WHY: Future tense in 1st conjugation uses -ab- + personal endings
  //      Different from 3rd conjugation which uses -e- + personal endings
  "abo": [{ tense: "future", person: 1, number: "singular", async: true }],
  "abis": [{ tense: "future", person: 2, number: "singular", async: true }],
  "abit": [{ tense: "future", person: 3, number: "singular", async: true }],
  "abimus": [{ tense: "future", person: 1, number: "plural", async: true }],
  "abitis": [{ tense: "future", person: 2, number: "plural", async: true }],
  "abunt": [{ tense: "future", person: 3, number: "plural", async: true }],
}
