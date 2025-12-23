/**
 * Prettier Printer for Faber Romanus
 *
 * ARCHITECTURE
 * ============
 * Recursive AST visitor using Prettier's doc builders. Each node type
 * has a dedicated print function that returns a Doc (formatting document).
 *
 * Style Rules:
 * - 4-space indentation
 * - Stroustrup braces (opening on same line, else/catch on new line)
 * - Break parameter lists at 3+ items
 * - 100 character print width
 *
 * @module prettier/printer
 */

import type { AstPath, Doc, ParserOptions } from 'prettier';
import { doc } from 'prettier';
import type {
    Program,
    Statement,
    Expression,
    BlockStatement,
    Parameter,
    TypeAnnotation,
    TypeParameter,
    ObjectProperty,
    ObjectPatternProperty,
    SwitchCase,
    GuardClause,
    CatchClause,
} from '../parser/ast.ts';
import type { AstNode, PrettierProgram } from './parser.ts';

const { group, indent, line, softline, hardline, join, ifBreak } = doc.builders;

// =============================================================================
// OPTIONS
// =============================================================================

interface FaberOptions extends ParserOptions {
    faberBreakThreshold?: number;
}

function getBreakThreshold(options: FaberOptions): number {
    return options.faberBreakThreshold ?? 3;
}

// =============================================================================
// MAIN PRINT FUNCTION
// =============================================================================

/**
 * Main entry point for Prettier printing.
 */
export function faberPrint(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue();

    switch (node.type) {
        // Program
        case 'Program':
            return printProgram(path as AstPath<PrettierProgram>, options, print);

        // Statements
        case 'ImportDeclaration':
            return printImportDeclaration(path, options, print);
        case 'VariableDeclaration':
            return printVariableDeclaration(path, options, print);
        case 'FunctionDeclaration':
            return printFunctionDeclaration(path, options, print);
        case 'TypeAliasDeclaration':
            return printTypeAliasDeclaration(path, options, print);
        case 'ExpressionStatement':
            return printExpressionStatement(path, options, print);
        case 'IfStatement':
            return printIfStatement(path, options, print);
        case 'WhileStatement':
            return printWhileStatement(path, options, print);
        case 'ForStatement':
            return printForStatement(path, options, print);
        case 'WithStatement':
            return printWithStatement(path, options, print);
        case 'SwitchStatement':
            return printSwitchStatement(path, options, print);
        case 'GuardStatement':
            return printGuardStatement(path, options, print);
        case 'AssertStatement':
            return printAssertStatement(path, options, print);
        case 'ReturnStatement':
            return printReturnStatement(path, options, print);
        case 'BlockStatement':
            return printBlockStatement(path, options, print);
        case 'ThrowStatement':
            return printThrowStatement(path, options, print);
        case 'TryStatement':
            return printTryStatement(path, options, print);
        case 'ScribeStatement':
            return printScribeStatement(path, options, print);

        // Expressions
        case 'Identifier':
            return printIdentifier(path, options, print);
        case 'Literal':
            return printLiteral(path, options, print);
        case 'TemplateLiteral':
            return printTemplateLiteral(path, options, print);
        case 'ArrayExpression':
            return printArrayExpression(path, options, print);
        case 'ObjectExpression':
            return printObjectExpression(path, options, print);
        case 'RangeExpression':
            return printRangeExpression(path, options, print);
        case 'BinaryExpression':
            return printBinaryExpression(path, options, print);
        case 'UnaryExpression':
            return printUnaryExpression(path, options, print);
        case 'CallExpression':
            return printCallExpression(path, options, print);
        case 'MemberExpression':
            return printMemberExpression(path, options, print);
        case 'ArrowFunctionExpression':
            return printArrowFunctionExpression(path, options, print);
        case 'AssignmentExpression':
            return printAssignmentExpression(path, options, print);
        case 'ConditionalExpression':
            return printConditionalExpression(path, options, print);
        case 'AwaitExpression':
            return printAwaitExpression(path, options, print);
        case 'NewExpression':
            return printNewExpression(path, options, print);

        // Type nodes
        case 'TypeAnnotation':
            return printTypeAnnotation(path, options, print);
        case 'Parameter':
            return printParameter(path, options, print);
        case 'ObjectPattern':
            return printObjectPattern(path, options, print);
        case 'ObjectPatternProperty':
            return printObjectPatternProperty(path, options, print);
        case 'ObjectProperty':
            return printObjectProperty(path, options, print);
        case 'SwitchCase':
            return printSwitchCase(path, options, print);
        case 'GuardClause':
            return printGuardClause(path, options, print);
        case 'CatchClause':
            return printCatchClause(path, options, print);

        default:
            throw new Error(`Unknown node type: ${(node as any).type}`);
    }
}

// =============================================================================
// PROGRAM
// =============================================================================

function printProgram(
    path: AstPath<PrettierProgram>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue();

    if (node.body.length === 0) {
        return '';
    }

    // Print statements, preserving blank lines from the original source
    const parts: Doc[] = [];
    path.each((stmtPath, index) => {
        const stmt = stmtPath.getValue() as Statement;
        const prevStmt = index > 0 ? node.body[index - 1] : null;

        // Preserve blank lines: if there was a gap > 1 line in the original,
        // insert a blank line in the output
        if (index > 0 && prevStmt) {
            const prevLine = prevStmt.position.line;
            const currLine = stmt.position.line;

            // If there's more than 1 line gap, preserve one blank line
            if (currLine - prevLine > 1) {
                parts.push(hardline);
            }
        }

        parts.push(print(stmtPath as AstPath<AstNode>));

        if (index < node.body.length - 1) {
            parts.push(hardline);
        }
    }, 'body');

    return [...parts, hardline];
}

// =============================================================================
// STATEMENTS
// =============================================================================

function printImportDeclaration(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.wildcard) {
        return ['ex ', node.source, ' importa *'];
    }

    const specifiers = path.map(print, 'specifiers');
    const threshold = getBreakThreshold(options);

    if (specifiers.length >= threshold) {
        return group([
            'ex ',
            node.source,
            ' importa ',
            indent([softline, join([',', line], specifiers)]),
        ]);
    }

    return ['ex ', node.source, ' importa ', join(', ', specifiers)];
}

function printVariableDeclaration(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [node.kind, ' '];

    // Type annotation (type-first syntax)
    if (node.typeAnnotation) {
        parts.push(path.call(print, 'typeAnnotation'), ' ');
    }

    // Name (identifier or destructuring pattern)
    parts.push(path.call(print, 'name'));

    // Initializer
    if (node.init) {
        parts.push(' = ', path.call(print, 'init'));
    }

    return parts;
}

function printFunctionDeclaration(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    // Async modifier
    if (node.async) {
        parts.push('futura ');
    }

    parts.push('functio ', path.call(print, 'name'));

    // Parameters
    const params = path.map(print, 'params');
    const threshold = getBreakThreshold(options);

    if (params.length >= threshold) {
        parts.push(group(['(', indent([softline, join([',', line], params)]), softline, ')']));
    } else if (params.length > 0) {
        parts.push('(', join(', ', params), ')');
    } else {
        parts.push('()');
    }

    // Return type
    if (node.returnType) {
        parts.push(' -> ', path.call(print, 'returnType'));
    }

    // Body
    parts.push(' ', path.call(print, 'body'));

    return parts;
}

function printTypeAliasDeclaration(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return ['typus ', path.call(print, 'name'), ' = ', path.call(print, 'typeAnnotation')];
}

function printExpressionStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return path.call(print, 'expression');
}

function printIfStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];

    // Catch clause (unique to Faber)
    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    // Else clause - Stroustrup style (else on new line)
    if (node.alternate) {
        if (node.alternate.type === 'IfStatement') {
            parts.push(hardline, 'aliter ', path.call(print, 'alternate'));
        } else {
            parts.push(hardline, 'aliter ', path.call(print, 'alternate'));
        }
    }

    return parts;
}

function printWhileStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['dum ', path.call(print, 'test'), ' ', path.call(print, 'body')];

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printForStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [
        node.kind,
        ' ',
        path.call(print, 'iterable'),
        ' pro ',
        path.call(print, 'variable'),
        ' ',
        path.call(print, 'body'),
    ];

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printWithStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['cum ', path.call(print, 'object'), ' ', path.call(print, 'body')];
}

function printSwitchStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['elige ', path.call(print, 'discriminant'), ' {'];

    // Cases
    if (node.cases.length > 0 || node.defaultCase) {
        const caseParts: Doc[] = [];

        path.each(casePath => {
            caseParts.push(hardline, print(casePath as AstPath<AstNode>));
        }, 'cases');

        if (node.defaultCase) {
            caseParts.push(hardline, 'aliter ', path.call(print, 'defaultCase'));
        }

        parts.push(indent(caseParts), hardline);
    }

    parts.push('}');

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printSwitchCase(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];
}

function printGuardStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const clauseParts: Doc[] = [];

    path.each((clausePath, index) => {
        if (index > 0) {
            clauseParts.push(hardline);
        }
        clauseParts.push(print(clausePath as AstPath<AstNode>));
    }, 'clauses');

    return ['custodi {', indent([hardline, ...clauseParts]), hardline, '}'];
}

function printGuardClause(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];
}

function printAssertStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['adfirma ', path.call(print, 'test')];

    if (node.message) {
        parts.push(', ', path.call(print, 'message'));
    }

    return parts;
}

function printReturnStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.argument) {
        return ['redde ', path.call(print, 'argument')];
    }

    return 'redde';
}

function printBlockStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.body.length === 0) {
        return '{}';
    }

    const stmtParts: Doc[] = [];

    path.each((stmtPath, index) => {
        const stmt = stmtPath.getValue() as Statement;
        const prevStmt = index > 0 ? node.body[index - 1] : null;

        // Preserve blank lines from original source
        if (index > 0 && prevStmt) {
            const prevLine = prevStmt.position.line;
            const currLine = stmt.position.line;

            if (currLine - prevLine > 1) {
                stmtParts.push(hardline);
            }
        }

        if (index > 0) {
            stmtParts.push(hardline);
        }
        stmtParts.push(print(stmtPath as AstPath<AstNode>));
    }, 'body');

    return ['{', indent([hardline, ...stmtParts]), hardline, '}'];
}

function printThrowStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['iace ', path.call(print, 'argument')];
}

function printTryStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['tempta ', path.call(print, 'block')];

    if (node.handler) {
        parts.push(hardline, path.call(print, 'handler'));
    }

    if (node.finalizer) {
        parts.push(hardline, 'demum ', path.call(print, 'finalizer'));
    }

    return parts;
}

function printCatchClause(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['cape ', path.call(print, 'param'), ' ', path.call(print, 'body')];
}

function printScribeStatement(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const args = path.map(print, 'arguments');
    return ['scribe ', join(', ', args)];
}

// =============================================================================
// EXPRESSIONS
// =============================================================================

function printIdentifier(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return node.name;
}

function printLiteral(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return node.raw;
}

function printTemplateLiteral(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return node.raw;
}

function printArrayExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.elements.length === 0) {
        return '[]';
    }

    const elements = path.map(print, 'elements');
    const threshold = getBreakThreshold(options);

    // Check if any element is an array (nested arrays should break)
    const hasNestedArray = node.elements.some((el: any) => el.type === 'ArrayExpression');

    if (elements.length >= threshold || hasNestedArray) {
        return group(['[', indent([softline, join([',', line], elements)]), softline, ']']);
    }

    return ['[', join(', ', elements), ']'];
}

function printObjectExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.properties.length === 0) {
        return '{}';
    }

    const properties = path.map(print, 'properties');
    const threshold = getBreakThreshold(options);

    if (properties.length >= threshold) {
        return group(['{', indent([line, join([',', line], properties)]), line, '}']);
    }

    return ['{ ', join(', ', properties), ' }'];
}

function printObjectProperty(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return [path.call(print, 'key'), ': ', path.call(print, 'value')];
}

function printObjectPattern(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const properties = path.map(print, 'properties');
    return ['{ ', join(', ', properties), ' }'];
}

function printObjectPatternProperty(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    // If key and value are the same, use shorthand
    if (node.key.name === node.value.name) {
        return path.call(print, 'key');
    }

    return [path.call(print, 'key'), ': ', path.call(print, 'value')];
}

function printRangeExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [path.call(print, 'start'), '..', path.call(print, 'end')];

    if (node.step) {
        parts.push(' per ', path.call(print, 'step'));
    }

    return parts;
}

function printBinaryExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return group([path.call(print, 'left'), ' ', node.operator, line, path.call(print, 'right')]);
}

function printUnaryExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.prefix) {
        // Handle Latin keyword operators that need a space before the operand
        if (node.operator === 'non' || node.operator === 'nulla' || node.operator === 'nonnulla') {
            return [node.operator, ' ', path.call(print, 'argument')];
        }
        return [node.operator, path.call(print, 'argument')];
    }

    return [path.call(print, 'argument'), node.operator];
}

function printCallExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const args = path.map(print, 'arguments');
    const threshold = getBreakThreshold(options);

    if (args.length >= threshold) {
        return [
            path.call(print, 'callee'),
            group(['(', indent([softline, join([',', line], args)]), softline, ')']),
        ];
    }

    return [path.call(print, 'callee'), '(', join(', ', args), ')'];
}

function printMemberExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    if (node.computed) {
        return [path.call(print, 'object'), '[', path.call(print, 'property'), ']'];
    }

    return [path.call(print, 'object'), '.', path.call(print, 'property')];
}

function printArrowFunctionExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    // Parameters
    const params = path.map(print, 'params');
    const threshold = getBreakThreshold(options);

    if (params.length >= threshold) {
        parts.push(group(['(', indent([softline, join([',', line], params)]), softline, ')']));
    } else if (params.length === 1 && !node.params[0].typeAnnotation) {
        // Single param without type can omit parens
        parts.push(params[0]);
    } else if (params.length > 0) {
        parts.push('(', join(', ', params), ')');
    } else {
        parts.push('()');
    }

    parts.push(' => ');

    // Body
    if (node.body.type === 'BlockStatement') {
        parts.push(path.call(print, 'body'));
    } else {
        parts.push(path.call(print, 'body'));
    }

    return parts;
}

function printAssignmentExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    return [path.call(print, 'left'), ' ', node.operator, ' ', path.call(print, 'right')];
}

function printConditionalExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return group([
        path.call(print, 'test'),
        indent([
            line,
            '? ',
            path.call(print, 'consequent'),
            line,
            ': ',
            path.call(print, 'alternate'),
        ]),
    ]);
}

function printAwaitExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    return ['cede ', path.call(print, 'argument')];
}

function printNewExpression(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const args = path.map(print, 'arguments');
    const threshold = getBreakThreshold(options);

    if (args.length >= threshold) {
        return [
            'novum ',
            path.call(print, 'callee'),
            group(['(', indent([softline, join([',', line], args)]), softline, ')']),
        ];
    }

    return ['novum ', path.call(print, 'callee'), '(', join(', ', args), ')'];
}

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

function printTypeAnnotation(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;

    // Union type
    if (node.union && node.union.length > 0) {
        const unionParts = path.map(print, 'union');
        return join(' | ', unionParts);
    }

    // Array shorthand: lista<T> with arrayShorthand flag -> T[]
    if (node.arrayShorthand && node.name === 'lista' && node.typeParameters?.length === 1) {
        const innerType = path.call(print, 'typeParameters', 0);
        const parts: Doc[] = [innerType, '[]'];
        if (node.nullable) {
            parts.push('?');
        }
        return parts;
    }

    const parts: Doc[] = [node.name];

    // Type parameters
    if (node.typeParameters && node.typeParameters.length > 0) {
        const typeParams = path.map(print, 'typeParameters');
        parts.push('<', join(', ', typeParams), '>');
    }

    // Nullable
    if (node.nullable) {
        parts.push('?');
    }

    return parts;
}

function printParameter(
    path: AstPath<AstNode>,
    options: FaberOptions,
    print: (path: AstPath<AstNode>) => Doc,
): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    // Preposition
    if (node.preposition) {
        parts.push(node.preposition, ' ');
    }

    // Type annotation (type-first syntax)
    if (node.typeAnnotation) {
        parts.push(path.call(print, 'typeAnnotation'), ' ');
    }

    // Name
    parts.push(path.call(print, 'name'));

    return parts;
}
