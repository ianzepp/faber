/**
 * AST Control Flow Types - Control flow statement definitions
 *
 * @module parser/ast/control
 */

import type { BaseNode } from './base';
import type { Expression, Identifier } from './expressions';
import type { CapeClause } from './actions';
import type { Annotation, ExitusModifier, ArrayPattern } from './declarations';

// Forward declaration for Statement (defined in index.ts)
import type { Statement } from './index';

// =============================================================================
// BLOCK STATEMENT
// =============================================================================

/**
 * Block statement (sequence of statements in braces).
 *
 * GRAMMAR (in EBNF):
 *   blockStmt := '{' statement* '}'
 *
 * INVARIANT: body is always an array (empty block = empty array).
 */
export interface BlockStatement extends BaseNode {
    type: 'BlockStatement';
    body: Statement[];
}

// =============================================================================
// EXPRESSION STATEMENT
// =============================================================================

/**
 * Expression statement (expression used as statement).
 *
 * GRAMMAR: exprStmt (see `EBNF.md` "Program Structure")
 *
 * INVARIANT: expression is never null.
 */
export interface ExpressionStatement extends BaseNode {
    type: 'ExpressionStatement';
    expression: Expression;
}

// =============================================================================
// CONDITIONAL STATEMENTS
// =============================================================================

/**
 * Conditional (if) statement.
 *
 * GRAMMAR: ifStmt (see `EBNF.md` "Control Flow")
 *
 * INVARIANT: catchClause is unique to Latin - allows error handling within conditionals.
 * INVARIANT: alternate can be BlockStatement (else) or SiStatement (else if chain).
 *
 * WHY: Latin 'cape' clause enables localized error handling in conditionals,
 *      not found in most target languages.
 *
 * Examples:
 *   si x > 0 { ... }
 *   si x > 0 { ... } cape erratum { ... }
 *   si x > 0 { ... } secus { ... }
 *   si x > 0 { ... } sin x < 0 { ... } secus { ... }
 */
export interface SiStatement extends BaseNode {
    type: 'SiStatement';
    test: Expression;
    consequent: BlockStatement;
    alternate?: BlockStatement | SiStatement;
    catchClause?: CapeClause;
}

// =============================================================================
// LOOP STATEMENTS
// =============================================================================

/**
 * While loop statement.
 *
 * GRAMMAR: whileStmt (see `EBNF.md` "Control Flow")
 *
 * INVARIANT: catchClause allows error handling within loop iterations.
 */
export interface DumStatement extends BaseNode {
    type: 'DumStatement';
    test: Expression;
    body: BlockStatement;
    catchClause?: CapeClause;
}

/**
 * Collection DSL transform operation.
 *
 * GRAMMAR: dslTransform (see `EBNF.md` "Collection DSL")
 *
 * WHY: DSL transforms provide concise syntax for collection operations.
 *      They desugar to method calls during code generation.
 *
 * Examples:
 *   prima 5                     -> .slice(0, 5)
 *   ultima 3                    -> .slice(-3)
 *   summa                       -> .reduce((a, b) => a + b, 0)
 *   summa pretium               -> .reduce((a, b) => a + b.pretium, 0)
 *   ordina per nomen            -> .sort((a, b) => compare(a.nomen, b.nomen))
 *   ordina per nomen descendens -> .sort((a, b) => compare(b.nomen, a.nomen))
 *   collige nomen               -> .map(x => x.nomen)
 *   grupa per categoria         -> groupBy(x => x.categoria)
 *   maximum                     -> Math.max(...arr)
 *   minimum                     -> Math.min(...arr)
 *   medium                      -> arr.reduce((a,b) => a+b, 0) / arr.length
 *   numera                      -> .length
 */
export interface CollectionDSLTransform extends BaseNode {
    type: 'CollectionDSLTransform';
    verb: string;
    argument?: Expression;
    property?: Expression;
    direction?: 'ascendens' | 'descendens';
}

export interface IteratioStatement extends BaseNode {
    type: 'IteratioStatement';
    kind: 'in' | 'ex';
    variable: Identifier | ArrayPattern;
    iterable: Expression;
    body: BlockStatement;
    async: boolean;
    catchClause?: CapeClause;
    transforms?: CollectionDSLTransform[];
}

/**
 * With statement (mutation block).
 *
 * GRAMMAR (in EBNF):
 *   withStmt := 'in' expression blockStmt
 *
 * WHY: Latin 'in' (into) establishes context for property mutation.
 *      Inside the block, bare identifiers in assignments refer to
 *      properties of the context object.
 *
 * Example:
 *   in user {
 *       nomen = "Marcus"
 *       email = "marcus@roma.it"
 *   }
 *   // Compiles to: user.nomen = "Marcus"; user.email = "marcus@roma.it";
 */
export interface InStatement extends BaseNode {
    type: 'InStatement';
    object: Expression;
    body: BlockStatement;
}

// =============================================================================
// SWITCH/MATCH STATEMENTS
// =============================================================================

/**
 * Switch statement (value matching).
 *
 * GRAMMAR (in EBNF):
 *   eligeStmt := 'elige' expression '{' eligeCase* defaultCase? '}' catchClause?
 *   eligeCase := 'casu' expression (blockStmt | 'ergo' expression)
 *   defaultCase := 'ceterum' (blockStmt | statement)
 *
 * WHY: Latin 'elige' (choose) for value-based switch.
 *      Use 'discerne' for variant pattern matching on discretio types.
 * NOTE: Codegen always lowers elige to if/else chains (no switch).
 *
 * Example:
 *   elige status {
 *       casu "pending" { processPending() }
 *       casu "active" { processActive() }
 *       ceterum { processDefault() }
 *   }
 */
export interface EligeStatement extends BaseNode {
    type: 'EligeStatement';
    discriminant: Expression;
    cases: EligeCasus[];
    defaultCase?: BlockStatement;
    catchClause?: CapeClause;
}

/**
 * Switch case for value matching (part of switch statement).
 */
export interface EligeCasus extends BaseNode {
    type: 'EligeCasus';
    test: Expression;
    consequent: BlockStatement;
}

/**
 * Variant matching statement (for discretio types).
 *
 * GRAMMAR (in EBNF):
 *   discerneStmt := 'discerne' discriminants '{' variantCase* '}'
 *   discriminants := expression (',' expression)*
 *   variantCase := 'casu' patterns (blockStmt | 'ergo' statement | 'reddit' expression)
 *   patterns := pattern (',' pattern)*
 *   pattern := '_' | (IDENTIFIER patternBind?)
 *   patternBind := ('ut' IDENTIFIER) | ('pro' IDENTIFIER (',' IDENTIFIER)*)
 *
 * WHY: 'discerne' (distinguish!) pairs with 'discretio' (the tagged union type).
 *      Uses 'casu' for match arms, 'ut' to bind whole variants, and 'pro' for positional bindings.
 *      Multi-discriminant matching reduces nesting when comparing multiple values.
 *
 * Examples:
 *   # Single discriminant (original syntax)
 *   discerne event {
 *       casu Click pro x, y { scribe "clicked at " + x + ", " + y }
 *       casu Keypress pro key { scribe "pressed " + key }
 *       casu Quit { mori "goodbye" }
 *   }
 *
 *   # Multi-discriminant (new syntax)
 *   discerne left, right {
 *       casu Primitivum ut l, Primitivum ut r { redde l.nomen == r.nomen }
 *       casu _, _ { redde falsum }
 *   }
 */
export interface DiscerneStatement extends BaseNode {
    type: 'DiscerneStatement';
    discriminants: Expression[];
    exhaustive: boolean;
    cases: VariantCase[];
    defaultCase?: BlockStatement;
}

/**
 * Single pattern in a variant case (matches one discriminant).
 *
 * WHY: Each pattern matches against one discriminant in a multi-discriminant discerne.
 *      Wildcard '_' matches any variant without binding.
 *
 * Examples:
 *   Primitivum ut p  -> variant=Primitivum, alias=p (bind whole variant)
 *   Click pro x, y   -> variant=Click, bindings=[x, y] (destructure)
 *   Quit             -> variant=Quit, no bindings
 *   _                -> wildcard, matches any variant
 */
export interface VariantPattern extends BaseNode {
    type: 'VariantPattern';
    variant: Identifier;
    isWildcard: boolean;
    alias?: Identifier;
    bindings: Identifier[];
}

/**
 * Variant case for pattern matching (part of discerne statement).
 *
 * GRAMMAR (in EBNF):
 *   variantCase := 'casu' patterns (blockStmt | 'ergo' statement | 'reddit' expression)
 *   patterns := pattern (',' pattern)*
 *
 * INVARIANT: patterns.length must equal discriminants.length (validated in semantic analysis).
 * INVARIANT: consequent is the block to execute on match.
 *
 * WHY: 'casu' (in the case of) for variant match arms.
 *      Multiple patterns enable multi-discriminant matching: `casu X ut x, Y ut y { ... }`
 *
 * Examples:
 *   casu Click ut c { ... }                  -> single pattern with alias
 *   casu Click pro x, y { ... }              -> single pattern with destructure
 *   casu Primitivum ut l, Primitivum ut r {} -> two patterns for two discriminants
 *   casu _, _ { ... }                        -> wildcard catch-all
 */
export interface VariantCase extends BaseNode {
    type: 'VariantCase';
    patterns: VariantPattern[];
    consequent: BlockStatement;
}

// =============================================================================
// GUARD STATEMENT
// =============================================================================

/**
 * Guard statement (grouped early-exit checks).
 *
 * GRAMMAR (in EBNF):
 *   guardStmt := 'custodi' '{' guardClause+ '}'
 *   guardClause := 'si' expression blockStmt
 *
 * WHY: Latin 'custodi' (guard!) groups early-exit conditions.
 *      Each clause should contain an early exit (redde, iace, rumpe, perge).
 *
 * Example:
 *   custodi {
 *       si user == nihil { redde nihil }
 *       si useri age < 0 { iace "Invalid age" }
 *   }
 */
export interface CustodiStatement extends BaseNode {
    type: 'CustodiStatement';
    clauses: CustodiClause[];
}

/**
 * Guard clause (part of guard statement).
 */
export interface CustodiClause extends BaseNode {
    type: 'CustodiClause';
    test: Expression;
    consequent: BlockStatement;
}

// =============================================================================
// ENTRY POINT STATEMENTS
// =============================================================================

/**
 * Entry point statement (sync).
 *
 * GRAMMAR (in EBNF):
 *   incipitStmt := 'incipit' (blockStmt | 'ergo' statement)
 *
 * INVARIANT: body is always a BlockStatement OR erpiStatement is set.
 *
 * WHY: Latin 'incipit' (it begins) marks the sync program entry point.
 *      This is a pure structural marker — it does not inject magic.
 *      Source is responsible for setup (allocators via cura, etc.).
 *
 *      The 'ergo' (therefore) form chains to a single statement, typically
 *      a cura block for allocator setup. This avoids extra nesting.
 *
 * Target mappings:
 *   TypeScript: top-level statements (no wrapper needed)
 *   Python:     if __name__ == "__main__": ...
 *   Zig:        pub fn main() void { ... }
 *   Rust:       fn main() { ... }
 *   C++:        int main() { ... }
 *
 * Examples:
 *   incipit {
 *       scribe "Hello"
 *   }
 *
 *   incipit ergo cura arena {
 *       // allocator-scoped work, one-liner header
 *   }
 *
 *   incipit ergo runMain()
 */
export interface IncipitStatement extends BaseNode {
    type: 'IncipitStatement';
    body: BlockStatement;
    annotations?: Annotation[];
    /** For CLI: binds parsed arguments to named variable. Usage: incipit argumenta args { } */
    argumentaBinding?: Identifier;
    /** For CLI: exit code modifier. Usage: incipit exitus 0 { } or incipit exitus code { } */
    exitusModifier?: ExitusModifier;
}

/**
 * Entry point statement (async).
 *
 * GRAMMAR (in EBNF):
 *   incipietStmt := 'incipiet' (blockStmt | 'ergo' statement)
 *
 * INVARIANT: body is always a BlockStatement OR ergoStatement is set.
 *
 * WHY: Latin 'incipiet' (it will begin) marks the async program entry point.
 *      Mirrors the fit/fiet pattern: present tense for sync, future for async.
 *
 *      The 'ergo' (therefore) form chains to a single statement, typically
 *      a cura block for allocator setup.
 *
 * Target mappings:
 *   TypeScript: (async () => { ... })()
 *   Python:     asyncio.run(main()) with async def main()
 *   Rust:       #[tokio::main] async fn main() { ... }
 *   C++:        int main() { std::async(...).get(); }
 *   Zig:        Not supported (Zig has no async main)
 *
 * Examples:
 *   incipiet {
 *       fixum data = cede fetchData()
 *       scribe data
 *   }
 *
 *   incipiet ergo cura arena {
 *       fixum data = cede fetchData()
 *   }
 */
export interface IncipietStatement extends BaseNode {
    type: 'IncipietStatement';
    body: BlockStatement;
    annotations?: Annotation[];
    /** For CLI: binds parsed arguments to named variable. Usage: incipiet argumenta args { } */
    argumentaBinding?: Identifier;
    /** For CLI: exit code modifier. Usage: incipiet exitus 0 { } or incipiet exitus code { } */
    exitusModifier?: ExitusModifier;
}
