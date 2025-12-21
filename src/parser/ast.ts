import type { Position } from "../tokenizer/types"
import type { Case, Number as GramNumber } from "../lexicon/types"

// Base node with position for error reporting
export interface BaseNode {
  position: Position
}

// Program is the root node
export interface Program extends BaseNode {
  type: "Program"
  body: Statement[]
}

// Statements
export type Statement =
  | VariableDeclaration
  | FunctionDeclaration
  | ExpressionStatement
  | IfStatement
  | WhileStatement
  | ForStatement
  | ReturnStatement
  | BlockStatement
  | ThrowStatement
  | TryStatement

export interface VariableDeclaration extends BaseNode {
  type: "VariableDeclaration"
  kind: "esto" | "fixum"  // let | const
  name: Identifier
  typeAnnotation?: TypeAnnotation
  init?: Expression
}

export interface FunctionDeclaration extends BaseNode {
  type: "FunctionDeclaration"
  name: Identifier
  params: Parameter[]
  returnType?: TypeAnnotation
  body: BlockStatement
  async: boolean  // futura functio
}

export interface Parameter extends BaseNode {
  type: "Parameter"
  name: Identifier
  typeAnnotation?: TypeAnnotation
  case?: Case  // Latin case for semantic role
  preposition?: string  // ad, cum, in, ex
}

export interface ExpressionStatement extends BaseNode {
  type: "ExpressionStatement"
  expression: Expression
}

export interface IfStatement extends BaseNode {
  type: "IfStatement"
  test: Expression
  consequent: BlockStatement
  alternate?: BlockStatement | IfStatement
  catchClause?: CatchClause  // si ... { } cape erratum { }
}

export interface WhileStatement extends BaseNode {
  type: "WhileStatement"
  test: Expression
  body: BlockStatement
  catchClause?: CatchClause
}

export interface ForStatement extends BaseNode {
  type: "ForStatement"
  kind: "in" | "ex"  // for...in vs for...of
  variable: Identifier
  iterable: Expression
  body: BlockStatement
  catchClause?: CatchClause
}

export interface ReturnStatement extends BaseNode {
  type: "ReturnStatement"
  argument?: Expression
}

export interface BlockStatement extends BaseNode {
  type: "BlockStatement"
  body: Statement[]
}

export interface ThrowStatement extends BaseNode {
  type: "ThrowStatement"
  argument: Expression
}

export interface TryStatement extends BaseNode {
  type: "TryStatement"
  block: BlockStatement
  handler?: CatchClause
  finalizer?: BlockStatement
}

export interface CatchClause extends BaseNode {
  type: "CatchClause"
  param: Identifier
  body: BlockStatement
}

// Expressions
export type Expression =
  | Identifier
  | Literal
  | BinaryExpression
  | UnaryExpression
  | CallExpression
  | MemberExpression
  | ArrowFunctionExpression
  | AssignmentExpression
  | ConditionalExpression
  | AwaitExpression
  | NewExpression
  | TemplateLiteral

export interface Identifier extends BaseNode {
  type: "Identifier"
  name: string
  // Latin morphology info (if parsed)
  morphology?: {
    stem: string
    case?: Case
    number?: GramNumber
  }
}

export interface Literal extends BaseNode {
  type: "Literal"
  value: string | number | boolean | null
  raw: string
}

export interface TemplateLiteral extends BaseNode {
  type: "TemplateLiteral"
  raw: string
  // For now, store as raw string. Full parsing would extract expressions.
}

export interface BinaryExpression extends BaseNode {
  type: "BinaryExpression"
  operator: string
  left: Expression
  right: Expression
}

export interface UnaryExpression extends BaseNode {
  type: "UnaryExpression"
  operator: string
  argument: Expression
  prefix: boolean
}

export interface CallExpression extends BaseNode {
  type: "CallExpression"
  callee: Expression
  arguments: Expression[]
}

export interface MemberExpression extends BaseNode {
  type: "MemberExpression"
  object: Expression
  property: Identifier
  computed: boolean  // obj[prop] vs obj.prop
}

export interface ArrowFunctionExpression extends BaseNode {
  type: "ArrowFunctionExpression"
  params: Parameter[]
  body: Expression | BlockStatement
  async: boolean
}

export interface AssignmentExpression extends BaseNode {
  type: "AssignmentExpression"
  operator: string
  left: Identifier | MemberExpression
  right: Expression
}

export interface ConditionalExpression extends BaseNode {
  type: "ConditionalExpression"
  test: Expression
  consequent: Expression
  alternate: Expression
}

export interface AwaitExpression extends BaseNode {
  type: "AwaitExpression"
  argument: Expression
}

export interface NewExpression extends BaseNode {
  type: "NewExpression"
  callee: Identifier
  arguments: Expression[]
}

// Type annotations
export interface TypeAnnotation extends BaseNode {
  type: "TypeAnnotation"
  name: string
  typeParameters?: TypeAnnotation[]
  nullable?: boolean
  union?: TypeAnnotation[]
}
