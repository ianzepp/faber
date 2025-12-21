// Grammatical types for Latin morphology

export type Case = "nominative" | "accusative" | "genitive" | "dative" | "ablative"
export type Number = "singular" | "plural"
export type Gender = "masculine" | "feminine" | "neuter"
export type Declension = 1 | 2 | 3 | 4 | 5

export type Conjugation = 1 | 2 | 3 | 4
export type Tense = "present" | "imperative" | "future"
export type Person = 1 | 2 | 3

export interface NounEntry {
  stem: string
  declension: Declension
  gender: Gender
  meaning: string
}

export interface VerbEntry {
  stem: string
  conjugation: Conjugation
  meaning: string
}

export interface ParsedNoun {
  stem: string
  declension: Declension
  gender: Gender
  case: Case
  number: Number
}

export interface ParsedVerb {
  stem: string
  conjugation: Conjugation
  tense: Tense
  person?: Person
  number?: Number
  async: boolean
}
