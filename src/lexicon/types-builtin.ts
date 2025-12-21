import type { NounEntry } from "./types"

// Built-in type names for Faber Romanus
// These map to JavaScript/TypeScript types

export interface TypeEntry extends NounEntry {
  jsType: string
  category: "primitive" | "collection" | "structural" | "iteration"
  generic?: boolean
}

export const builtinTypes: TypeEntry[] = [
  // Primitives
  { stem: "Text", declension: 4, gender: "masculine", meaning: "text/string", jsType: "string", category: "primitive" },
  { stem: "Numer", declension: 2, gender: "masculine", meaning: "number", jsType: "number", category: "primitive" },
  { stem: "Bivalen", declension: 3, gender: "masculine", meaning: "boolean", jsType: "boolean", category: "primitive" },
  { stem: "Sign", declension: 2, gender: "neuter", meaning: "symbol", jsType: "symbol", category: "primitive" },
  { stem: "Incert", declension: 2, gender: "neuter", meaning: "undefined", jsType: "undefined", category: "primitive" },

  // Collections (generic)
  { stem: "List", declension: 1, gender: "feminine", meaning: "list/array", jsType: "Array", category: "collection", generic: true },
  { stem: "Tabul", declension: 1, gender: "feminine", meaning: "table/map", jsType: "Map", category: "collection", generic: true },
  { stem: "Copi", declension: 1, gender: "feminine", meaning: "set/collection", jsType: "Set", category: "collection", generic: true },

  // Structural
  { stem: "R", declension: 3, gender: "feminine", meaning: "thing/object", jsType: "object", category: "structural" },  // res, rei (irregular but we handle it)
  { stem: "Function", declension: 3, gender: "feminine", meaning: "function", jsType: "Function", category: "structural" },
  { stem: "Promiss", declension: 2, gender: "neuter", meaning: "promise", jsType: "Promise", category: "structural", generic: true },
  { stem: "Tempor", declension: 3, gender: "neuter", meaning: "time/date", jsType: "Date", category: "structural" },
  { stem: "Errat", declension: 2, gender: "neuter", meaning: "error", jsType: "Error", category: "structural" },
  { stem: "Vacu", declension: 2, gender: "neuter", meaning: "void/empty", jsType: "void", category: "structural" },
  { stem: "Quodlibet", declension: 3, gender: "neuter", meaning: "any", jsType: "any", category: "structural" },
  { stem: "Ignot", declension: 2, gender: "neuter", meaning: "unknown", jsType: "unknown", category: "structural" },

  // Iteration & Streaming
  { stem: "Cursor", declension: 3, gender: "masculine", meaning: "cursor/iterator", jsType: "Iterator", category: "iteration", generic: true },
  { stem: "Flux", declension: 4, gender: "masculine", meaning: "flow/stream", jsType: "AsyncIterable", category: "iteration", generic: true },
]

const typeMap = new Map(builtinTypes.map((t) => [t.stem.toLowerCase(), t]))

export function isBuiltinType(stem: string): boolean {
  return typeMap.has(stem.toLowerCase())
}

export function getBuiltinType(stem: string): TypeEntry | undefined {
  return typeMap.get(stem.toLowerCase())
}
