import { test, expect, describe } from "bun:test"
import { parseNoun, parseVerb, parseType, isKeyword, getKeyword, isBuiltinType } from "./index"

describe("parseNoun", () => {
  describe("2nd declension masculine", () => {
    test("nominative singular: nuntius", () => {
      const results = parseNoun("nuntius")
      expect(results).not.toBeNull()
      expect(results).toHaveLength(1)
      expect(results![0]).toEqual({
        stem: "nunti",
        declension: 2,
        gender: "masculine",
        case: "nominative",
        number: "singular",
      })
    })

    test("accusative singular: nuntium", () => {
      const results = parseNoun("nuntium")
      expect(results).not.toBeNull()
      expect(results).toHaveLength(1)
      expect(results![0].case).toBe("accusative")
      expect(results![0].number).toBe("singular")
    })

    test("genitive singular: nuntii", () => {
      const results = parseNoun("nuntii")
      expect(results).not.toBeNull()
      // Could be genitive singular OR nominative plural
      expect(results!.length).toBeGreaterThanOrEqual(1)
      expect(results!.some((r) => r.case === "genitive")).toBe(true)
    })

    test("dative/ablative singular: nuntio (ambiguous)", () => {
      const results = parseNoun("nuntio")
      expect(results).not.toBeNull()
      expect(results).toHaveLength(2)
      expect(results!.map((r) => r.case)).toContain("dative")
      expect(results!.map((r) => r.case)).toContain("ablative")
    })

    test("accusative plural: nuntios", () => {
      const results = parseNoun("nuntios")
      expect(results).not.toBeNull()
      expect(results![0].case).toBe("accusative")
      expect(results![0].number).toBe("plural")
    })

    test("genitive plural: nuntiorum", () => {
      const results = parseNoun("nuntiorum")
      expect(results).not.toBeNull()
      expect(results![0].case).toBe("genitive")
      expect(results![0].number).toBe("plural")
    })
  })

  describe("unknown words", () => {
    test("returns null for unknown word", () => {
      expect(parseNoun("asdfgh")).toBeNull()
    })

    test("returns null for known stem with invalid ending", () => {
      expect(parseNoun("nuntixx")).toBeNull()
    })
  })
})

describe("parseVerb", () => {
  describe("3rd conjugation: mittere (send)", () => {
    test("imperative: mitte (sync)", () => {
      const results = parseVerb("mitte")
      expect(results).not.toBeNull()
      expect(results![0].stem).toBe("mitt")
      expect(results![0].tense).toBe("imperative")
      expect(results![0].async).toBe(false)
    })

    test("present 3rd person: mittit (sync)", () => {
      const results = parseVerb("mittit")
      expect(results).not.toBeNull()
      expect(results![0].tense).toBe("present")
      expect(results![0].person).toBe(3)
      expect(results![0].async).toBe(false)
    })

    test("future 3rd person: mittet (async)", () => {
      const results = parseVerb("mittet")
      expect(results).not.toBeNull()
      expect(results![0].tense).toBe("future")
      expect(results![0].person).toBe(3)
      expect(results![0].async).toBe(true)
    })
  })

  describe("1st conjugation: creare (create)", () => {
    test("imperative: crea (sync)", () => {
      const results = parseVerb("crea")
      expect(results).not.toBeNull()
      expect(results![0].stem).toBe("cre")
      expect(results![0].tense).toBe("imperative")
      expect(results![0].async).toBe(false)
    })

    test("present 3rd person: creat (sync)", () => {
      const results = parseVerb("creat")
      expect(results).not.toBeNull()
      expect(results![0].tense).toBe("present")
      expect(results![0].async).toBe(false)
    })

    test("future 3rd person: creabit (async)", () => {
      const results = parseVerb("creabit")
      expect(results).not.toBeNull()
      expect(results![0].tense).toBe("future")
      expect(results![0].async).toBe(true)
    })
  })

  describe("unknown verbs", () => {
    test("returns null for unknown verb", () => {
      expect(parseVerb("asdfgh")).toBeNull()
    })
  })
})

describe("keywords", () => {
  test("recognizes control flow keywords", () => {
    expect(isKeyword("si")).toBe(true)
    expect(isKeyword("aliter")).toBe(true)
    expect(isKeyword("dum")).toBe(true)
    expect(isKeyword("redde")).toBe(true)
  })

  test("recognizes declaration keywords", () => {
    expect(isKeyword("esto")).toBe(true)
    expect(isKeyword("fixum")).toBe(true)
    expect(isKeyword("functio")).toBe(true)
  })

  test("recognizes values", () => {
    expect(isKeyword("verum")).toBe(true)
    expect(isKeyword("falsum")).toBe(true)
    expect(isKeyword("nihil")).toBe(true)
  })

  test("returns false for non-keywords", () => {
    expect(isKeyword("nuntius")).toBe(false)
    expect(isKeyword("asdfgh")).toBe(false)
  })

  test("getKeyword returns JS equivalent", () => {
    expect(getKeyword("si")?.javascript).toBe("if")
    expect(getKeyword("esto")?.javascript).toBe("let")
    expect(getKeyword("fixum")?.javascript).toBe("const")
    expect(getKeyword("verum")?.javascript).toBe("true")
    expect(getKeyword("futura")?.javascript).toBe("async")
  })

  test("getKeyword is case insensitive", () => {
    expect(getKeyword("SI")?.javascript).toBe("if")
    expect(getKeyword("Fixum")?.javascript).toBe("const")
  })
})

describe("parseType", () => {
  describe("primitives", () => {
    test("Textus (4th declension)", () => {
      const results = parseType("Textus")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("string")
      expect(results![0].category).toBe("primitive")
    })

    test("Numerus (2nd declension masculine)", () => {
      const results = parseType("Numerus")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("number")
    })

    test("Numerum (accusative)", () => {
      const results = parseType("Numerum")
      expect(results).not.toBeNull()
      expect(results![0].case).toBe("accusative")
      expect(results![0].jsType).toBe("number")
    })
  })

  describe("collections", () => {
    test("Lista (1st declension feminine)", () => {
      const results = parseType("Lista")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Array")
      expect(results![0].category).toBe("collection")
      expect(results![0].generic).toBe(true)
    })

    test("Listam (accusative)", () => {
      const results = parseType("Listam")
      expect(results).not.toBeNull()
      expect(results![0].case).toBe("accusative")
    })

    test("Tabula (Map)", () => {
      const results = parseType("Tabula")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Map")
    })

    test("Copia (Set)", () => {
      const results = parseType("Copia")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Set")
    })
  })

  describe("structural", () => {
    test("Promissum (2nd declension neuter)", () => {
      const results = parseType("Promissum")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Promise")
      expect(results![0].generic).toBe(true)
    })

    test("Erratum (Error)", () => {
      const results = parseType("Erratum")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Error")
    })

    test("Cursor (3rd declension, no ending)", () => {
      const results = parseType("Cursor")
      expect(results).not.toBeNull()
      expect(results![0].jsType).toBe("Iterator")
      expect(results![0].case).toBe("nominative")
    })
  })

  describe("builtin type helpers", () => {
    test("isBuiltinType recognizes type stems", () => {
      expect(isBuiltinType("Text")).toBe(true)
      expect(isBuiltinType("Numer")).toBe(true)
      expect(isBuiltinType("List")).toBe(true)
      expect(isBuiltinType("asdfgh")).toBe(false)
    })
  })
})
