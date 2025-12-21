import type { VerbEntry } from "./types"

export const verbs: VerbEntry[] = [
  { stem: "mitt", conjugation: 3, meaning: "send" },
  { stem: "leg", conjugation: 3, meaning: "read" },
  { stem: "scrib", conjugation: 3, meaning: "write" },
  { stem: "cre", conjugation: 1, meaning: "create" },
  { stem: "port", conjugation: 1, meaning: "carry" },
]

// 3rd conjugation endings (present/imperative/future)
export const conjugation3Endings: Record<string, { tense: string; person?: number; number?: string; async: boolean }[]> = {
  // Imperative (sync)
  "e": [{ tense: "imperative", person: 2, number: "singular", async: false }],
  "ite": [{ tense: "imperative", person: 2, number: "plural", async: false }],

  // Present (sync)
  "o": [{ tense: "present", person: 1, number: "singular", async: false }],
  "is": [{ tense: "present", person: 2, number: "singular", async: false }],
  "it": [{ tense: "present", person: 3, number: "singular", async: false }],
  "imus": [{ tense: "present", person: 1, number: "plural", async: false }],
  "itis": [{ tense: "present", person: 2, number: "plural", async: false }],
  "unt": [{ tense: "present", person: 3, number: "plural", async: false }],

  // Future (async)
  "am": [{ tense: "future", person: 1, number: "singular", async: true }],
  "es": [{ tense: "future", person: 2, number: "singular", async: true }],
  "et": [{ tense: "future", person: 3, number: "singular", async: true }],
  "emus": [{ tense: "future", person: 1, number: "plural", async: true }],
  "etis": [{ tense: "future", person: 2, number: "plural", async: true }],
  "ent": [{ tense: "future", person: 3, number: "plural", async: true }],
}

// 1st conjugation endings
export const conjugation1Endings: Record<string, { tense: string; person?: number; number?: string; async: boolean }[]> = {
  // Imperative (sync)
  "a": [{ tense: "imperative", person: 2, number: "singular", async: false }],
  "ate": [{ tense: "imperative", person: 2, number: "plural", async: false }],

  // Present (sync)
  "o": [{ tense: "present", person: 1, number: "singular", async: false }],
  "as": [{ tense: "present", person: 2, number: "singular", async: false }],
  "at": [{ tense: "present", person: 3, number: "singular", async: false }],
  "amus": [{ tense: "present", person: 1, number: "plural", async: false }],
  "atis": [{ tense: "present", person: 2, number: "plural", async: false }],
  "ant": [{ tense: "present", person: 3, number: "plural", async: false }],

  // Future (async)
  "abo": [{ tense: "future", person: 1, number: "singular", async: true }],
  "abis": [{ tense: "future", person: 2, number: "singular", async: true }],
  "abit": [{ tense: "future", person: 3, number: "singular", async: true }],
  "abimus": [{ tense: "future", person: 1, number: "plural", async: true }],
  "abitis": [{ tense: "future", person: 2, number: "plural", async: true }],
  "abunt": [{ tense: "future", person: 3, number: "plural", async: true }],
}
