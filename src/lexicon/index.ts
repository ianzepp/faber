import type { ParsedNoun, ParsedVerb, Case, Number, Tense } from "./types"
import { nouns, getEndingsForDeclension } from "./nouns"
import { verbs, conjugation1Endings, conjugation3Endings } from "./verbs"
import { builtinTypes, type TypeEntry } from "./types-builtin"

export interface ParsedType extends ParsedNoun {
  jsType: string
  category: "primitive" | "collection" | "structural" | "iteration"
  generic?: boolean
}

export function parseNoun(word: string): ParsedNoun[] | null {
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

export function parseType(word: string): ParsedType[] | null {
  // Types are TitleCase, so preserve case for matching
  for (const typeEntry of builtinTypes) {
    // Check if word starts with the type stem (case-sensitive for types)
    if (!word.startsWith(typeEntry.stem)) continue

    const ending = word.slice(typeEntry.stem.length).toLowerCase()
    const endingsTable = getEndingsForDeclension(typeEntry.declension, typeEntry.gender)

    if (!endingsTable) continue

    // Handle nominative with no ending for 3rd declension
    // e.g., "Cursor" has no ending beyond the stem in nominative
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

export function parseVerb(word: string): ParsedVerb[] | null {
  const lowerWord = word.toLowerCase()

  for (const verb of verbs) {
    if (!lowerWord.startsWith(verb.stem)) continue

    const ending = lowerWord.slice(verb.stem.length)

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
        async: m.async,
      }))
    }
  }

  return null
}

export * from "./types"
export { isKeyword, getKeyword, keywords } from "./keywords"
export { isBuiltinType, getBuiltinType, builtinTypes } from "./types-builtin"
export type { TypeEntry } from "./types-builtin"
