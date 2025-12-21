export interface KeywordEntry {
  latin: string
  javascript: string
  category: "control" | "declaration" | "operator" | "value" | "preposition" | "modifier"
}

export const keywords: KeywordEntry[] = [
  // Control flow
  { latin: "si", javascript: "if", category: "control" },
  { latin: "aliter", javascript: "else", category: "control" },
  { latin: "dum", javascript: "while", category: "control" },
  { latin: "fac", javascript: "do", category: "control" },
  { latin: "pro", javascript: "for", category: "control" },
  { latin: "elige", javascript: "switch", category: "control" },
  { latin: "quando", javascript: "case", category: "control" },
  { latin: "rumpe", javascript: "break", category: "control" },
  { latin: "perge", javascript: "continue", category: "control" },
  { latin: "redde", javascript: "return", category: "control" },
  { latin: "tempta", javascript: "try", category: "control" },
  { latin: "cape", javascript: "catch", category: "control" },
  { latin: "demum", javascript: "finally", category: "control" },
  { latin: "iace", javascript: "throw", category: "control" },
  { latin: "exspecta", javascript: "await", category: "control" },

  // Declarations
  { latin: "esto", javascript: "let", category: "declaration" },
  { latin: "fixum", javascript: "const", category: "declaration" },
  { latin: "functio", javascript: "function", category: "declaration" },
  { latin: "novum", javascript: "new", category: "declaration" },

  // Modifiers
  { latin: "futura", javascript: "async", category: "modifier" },

  // Operators
  { latin: "et", javascript: "&&", category: "operator" },
  { latin: "aut", javascript: "||", category: "operator" },
  { latin: "non", javascript: "!", category: "operator" },

  // Values
  { latin: "verum", javascript: "true", category: "value" },
  { latin: "falsum", javascript: "false", category: "value" },
  { latin: "nihil", javascript: "null", category: "value" },

  // Prepositions (used with cases)
  { latin: "in", javascript: "in", category: "preposition" },
  { latin: "ex", javascript: "of", category: "preposition" },
  { latin: "cum", javascript: "with", category: "preposition" },
  { latin: "ad", javascript: "to", category: "preposition" },
]

const keywordMap = new Map(keywords.map((k) => [k.latin, k]))

export function isKeyword(word: string): boolean {
  return keywordMap.has(word.toLowerCase())
}

export function getKeyword(word: string): KeywordEntry | undefined {
  return keywordMap.get(word.toLowerCase())
}
