import { test, expect, describe } from "bun:test"
import { parseNoun, parseVerb, isKeyword, getKeyword } from "./index"

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
