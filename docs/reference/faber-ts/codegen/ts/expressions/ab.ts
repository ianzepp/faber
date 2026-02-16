/**
 * TypeScript Code Generator - Ab Expression (Collection Filtering DSL)
 *
 * TRANSFORMS:
 *   ab users activus                     -> users.filter(u => u.activus)
 *   ab users non banned                  -> users.filter(u => !u.banned)
 *   ab users ubi aetas >= 18             -> users.filter(u => u.aetas >= 18)
 *   ab users non ubi banned et suspended -> users.filter(u => !(u.banned && u.suspended))
 *   ab users activus, prima 10           -> users.filter(u => u.activus).slice(0, 10)
 *
 * WHY: 'ab' is the dedicated DSL entry point for collection filtering.
 *      The 'ex' preposition remains unchanged for iteration/import/destructuring.
 */

import type { AbExpression, Expression, Identifier, MemberExpression, AssignmentExpression } from '../../../parser/ast';
import type { TsGenerator } from '../generator';
import { applyDSLTransforms } from '../statements/iteratio';

export function genAbExpression(node: AbExpression, g: TsGenerator): string {
    const source = g.genExpression(node.source);

    // If no filter, just apply transforms (edge case)
    if (!node.filter) {
        if (node.transforms) {
            return applyDSLTransforms(source, node.transforms, g);
        }
        return source;
    }

    // Generate the filter callback
    // Use a short lambda parameter name
    const param = '_x';
    let condition: string;

    if (node.filter.hasUbi) {
        // Full condition: ab users ubi aetas >= 18
        // Need to rewrite the condition to use the lambda parameter
        condition = rewriteCondition(node.filter.condition, param, g);
    } else {
        // Boolean property shorthand: ab users activus
        // The condition is just an identifier - use it as property access
        const propName = g.genExpression(node.filter.condition);
        condition = `${param}.${propName}`;
    }

    // Apply negation if present
    if (node.negated) {
        condition = `!(${condition})`;
    }

    // Build the filter call
    let result = `${source}.filter(${param} => ${condition})`;

    // Apply any additional transforms
    if (node.transforms) {
        result = applyDSLTransforms(result, node.transforms, g);
    }

    return result;
}

/**
 * Rewrite a condition expression to use the lambda parameter.
 *
 * WHY: In `ab users ubi aetas >= 18`, `aetas` refers to a property of the current
 * element. Semantic analysis marks unknown identifiers inside `ubi` conditions as
 * `isImplicitProperty`; codegen rewrites those to `${param}.<name>`.
 */
function rewriteCondition(expr: Expression, param: string, g: TsGenerator): string {
    const rewritten = rewriteImplicitProperties(expr, param);
    return g.genExpression(rewritten);
}

function rewriteImplicitProperties(expr: Expression, param: string): Expression {
    switch (expr.type) {
        case 'Identifier': {
            const ident = expr as Identifier;
            if (!ident.isImplicitProperty) {
                return ident;
            }

            const paramIdent: Identifier = {
                type: 'Identifier',
                name: param,
                position: ident.position,
            };

            const property: Identifier = {
                type: 'Identifier',
                name: ident.name,
                position: ident.position,
            };

            const member: MemberExpression = {
                type: 'MemberExpression',
                object: paramIdent,
                property,
                computed: false,
                position: ident.position,
            };

            return member;
        }

        case 'MemberExpression': {
            const member = expr as MemberExpression;
            const object = rewriteImplicitProperties(member.object, param);
            const property = member.computed ? rewriteImplicitProperties(member.property, param) : member.property;

            if (object === member.object && property === member.property) {
                return member;
            }

            return {
                ...member,
                object,
                property,
            };
        }

        case 'UnaryExpression':
            return { ...expr, argument: rewriteImplicitProperties((expr as any).argument, param) };

        case 'BinaryExpression':
            return {
                ...expr,
                left: rewriteImplicitProperties((expr as any).left, param),
                right: rewriteImplicitProperties((expr as any).right, param),
            };

        case 'AssignmentExpression': {
            const assignment = expr as AssignmentExpression;
            const left =
                assignment.left.type === 'Identifier'
                    ? rewriteImplicitProperties(assignment.left, param)
                    : rewriteImplicitProperties(assignment.left, param);
            const right = rewriteImplicitProperties(assignment.right, param);

            return {
                ...assignment,
                left: left as any,
                right,
            };
        }

        case 'CallExpression': {
            const callee = rewriteImplicitProperties((expr as any).callee, param);
            const args = (expr as any).arguments.map((arg: any) =>
                arg.type === 'SpreadElement'
                    ? { ...arg, argument: rewriteImplicitProperties(arg.argument, param) }
                    : rewriteImplicitProperties(arg, param),
            );

            return {
                ...expr,
                callee,
                arguments: args,
            };
        }

        case 'ConditionalExpression':
            return {
                ...expr,
                test: rewriteImplicitProperties((expr as any).test, param),
                consequent: rewriteImplicitProperties((expr as any).consequent, param),
                alternate: rewriteImplicitProperties((expr as any).alternate, param),
            };

        default:
            return expr;
    }
}
