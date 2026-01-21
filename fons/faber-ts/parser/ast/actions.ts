/**
 * AST Action Types - Action statement definitions (throw, return, print, try-catch, etc.)
 *
 * @module parser/ast/actions
 */

import type { BaseNode } from './base';
import type { Expression, Identifier, Literal } from './expressions';
import type { BlockStatement } from './control';

// =============================================================================
// CONTROL FLOW ACTIONS
// =============================================================================

/**
 * Assert statement.
 *
 * GRAMMAR (in EBNF):
 *   assertStmt := 'adfirma' expression (',' STRING)?
 *
 * WHY: Latin 'adfirma' (affirm/assert) for runtime invariant checks.
 *      Always-on runtime checks - if condition is false, throws an error.
 *      Optional message for custom error text; otherwise auto-generated.
 *
 * Example:
 *   adfirma x > 0
 *   adfirma x > 0, "x must be positive"
 */
export interface AdfirmaStatement extends BaseNode {
    type: 'AdfirmaStatement';
    test: Expression;
    message?: Expression;
}

/**
 * Return statement.
 *
 * GRAMMAR (in EBNF):
 *   returnStmt := 'redde' expression?
 *
 * INVARIANT: argument is optional (void return).
 */
export interface ReddeStatement extends BaseNode {
    type: 'ReddeStatement';
    argument?: Expression;
}

/**
 * Break statement (loop exit).
 *
 * GRAMMAR (in EBNF):
 *   breakStmt := 'rumpe'
 *
 * WHY: Latin 'rumpe' (break!) exits the innermost loop.
 *      Used within dum, ex...pro, and de...pro loops.
 *
 * Example:
 *   dum verum {
 *       si found { rumpe }
 *   }
 */
export interface RumpeStatement extends BaseNode {
    type: 'RumpeStatement';
}

/**
 * Continue statement (loop skip).
 *
 * GRAMMAR (in EBNF):
 *   continueStmt := 'perge'
 *
 * WHY: Latin 'perge' (continue/proceed!) skips to next iteration.
 *      Used within dum, ex...pro, and de...pro loops.
 *
 * Example:
 *   ex items pro item {
 *       si item.skip { perge }
 *       process(item)
 *   }
 */
export interface PergeStatement extends BaseNode {
    type: 'PergeStatement';
}

/**
 * No-op statement (intentional silence).
 *
 * GRAMMAR (in EBNF):
 *   tacetStmt := 'tacet'
 *
 * WHY: Latin 'tacet' (it is silent) for explicit empty blocks.
 *      From musical notation where tacet indicates a rest/silence.
 *      Makes intentional emptiness explicit vs forgotten implementation.
 *
 * Example:
 *   si debug tacet secus process()
 *   casu Littera ut l tacet
 */
export interface TacetStatement extends BaseNode {
    type: 'TacetStatement';
}

// =============================================================================
// EXCEPTION HANDLING
// =============================================================================

/**
 * Throw/panic statement.
 *
 * GRAMMAR (in EBNF):
 *   throwStmt := ('iace' | 'mori') expression
 *
 * INVARIANT: argument is never null.
 *
 * WHY: Latin error keywords for two severity levels:
 *   iace (throw!) → recoverable error, can be caught
 *   mori (die!)   → fatal/panic, unrecoverable
 *
 * Target mappings:
 *   iace → throw (TS/Py), return error.X (Zig), return Err (Rust)
 *   mori → throw (TS/Py), @panic (Zig), panic! (Rust)
 */
export interface IaceStatement extends BaseNode {
    type: 'IaceStatement';
    fatal: boolean;
    argument: Expression;
}

/**
 * Try-catch-finally statement.
 *
 * GRAMMAR (in EBNF):
 *   tryStmt := 'tempta' blockStmt ('cape' IDENTIFIER blockStmt)? ('demum' blockStmt)?
 *
 * INVARIANT: At least one of handler or finalizer SHOULD be present.
 *
 * WHY: Latin keywords:
 *      tempta = try (attempt)
 *      cape = catch (seize/capture)
 *      demum = finally (at last)
 *
 * Target mappings:
 *   tempta → try (TS/Py), N/A (Zig uses error unions)
 *   cape   → catch (TS), except (Py), catch |err| (Zig)
 *   demum  → finally (TS), finally (Py), defer (Zig)
 */
export interface TemptaStatement extends BaseNode {
    type: 'TemptaStatement';
    block: BlockStatement;
    handler?: CapeClause;
    finalizer?: BlockStatement;
}

/**
 * Catch clause (part of try or control flow statements).
 *
 * GRAMMAR (in EBNF):
 *   catchClause := 'cape' IDENTIFIER blockStmt
 *
 * INVARIANT: param is the error variable name.
 *
 * WHY: Reusable in both TemptaStatement and control flow (SiStatement, loops).
 */
export interface CapeClause extends BaseNode {
    type: 'CapeClause';
    param: Identifier;
    body: BlockStatement;
}

// =============================================================================
// OUTPUT STATEMENTS
// =============================================================================

/**
 * Output level for scribe/vide/mone statements.
 *
 * WHY: Latin has three output keywords mapping to different console levels:
 *   scribe (write!) → console.log  - normal output
 *   vide (see!)     → console.debug - developer/debug output
 *   mone (warn!)    → console.warn  - warning output
 */
export type OutputLevel = 'log' | 'debug' | 'warn';

/**
 * Scribe (print) statement.
 *
 * GRAMMAR (in EBNF):
 *   scribeStmt := ('scribe' | 'vide' | 'mone') expression (',' expression)*
 *
 * WHY: Latin output keywords as statement forms, not function calls.
 *
 * Target mappings:
 *   scribe → console.log (TS), print() (Py), std.debug.print (Zig)
 *   vide   → console.debug (TS), print("[DEBUG]") (Py), std.debug.print (Zig)
 *   mone   → console.warn (TS), print("[WARN]") (Py), std.debug.print (Zig)
 *
 * Examples:
 *   scribe "hello"
 *   vide "debugging:", value
 *   mone "warning: value is", x
 */
export interface ScribeStatement extends BaseNode {
    type: 'ScribeStatement';
    level: OutputLevel;
    arguments: Expression[];
}

// =============================================================================
// FAC (DO) BLOCK
// =============================================================================

/**
 * Fac block statement (explicit scope block or do-while loop).
 *
 * GRAMMAR (in EBNF):
 *   facBlockStmt := 'fac' blockStmt ('cape' IDENTIFIER blockStmt)? ('dum' expression)?
 *
 * INVARIANT: body is always a BlockStatement.
 * INVARIANT: catchClause is optional - for error handling.
 * INVARIANT: test is optional - when present, creates do-while loop.
 *
 * WHY: Latin 'fac' (do!) creates an explicit scope boundary.
 *      Unlike `si verum { }`, this communicates intent clearly.
 *      When paired with 'cape', provides error boundary semantics.
 *      When paired with 'dum', creates do-while loop (body executes first).
 *
 * Examples:
 *   fac { riskyOperation() }
 *   fac { riskyOperation() } cape err { handleError(err) }
 *   fac { process() } dum hasMore()
 *   fac { process() } cape err { log(err) } dum hasMore()
 */
export interface FacBlockStatement extends BaseNode {
    type: 'FacBlockStatement';
    body: BlockStatement;
    catchClause?: CapeClause;
    test?: Expression;
}

