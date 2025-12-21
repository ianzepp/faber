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
 * OUTPUT: Array of possible grammatical interpretations (may be multiple due to
 *         homographs - same form, different meanings)
 * ERRORS: Returns null when word is not recognized in lexicon
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

import type { ParsedNoun, ParsedVerb, Case, Number, Tense } from "./types"
import { nouns, getEndingsForDeclension } from "./nouns"
import { verbs, conjugation1Endings, conjugation3Endings } from "./verbs"
import { builtinTypes, type TypeEntry } from "./types-builtin"

// =============================================================================
// TYPES
// =============================================================================

/**
 * Parsed type with JavaScript/TypeScript target information.
 *
 * DESIGN: Extends ParsedNoun because Latin type names follow noun declension
 *         patterns. jsType maps to target language primitives.
 */
export interface ParsedType extends ParsedNoun {
  jsType: string
  category: "primitive" | "collection" | "structural" | "iteration"
  generic?: boolean
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
 * @returns Array of possible interpretations, or null if not recognized
 */
export function parseNoun(word: string): ParsedNoun[] | null {
  // WHY: Latin nouns in user code are case-insensitive (unlike types which are TitleCase)
  const lowerWord = word.toLowerCase()

  for (const noun of nouns) {
    if (!lowerWord.startsWith(noun.stem)) continue

    const ending = lowerWord.slice(noun.stem.length)
    const endingsTable = getEndingsForDeclension(noun.declension, noun.gender)

    if (!endingsTable) continue

    const matches = endingsTable[ending]
    if (matches) {
      return matches.map((m) => ({
        stem: noun.stem,
        declension: noun.declension,
        gender: noun.gender,
        case: m.case,
        number: m.number,
      }))
    }
  }

  return null
}

// ---------------------------------------------------------------------------
// Type Parsing
// ---------------------------------------------------------------------------

/**
 * Parse an inflected Latin type name to determine its grammatical properties.
 *
 * ALGORITHM:
 *   1. Preserve original case for stem matching (types are TitleCase)
 *   2. Try each built-in type stem
 *   3. Extract ending (convert to lowercase for ending lookup)
 *   4. Lookup ending in appropriate declension table
 *   5. Return all matching interpretations
 *
 * WHY: Types use TitleCase (Textus, Numerus) to distinguish from user nouns.
 *      This follows TypeScript convention where types start with uppercase.
 *
 * EDGE: 3rd declension types (Cursor, Functio) have no ending in nominative
 *       singular. Must handle empty ending specially.
 *
 * @param word - The inflected type name (e.g., "Textus", "Cursorem")
 * @returns Array of possible interpretations with target type info, or null
 */
export function parseType(word: string): ParsedType[] | null {
  // WHY: Case-sensitive stem matching preserves TitleCase convention for types
  for (const typeEntry of builtinTypes) {
    if (!word.startsWith(typeEntry.stem)) continue

    // WHY: Endings are lowercase in declension tables, but stem is TitleCase
    const ending = word.slice(typeEntry.stem.length).toLowerCase()
    const endingsTable = getEndingsForDeclension(typeEntry.declension, typeEntry.gender)

    if (!endingsTable) continue

    // EDGE: 3rd declension nominative singular has no ending
    //       Example: "Cursor" (not "Cursorus"), "Functio" (as-is)
    if (ending === "" && typeEntry.declension === 3) {
      return [{
        stem: typeEntry.stem,
        declension: typeEntry.declension,
        gender: typeEntry.gender,
        case: "nominative",
        number: "singular",
        jsType: typeEntry.jsType,
        category: typeEntry.category,
        generic: typeEntry.generic,
      }]
    }

    const matches = endingsTable[ending]
    if (matches) {
      return matches.map((m) => ({
        stem: typeEntry.stem,
        declension: typeEntry.declension,
        gender: typeEntry.gender,
        case: m.case,
        number: m.number,
        jsType: typeEntry.jsType,
        category: typeEntry.category,
        generic: typeEntry.generic,
      }))
    }
  }

  return null
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
 * @returns Array of possible interpretations, or null if not recognized
 */
export function parseVerb(word: string): ParsedVerb[] | null {
  const lowerWord = word.toLowerCase()

  for (const verb of verbs) {
    if (!lowerWord.startsWith(verb.stem)) continue

    const ending = lowerWord.slice(verb.stem.length)

    // WHY: Currently only 1st and 3rd conjugations implemented
    //      2nd and 4th conjugations can be added as needed
    let endingsTable: typeof conjugation1Endings | null = null
    if (verb.conjugation === 1) endingsTable = conjugation1Endings
    if (verb.conjugation === 3) endingsTable = conjugation3Endings

    if (!endingsTable) continue

    const matches = endingsTable[ending]
    if (matches) {
      return matches.map((m) => ({
        stem: verb.stem,
        conjugation: verb.conjugation,
        tense: m.tense as Tense,
        person: m.person,
        number: m.number as Number | undefined,
        // WHY: Future tense verbs are async in our mapping
        async: m.async,
      }))
    }
  }

  return null
}

// =============================================================================
// RE-EXPORTS
// =============================================================================

export * from "./types"
export { isKeyword, getKeyword, keywords } from "./keywords"
export { isBuiltinType, getBuiltinType, builtinTypes } from "./types-builtin"
export type { TypeEntry } from "./types-builtin"
