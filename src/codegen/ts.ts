/**
 * TypeScript Code Generator - Emit JavaScript with type annotations
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into TypeScript source code.
 * It preserves JavaScript runtime semantics while adding TypeScript type
 * annotations derived from Latin type declarations.
 *
 * The generator uses a recursive descent pattern that mirrors the AST structure.
 * Each AST node type has a corresponding gen* function that produces a string
 * fragment. These fragments are composed bottom-up to build the complete output.
 *
 * Indentation is managed via a depth counter that tracks nesting level. The
 * ind() helper function generates the appropriate indentation string for the
 * current depth.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid TypeScript source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * TypeScript preserves JavaScript semantics:
 * - Dynamic typing with optional annotations
 * - Prototype-based objects
 * - Async/await for concurrency
 * - Exception-based error handling
 * - Nullable types via union with null
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid TypeScript
 * INV-2: All Latin type names are mapped to TypeScript equivalents
 * INV-3: Indentation depth is correctly maintained (incremented/decremented)
 */

import type {
  Program,
  Statement,
  Expression,
  ImportDeclaration,
  VariableDeclaration,
  FunctionDeclaration,
  IfStatement,
  WhileStatement,
  ForStatement,
  ReturnStatement,
  BlockStatement,
  ThrowStatement,
  TryStatement,
  ExpressionStatement,
  BinaryExpression,
  UnaryExpression,
  CallExpression,
  MemberExpression,
  ArrowFunctionExpression,
  AssignmentExpression,
  NewExpression,
  Literal,
  Parameter,
  TypeAnnotation,
} from "../parser/ast"
import type { CodegenOptions } from "./types"

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to TypeScript types.
 *
 * WHY: Latin uses descriptive names (Textus = text), TypeScript uses
 *      JavaScript primitive names (string). This mapping preserves semantics
 *      while emitting idiomatic TypeScript.
 *
 * TARGET MAPPING:
 * | Latin      | TypeScript |
 * |------------|------------|
 * | Textus     | string     |
 * | Numerus    | number     |
 * | Bivalens   | boolean    |
 * | Nihil      | null       |
 * | Lista      | Array      |
 * | Tabula     | Map        |
 * | Copia      | Set        |
 * | Promissum  | Promise    |
 * | Erratum    | Error      |
 * | Cursor     | Iterator   |
 */
const typeMap: Record<string, string> = {
  Textus: "string",
  Numerus: "number",
  Bivalens: "boolean",
  Nihil: "null",
  Lista: "Array",
  Tabula: "Map",
  Copia: "Set",
  Promissum: "Promise",
  Erratum: "Error",
  Cursor: "Iterator",
}

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate TypeScript source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> TypeScript source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent, semicolons)
 * @returns TypeScript source code
 */
export function generateTs(program: Program, options: CodegenOptions = {}): string {
  // WHY: 2 spaces is TypeScript convention
  const indent = options.indent ?? "  "
  // WHY: Semicolons are recommended in TypeScript style guides
  const semi = options.semicolons ?? true

  // Track indentation depth for nested blocks
  let depth = 0

  /**
   * Generate indentation for current depth.
   * WHY: Centralized indentation logic ensures consistent formatting.
   */
  function ind(): string {
    return indent.repeat(depth)
  }

  // ---------------------------------------------------------------------------
  // Top-level
  // ---------------------------------------------------------------------------

  function genProgram(node: Program): string {
    // WHY: Each top-level statement is separated by newline
    return node.body.map(genStatement).join("\n")
  }

  // ---------------------------------------------------------------------------
  // Statements
  // ---------------------------------------------------------------------------

  /**
   * Generate code for any statement type.
   * WHY: Exhaustive switch ensures all statement types are handled.
   */
  function genStatement(node: Statement): string {
    switch (node.type) {
      case "ImportDeclaration":
        return genImportDeclaration(node)
      case "VariableDeclaration":
        return genVariableDeclaration(node)
      case "FunctionDeclaration":
        return genFunctionDeclaration(node)
      case "IfStatement":
        return genIfStatement(node)
      case "WhileStatement":
        return genWhileStatement(node)
      case "ForStatement":
        return genForStatement(node)
      case "ReturnStatement":
        return genReturnStatement(node)
      case "ThrowStatement":
        return genThrowStatement(node)
      case "TryStatement":
        return genTryStatement(node)
      case "BlockStatement":
        return genBlockStatement(node)
      case "ExpressionStatement":
        return genExpressionStatement(node)
      default:
        throw new Error(`Unknown statement type: ${(node as any).type}`)
    }
  }

  /**
   * Generate import declaration.
   *
   * TRANSFORMS:
   *   ex norma importa * -> import * as norma from "norma"
   *   ex norma importa scribe, lege -> import { scribe, lege } from "norma"
   */
  function genImportDeclaration(node: ImportDeclaration): string {
    const source = node.source

    if (node.wildcard) {
      return `${ind()}import * as ${source} from "${source}"${semi ? ";" : ""}`
    }

    const names = node.specifiers.map(s => s.name).join(", ")

    return `${ind()}import { ${names} } from "${source}"${semi ? ";" : ""}`
  }

  /**
   * Generate variable declaration.
   *
   * TRANSFORMS:
   *   esto x: Numerus = 5 -> let x: number = 5
   *   fixum y: Textus = "hello" -> const y: string = "hello"
   */
  function genVariableDeclaration(node: VariableDeclaration): string {
    // WHY: 'esto' (let it be) maps to mutable 'let', 'fixum' (fixed) to immutable 'const'
    const kind = node.kind === "esto" ? "let" : "const"
    const name = node.name.name
    const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : ""
    const init = node.init ? ` = ${genExpression(node.init)}` : ""

    return `${ind()}${kind} ${name}${typeAnno}${init}${semi ? ";" : ""}`
  }

  /**
   * Generate type annotation from Latin type.
   *
   * TRANSFORMS:
   *   Textus -> string
   *   Lista<Numerus> -> Array<number>
   *   Textus? -> string | null
   */
  function genType(node: TypeAnnotation): string {
    // Map Latin type name to TS type
    const base = typeMap[node.name] ?? node.name

    // Handle generic type parameters: Lista<Textus> -> Array<string>
    let result = base

    if (node.typeParameters && node.typeParameters.length > 0) {
      const params = node.typeParameters.map(genType).join(", ")

      result = `${base}<${params}>`
    }

    // Handle nullable: Textus? -> string | null
    if (node.nullable) {
      result = `${result} | null`
    }

    // Handle union types
    if (node.union && node.union.length > 0) {
      result = node.union.map(genType).join(" | ")
    }

    return result
  }

  /**
   * Generate function declaration.
   *
   * TRANSFORMS:
   *   functio salve(nomen: Textus): Nihil -> function salve(nomen: string): null
   *   futura functio f(): Numerus -> async function f(): number
   */
  function genFunctionDeclaration(node: FunctionDeclaration): string {
    const async = node.async ? "async " : ""
    const name = node.name.name
    const params = node.params.map(genParameter).join(", ")
    const returnType = node.returnType ? `: ${genType(node.returnType)}` : ""
    const body = genBlockStatement(node.body)

    return `${ind()}${async}function ${name}(${params})${returnType} ${body}`
  }

  /**
   * Generate function parameter.
   *
   * TRANSFORMS:
   *   nomen: Textus -> nomen: string
   */
  function genParameter(node: Parameter): string {
    const name = node.name.name
    const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : ""

    return `${name}${typeAnno}`
  }

  /**
   * Generate if statement.
   *
   * TRANSFORMS:
   *   si (conditio) { ... } -> if (conditio) { ... }
   *   si (conditio) { ... } aliter { ... } -> if (conditio) { ... } else { ... }
   *
   * WHY: Latin if-statements can have optional catch clauses for exception handling.
   *      When present, we wrap the entire if in a try-catch block.
   */
  function genIfStatement(node: IfStatement): string {
    let result = ""

    // WHY: Latin allows 'capta' (catch) clause on if-statements for brevity
    if (node.catchClause) {
      result += `${ind()}try {\n`
      depth++
      result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`
      depth--
      result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`
    }
    else {
      result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`
    }

    if (node.alternate) {
      if (node.alternate.type === "IfStatement") {
        result += ` else ${genIfStatement(node.alternate).trim()}`
      }
      else {
        result += ` else ${genBlockStatement(node.alternate)}`
      }
    }

    return result
  }

  function genWhileStatement(node: WhileStatement): string {
    const test = genExpression(node.test)
    const body = genBlockStatement(node.body)

    if (node.catchClause) {
      let result = `${ind()}try {\n`

      depth++
      result += `${ind()}while (${test}) ${body}`
      depth--
      result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`

      return result
    }

    return `${ind()}while (${test}) ${body}`
  }

  function genForStatement(node: ForStatement): string {
    const varName = node.variable.name
    const iterable = genExpression(node.iterable)
    const keyword = node.kind === "in" ? "in" : "of"
    const body = genBlockStatement(node.body)

    if (node.catchClause) {
      let result = `${ind()}try {\n`

      depth++
      result += `${ind()}for (const ${varName} ${keyword} ${iterable}) ${body}`
      depth--
      result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`

      return result
    }

    return `${ind()}for (const ${varName} ${keyword} ${iterable}) ${body}`
  }

  function genReturnStatement(node: ReturnStatement): string {
    if (node.argument) {
      return `${ind()}return ${genExpression(node.argument)}${semi ? ";" : ""}`
    }

    return `${ind()}return${semi ? ";" : ""}`
  }

  function genThrowStatement(node: ThrowStatement): string {
    return `${ind()}throw ${genExpression(node.argument)}${semi ? ";" : ""}`
  }

  function genTryStatement(node: TryStatement): string {
    let result = `${ind()}try ${genBlockStatement(node.block)}`

    if (node.handler) {
      result += ` catch (${node.handler.param.name}) ${genBlockStatement(node.handler.body)}`
    }

    if (node.finalizer) {
      result += ` finally ${genBlockStatement(node.finalizer)}`
    }

    return result
  }

  function genBlockStatement(node: BlockStatement): string {
    if (node.body.length === 0) {
      return "{}"
    }

    depth++
    const body = node.body.map(genStatement).join("\n")

    depth--

    return `{\n${body}\n${ind()}}`
  }

  function genExpressionStatement(node: ExpressionStatement): string {
    return `${ind()}${genExpression(node.expression)}${semi ? ";" : ""}`
  }

  // ---------------------------------------------------------------------------
  // Expressions
  // ---------------------------------------------------------------------------

  /**
   * Generate code for any expression type.
   * WHY: Exhaustive switch ensures all expression types are handled.
   */
  function genExpression(node: Expression): string {
    switch (node.type) {
      case "Identifier":
        return node.name
      case "Literal":
        return genLiteral(node)
      case "TemplateLiteral":
        return `\`${node.raw}\``
      case "BinaryExpression":
        return genBinaryExpression(node)
      case "UnaryExpression":
        return genUnaryExpression(node)
      case "CallExpression":
        return genCallExpression(node)
      case "MemberExpression":
        return genMemberExpression(node)
      case "ArrowFunctionExpression":
        return genArrowFunction(node)
      case "AssignmentExpression":
        return genAssignmentExpression(node)
      case "AwaitExpression":
        return `await ${genExpression(node.argument)}`
      case "NewExpression":
        return genNewExpression(node)
      case "ConditionalExpression":
        return `${genExpression(node.test)} ? ${genExpression(node.consequent)} : ${genExpression(node.alternate)}`
      default:
        throw new Error(`Unknown expression type: ${(node as any).type}`)
    }
  }

  /**
   * Generate literal value.
   *
   * TRANSFORMS:
   *   "hello" -> "hello" (JSON-escaped)
   *   42 -> 42
   *   verum -> true
   *   nihil -> null
   *
   * WHY: JSON.stringify ensures proper escaping of string literals.
   */
  function genLiteral(node: Literal): string {
    if (node.value === null) {
      return "null"
    }

    if (typeof node.value === "string") {
      return JSON.stringify(node.value)
    }

    if (typeof node.value === "boolean") {
      return node.value ? "true" : "false"
    }

    return String(node.value)
  }

  /**
   * Generate binary expression.
   *
   * TRANSFORMS:
   *   x + y -> (x + y)
   *   a && b -> (a && b)
   *
   * WHY: Parentheses ensure correct precedence in all contexts.
   */
  function genBinaryExpression(node: BinaryExpression): string {
    const left = genExpression(node.left)
    const right = genExpression(node.right)

    return `(${left} ${node.operator} ${right})`
  }

  /**
   * Generate unary expression.
   *
   * TRANSFORMS:
   *   !x -> !x (prefix)
   *   x++ -> x++ (postfix)
   */
  function genUnaryExpression(node: UnaryExpression): string {
    const arg = genExpression(node.argument)

    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`
  }

  /**
   * Generate function call.
   *
   * TRANSFORMS:
   *   scribe("hello") -> scribe("hello")
   */
  function genCallExpression(node: CallExpression): string {
    const callee = genExpression(node.callee)
    const args = node.arguments.map(genExpression).join(", ")

    return `${callee}(${args})`
  }

  /**
   * Generate member access.
   *
   * TRANSFORMS:
   *   obj.prop -> obj.prop
   *   obj[key] -> obj[key]
   */
  function genMemberExpression(node: MemberExpression): string {
    const obj = genExpression(node.object)

    if (node.computed) {
      return `${obj}[${genExpression(node.property)}]`
    }

    return `${obj}.${node.property.name}`
  }

  /**
   * Generate arrow function.
   *
   * TRANSFORMS:
   *   (x) => x + 1 -> (x) => x + 1
   *   (x) => { redde x + 1; } -> (x) => { return x + 1; }
   */
  function genArrowFunction(node: ArrowFunctionExpression): string {
    const params = node.params.map(genParameter).join(", ")

    if (node.body.type === "BlockStatement") {
      const body = genBlockStatement(node.body)

      return `(${params}) => ${body}`
    }

    const body = genExpression(node.body as Expression)

    return `(${params}) => ${body}`
  }

  function genAssignmentExpression(node: AssignmentExpression): string {
    const left = node.left.type === "Identifier"
      ? node.left.name
      : genExpression(node.left)

    return `${left} ${node.operator} ${genExpression(node.right)}`
  }

  function genNewExpression(node: NewExpression): string {
    const callee = node.callee.name
    const args = node.arguments.map(genExpression).join(", ")

    return `new ${callee}(${args})`
  }

  return genProgram(program)
}
