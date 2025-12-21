import type { NounEntry } from "./types"

// WHY: Start with 2nd declension masculine - most common pattern for our types
export const nouns: NounEntry[] = [
  { stem: "nunti", declension: 2, gender: "masculine", meaning: "message" },
  { stem: "numer", declension: 2, gender: "masculine", meaning: "number" },
  { stem: "usuar", declension: 2, gender: "masculine", meaning: "user" },
]

// 2nd declension masculine endings
export const declension2MascEndings: Record<string, { case: string; number: string }[]> = {
  "us": [{ case: "nominative", number: "singular" }],
  "um": [{ case: "accusative", number: "singular" }],
  "i": [
    { case: "genitive", number: "singular" },
    { case: "nominative", number: "plural" },
  ],
  "o": [
    { case: "dative", number: "singular" },
    { case: "ablative", number: "singular" },
  ],
  "os": [{ case: "accusative", number: "plural" }],
  "orum": [{ case: "genitive", number: "plural" }],
  "is": [
    { case: "dative", number: "plural" },
    { case: "ablative", number: "plural" },
  ],
}
