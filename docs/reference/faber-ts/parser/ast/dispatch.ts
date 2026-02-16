/**
 * AST Dispatch Types - Ad statement and dispatch binding definitions
 *
 * @module parser/ast/dispatch
 */

import type { BaseNode } from './base';
import type { Expression, Identifier, SpreadElement } from './expressions';
import type { TypeAnnotation } from './types';
import type { BlockStatement } from './control';
import type { CapeClause } from './actions';

// =============================================================================
// AD (DISPATCH) STATEMENT
// =============================================================================

/**
 * Binding verb for ad statement result binding.
 *
 * WHY: Mirrors function return type verbs for consistency.
 *      Encodes both sync/async and single/plural semantics.
 *
 * | Verb   | Async | Plural | Meaning              |
 * |--------|-------|--------|----------------------|
 * | fit    | no    | no     | becomes (sync)       |
 * | fiet   | yes   | no     | will become (async)  |
 * | fiunt  | no    | yes    | become (sync plural) |
 * | fient  | yes   | yes    | will become (async)  |
 */
export type AdBindingVerb = 'fit' | 'fiet' | 'fiunt' | 'fient';

/**
 * Binding clause for ad statement result.
 *
 * GRAMMAR (in EBNF):
 *   adBinding := adBindingVerb typeAnnotation? 'pro' IDENTIFIER ('ut' IDENTIFIER)?
 *   adBindingVerb := 'fit' | 'fiet' | 'fiunt' | 'fient'
 *
 * INVARIANT: verb encodes sync/async and single/plural.
 * INVARIANT: typeAnnotation is optional (can be inferred from syscall table).
 * INVARIANT: name is the binding variable.
 * INVARIANT: alias is optional 'ut' rename (like import aliases).
 *
 * WHY: 'pro' introduces the binding (consistent with iteration/lambda bindings).
 *      Optional 'ut' provides an alias when the binding name should differ.
 *
 * Examples:
 *   fit textus pro content          -> sync, explicit type
 *   fiet Response pro response      -> async, explicit type
 *   fiunt textus pro lines          -> sync plural (stream)
 *   pro content                     -> type inferred, fit implied
 *   fit textus pro content ut c     -> with alias
 */
export interface AdBinding extends BaseNode {
    type: 'AdBinding';
    verb: AdBindingVerb;
    typeAnnotation?: TypeAnnotation;
    name: Identifier;
    alias?: Identifier;
}

/**
 * Ad (dispatch) statement.
 *
 * GRAMMAR (in EBNF):
 *   adStmt := 'ad' STRING '(' argumentList ')' adBinding? blockStmt? catchClause?
 *   argumentList := (expression (',' expression)*)?
 *
 * INVARIANT: target is a string literal (dispatch endpoint).
 * INVARIANT: arguments is always an array (empty for zero-arg calls).
 * INVARIANT: binding is optional (fire-and-forget if omitted).
 * INVARIANT: body is optional (can bind without block for simple cases).
 * INVARIANT: catchClause is optional error handling.
 *
 * WHY: Latin 'ad' (to/toward) dispatches to named endpoints:
 *      - Stdlib syscalls: "fasciculus:lege", "console:log"
 *      - External packages: "hono/Hono", "hono/app:serve"
 *      - Remote services: "https://api.example.com/users"
 *
 * Target resolution:
 *   - "module:method" -> stdlib dispatch
 *   - "package/export" -> external package
 *   - "http://", "https://" -> routed to caelum:request
 *   - "file://" -> routed to fasciculus:lege
 *   - "ws://", "wss://" -> routed to caelum:websocket
 *
 * Examples:
 *   // Fire-and-forget
 *   ad "console:log" ("hello")
 *
 *   // Sync binding with block
 *   ad "fasciculus:lege" ("file.txt") fit textus pro content {
 *       scribe content
 *   }
 *
 *   // Async binding
 *   ad "http:get" (url) fiet Response pro response {
 *       scribe response.body
 *   }
 *
 *   // Plural/streaming
 *   ad "http:batch" (urls) fient Response[] pro responses {
 *       ex responses pro r { scribe r.status }
 *   }
 *
 *   // With error handling
 *   ad "fasciculus:lege" ("file.txt") fit textus pro content {
 *       scribe content
 *   } cape err {
 *       scribe "Error: " + err.message
 *   }
 */
export interface AdStatement extends BaseNode {
    type: 'AdStatement';
    target: string;
    arguments: (Expression | SpreadElement)[];
    binding?: AdBinding;
    body?: BlockStatement;
    catchClause?: CapeClause;
}
