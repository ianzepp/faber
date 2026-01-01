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
    EligeCasus,
    CustodiClause,
    CapeClause,
    OrdoMember,
    OrdoDeclaration,
    FieldDeclaration,
    GenusDeclaration,
    PactumDeclaration,
    PactumMethod,
    RumpeStatement,
    PergeStatement,
    FacBlockStatement,
    LambdaExpression,
    EgoExpression,
    ImportSpecifier,
    DestructureDeclaration,
    ArrayPattern,
    ArrayPatternElement,
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
export function faberPrint(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue();

    switch (node.type) {
        // Program
        case 'Program':
            return printProgram(path as AstPath<PrettierProgram>, options, print);

        // Statements
        case 'ImportaDeclaration':
            return printImportaDeclaration(path, options, print);
        case 'ImportSpecifier':
            return printImportSpecifier(path, options, print);
        case 'DestructureDeclaration':
            return printDestructureDeclaration(path, options, print);
        case 'VariaDeclaration':
            return printVariaDeclaration(path, options, print);
        case 'FunctioDeclaration':
            return printFunctioDeclaration(path, options, print);
        case 'TypeAliasDeclaration':
            return printTypeAliasDeclaration(path, options, print);
        case 'OrdoDeclaration':
            return printOrdoDeclaration(path, options, print);
        case 'OrdoMember':
            return printOrdoMember(path, options, print);
        case 'GenusDeclaration':
            return printGenusDeclaration(path, options, print);
        case 'FieldDeclaration':
            return printFieldDeclaration(path, options, print);
        case 'PactumDeclaration':
            return printPactumDeclaration(path, options, print);
        case 'PactumMethod':
            return printPactumMethod(path, options, print);
        case 'ExpressionStatement':
            return printExpressionStatement(path, options, print);
        case 'SiStatement':
            return printSiStatement(path, options, print);
        case 'DumStatement':
            return printDumStatement(path, options, print);
        case 'IteratioStatement':
            return printIteratioStatement(path, options, print);
        case 'InStatement':
            return printInStatement(path, options, print);
        case 'EligeStatement':
            return printEligeStatement(path, options, print);
        case 'DiscerneStatement':
            return printDiscerneStatement(path, options, print);
        case 'CustodiStatement':
            return printCustodiStatement(path, options, print);
        case 'AdfirmaStatement':
            return printAdfirmaStatement(path, options, print);
        case 'ReddeStatement':
            return printReddeStatement(path, options, print);
        case 'RumpeStatement':
            return 'rumpe';
        case 'PergeStatement':
            return 'perge';
        case 'FacBlockStatement':
            return printFacBlockStatement(path, options, print);
        case 'BlockStatement':
            return printBlockStatement(path, options, print);
        case 'IaceStatement':
            return printIaceStatement(path, options, print);
        case 'TemptaStatement':
            return printTemptaStatement(path, options, print);
        case 'ScribeStatement':
            return printScribeStatement(path, options, print);

        // Expressions
        case 'Identifier':
            return printIdentifier(path, options, print);
        case 'EgoExpression':
            return 'ego';
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
        case 'AssignmentExpression':
            return printAssignmentExpression(path, options, print);
        case 'ConditionalExpression':
            return printConditionalExpression(path, options, print);
        case 'CedeExpression':
            return printCedeExpression(path, options, print);
        case 'NovumExpression':
            return printNovumExpression(path, options, print);
        case 'LambdaExpression':
            return printLambdaExpression(path, options, print);
        case 'PraefixumExpression':
            return printPraefixumExpression(path, options, print);
        case 'ScriptumExpression':
            return printScriptumExpression(path, options, print);
        case 'LegeExpression':
            return 'lege()';
        case 'CollectionDSLExpression':
            return printCollectionDSLExpression(path, options, print);
        case 'EstExpression':
            return printEstExpression(path, options, print);
        case 'QuaExpression':
            return printQuaExpression(path, options, print);

        // Type nodes
        case 'TypeAnnotation':
            return printTypeAnnotation(path, options, print);
        case 'Parameter':
            return printParameter(path, options, print);
        case 'ObjectPattern':
            return printObjectPattern(path, options, print);
        case 'ObjectPatternProperty':
            return printObjectPatternProperty(path, options, print);
        case 'ArrayPattern':
            return printArrayPattern(path, options, print);
        case 'ArrayPatternElement':
            return printArrayPatternElement(path, options, print);
        case 'ObjectProperty':
            return printObjectProperty(path, options, print);
        case 'EligeCasus':
            return printEligeCasus(path, options, print);
        case 'VariantCase':
            return printVariantCase(path, options, print);
        case 'CustodiClause':
            return printCustodiClause(path, options, print);
        case 'CapeClause':
            return printCapeClause(path, options, print);

        default:
            throw new Error(`Unknown node type: ${(node as any).type}`);
    }
}

// =============================================================================
// PROGRAM
// =============================================================================

function printProgram(path: AstPath<PrettierProgram>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printImportaDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    if (node.wildcard) {
        return ['ex ', node.source, ' importa *'];
    }

    const specifiers = path.map(print as any, 'specifiers') as Doc[];
    const threshold = getBreakThreshold(options);

    if (specifiers.length >= threshold) {
        return group(['ex ', node.source, ' importa ', indent([softline, join([',', line], specifiers)])]);
    }

    return ['ex ', node.source, ' importa ', join(', ', specifiers)];
}

function printImportSpecifier(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    // If imported and local are the same, just print the name
    if (node.imported.name === node.local.name) {
        return node.imported.name;
    }

    // Otherwise print with 'ut' alias: imported ut local
    return [node.imported.name, ' ut ', node.local.name];
}

function printDestructureDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['ex ', path.call(print as any, 'source'), ' ', node.kind, ' '];

    const specifiers = path.map(print as any, 'specifiers') as Doc[];
    const threshold = getBreakThreshold(options);

    if (specifiers.length >= threshold) {
        parts.push(indent([softline, join([',', line], specifiers)]));
    } else {
        parts.push(join(', ', specifiers));
    }

    return group(parts);
}

function printVariaDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printFunctioDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

    // Body (optional for abstract methods)
    if (node.body) {
        parts.push(' ', path.call(print, 'body'));
    }

    return parts;
}

function printTypeAliasDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    return ['typus ', path.call(print, 'name'), ' = ', path.call(print, 'typeAnnotation')];
}

function printOrdoDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const members = path.map(print, 'members');

    if (members.length === 0) {
        return ['ordo ', path.call(print, 'name'), ' {}'];
    }

    return ['ordo ', path.call(print, 'name'), ' {', indent([hardline, join([',', hardline], members)]), hardline, '}'];
}

function printOrdoMember(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [path.call(print, 'name')];

    if (node.value) {
        parts.push(' = ', path.call(print, 'value'));
    }

    return parts;
}

function printGenusDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['genus ', path.call(print, 'name')];

    // Type parameters
    if (node.typeParameters && node.typeParameters.length > 0) {
        const typeParams = path.map(print, 'typeParameters');
        parts.push('<', join(', ', typeParams), '>');
    }

    // Implements clause
    if (node.implements && node.implements.length > 0) {
        const impls = path.map(print, 'implements');
        parts.push(' implet ', join(', ', impls));
    }

    parts.push(' {');

    const bodyParts: Doc[] = [];

    // Fields
    path.each((fieldPath, index) => {
        if (index > 0 || bodyParts.length > 0) {
            bodyParts.push(hardline);
        }
        bodyParts.push(print(fieldPath as AstPath<AstNode>));
    }, 'fields');

    // Constructor
    if (node.constructor) {
        if (bodyParts.length > 0) {
            bodyParts.push(hardline, hardline);
        }
        bodyParts.push(path.call(print, 'constructor'));
    }

    // Methods
    path.each((methodPath, index) => {
        if (bodyParts.length > 0) {
            bodyParts.push(hardline, hardline);
        }
        bodyParts.push(print(methodPath as AstPath<AstNode>));
    }, 'methods');

    if (bodyParts.length > 0) {
        parts.push(indent([hardline, ...bodyParts]), hardline);
    }

    parts.push('}');
    return parts;
}

function printFieldDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    if (node.isPrivate) {
        parts.push('privatus ');
    }

    if (node.isStatic) {
        parts.push('generis ');
    }

    if (node.isReactive) {
        parts.push('nexum ');
    }

    parts.push(path.call(print, 'fieldType'), ' ', path.call(print, 'name'));

    if (node.init) {
        parts.push(': ', path.call(print, 'init'));
    }

    return parts;
}

function printComputedFieldDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    if (node.isPrivate) {
        parts.push('privatus ');
    }

    if (node.isStatic) {
        parts.push('generis ');
    }

    parts.push(path.call(print, 'fieldType'), ' ', path.call(print, 'name'), ' => ', path.call(print, 'expression'));

    return parts;
}

function printPactumDeclaration(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['pactum ', path.call(print, 'name')];

    // Type parameters
    if (node.typeParameters && node.typeParameters.length > 0) {
        const typeParams = path.map(print, 'typeParameters');
        parts.push('<', join(', ', typeParams), '>');
    }

    parts.push(' {');

    const methodParts: Doc[] = [];
    path.each((methodPath, index) => {
        if (index > 0) {
            methodParts.push(hardline);
        }
        methodParts.push(print(methodPath as AstPath<AstNode>));
    }, 'methods');

    if (methodParts.length > 0) {
        parts.push(indent([hardline, ...methodParts]), hardline);
    }

    parts.push('}');
    return parts;
}

function printPactumMethod(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

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

    return parts;
}

function printFacBlockStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['fac ', path.call(print, 'body')];

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    if (node.test) {
        parts.push(' dum ', path.call(print, 'test'));
    }

    return parts;
}

function printLambdaExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['pro'];

    if (node.params.length > 0) {
        const params = path.map(print, 'params');
        parts.push(' ', join(', ', params));
    }

    // Optional return type: pro x -> numerus: expr
    if (node.returnType) {
        parts.push(' -> ', path.call(print, 'returnType'));
    }

    if (node.body.type === 'BlockStatement') {
        parts.push(' ', path.call(print, 'body'));
    } else {
        // Expression form uses colon: pro x: expr
        parts.push(': ', path.call(print, 'body'));
    }

    return parts;
}

function printExpressionStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return path.call(print, 'expression');
}

function printSiStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];

    // Catch clause (unique to Faber)
    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    // Else clause - Stroustrup style (else on new line)
    if (node.alternate) {
        if (node.alternate.type === 'SiStatement') {
            parts.push(hardline, 'secus ', path.call(print, 'alternate'));
        } else {
            parts.push(hardline, 'secus ', path.call(print, 'alternate'));
        }
    }

    return parts;
}

function printDumStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['dum ', path.call(print, 'test'), ' ', path.call(print, 'body')];

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printIteratioStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [node.kind, ' ', path.call(print, 'iterable'), ' pro ', path.call(print, 'variable'), ' ', path.call(print, 'body')];

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printInStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return ['in ', path.call(print, 'object'), ' ', path.call(print, 'body')];
}

function printEligeStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['elige ', path.call(print, 'discriminant'), ' {'];

    // Cases
    if (node.cases.length > 0 || node.defaultCase) {
        const caseParts: Doc[] = [];

        path.each(casePath => {
            caseParts.push(hardline, print(casePath as AstPath<AstNode>));
        }, 'cases');

        if (node.defaultCase) {
            caseParts.push(hardline, 'secus ', path.call(print, 'defaultCase'));
        }

        parts.push(indent(caseParts), hardline);
    }

    parts.push('}');

    if (node.catchClause) {
        parts.push(hardline, path.call(print, 'catchClause'));
    }

    return parts;
}

function printEligeCasus(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];
}

function printDiscerneStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['discerne ', path.call(print, 'discriminant'), ' {'];

    // Cases
    if (node.cases.length > 0) {
        const caseParts: Doc[] = [];

        path.each(casePath => {
            caseParts.push(hardline, print(casePath as AstPath<AstNode>));
        }, 'cases');

        parts.push(indent(caseParts), hardline);
    }

    parts.push('}');

    return parts;
}

function printVariantCase(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['si ', path.call(print, 'variant')];

    if (node.bindings && node.bindings.length > 0) {
        parts.push(' pro ');
        path.each((bindingPath, index) => {
            if (index > 0) {
                parts.push(', ');
            }
            parts.push(print(bindingPath as AstPath<AstNode>));
        }, 'bindings');
    }

    parts.push(' ', path.call(print, 'consequent'));

    return parts;
}

function printCustodiStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const clauseParts: Doc[] = [];

    path.each((clausePath, index) => {
        if (index > 0) {
            clauseParts.push(hardline);
        }
        clauseParts.push(print(clausePath as AstPath<AstNode>));
    }, 'clauses');

    return ['custodi {', indent([hardline, ...clauseParts]), hardline, '}'];
}

function printCustodiClause(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return ['si ', path.call(print, 'test'), ' ', path.call(print, 'consequent')];
}

function printAdfirmaStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['adfirma ', path.call(print, 'test')];

    if (node.message) {
        parts.push(', ', path.call(print, 'message'));
    }

    return parts;
}

function printReddeStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    if (node.argument) {
        return ['redde ', path.call(print, 'argument')];
    }

    return 'redde';
}

function printBlockStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printIaceStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const keyword = node.fatal ? 'mori' : 'iace';
    return [keyword, ' ', path.call(print, 'argument')];
}

function printTemptaStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printCapeClause(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return ['cape ', path.call(print, 'param'), ' ', path.call(print, 'body')];
}

function printScribeStatement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const keyword = node.level === 'debug' ? 'vide' : node.level === 'warn' ? 'mone' : 'scribe';
    const args = path.map(print, 'arguments');
    return [keyword, ' ', join(', ', args)];
}

// =============================================================================
// EXPRESSIONS
// =============================================================================

function printIdentifier(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    return node.name;
}

function printLiteral(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    // Prefer raw representation, fall back to stringified value
    if (node.raw !== undefined) {
        return node.raw;
    }

    // Handle missing raw field (e.g., enum member values)
    if (node.value === null) {
        return 'nihil';
    }
    if (typeof node.value === 'boolean') {
        return node.value ? 'verum' : 'falsum';
    }
    if (typeof node.value === 'string') {
        return `"${node.value}"`;
    }
    return String(node.value);
}

function printTemplateLiteral(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    // Ensure backticks are present (tokenizer may strip them)
    const raw = node.raw || '';
    if (raw.startsWith('`') && raw.endsWith('`')) {
        return raw;
    }
    return `\`${raw}\``;
}

function printArrayExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printObjectExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printObjectProperty(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return [path.call(print, 'key'), ': ', path.call(print, 'value')];
}

function printObjectPattern(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const properties = path.map(print, 'properties');
    return ['{ ', join(', ', properties), ' }'];
}

function printObjectPatternProperty(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    // If key and value are the same, use shorthand
    if (node.key.name === node.value.name) {
        return path.call(print, 'key');
    }

    return [path.call(print, 'key'), ': ', path.call(print, 'value')];
}

function printArrayPattern(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const elements = path.map(print as any, 'elements');
    return ['[', join(', ', elements), ']'];
}

function printArrayPatternElement(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    // Skip element (underscore)
    if (node.skip) {
        return '_';
    }

    // Rest element (ceteri pattern)
    if (node.rest) {
        return ['ceteri ', path.call(print as any, 'name')];
    }

    // Regular element
    return path.call(print as any, 'name');
}

function printRangeExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [path.call(print, 'start'), '..', path.call(print, 'end')];

    if (node.step) {
        parts.push(' per ', path.call(print, 'step'));
    }

    return parts;
}

function printBinaryExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    return group([path.call(print, 'left'), ' ', node.operator, line, path.call(print, 'right')]);
}

function printUnaryExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printCallExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const args = path.map(print, 'arguments');
    const threshold = getBreakThreshold(options);

    if (args.length >= threshold) {
        return [path.call(print, 'callee'), group(['(', indent([softline, join([',', line], args)]), softline, ')'])];
    }

    return [path.call(print, 'callee'), '(', join(', ', args), ')'];
}

function printMemberExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;

    if (node.computed) {
        return [path.call(print, 'object'), '[', path.call(print, 'property'), ']'];
    }

    return [path.call(print, 'object'), '.', path.call(print, 'property')];
}

function printAssignmentExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    return [path.call(print, 'left'), ' ', node.operator, ' ', path.call(print, 'right')];
}

function printConditionalExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return group([path.call(print, 'test'), indent([line, '? ', path.call(print, 'consequent'), line, ': ', path.call(print, 'alternate')])]);
}

function printCedeExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return ['cede ', path.call(print, 'argument')];
}

function printNovumExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const args = path.map(print, 'arguments');
    const threshold = getBreakThreshold(options);

    if (args.length >= threshold) {
        return ['novum ', path.call(print, 'callee'), group(['(', indent([softline, join([',', line], args)]), softline, ')'])];
    }

    return ['novum ', path.call(print, 'callee'), '(', join(', ', args), ')'];
}

function printPraefixumExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    if (node.body?.type === 'BlockStatement') {
        return ['praefixum ', path.call(print, 'body')];
    }
    return ['praefixum(', path.call(print, 'body'), ')'];
}

function printScriptumExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const args = path.map(print, 'arguments');
    const threshold = getBreakThreshold(options);

    if (args.length >= threshold) {
        return group(['scriptum(', path.call(print, 'format'), ',', indent([line, join([',', line], args)]), softline, ')']);
    }

    const allArgs = [path.call(print, 'format'), ...args];
    return ['scriptum(', join(', ', allArgs), ')'];
}

function printCollectionDSLExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = ['ex ', path.call(print, 'source')];
    for (const transform of node.transforms || []) {
        parts.push(' ', transform.verb);
        if (transform.argument) {
            parts.push(' ', path.call(print, 'transforms', node.transforms.indexOf(transform), 'argument'));
        }
    }
    return parts;
}

function printEstExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return [path.call(print, 'expression'), ' est ', path.call(print, 'typeAnnotation')];
}

function printQuaExpression(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    return [path.call(print, 'expression'), ' qua ', path.call(print, 'targetType')];
}

// =============================================================================
// TYPE ANNOTATIONS
// =============================================================================

function printTypeAnnotation(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
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

function printParameter(path: AstPath<AstNode>, options: FaberOptions, print: (path: AstPath<AstNode>) => Doc): Doc {
    const node = path.getValue() as any;
    const parts: Doc[] = [];

    // Rest parameter prefix
    if (node.rest) {
        parts.push('ceteri ');
    }

    // Preposition
    if (node.preposition) {
        parts.push(node.preposition, ' ');
    }

    // Type annotation (type-first syntax)
    if (node.typeAnnotation) {
        parts.push(path.call(print, 'typeAnnotation'), ' ');
    }

    // Name (external name)
    parts.push(path.call(print, 'name'));

    // Alias (internal name) - dual parameter naming: textus location ut loc
    if (node.alias) {
        parts.push(' ut ', path.call(print, 'alias'));
    }

    // Default value - textus name vel "World"
    if (node.defaultValue) {
        parts.push(' vel ', path.call(print, 'defaultValue'));
    }

    return parts;
}
