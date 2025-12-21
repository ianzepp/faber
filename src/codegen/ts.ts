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
  AwaitExpression,
  NewExpression,
  Identifier,
  Literal,
  TemplateLiteral,
  Parameter,
  TypeAnnotation,
  CatchClause,
} from "../parser/ast"
import type { CodegenOptions } from "./types"

// Map Latin type names to TypeScript types
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

export function generateTs(program: Program, options: CodegenOptions = {}): string {
  const indent = options.indent ?? "  "
  const semi = options.semicolons ?? true

  let depth = 0

  function ind(): string {
    return indent.repeat(depth)
  }

  function genProgram(node: Program): string {
    return node.body.map(genStatement).join("\n")
  }

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

  function genImportDeclaration(node: ImportDeclaration): string {
    const source = node.source
    if (node.wildcard) {
      return `${ind()}import * as ${source} from "${source}"${semi ? ";" : ""}`
    }
    const names = node.specifiers.map(s => s.name).join(", ")
    return `${ind()}import { ${names} } from "${source}"${semi ? ";" : ""}`
  }

  function genVariableDeclaration(node: VariableDeclaration): string {
    const kind = node.kind === "esto" ? "let" : "const"
    const name = node.name.name
    const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : ""
    const init = node.init ? ` = ${genExpression(node.init)}` : ""
    return `${ind()}${kind} ${name}${typeAnno}${init}${semi ? ";" : ""}`
  }

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

  function genFunctionDeclaration(node: FunctionDeclaration): string {
    const async = node.async ? "async " : ""
    const name = node.name.name
    const params = node.params.map(genParameter).join(", ")
    const returnType = node.returnType ? `: ${genType(node.returnType)}` : ""
    const body = genBlockStatement(node.body)
    return `${ind()}${async}function ${name}(${params})${returnType} ${body}`
  }

  function genParameter(node: Parameter): string {
    const name = node.name.name
    const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : ""
    return `${name}${typeAnno}`
  }

  function genIfStatement(node: IfStatement): string {
    let result = ""

    // If the if has a catch clause, wrap in try
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

  function genLiteral(node: Literal): string {
    if (node.value === null) return "null"
    if (typeof node.value === "string") return JSON.stringify(node.value)
    if (typeof node.value === "boolean") return node.value ? "true" : "false"
    return String(node.value)
  }

  function genBinaryExpression(node: BinaryExpression): string {
    const left = genExpression(node.left)
    const right = genExpression(node.right)
    return `(${left} ${node.operator} ${right})`
  }

  function genUnaryExpression(node: UnaryExpression): string {
    const arg = genExpression(node.argument)
    return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`
  }

  function genCallExpression(node: CallExpression): string {
    const callee = genExpression(node.callee)
    const args = node.arguments.map(genExpression).join(", ")
    return `${callee}(${args})`
  }

  function genMemberExpression(node: MemberExpression): string {
    const obj = genExpression(node.object)
    if (node.computed) {
      return `${obj}[${genExpression(node.property)}]`
    }
    return `${obj}.${node.property.name}`
  }

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
