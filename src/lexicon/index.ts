import type { ParsedNoun, ParsedVerb, Case, Number, Tense } from "./types"
import { nouns, declension2MascEndings } from "./nouns"
import { verbs, conjugation1Endings, conjugation3Endings } from "./verbs"

export function parseNoun(word: string): ParsedNoun[] | null {
  const lowerWord = word.toLowerCase()

  // Try each noun in the lexicon
  for (const noun of nouns) {
    // Check if word starts with this stem
    if (!lowerWord.startsWith(noun.stem)) continue

    const ending = lowerWord.slice(noun.stem.length)

    // Only supporting 2nd declension masculine for now
    if (noun.declension === 2 && noun.gender === "masculine") {
      const matches = declension2MascEndings[ending]
      if (matches) {
        return matches.map((m) => ({
          stem: noun.stem,
          declension: noun.declension,
          gender: noun.gender,
          case: m.case as Case,
          number: m.number as Number,
        }))
      }
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
