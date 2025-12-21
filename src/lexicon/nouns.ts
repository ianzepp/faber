import type { NounEntry, Declension, Gender, Case, Number } from "./types"

type EndingMap = Record<string, { case: Case; number: Number }[]>

// 1st declension (feminine, -a): Lista, Tabula, Copia
export const declension1Endings: EndingMap = {
  "a": [
    { case: "nominative", number: "singular" },
    { case: "ablative", number: "singular" },
  ],
  "am": [{ case: "accusative", number: "singular" }],
  "ae": [
    { case: "genitive", number: "singular" },
    { case: "dative", number: "singular" },
    { case: "nominative", number: "plural" },
  ],
  "as": [{ case: "accusative", number: "plural" }],
  "arum": [{ case: "genitive", number: "plural" }],
  "is": [
    { case: "dative", number: "plural" },
    { case: "ablative", number: "plural" },
  ],
}

// 2nd declension masculine (-us): Numerus, Usuarius
export const declension2MascEndings: EndingMap = {
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

// 2nd declension neuter (-um): Promissum, Erratum, Datum
export const declension2NeutEndings: EndingMap = {
  "um": [
    { case: "nominative", number: "singular" },
    { case: "accusative", number: "singular" },
  ],
  "i": [{ case: "genitive", number: "singular" }],
  "o": [
    { case: "dative", number: "singular" },
    { case: "ablative", number: "singular" },
  ],
  "a": [
    { case: "nominative", number: "plural" },
    { case: "accusative", number: "plural" },
  ],
  "orum": [{ case: "genitive", number: "plural" }],
  "is": [
    { case: "dative", number: "plural" },
    { case: "ablative", number: "plural" },
  ],
}

// 3rd declension (-io, -or, -us, etc.): Functio, Cursor, Tempus
export const declension3Endings: EndingMap = {
  // Singular varies by stem, but these are common accusative/genitive/etc.
  "em": [{ case: "accusative", number: "singular" }],
  "is": [{ case: "genitive", number: "singular" }],
  "i": [{ case: "dative", number: "singular" }],
  "e": [{ case: "ablative", number: "singular" }],
  "es": [
    { case: "nominative", number: "plural" },
    { case: "accusative", number: "plural" },
  ],
  "um": [{ case: "genitive", number: "plural" }],
  "ibus": [
    { case: "dative", number: "plural" },
    { case: "ablative", number: "plural" },
  ],
}

// 4th declension (-us masc): Textus, Fluxus
export const declension4Endings: EndingMap = {
  "us": [
    { case: "nominative", number: "singular" },
    { case: "genitive", number: "singular" },
  ],
  "um": [{ case: "accusative", number: "singular" }],
  "ui": [{ case: "dative", number: "singular" }],
  "u": [{ case: "ablative", number: "singular" }],
  "uum": [{ case: "genitive", number: "plural" }],
  "ibus": [
    { case: "dative", number: "plural" },
    { case: "ablative", number: "plural" },
  ],
}

export function getEndingsForDeclension(
  declension: Declension,
  gender: Gender
): EndingMap | null {
  if (declension === 1) return declension1Endings
  if (declension === 2 && gender === "masculine") return declension2MascEndings
  if (declension === 2 && gender === "neuter") return declension2NeutEndings
  if (declension === 3) return declension3Endings
  if (declension === 4) return declension4Endings
  return null
}

// Common nouns for user code
export const nouns: NounEntry[] = [
  { stem: "nunti", declension: 2, gender: "masculine", meaning: "message" },
  { stem: "numer", declension: 2, gender: "masculine", meaning: "number" },
  { stem: "usuar", declension: 2, gender: "masculine", meaning: "user" },
  { stem: "dat", declension: 2, gender: "neuter", meaning: "data" },
]
