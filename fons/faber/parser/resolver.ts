/**
 * Resolver - Parser Interface for Mutual Recursion
 *
 * Solves mutual recursion between expression and statement parsers.
 * Each parsing function receives a Resolver instead of raw ParserContext,
 * and calls methods for cross-module parsing.
 *
 * WHY: Expression parsers need to parse blocks (for lambdas), and
 *      statement parsers need to parse expressions. This creates
 *      circular imports. By using an interface, each module
 *      depends only on Resolver, not on each other.
 *
 * The concrete implementation (ParserImpl) is defined in index.ts,
 * which imports all the actual parsing functions.
 *
 * @module parser/resolver
 */

import type { ParserContext } from './context';
import type { Expression, Statement, BlockStatement, TypeAnnotation, Annotation } from './ast';

/**
 * Parser interface for cross-module parsing.
 *
 * Usage in parsing functions:
 *   function parseFoo(r: Resolver): Foo {
 *       const ctx = r.ctx();              // get parser state
 *       const expr = r.expression();      // parse expression
 *       const block = r.block();          // parse block
 *       const typ = r.typeAnnotation();   // parse type annotation
 *       ...
 *   }
 */
export interface Resolver {
    /** Get the underlying parser context (state) */
    ctx(): ParserContext;

    /** Parse an expression (handles precedence) */
    expression(): Expression;

    /** Parse a single statement */
    statement(): Statement;

    /** Parse a block of statements (BlockStatement) */
    block(): BlockStatement;

    /** Parse a type annotation */
    typeAnnotation(): TypeAnnotation;

    /** Parse annotations (@ decorators) */
    annotations(): Annotation[];
}
