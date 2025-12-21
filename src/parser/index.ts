import type { Token, TokenType, Position } from "../tokenizer/types"
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
  ExpressionStatement,
  Identifier,
  Literal,
  BinaryExpression,
  UnaryExpression,
  CallExpression,
  MemberExpression,
  ArrowFunctionExpression,
  Parameter,
  TypeAnnotation,
  CatchClause,
  AwaitExpression,
  NewExpression,
  TemplateLiteral,
} from "./ast"

export interface ParserError {
  message: string
  position: Position
}

export interface ParserResult {
  program: Program | null
  errors: ParserError[]
}

export function parse(tokens: Token[]): ParserResult {
  const errors: ParserError[] = []
  let current = 0

  function peek(offset = 0): Token {
    return tokens[current + offset] ?? tokens[tokens.length - 1]
  }

  function isAtEnd(): boolean {
    return peek().type === "EOF"
  }

  function advance(): Token {
    if (!isAtEnd()) current++
    return tokens[current - 1]
  }

  function check(type: TokenType): boolean {
    return peek().type === type
  }

  function checkKeyword(keyword: string): boolean {
    return peek().type === "KEYWORD" && peek().keyword === keyword
  }

  function match(...types: TokenType[]): boolean {
    for (const type of types) {
      if (check(type)) {
        advance()
        return true
      }
    }
    return false
  }

  function matchKeyword(keyword: string): boolean {
    if (checkKeyword(keyword)) {
      advance()
      return true
    }
    return false
  }

  function expect(type: TokenType, message: string): Token {
    if (check(type)) return advance()
    const token = peek()
    errors.push({ message: `${message}, got '${token.value}'`, position: token.position })
    return token
  }

  function expectKeyword(keyword: string, message: string): Token {
    if (checkKeyword(keyword)) return advance()
    const token = peek()
    errors.push({ message: `${message}, got '${token.value}'`, position: token.position })
    return token
  }

  function error(message: string): never {
    const token = peek()
    errors.push({ message, position: token.position })
    throw new Error(message)
  }

  // Parsing functions
  function parseProgram(): Program {
    const body: Statement[] = []
    const position = peek().position

    while (!isAtEnd()) {
      try {
        body.push(parseStatement())
      }
      catch {
        // Synchronize on errors
        synchronize()
      }
    }

    return { type: "Program", body, position }
  }

  function synchronize(): void {
    advance()
    while (!isAtEnd()) {
      // Sync at statement boundaries
      if (checkKeyword("functio") || checkKeyword("esto") || checkKeyword("fixum") ||
          checkKeyword("si") || checkKeyword("dum") || checkKeyword("pro") ||
          checkKeyword("redde") || checkKeyword("tempta")) {
        return
      }
      advance()
    }
  }

  function parseStatement(): Statement {
    // Import: ex norma importa scribe, lege
    if (checkKeyword("ex")) {
      return parseImportDeclaration()
    }
    if (checkKeyword("esto") || checkKeyword("fixum")) {
      return parseVariableDeclaration()
    }
    if (checkKeyword("functio") || checkKeyword("futura")) {
      return parseFunctionDeclaration()
    }
    if (checkKeyword("si")) {
      return parseIfStatement()
    }
    if (checkKeyword("dum")) {
      return parseWhileStatement()
    }
    if (checkKeyword("pro")) {
      return parseForStatement()
    }
    if (checkKeyword("redde")) {
      return parseReturnStatement()
    }
    if (checkKeyword("iace")) {
      return parseThrowStatement()
    }
    if (checkKeyword("tempta")) {
      return parseTryStatement()
    }
    if (check("LBRACE")) {
      return parseBlockStatement()
    }

    return parseExpressionStatement()
  }

  function parseImportDeclaration(): ImportDeclaration {
    const position = peek().position
    expectKeyword("ex", "Expected 'ex'")

    // Module name
    const sourceToken = expect("IDENTIFIER", "Expected module name after 'ex'")
    const source = sourceToken.value

    expectKeyword("importa", "Expected 'importa' after module name")

    // Check for wildcard: ex norma importa *
    if (match("STAR")) {
      return { type: "ImportDeclaration", source, specifiers: [], wildcard: true, position }
    }

    // Parse comma-separated identifiers
    const specifiers: Identifier[] = []
    do {
      specifiers.push(parseIdentifier())
    } while (match("COMMA"))

    return { type: "ImportDeclaration", source, specifiers, wildcard: false, position }
  }

  function parseVariableDeclaration(): VariableDeclaration {
    const position = peek().position
    const kind = peek().keyword as "esto" | "fixum"
    advance() // esto or fixum

    const name = parseIdentifier()

    let typeAnnotation: TypeAnnotation | undefined
    if (match("COLON")) {
      typeAnnotation = parseTypeAnnotation()
    }

    let init: Expression | undefined
    if (match("EQUAL")) {
      init = parseExpression()
    }

    return { type: "VariableDeclaration", kind, name, typeAnnotation, init, position }
  }

  function parseFunctionDeclaration(): FunctionDeclaration {
    const position = peek().position
    let async = false

    if (matchKeyword("futura")) {
      async = true
    }

    expectKeyword("functio", "Expected 'functio'")

    const name = parseIdentifier()

    expect("LPAREN", "Expected '(' after function name")
    const params = parseParameterList()
    expect("RPAREN", "Expected ')' after parameters")

    let returnType: TypeAnnotation | undefined
    if (match("THIN_ARROW")) {
      returnType = parseTypeAnnotation()
    }

    const body = parseBlockStatement()

    return { type: "FunctionDeclaration", name, params, returnType, body, async, position }
  }

  function parseParameterList(): Parameter[] {
    const params: Parameter[] = []

    if (check("RPAREN")) return params

    do {
      params.push(parseParameter())
    } while (match("COMMA"))

    return params
  }

  function parseParameter(): Parameter {
    const position = peek().position

    // Check for preposition (ad, cum, in, ex)
    let preposition: string | undefined
    if (peek().type === "KEYWORD" &&
        ["ad", "cum", "in", "ex"].includes(peek().keyword ?? "")) {
      preposition = advance().keyword
    }

    const name = parseIdentifier()

    let typeAnnotation: TypeAnnotation | undefined
    if (match("COLON")) {
      typeAnnotation = parseTypeAnnotation()
    }

    return { type: "Parameter", name, typeAnnotation, preposition, position }
  }

  function parseIfStatement(): IfStatement {
    const position = peek().position
    expectKeyword("si", "Expected 'si'")

    const test = parseExpression()
    const consequent = parseBlockStatement()

    // Check for cape (catch) clause
    let catchClause: CatchClause | undefined
    if (checkKeyword("cape")) {
      catchClause = parseCatchClause()
    }

    // Check for alternate (aliter)
    let alternate: BlockStatement | IfStatement | undefined
    if (matchKeyword("aliter")) {
      if (checkKeyword("si")) {
        alternate = parseIfStatement()
      }
      else {
        alternate = parseBlockStatement()
      }
    }

    return { type: "IfStatement", test, consequent, alternate, catchClause, position }
  }

  function parseWhileStatement(): WhileStatement {
    const position = peek().position
    expectKeyword("dum", "Expected 'dum'")

    const test = parseExpression()
    const body = parseBlockStatement()

    let catchClause: CatchClause | undefined
    if (checkKeyword("cape")) {
      catchClause = parseCatchClause()
    }

    return { type: "WhileStatement", test, body, catchClause, position }
  }

  function parseForStatement(): ForStatement {
    const position = peek().position
    expectKeyword("pro", "Expected 'pro'")

    const variable = parseIdentifier()

    let kind: "in" | "ex" = "in"
    if (matchKeyword("in")) {
      kind = "in"
    }
    else if (matchKeyword("ex")) {
      kind = "ex"
    }
    else {
      error("Expected 'in' or 'ex' after variable in for loop")
    }

    const iterable = parseExpression()
    const body = parseBlockStatement()

    let catchClause: CatchClause | undefined
    if (checkKeyword("cape")) {
      catchClause = parseCatchClause()
    }

    return { type: "ForStatement", kind, variable, iterable, body, catchClause, position }
  }

  function parseReturnStatement(): ReturnStatement {
    const position = peek().position
    expectKeyword("redde", "Expected 'redde'")

    let argument: Expression | undefined
    if (!check("RBRACE") && !isAtEnd()) {
      argument = parseExpression()
    }

    return { type: "ReturnStatement", argument, position }
  }

  function parseThrowStatement(): ThrowStatement {
    const position = peek().position
    expectKeyword("iace", "Expected 'iace'")

    const argument = parseExpression()

    return { type: "ThrowStatement", argument, position }
  }

  function parseTryStatement(): Statement {
    const position = peek().position
    expectKeyword("tempta", "Expected 'tempta'")

    const block = parseBlockStatement()

    let handler: CatchClause | undefined
    if (checkKeyword("cape")) {
      handler = parseCatchClause()
    }

    let finalizer: BlockStatement | undefined
    if (matchKeyword("demum")) {
      finalizer = parseBlockStatement()
    }

    return { type: "TryStatement", block, handler, finalizer, position }
  }

  function parseCatchClause(): CatchClause {
    const position = peek().position
    expectKeyword("cape", "Expected 'cape'")

    const param = parseIdentifier()
    const body = parseBlockStatement()

    return { type: "CatchClause", param, body, position }
  }

  function parseBlockStatement(): BlockStatement {
    const position = peek().position
    expect("LBRACE", "Expected '{'")

    const body: Statement[] = []
    while (!check("RBRACE") && !isAtEnd()) {
      body.push(parseStatement())
    }

    expect("RBRACE", "Expected '}'")

    return { type: "BlockStatement", body, position }
  }

  function parseExpressionStatement(): ExpressionStatement {
    const position = peek().position
    const expression = parseExpression()
    return { type: "ExpressionStatement", expression, position }
  }

  // Expression parsing with precedence climbing
  function parseExpression(): Expression {
    return parseAssignment()
  }

  function parseAssignment(): Expression {
    const expr = parseOr()

    if (match("EQUAL")) {
      const position = peek().position
      const value = parseAssignment()
      if (expr.type === "Identifier" || expr.type === "MemberExpression") {
        return { type: "AssignmentExpression", operator: "=", left: expr, right: value, position }
      }
      error("Invalid assignment target")
    }

    return expr
  }

  function parseOr(): Expression {
    let left = parseAnd()

    while (match("OR") || matchKeyword("aut")) {
      const position = peek().position
      const right = parseAnd()
      left = { type: "BinaryExpression", operator: "||", left, right, position }
    }

    return left
  }

  function parseAnd(): Expression {
    let left = parseEquality()

    while (match("AND") || matchKeyword("et")) {
      const position = peek().position
      const right = parseEquality()
      left = { type: "BinaryExpression", operator: "&&", left, right, position }
    }

    return left
  }

  function parseEquality(): Expression {
    let left = parseComparison()

    while (match("EQUAL_EQUAL", "BANG_EQUAL")) {
      const operator = tokens[current - 1].value
      const position = tokens[current - 1].position
      const right = parseComparison()
      left = { type: "BinaryExpression", operator, left, right, position }
    }

    return left
  }

  function parseComparison(): Expression {
    let left = parseAdditive()

    while (match("LESS", "LESS_EQUAL", "GREATER", "GREATER_EQUAL")) {
      const operator = tokens[current - 1].value
      const position = tokens[current - 1].position
      const right = parseAdditive()
      left = { type: "BinaryExpression", operator, left, right, position }
    }

    return left
  }

  function parseAdditive(): Expression {
    let left = parseMultiplicative()

    while (match("PLUS", "MINUS")) {
      const operator = tokens[current - 1].value
      const position = tokens[current - 1].position
      const right = parseMultiplicative()
      left = { type: "BinaryExpression", operator, left, right, position }
    }

    return left
  }

  function parseMultiplicative(): Expression {
    let left = parseUnary()

    while (match("STAR", "SLASH", "PERCENT")) {
      const operator = tokens[current - 1].value
      const position = tokens[current - 1].position
      const right = parseUnary()
      left = { type: "BinaryExpression", operator, left, right, position }
    }

    return left
  }

  function parseUnary(): Expression {
    if (match("BANG") || matchKeyword("non")) {
      const position = tokens[current - 1].position
      const argument = parseUnary()
      return { type: "UnaryExpression", operator: "!", argument, prefix: true, position }
    }

    if (match("MINUS")) {
      const position = tokens[current - 1].position
      const argument = parseUnary()
      return { type: "UnaryExpression", operator: "-", argument, prefix: true, position }
    }

    if (matchKeyword("exspecta")) {
      const position = tokens[current - 1].position
      const argument = parseUnary()
      return { type: "AwaitExpression", argument, position }
    }

    if (matchKeyword("novum")) {
      return parseNewExpression()
    }

    return parseCall()
  }

  function parseNewExpression(): NewExpression {
    const position = tokens[current - 1].position
    const callee = parseIdentifier()

    expect("LPAREN", "Expected '(' after constructor")
    const args = parseArgumentList()
    expect("RPAREN", "Expected ')' after arguments")

    return { type: "NewExpression", callee, arguments: args, position }
  }

  function parseCall(): Expression {
    let expr = parsePrimary()

    while (true) {
      if (match("LPAREN")) {
        const position = tokens[current - 1].position
        const args = parseArgumentList()
        expect("RPAREN", "Expected ')' after arguments")
        expr = { type: "CallExpression", callee: expr, arguments: args, position }
      }
      else if (match("DOT")) {
        const position = tokens[current - 1].position
        const property = parseIdentifier()
        expr = { type: "MemberExpression", object: expr, property, computed: false, position }
      }
      else if (match("LBRACKET")) {
        const position = tokens[current - 1].position
        const property = parseExpression() as Identifier
        expect("RBRACKET", "Expected ']'")
        expr = { type: "MemberExpression", object: expr, property, computed: true, position }
      }
      else {
        break
      }
    }

    return expr
  }

  function parseArgumentList(): Expression[] {
    const args: Expression[] = []

    if (check("RPAREN")) return args

    do {
      args.push(parseExpression())
    } while (match("COMMA"))

    return args
  }

  function parsePrimary(): Expression {
    const position = peek().position

    // Boolean literals
    if (matchKeyword("verum")) {
      return { type: "Literal", value: true, raw: "verum", position }
    }
    if (matchKeyword("falsum")) {
      return { type: "Literal", value: false, raw: "falsum", position }
    }
    if (matchKeyword("nihil")) {
      return { type: "Literal", value: null, raw: "nihil", position }
    }

    // Number literal
    if (check("NUMBER")) {
      const token = advance()
      const value = token.value.includes(".") ? parseFloat(token.value) : parseInt(token.value, 10)
      return { type: "Literal", value, raw: token.value, position }
    }

    // String literal
    if (check("STRING")) {
      const token = advance()
      return { type: "Literal", value: token.value, raw: `"${token.value}"`, position }
    }

    // Template string
    if (check("TEMPLATE_STRING")) {
      const token = advance()
      return { type: "TemplateLiteral", raw: token.value, position }
    }

    // Parenthesized expression or arrow function
    if (match("LPAREN")) {
      // Could be arrow function: (x) => ...
      // Or grouped expression: (a + b)
      // Peek ahead to see if this is an arrow function

      // Simple heuristic: if we see ) => then it's an arrow function
      const startPos = current
      let parenDepth = 1

      while (parenDepth > 0 && !isAtEnd()) {
        if (check("LPAREN")) parenDepth++
        if (check("RPAREN")) parenDepth--
        if (parenDepth > 0) advance()
      }

      if (check("RPAREN")) {
        advance() // consume )
        if (check("ARROW")) {
          // It's an arrow function, backtrack and parse properly
          current = startPos
          return parseArrowFunction(position)
        }
      }

      // Not an arrow function, backtrack and parse as grouped expression
      current = startPos
      const expr = parseExpression()
      expect("RPAREN", "Expected ')'")
      return expr
    }

    // Identifier
    if (check("IDENTIFIER")) {
      return parseIdentifier()
    }

    error(`Unexpected token: ${peek().value}`)
  }

  function parseArrowFunction(position: Position): ArrowFunctionExpression {
    const params = parseParameterList()
    expect("RPAREN", "Expected ')' after arrow function parameters")
    expect("ARROW", "Expected '=>'")

    let body: Expression | BlockStatement
    if (check("LBRACE")) {
      body = parseBlockStatement()
    }
    else {
      body = parseExpression()
    }

    return { type: "ArrowFunctionExpression", params, body, async: false, position }
  }

  function parseIdentifier(): Identifier {
    const token = expect("IDENTIFIER", "Expected identifier")
    return { type: "Identifier", name: token.value, position: token.position }
  }

  function parseTypeAnnotation(): TypeAnnotation {
    const position = peek().position
    const token = expect("IDENTIFIER", "Expected type name")
    let name = token.value

    // Check for nullable (?)
    let nullable = false
    if (match("QUESTION")) {
      nullable = true
    }

    // Check for generic parameters (<T>)
    let typeParameters: TypeAnnotation[] | undefined
    if (match("LESS")) {
      typeParameters = []
      do {
        typeParameters.push(parseTypeAnnotation())
      } while (match("COMMA"))
      expect("GREATER", "Expected '>' after type parameters")
    }

    // Check for union types (|)
    let union: TypeAnnotation[] | undefined
    if (check("PIPE")) {
      union = [{ type: "TypeAnnotation", name, typeParameters, nullable, position }]
      while (match("PIPE")) {
        union.push(parseTypeAnnotation())
      }
      return { type: "TypeAnnotation", name: "union", union, position }
    }

    return { type: "TypeAnnotation", name, typeParameters, nullable, position }
  }

  // Main parse
  try {
    const program = parseProgram()
    return { program, errors }
  }
  catch {
    return { program: null, errors }
  }
}

export * from "./ast"
