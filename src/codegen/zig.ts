/**
 * Zig Code Generator - Emit systems programming code
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into Zig source code. Unlike
 * the TypeScript generator which preserves JavaScript semantics, this generator
 * adapts Latin constructs to Zig's systems programming model.
 *
 * Key transformations:
 * - Top-level code is wrapped in pub fn main()
 * - Comptime (compile-time) values are hoisted to module scope
 * - Function declarations remain at module scope
 * - Async becomes error union types (!T)
 * - Exceptions map to error returns
 * - scribe() maps to std.debug.print()
 *
 * The generator uses the same recursive descent pattern as the TypeScript
 * generator but with Zig-specific semantics for types, error handling, and
 * memory management.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid Zig source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * Zig uses systems programming semantics:
 * - Compile-time vs runtime execution (comptime)
 * - Explicit types required for var declarations
 * - Error unions instead of exceptions (!T)
 * - No garbage collection (manual memory)
 * - Nullable via optional types (?T)
 * - No arrow functions (use struct { fn ... }.call pattern)
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid Zig
 * INV-2: All Latin type names are mapped to Zig equivalents
 * INV-3: Top-level declarations separate from runtime code
 * INV-4: Main function is only emitted if there are runtime statements
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

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to Zig types.
 *
 * WHY: Zig uses explicit, low-level types instead of JavaScript's dynamic types.
 *      Strings are slices of bytes, numbers are sized integers, void replaces null.
 *
 * TARGET MAPPING:
 * | Latin     | TypeScript | Zig        |
 * |-----------|------------|------------|
 * | Textus    | string     | []const u8 |
 * | Numerus   | number     | i64        |
 * | Bivalens  | boolean    | bool       |
 * | Nihil     | null       | void       |
 *
 * LIMITATION: Complex types (Lista, Tabula, etc.) not yet supported in Zig target.
 */
const typeMap: Record<string, string> = {
  Textus: "[]const u8",
  Numerus: "i64",
  Bivalens: "bool",
  Nihil: "void",
}

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate Zig source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> Zig source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent only, semicolons always used)
 * @returns Zig source code
 */
export function generateZig(program: Program, options: CodegenOptions = {}): string {
  // WHY: 4 spaces is Zig convention
  const indent = options.indent ?? "    "

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

  /**
   * Generate top-level program structure.
   *
   * WHY: Zig requires separation between compile-time and runtime code.
   *      Functions and comptime constants go at module scope.
   *      Runtime statements are wrapped in pub fn main().
   */
  function genProgram(node: Program): string {
    const lines: string[] = []

    // WHY: scribe() maps to std.debug.print(), so we need std import
    const needsStd = programUsesScribe(node)
    if (needsStd) {
      lines.push("const std = @import(\"std\");")
      lines.push("")
    }

    // WHY: Zig distinguishes compile-time (const, fn) from runtime (var, statements)
    const topLevel: Statement[] = []
    const runtime: Statement[] = []

    for (const stmt of node.body) {
      if (isTopLevelDeclaration(stmt)) {
        topLevel.push(stmt)
      }
      else {
        runtime.push(stmt)
      }
    }

    lines.push(...topLevel.map(genStatement))

    // WHY: Only emit main() if there's runtime code to execute
    if (runtime.length > 0) {
      if (topLevel.length > 0) lines.push("")
      lines.push("pub fn main() void {")
      depth++
      lines.push(...runtime.map(genStatement))
      depth--
      lines.push("}")
    }

    return lines.join("\n")
  }

  /**
   * Determine if a statement belongs at module scope.
   *
   * WHY: Zig executes at compile-time (comptime) by default.
   *      Only runtime-dependent code goes in main().
   *
   * TARGET: Functions and imports are always top-level in Zig.
   *         Const declarations with comptime values are hoisted.
   */
  function isTopLevelDeclaration(node: Statement): boolean {
    if (node.type === "FunctionDeclaration") return true
    if (node.type === "ImportDeclaration") return true
    // WHY: fixum with literal is comptime in Zig
    if (node.type === "VariableDeclaration" && node.kind === "fixum") {
      if (node.init && isComptimeValue(node.init)) return true
    }
    return false
  }

  /**
   * Determine if an expression can be evaluated at compile-time.
   *
   * WHY: Zig's comptime system allows literals and constants to be
   *      evaluated during compilation, not runtime.
   */
  function isComptimeValue(node: Expression): boolean {
    if (node.type === "Literal") return true
    if (node.type === "TemplateLiteral") return true
    // WHY: verum, falsum, nihil are Latin keywords for literal values
    if (node.type === "Identifier") {
      return ["verum", "falsum", "nihil"].includes(node.name)
    }
    return false
  }

  /**
   * Check if program uses scribe() function.
   *
   * WHY: scribe() maps to std.debug.print(), so we need to import std.
   *      Quick check via JSON serialization instead of full AST walk.
   */
  function programUsesScribe(node: Program): boolean {
    const source = JSON.stringify(node)
    return source.includes('"name":"scribe"')
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
        return genBlockStatementContent(node)
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
   *   ex norma importa * -> const norma = @import("norma");
   *   ex norma importa scribe, lege -> const _norma = @import("norma");
   *                                     const scribe = _norma.scribe;
   *                                     const lege = _norma.lege;
   *
   * TARGET: Zig uses @import() builtin. Specific imports create const bindings.
   */
  function genImportDeclaration(node: ImportDeclaration): string {
    const source = node.source
    if (node.wildcard) {
      return `${ind()}const ${source} = @import("${source}");`
    }
    // WHY: Import module once with underscore prefix, then bind specific names
    const lines: string[] = []
    const modVar = `_${source}`
    lines.push(`${ind()}const ${modVar} = @import("${source}");`)
    for (const spec of node.specifiers) {
      lines.push(`${ind()}const ${spec.name} = ${modVar}.${spec.name};`)
    }
    return lines.join("\n")
  }

  /**
   * Generate variable declaration.
   *
   * TRANSFORMS:
   *   esto x: Numerus = 5 -> var x: i64 = 5;
   *   fixum y: Textus = "hello" -> const y: []const u8 = "hello";
   *
   * TARGET: Zig requires explicit types for var (mutable) declarations.
   *         Const can infer but we add type for clarity.
   */
  function genVariableDeclaration(node: VariableDeclaration): string {
    const kind = node.kind === "esto" ? "var" : "const"
    const name = node.name.name

    // TARGET: Zig requires explicit types for var, we infer if not provided
    let typeAnno = ""
    if (node.typeAnnotation) {
      typeAnno = `: ${genType(node.typeAnnotation)}`
    }
    else if (kind === "var" && node.init) {
      typeAnno = `: ${inferZigType(node.init)}`
    }

    const init = node.init ? ` = ${genExpression(node.init)}` : " = undefined"
    return `${ind()}${kind} ${name}${typeAnno}${init};`
  }

  /**
   * Infer Zig type from expression when no explicit annotation.
   *
   * WHY: Zig var declarations require types. We infer from literals.
   *      Falls back to anytype for complex expressions.
   */
  function inferZigType(node: Expression): string {
    if (node.type === "Literal") {
      if (typeof node.value === "number") {
        // WHY: Distinguish integer from float for Zig's type system
        return Number.isInteger(node.value) ? "i64" : "f64"
      }
      if (typeof node.value === "string") return "[]const u8"
      if (typeof node.value === "boolean") return "bool"
    }
    if (node.type === "Identifier") {
      if (node.name === "verum" || node.name === "falsum") return "bool"
      if (node.name === "nihil") return "?void"
    }
    // WHY: anytype lets Zig infer from assignment
    return "anytype"
  }

  /**
   * Generate type annotation from Latin type.
   *
   * TRANSFORMS:
   *   Textus -> []const u8
   *   Numerus -> i64
   *   Textus? -> ?[]const u8
   *
   * TARGET: Zig uses ? prefix for optional types, not | null suffix.
   */
  function genType(node: TypeAnnotation): string {
    const base = typeMap[node.name] ?? node.name
    if (node.nullable) {
      return `?${base}`
    }
    return base
  }

  /**
   * Generate function declaration.
   *
   * TRANSFORMS:
   *   functio salve(nomen: Textus): Nihil -> fn salve(nomen: []const u8) void
   *   futura functio f(): Numerus -> fn f() !i64
   *
   * TARGET: Zig uses fn not function. Async becomes error union (!T).
   */
  function genFunctionDeclaration(node: FunctionDeclaration): string {
    const name = node.name.name
    const params = node.params.map(genParameter).join(", ")
    const returnType = node.returnType ? genType(node.returnType) : "void"

    // TARGET: Async in Zig uses error unions (!T) not async/await
    const retType = node.async ? `!${returnType}` : returnType

    const body = genBlockStatement(node.body)
    return `${ind()}fn ${name}(${params}) ${retType} ${body}`
  }

  /**
   * Generate function parameter.
   *
   * TRANSFORMS:
   *   nomen: Textus -> nomen: []const u8
   *
   * TARGET: Zig requires type after parameter name (name: type).
   */
  function genParameter(node: Parameter): string {
    const name = node.name.name
    const type = node.typeAnnotation ? genType(node.typeAnnotation) : "anytype"
    return `${name}: ${type}`
  }

  function genIfStatement(node: IfStatement): string {
    let result = ""

    // Zig doesn't have try/catch like JS, we'll use error handling differently
    // For now, ignore catchClause in Zig output
    result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`

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
    return `${ind()}while (${test}) ${body}`
  }

  function genForStatement(node: ForStatement): string {
    const varName = node.variable.name
    const iterable = genExpression(node.iterable)
    const body = genBlockStatement(node.body)

    // Zig uses for (slice) |item| syntax
    return `${ind()}for (${iterable}) |${varName}| ${body}`
  }

  function genReturnStatement(node: ReturnStatement): string {
    if (node.argument) {
      return `${ind()}return ${genExpression(node.argument)};`
    }
    return `${ind()}return;`
  }

  function genThrowStatement(node: ThrowStatement): string {
    // Zig uses return error.X for errors
    return `${ind()}return error.${genExpression(node.argument)};`
  }

  function genTryStatement(node: TryStatement): string {
    // Zig handles errors differently — this is a simplified mapping
    // Real Zig would use catch |err| { } syntax on expressions
    let result = `${ind()}// try block\n`
    result += genBlockStatementContent(node.block)

    if (node.handler) {
      result += `\n${ind()}// catch handling would use: catch |${node.handler.param.name}| { ... }`
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

  function genBlockStatementContent(node: BlockStatement): string {
    return node.body.map(genStatement).join("\n")
  }

  /**
   * Generate expression statement.
   *
   * TARGET: Zig requires expressions to be used or discarded with _ = prefix.
   *         Calls and assignments don't need discard prefix.
   */
  function genExpressionStatement(node: ExpressionStatement): string {
    const expr = genExpression(node.expression)
    // TARGET: Zig assignments and calls are statements, not expressions
    if (node.expression.type === "CallExpression" || node.expression.type === "AssignmentExpression") {
      return `${ind()}${expr};`
    }
    // WHY: Zig requires explicit discard with _ = for unused expression results
    return `${ind()}_ = ${expr};`
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
        return genIdentifier(node)
      case "Literal":
        return genLiteral(node)
      case "TemplateLiteral":
        // Zig doesn't have template literals, convert to string
        return `"${node.raw.replace(/`/g, "")}"`
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
        // Zig async is different, use try for error handling
        return `try ${genExpression(node.argument)}`
      case "NewExpression":
        return genNewExpression(node)
      case "ConditionalExpression":
        return `if (${genExpression(node.test)}) ${genExpression(node.consequent)} else ${genExpression(node.alternate)}`
      default:
        throw new Error(`Unknown expression type: ${(node as any).type}`)
    }
  }

  /**
   * Generate identifier.
   *
   * TRANSFORMS:
   *   verum -> true
   *   falsum -> false
   *   nihil -> null
   *
   * WHY: Latin uses Latin words for boolean/null literals.
   */
  function genIdentifier(node: Identifier): string {
    switch (node.name) {
      case "verum": return "true"
      case "falsum": return "false"
      case "nihil": return "null"
      default: return node.name
    }
  }

  /**
   * Generate literal value.
   *
   * TRANSFORMS:
   *   "hello" -> "hello"
   *   42 -> 42
   *   verum -> true
   *   nihil -> null
   *
   * TARGET: Zig string literals use double quotes, no escaping needed for simple strings.
   */
  function genLiteral(node: Literal): string {
    if (node.value === null) return "null"
    if (typeof node.value === "string") return `"${node.value}"`
    if (typeof node.value === "boolean") return node.value ? "true" : "false"
    return String(node.value)
  }

  /**
   * Generate binary expression.
   *
   * TRANSFORMS:
   *   x + y -> (x + y)
   *   a && b -> (a and b)
   *   a || b -> (a or b)
   *
   * TARGET: Zig uses 'and'/'or' keywords not &&/|| operators.
   */
  function genBinaryExpression(node: BinaryExpression): string {
    const left = genExpression(node.left)
    const right = genExpression(node.right)
    const op = mapOperator(node.operator)
    return `(${left} ${op} ${right})`
  }

  /**
   * Map JavaScript operators to Zig equivalents.
   *
   * TARGET: Zig uses keyword operators for boolean logic.
   *
   * LIMITATION: String concatenation (++) vs numeric addition (+) can't be
   *             distinguished without type information. Using + for both.
   */
  function mapOperator(op: string): string {
    switch (op) {
      case "&&": return "and"
      case "||": return "or"
      default: return op
    }
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
   *   scribe("hello") -> std.debug.print("{s}\n", .{"hello"})
   *   foo(x, y) -> foo(x, y)
   *
   * TARGET: scribe() is Latin's print function, maps to std.debug.print().
   *         Zig print uses format strings and anonymous tuple syntax (.{...}).
   */
  function genCallExpression(node: CallExpression): string {
    // TARGET: scribe() maps to Zig's std.debug.print()
    if (node.callee.type === "Identifier" && node.callee.name === "scribe") {
      const args = node.arguments.map(genExpression)
      // WHY: {s} for string literals, {any} for runtime values
      const formatSpecs = node.arguments.map(arg => {
        if (arg.type === "Literal" && typeof arg.value === "string") return "{s}"
        if (arg.type === "Identifier") return "{any}"
        return "{any}"
      })
      const format = formatSpecs.join(" ") + "\\n"
      return `std.debug.print("${format}", .{${args.join(", ")}})`
    }

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
   * Generate arrow function (emulated).
   *
   * TRANSFORMS:
   *   (x) => x + 1 -> struct { fn call(x: anytype) anytype { return x + 1; } }.call
   *
   * TARGET: Zig doesn't have arrow functions. We emulate with anonymous struct
   *         containing a function. This is a simplification for basic cases.
   *
   * LIMITATION: This pattern doesn't capture closures properly. Full implementation
   *             would require passing captured variables explicitly.
   */
  function genArrowFunction(node: ArrowFunctionExpression): string {
    const params = node.params.map(genParameter).join(", ")

    if (node.body.type === "BlockStatement") {
      const body = genBlockStatement(node.body)
      return `struct { fn call(${params}) anytype ${body} }.call`
    }

    const body = genExpression(node.body as Expression)
    return `struct { fn call(${params}) anytype { return ${body}; } }.call`
  }

  /**
   * Generate assignment expression.
   *
   * TRANSFORMS:
   *   x = 5 -> x = 5
   *   x += 1 -> x += 1
   */
  function genAssignmentExpression(node: AssignmentExpression): string {
    const left = node.left.type === "Identifier"
      ? node.left.name
      : genExpression(node.left)
    return `${left} ${node.operator} ${genExpression(node.right)}`
  }

  /**
   * Generate new expression.
   *
   * TRANSFORMS:
   *   new Foo(x, y) -> Foo.init(x, y)
   *
   * TARGET: Zig doesn't have 'new' keyword. Idiomatic pattern is Type.init().
   */
  function genNewExpression(node: NewExpression): string {
    const callee = node.callee.name
    const args = node.arguments.map(genExpression).join(", ")
    return `${callee}.init(${args})`
  }

  return genProgram(program)
}
