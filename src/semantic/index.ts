/**
 * Semantic Analyzer - Type Resolution and Validation
 *
 * COMPILER PHASE
 * ==============
 * semantic
 *
 * ARCHITECTURE
 * ============
 * This module performs semantic analysis on the AST produced by the parser.
 * It resolves types, builds symbol tables, validates type compatibility,
 * and annotates AST nodes with resolved type information.
 *
 * The analyzer walks the AST in a single pass:
 * 1. Statements are processed to build symbol tables
 * 2. Expressions are typed bottom-up
 * 3. Type errors are collected (not thrown) for multi-error reporting
 * 4. AST nodes are annotated with resolvedType field
 *
 * Key responsibilities:
 * - Variable type resolution (from annotation or initializer inference)
 * - Function signature recording
 * - Expression type inference
 * - Type compatibility checking
 * - Undefined variable detection
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST from parser
 * OUTPUT: SemanticResult with typed AST and error list
 * ERRORS: Type mismatches, undefined variables, redefinitions
 *
 * INVARIANTS
 * ==========
 * INV-1: All expressions have resolvedType after analysis (may be unknown)
 * INV-2: All variable references are validated against symbol table
 * INV-3: Analyzer never throws - errors collected in result
 *
 * @module semantic
 */

import type {
  Program,
  Statement,
  Expression,
  VariableDeclaration,
  FunctionDeclaration,
  IfStatement,
  WhileStatement,
  ForStatement,
  ReturnStatement,
  BlockStatement,
  ExpressionStatement,
  Identifier,
  Literal,
  BinaryExpression,
  UnaryExpression,
  CallExpression,
  MemberExpression,
  ArrowFunctionExpression,
  AssignmentExpression,
  AwaitExpression,
  NewExpression,
  TypeAnnotation,
  Parameter,
  ThrowStatement,
  TryStatement,
} from "../parser/ast"
import type { Position } from "../tokenizer/types"
import type { Scope, Symbol } from "./scope"
import {
  createGlobalScope,
  createScope,
  defineSymbol,
  lookupSymbol,
} from "./scope"
import type { SemanticType, FunctionType } from "./types"
import {
  primitiveType,
  genericType,
  functionType,
  unionType,
  unknownType,
  userType,
  TEXTUS,
  NUMERUS,
  BIVALENS,
  NIHIL,
  VACUUM,
  UNKNOWN,
  formatType,
  isAssignableTo,
} from "./types"

// =============================================================================
// TYPES
// =============================================================================

/**
 * Semantic error with source location.
 */
export interface SemanticError {
  message: string
  position: Position
}

/**
 * Result of semantic analysis.
 */
export interface SemanticResult {
  program: Program
  errors: SemanticError[]
}


// =============================================================================
// LATIN TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to semantic types.
 */
const LATIN_TYPE_MAP: Record<string, SemanticType> = {
  Textus: TEXTUS,
  Numerus: NUMERUS,
  Bivalens: BIVALENS,
  Nihil: NIHIL,
  Vacuum: VACUUM,
}

/**
 * Generic types that take type parameters.
 */
const GENERIC_TYPES = new Set(["Lista", "Tabula", "Copia", "Promissum", "Cursor", "Fluxus"])

// =============================================================================
// MAIN ANALYZER
// =============================================================================

/**
 * Perform semantic analysis on a program.
 */
export function analyze(program: Program): SemanticResult {
  const errors: SemanticError[] = []
  let currentScope: Scope = createGlobalScope()
  let currentFunctionReturnType: SemanticType | null = null

  // Define built-in functions
  defineBuiltins()

  /**
   * Define built-in functions in global scope.
   */
  function defineBuiltins(): void {
    // scribe - print function (variadic, returns void)
    currentScope.symbols.set("scribe", {
      name: "scribe",
      type: functionType([], VACUUM),
      kind: "function",
      mutable: false,
      position: { line: 0, column: 0 },
    })
  }

  /**
   * Report a semantic error.
   */
  function error(message: string, position: Position): void {
    errors.push({ message, position })
  }

  /**
   * Enter a new scope.
   */
  function enterScope(kind: Scope["kind"] = "block"): void {
    currentScope = createScope(currentScope, kind)
  }

  /**
   * Exit the current scope.
   */
  function exitScope(): void {
    if (currentScope.parent) {
      currentScope = currentScope.parent
    }
  }

  /**
   * Define a symbol in current scope, reporting error if duplicate.
   */
  function define(symbol: Symbol): void {
    const err = defineSymbol(currentScope, symbol)
    if (err) {
      error(err, symbol.position)
    }
  }

  // ---------------------------------------------------------------------------
  // Type Resolution
  // ---------------------------------------------------------------------------

  /**
   * Resolve a TypeAnnotation AST node to a SemanticType.
   */
  function resolveTypeAnnotation(node: TypeAnnotation): SemanticType {
    // Handle union types
    if (node.union && node.union.length > 0) {
      const types = node.union.map(resolveTypeAnnotation)
      return unionType(types)
    }

    // Check for built-in primitive type
    const primitive = LATIN_TYPE_MAP[node.name]
    if (primitive) {
      if (node.nullable) {
        return { ...primitive, nullable: true }
      }
      return primitive
    }

    // Check for generic type
    if (GENERIC_TYPES.has(node.name)) {
      const typeParams = (node.typeParameters ?? []).map(resolveTypeAnnotation)
      return genericType(node.name, typeParams, node.nullable)
    }

    // User-defined type
    return userType(node.name, node.nullable)
  }

  /**
   * Infer type from a literal value.
   */
  function inferLiteralType(node: Literal): SemanticType {
    if (node.value === null) return NIHIL
    if (typeof node.value === "string") return TEXTUS
    if (typeof node.value === "number") return NUMERUS
    if (typeof node.value === "boolean") return BIVALENS
    return UNKNOWN
  }

  // ---------------------------------------------------------------------------
  // Expression Type Resolution
  // ---------------------------------------------------------------------------

  /**
   * Resolve the type of an expression.
   */
  function resolveExpression(node: Expression): SemanticType {
    switch (node.type) {
      case "Identifier":
        return resolveIdentifier(node)
      case "Literal":
        return resolveLiteral(node)
      case "TemplateLiteral":
        node.resolvedType = TEXTUS
        return TEXTUS
      case "BinaryExpression":
        return resolveBinaryExpression(node)
      case "UnaryExpression":
        return resolveUnaryExpression(node)
      case "CallExpression":
        return resolveCallExpression(node)
      case "MemberExpression":
        return resolveMemberExpression(node)
      case "ArrowFunctionExpression":
        return resolveArrowFunction(node)
      case "AssignmentExpression":
        return resolveAssignment(node)
      case "AwaitExpression":
        return resolveAwait(node)
      case "NewExpression":
        return resolveNew(node)
      case "ConditionalExpression":
        return resolveConditional(node)
      default:
        return UNKNOWN
    }
  }

  function resolveIdentifier(node: Identifier): SemanticType {
    // Handle Latin boolean/null keywords
    if (node.name === "verum" || node.name === "falsum") {
      node.resolvedType = BIVALENS
      return BIVALENS
    }
    if (node.name === "nihil") {
      node.resolvedType = NIHIL
      return NIHIL
    }

    // Look up in symbol table
    const symbol = lookupSymbol(currentScope, node.name)
    if (!symbol) {
      error(`Undefined variable '${node.name}'`, node.position)
      node.resolvedType = UNKNOWN
      return UNKNOWN
    }

    node.resolvedType = symbol.type
    return symbol.type
  }

  function resolveLiteral(node: Literal): SemanticType {
    const type = inferLiteralType(node)
    node.resolvedType = type
    return type
  }

  function resolveBinaryExpression(node: BinaryExpression): SemanticType {
    const leftType = resolveExpression(node.left)
    const rightType = resolveExpression(node.right)

    // Arithmetic operators: +, -, *, /, %
    if (["+", "-", "*", "/", "%"].includes(node.operator)) {
      // String concatenation
      if (node.operator === "+" && leftType.kind === "primitive" && leftType.name === "Textus") {
        node.resolvedType = TEXTUS
        return TEXTUS
      }
      // Numeric arithmetic
      if (leftType.kind === "primitive" && leftType.name === "Numerus" &&
          rightType.kind === "primitive" && rightType.name === "Numerus") {
        node.resolvedType = NUMERUS
        return NUMERUS
      }
      // Mixed or unknown
      node.resolvedType = NUMERUS
      return NUMERUS
    }

    // Comparison operators: <, >, <=, >=
    if (["<", ">", "<=", ">="].includes(node.operator)) {
      node.resolvedType = BIVALENS
      return BIVALENS
    }

    // Equality operators: ==, !=
    if (["==", "!="].includes(node.operator)) {
      node.resolvedType = BIVALENS
      return BIVALENS
    }

    // Logical operators: &&, ||
    if (["&&", "||"].includes(node.operator)) {
      node.resolvedType = BIVALENS
      return BIVALENS
    }

    node.resolvedType = UNKNOWN
    return UNKNOWN
  }

  function resolveUnaryExpression(node: UnaryExpression): SemanticType {
    const argType = resolveExpression(node.argument)

    if (node.operator === "!" || node.operator === "non") {
      node.resolvedType = BIVALENS
      return BIVALENS
    }

    if (node.operator === "-") {
      node.resolvedType = NUMERUS
      return NUMERUS
    }

    node.resolvedType = argType
    return argType
  }

  function resolveCallExpression(node: CallExpression): SemanticType {
    const calleeType = resolveExpression(node.callee)

    // Resolve argument types
    for (const arg of node.arguments) {
      resolveExpression(arg)
    }

    // If callee is a function type, return its return type
    if (calleeType.kind === "function") {
      node.resolvedType = calleeType.returnType
      return calleeType.returnType
    }

    // Built-in function handling (scribe, etc.)
    if (node.callee.type === "Identifier") {
      const name = node.callee.name
      if (name === "scribe") {
        node.resolvedType = VACUUM
        return VACUUM
      }
    }

    node.resolvedType = UNKNOWN
    return UNKNOWN
  }

  function resolveMemberExpression(node: MemberExpression): SemanticType {
    resolveExpression(node.object)
    // Property access type depends on the object type - for now return unknown
    node.resolvedType = UNKNOWN
    return UNKNOWN
  }

  function resolveArrowFunction(node: ArrowFunctionExpression): SemanticType {
    enterScope("function")

    // Define parameters
    const paramTypes: SemanticType[] = []
    for (const param of node.params) {
      const paramType = param.typeAnnotation
        ? resolveTypeAnnotation(param.typeAnnotation)
        : UNKNOWN
      paramTypes.push(paramType)
      define({
        name: param.name.name,
        type: paramType,
        kind: "parameter",
        mutable: false,
        position: param.position,
      })
    }

    // Resolve body
    let returnType: SemanticType
    if (node.body.type === "BlockStatement") {
      analyzeBlock(node.body)
      returnType = VACUUM
    }
    else {
      returnType = resolveExpression(node.body as Expression)
    }

    exitScope()

    const fnType = functionType(paramTypes, returnType, node.async)
    node.resolvedType = fnType
    return fnType
  }

  function resolveAssignment(node: AssignmentExpression): SemanticType {
    const rightType = resolveExpression(node.right)

    if (node.left.type === "Identifier") {
      const symbol = lookupSymbol(currentScope, node.left.name)
      if (!symbol) {
        error(`Undefined variable '${node.left.name}'`, node.left.position)
      }
      else if (!symbol.mutable) {
        error(`Cannot assign to immutable variable '${node.left.name}'`, node.position)
      }
      else if (!isAssignableTo(rightType, symbol.type)) {
        error(
          `Type '${formatType(rightType)}' is not assignable to type '${formatType(symbol.type)}'`,
          node.position
        )
      }
    }
    else {
      resolveExpression(node.left)
    }

    node.resolvedType = rightType
    return rightType
  }

  function resolveAwait(node: AwaitExpression): SemanticType {
    const argType = resolveExpression(node.argument)

    // If awaiting a Promise, unwrap it
    if (argType.kind === "generic" && argType.name === "Promissum") {
      const unwrapped = argType.typeParameters[0] ?? UNKNOWN
      node.resolvedType = unwrapped
      return unwrapped
    }

    node.resolvedType = argType
    return argType
  }

  function resolveNew(node: NewExpression): SemanticType {
    // Resolve constructor arguments
    for (const arg of node.arguments) {
      resolveExpression(arg)
    }

    // Return user type based on constructor name
    const type = userType(node.callee.name)
    node.resolvedType = type
    return type
  }

  function resolveConditional(node: Expression & { type: "ConditionalExpression" }): SemanticType {
    resolveExpression(node.test)
    const consequentType = resolveExpression(node.consequent)
    const alternateType = resolveExpression(node.alternate)

    // Return union of both branches if different
    if (consequentType.kind === alternateType.kind &&
        consequentType.kind === "primitive" &&
        (consequentType as any).name === (alternateType as any).name) {
      node.resolvedType = consequentType
      return consequentType
    }

    const result = unionType([consequentType, alternateType])
    node.resolvedType = result
    return result
  }

  // ---------------------------------------------------------------------------
  // Statement Analysis
  // ---------------------------------------------------------------------------

  function analyzeStatement(node: Statement): void {
    switch (node.type) {
      case "ImportDeclaration":
        // Imports don't affect type checking for now
        break
      case "VariableDeclaration":
        analyzeVariableDeclaration(node)
        break
      case "FunctionDeclaration":
        analyzeFunctionDeclaration(node)
        break
      case "IfStatement":
        analyzeIfStatement(node)
        break
      case "WhileStatement":
        analyzeWhileStatement(node)
        break
      case "ForStatement":
        analyzeForStatement(node)
        break
      case "ReturnStatement":
        analyzeReturnStatement(node)
        break
      case "BlockStatement":
        analyzeBlock(node)
        break
      case "ExpressionStatement":
        resolveExpression(node.expression)
        break
      case "ThrowStatement":
        analyzeThrowStatement(node)
        break
      case "TryStatement":
        analyzeTryStatement(node)
        break
    }
  }

  function analyzeVariableDeclaration(node: VariableDeclaration): void {
    // Resolve type from annotation or infer from initializer
    let type: SemanticType
    if (node.typeAnnotation) {
      type = resolveTypeAnnotation(node.typeAnnotation)
    }
    else if (node.init) {
      type = resolveExpression(node.init)
    }
    else {
      type = UNKNOWN
      error(`Variable '${node.name.name}' has no type annotation or initializer`, node.position)
    }

    // Check initializer type compatibility
    if (node.typeAnnotation && node.init) {
      const initType = resolveExpression(node.init)
      if (!isAssignableTo(initType, type)) {
        error(
          `Type '${formatType(initType)}' is not assignable to type '${formatType(type)}'`,
          node.position
        )
      }
    }

    // Define in symbol table
    define({
      name: node.name.name,
      type,
      kind: "variable",
      mutable: node.kind === "esto",
      position: node.position,
    })

    node.resolvedType = type
    node.name.resolvedType = type
  }

  function analyzeFunctionDeclaration(node: FunctionDeclaration): void {
    // Build function type
    const paramTypes: SemanticType[] = node.params.map(p =>
      p.typeAnnotation ? resolveTypeAnnotation(p.typeAnnotation) : UNKNOWN
    )
    const returnType = node.returnType
      ? resolveTypeAnnotation(node.returnType)
      : VACUUM

    const fnType = functionType(paramTypes, returnType, node.async)

    // Define function in current scope
    define({
      name: node.name.name,
      type: fnType,
      kind: "function",
      mutable: false,
      position: node.position,
    })

    // Analyze function body in new scope
    enterScope("function")
    const previousReturnType = currentFunctionReturnType
    currentFunctionReturnType = returnType

    // Define parameters
    for (let i = 0; i < node.params.length; i++) {
      const param = node.params[i]
      define({
        name: param.name.name,
        type: paramTypes[i],
        kind: "parameter",
        mutable: false,
        position: param.position,
      })
    }

    // Analyze body
    analyzeBlock(node.body)

    currentFunctionReturnType = previousReturnType
    exitScope()

    node.resolvedType = fnType
  }

  function analyzeIfStatement(node: IfStatement): void {
    const testType = resolveExpression(node.test)
    if (testType.kind === "primitive" && testType.name !== "Bivalens") {
      // Warn but don't error - truthy/falsy is valid
    }

    enterScope()
    analyzeBlock(node.consequent)
    exitScope()

    if (node.alternate) {
      if (node.alternate.type === "IfStatement") {
        analyzeIfStatement(node.alternate)
      }
      else {
        enterScope()
        analyzeBlock(node.alternate)
        exitScope()
      }
    }

    if (node.catchClause) {
      analyzeCatchClause(node.catchClause)
    }
  }

  function analyzeWhileStatement(node: WhileStatement): void {
    resolveExpression(node.test)
    enterScope()
    analyzeBlock(node.body)
    exitScope()

    if (node.catchClause) {
      analyzeCatchClause(node.catchClause)
    }
  }

  function analyzeForStatement(node: ForStatement): void {
    resolveExpression(node.iterable)

    enterScope()
    // Define loop variable
    define({
      name: node.variable.name,
      type: UNKNOWN, // Would need to infer from iterable element type
      kind: "variable",
      mutable: false,
      position: node.variable.position,
    })

    analyzeBlock(node.body)
    exitScope()

    if (node.catchClause) {
      analyzeCatchClause(node.catchClause)
    }
  }

  function analyzeReturnStatement(node: ReturnStatement): void {
    if (node.argument) {
      const returnType = resolveExpression(node.argument)
      if (currentFunctionReturnType && !isAssignableTo(returnType, currentFunctionReturnType)) {
        error(
          `Return type '${formatType(returnType)}' is not assignable to function return type '${formatType(currentFunctionReturnType)}'`,
          node.position
        )
      }
    }
  }

  function analyzeThrowStatement(node: ThrowStatement): void {
    resolveExpression(node.argument)
  }

  function analyzeTryStatement(node: TryStatement): void {
    enterScope()
    analyzeBlock(node.block)
    exitScope()

    if (node.handler) {
      analyzeCatchClause(node.handler)
    }

    if (node.finalizer) {
      enterScope()
      analyzeBlock(node.finalizer)
      exitScope()
    }
  }

  function analyzeCatchClause(node: { param: Identifier; body: BlockStatement }): void {
    enterScope()
    define({
      name: node.param.name,
      type: userType("Error"),
      kind: "parameter",
      mutable: false,
      position: node.param.position,
    })
    analyzeBlock(node.body)
    exitScope()
  }

  function analyzeBlock(node: BlockStatement): void {
    for (const stmt of node.body) {
      analyzeStatement(stmt)
    }
  }

  // ---------------------------------------------------------------------------
  // Main Analysis
  // ---------------------------------------------------------------------------

  for (const stmt of program.body) {
    analyzeStatement(stmt)
  }

  return { program, errors }
}

// Re-export types
export * from "./types"
export * from "./scope"
