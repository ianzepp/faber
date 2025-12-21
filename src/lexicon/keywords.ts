export interface KeywordEntry {
  latin: string
  meaning: string
  category: "control" | "declaration" | "operator" | "value" | "preposition" | "modifier"
}

export const keywords: KeywordEntry[] = [
  // Control flow
  { latin: "si", meaning: "if", category: "control" },
  { latin: "aliter", meaning: "else", category: "control" },
  { latin: "dum", meaning: "while", category: "control" },
  { latin: "fac", meaning: "do", category: "control" },
  { latin: "pro", meaning: "for", category: "control" },
  { latin: "elige", meaning: "switch", category: "control" },
  { latin: "quando", meaning: "case", category: "control" },
  { latin: "rumpe", meaning: "break", category: "control" },
  { latin: "perge", meaning: "continue", category: "control" },
  { latin: "redde", meaning: "return", category: "control" },
  { latin: "tempta", meaning: "try", category: "control" },
  { latin: "cape", meaning: "catch", category: "control" },
  { latin: "demum", meaning: "finally", category: "control" },
  { latin: "iace", meaning: "throw", category: "control" },
  { latin: "exspecta", meaning: "await", category: "control" },

  // Declarations
  { latin: "esto", meaning: "let", category: "declaration" },
  { latin: "fixum", meaning: "const", category: "declaration" },
  { latin: "functio", meaning: "function", category: "declaration" },
  { latin: "novum", meaning: "new", category: "declaration" },
  { latin: "importa", meaning: "import", category: "declaration" },

  // Modifiers
  { latin: "futura", meaning: "async", category: "modifier" },

  // Operators
  { latin: "et", meaning: "&&", category: "operator" },
  { latin: "aut", meaning: "||", category: "operator" },
  { latin: "non", meaning: "!", category: "operator" },

  // Values
  { latin: "verum", meaning: "true", category: "value" },
  { latin: "falsum", meaning: "false", category: "value" },
  { latin: "nihil", meaning: "null", category: "value" },

  // Prepositions (used with cases)
  { latin: "in", meaning: "in", category: "preposition" },
  { latin: "ex", meaning: "of", category: "preposition" },
  { latin: "cum", meaning: "with", category: "preposition" },
  { latin: "ad", meaning: "to", category: "preposition" },
]

const keywordMap = new Map(keywords.map((k) => [k.latin, k]))

export function isKeyword(word: string): boolean {
  return keywordMap.has(word.toLowerCase())
}

export function getKeyword(word: string): KeywordEntry | undefined {
  return keywordMap.get(word.toLowerCase())
}
