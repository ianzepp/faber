/**
 * TypeScript Code Generator - Emit JavaScript with type annotations
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into TypeScript source code.
 * It preserves JavaScript runtime semantics while adding TypeScript type
 * annotations derived from Latin type declarations.
 *
 * The generator uses a recursive descent pattern that mirrors the AST structure.
 * Each AST node type has a corresponding gen* function that produces a string
 * fragment. These fragments are composed bottom-up to build the complete output.
 *
 * Indentation is managed via a depth counter that tracks nesting level. The
 * ind() helper function generates the appropriate indentation string for the
 * current depth.
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid TypeScript source code string
 * ERRORS: Throws on unknown AST node types (should never happen with valid AST)
 *
 * TARGET DIFFERENCES
 * ==================
 * TypeScript preserves JavaScript semantics:
 * - Dynamic typing with optional annotations
 * - Prototype-based objects
 * - Async/await for concurrency
 * - Exception-based error handling
 * - Nullable types via union with null
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid TypeScript
 * INV-2: All Latin type names are mapped to TypeScript equivalents
 * INV-3: Indentation depth is correctly maintained (incremented/decremented)
 */

import type {
    Program,
    Statement,
    Expression,
    ImportDeclaration,
    VariableDeclaration,
    FunctionDeclaration,
    GenusDeclaration,
    FieldDeclaration,
    ComputedFieldDeclaration,
    PactumDeclaration,
    PactumMethod,
    TypeAliasDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    WithStatement,
    SwitchStatement,
    GuardStatement,
    AssertStatement,
    ReturnStatement,
    BlockStatement,
    ThrowStatement,
    ScribeStatement,
    TryStatement,
    ExpressionStatement,
    ArrayExpression,
    ObjectExpression,
    RangeExpression,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrowFunctionExpression,
    AssignmentExpression,
    NewExpression,
    Literal,
    ThisExpression,
    Parameter,
    TypeAnnotation,
    TypeParameter,
} from '../parser/ast';
import type { CodegenOptions } from './types';

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to TypeScript types.
 *
 * WHY: Latin uses descriptive names (textus = text), TypeScript uses
 *      JavaScript primitive names (string). This mapping preserves semantics
 *      while emitting idiomatic TypeScript.
 *
 * TARGET MAPPING:
 * | Latin      | TypeScript |
 * |------------|------------|
 * | textus     | string     |
 * | numerus    | number     |
 * | bivalens   | boolean    |
 * | nihil      | null       |
 * | lista      | Array      |
 * | tabula     | Map        |
 * | copia      | Set        |
 * | promissum  | Promise    |
 * | erratum    | Error      |
 * | cursor     | Iterator   |
 *
 * CASE: Keys are lowercase. Lookup normalizes input to lowercase for
 *       case-insensitive matching (textus, textus, TEXTUS all work).
 */
const typeMap: Record<string, string> = {
    textus: 'string',
    numerus: 'number',
    bivalens: 'boolean',
    nihil: 'null',
    lista: 'Array',
    tabula: 'Map',
    copia: 'Set',
    promissum: 'Promise',
    erratum: 'Error',
    cursor: 'Iterator',
    vacuum: 'void',
};

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate TypeScript source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> TypeScript source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration (indent, semicolons)
 * @returns TypeScript source code
 */
export function generateTs(program: Program, options: CodegenOptions = {}): string {
    // WHY: 2 spaces is TypeScript convention
    const indent = options.indent ?? '  ';
    // WHY: Semicolons are recommended in TypeScript style guides
    const semi = options.semicolons ?? true;

    // Track indentation depth for nested blocks
    let depth = 0;

    /**
     * Generate indentation for current depth.
     * WHY: Centralized indentation logic ensures consistent formatting.
     */
    function ind(): string {
        return indent.repeat(depth);
    }

    // ---------------------------------------------------------------------------
    // Top-level
    // ---------------------------------------------------------------------------

    function genProgram(node: Program): string {
        // WHY: Each top-level statement is separated by newline
        return node.body.map(genStatement).join('\n');
    }

    // ---------------------------------------------------------------------------
    // Statements
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any statement type.
     * WHY: Exhaustive switch ensures all statement types are handled.
     */
    function genStatement(node: Statement): string {
        switch (node.type) {
            case 'ImportDeclaration':
                return genImportDeclaration(node);
            case 'VariableDeclaration':
                return genVariableDeclaration(node);
            case 'FunctionDeclaration':
                return genFunctionDeclaration(node);
            case 'GenusDeclaration':
                return genGenusDeclaration(node);
            case 'PactumDeclaration':
                return genPactumDeclaration(node);
            case 'TypeAliasDeclaration':
                return genTypeAliasDeclaration(node);
            case 'IfStatement':
                return genIfStatement(node);
            case 'WhileStatement':
                return genWhileStatement(node);
            case 'ForStatement':
                return genForStatement(node);
            case 'WithStatement':
                return genWithStatement(node);
            case 'SwitchStatement':
                return genSwitchStatement(node);
            case 'GuardStatement':
                return genGuardStatement(node);
            case 'AssertStatement':
                return genAssertStatement(node);
            case 'ReturnStatement':
                return genReturnStatement(node);
            case 'ThrowStatement':
                return genThrowStatement(node);
            case 'ScribeStatement':
                return genScribeStatement(node);
            case 'TryStatement':
                return genTryStatement(node);
            case 'BlockStatement':
                return genBlockStatement(node);
            case 'ExpressionStatement':
                return genExpressionStatement(node);
            default:
                throw new Error(`Unknown statement type: ${(node as any).type}`);
        }
    }

    /**
     * Generate import declaration.
     *
     * TRANSFORMS:
     *   ex norma importa * -> import * as norma from "norma"
     *   ex norma importa scribe, lege -> import { scribe, lege } from "norma"
     */
    function genImportDeclaration(node: ImportDeclaration): string {
        const source = node.source;

        if (node.wildcard) {
            return `${ind()}import * as ${source} from "${source}"${semi ? ';' : ''}`;
        }

        const names = node.specifiers.map(s => s.name).join(', ');

        return `${ind()}import { ${names} } from "${source}"${semi ? ';' : ''}`;
    }

    /**
     * Generate variable declaration.
     *
     * TRANSFORMS:
     *   varia x: numerus = 5 -> let x: number = 5
     *   fixum y: textus = "hello" -> const y: string = "hello"
     *   fixum { nomen, aetas } = persona -> const { nomen, aetas } = persona
     *   fixum { nomen: localName } = persona -> const { nomen: localName } = persona
     */
    function genVariableDeclaration(node: VariableDeclaration): string {
        // WHY: 'varia' (let it be) maps to mutable 'let', 'fixum' (fixed) to immutable 'const'
        const kind = node.kind === 'varia' ? 'let' : 'const';

        let name: string;

        if (node.name.type === 'ObjectPattern') {
            // Generate object destructuring pattern
            const props = node.name.properties.map(prop => {
                if (prop.key.name === prop.value.name) {
                    return prop.key.name;
                }

                return `${prop.key.name}: ${prop.value.name}`;
            });

            name = `{ ${props.join(', ')} }`;
        } else {
            name = node.name.name;
        }

        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        return `${ind()}${kind} ${name}${typeAnno}${init}${semi ? ';' : ''}`;
    }

    /**
     * Generate genus (struct) declaration as TypeScript class.
     *
     * TRANSFORMS:
     *   genus persona { textus nomen numerus aetas }
     *   ->
     *   class persona {
     *       nomen: string;
     *       aetas: number;
     *   }
     *
     * WHY: TypeScript classes provide the closest analog to Latin genus -
     *      encapsulated data with methods.
     */
    function genGenusDeclaration(node: GenusDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters
            ? `<${node.typeParameters.map(p => p.name).join(', ')}>`
            : '';
        const impl = node.implements
            ? ` implements ${node.implements.map(i => i.name).join(', ')}`
            : '';

        const lines: string[] = [];
        lines.push(`${ind()}class ${name}${typeParams}${impl} {`);

        depth++;

        const sections: string[][] = [];

        if (node.fields.length > 0) {
            sections.push(node.fields.map(genFieldDeclaration));
        }

        if (node.computedFields.length > 0) {
            sections.push(node.computedFields.map(genComputedFieldDeclaration));
        }

        if (node.constructor) {
            sections.push([genConstructorDeclaration(node.constructor)]);
        }

        if (node.methods.length > 0) {
            sections.push(node.methods.map(genMethodDeclaration));
        }

        sections.forEach((section, index) => {
            if (index > 0) {
                lines.push('');
            }
            lines.push(...section);
        });

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    /**
     * Generate field declaration within a class.
     */
    function genFieldDeclaration(node: FieldDeclaration): string {
        const visibility = node.isPublic ? '' : 'private ';
        const staticMod = node.isStatic ? 'static ' : '';
        const name = node.name.name;
        const type = genType(node.fieldType);
        const init = node.init ? ` = ${genExpression(node.init)}` : '';

        return `${ind()}${visibility}${staticMod}${name}: ${type}${init}${semi ? ';' : ''}`;
    }

    /**
     * Generate computed field declaration as a getter.
     */
    function genComputedFieldDeclaration(node: ComputedFieldDeclaration): string {
        const visibility = node.isPublic ? '' : 'private ';
        const staticMod = node.isStatic ? 'static ' : '';
        const name = node.name.name;
        const type = genType(node.fieldType);
        const expression = genExpression(node.expression);

        return `${ind()}${visibility}${staticMod}get ${name}(): ${type} { return ${expression}; }`;
    }

    /**
     * Generate method declaration within a class.
     */
    function genMethodDeclaration(node: FunctionDeclaration): string {
        const asyncMod = node.async ? 'async ' : '';
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        const returnType = node.returnType ? `: ${genType(node.returnType)}` : '';
        const body = genBlockStatement(node.body);

        return `${ind()}${asyncMod}${name}(${params})${returnType} ${body}`;
    }

    function genConstructorDeclaration(node: FunctionDeclaration): string {
        const params = node.params.map(genParameter).join(', ');
        const body = genBlockStatement(node.body);

        return `${ind()}constructor(${params}) ${body}`;
    }

    /**
     * Generate pactum declaration as a TypeScript interface.
     */
    function genPactumDeclaration(node: PactumDeclaration): string {
        const name = node.name.name;
        const typeParams = node.typeParameters
            ? `<${node.typeParameters.map(p => p.name).join(', ')}>`
            : '';
        const lines: string[] = [];

        lines.push(`${ind()}interface ${name}${typeParams} {`);
        depth++;

        for (const method of node.methods) {
            lines.push(genPactumMethod(method));
        }

        depth--;
        lines.push(`${ind()}}`);

        return lines.join('\n');
    }

    function genPactumMethod(node: PactumMethod): string {
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        let returnType = node.returnType ? genType(node.returnType) : 'void';

        if (node.async) {
            returnType = `Promise<${returnType}>`;
        }

        return `${ind()}${name}(${params}): ${returnType}${semi ? ';' : ''}`;
    }

    /**
     * Generate type alias declaration.
     *
     * TRANSFORMS:
     *   typus ID = textus -> type ID = string;
     *   typus UserID = numerus<32, Naturalis> -> type UserID = number;
     *
     * WHY: TypeScript type aliases provide semantic naming without runtime cost.
     */
    function genTypeAliasDeclaration(node: TypeAliasDeclaration): string {
        const name = node.name.name;
        const typeAnno = genType(node.typeAnnotation);

        return `${ind()}type ${name} = ${typeAnno}${semi ? ';' : ''}`;
    }

    /**
     * Generate type parameter (type, literal, or modifier).
     *
     * TRANSFORMS:
     *   textus -> string (nested type)
     *   32 -> ignored (size constraint)
     *   Naturalis -> ignored (modifier)
     *
     * WHY: TypeScript doesn't support numeric size constraints or modifiers.
     *      These are semantic hints for other targets (Zig, C++) but not TS.
     */
    function genTypeParameter(param: TypeParameter): string | null {
        if (param.type === 'TypeAnnotation') {
            return genType(param);
        }

        if (param.type === 'Literal') {
            // EDGE: Numeric params like numerus<32> indicate size/width
            // For TypeScript, we ignore size - all numbers are float64
            return null;
        }

        if (param.type === 'ModifierParameter') {
            // EDGE: Modifiers like Naturalis (unsigned) don't exist in TS
            // TypeScript has no unsigned types or ownership semantics
            return null;
        }

        return null;
    }

    /**
     * Generate type annotation from Latin type.
     *
     * TRANSFORMS:
     *   textus -> string
     *   lista<numerus> -> Array<number>
     *   textus? -> string | null
     *   numerus<32> -> number (size ignored)
     *   numerus<Naturalis> -> number (modifier ignored)
     */
    function genType(node: TypeAnnotation): string {
        // Map Latin type name to TS type (case-insensitive lookup)
        const base = typeMap[node.name.toLowerCase()] ?? node.name;

        // Handle generic type parameters: lista<textus> -> Array<string>
        let result = base;

        if (node.typeParameters && node.typeParameters.length > 0) {
            // Filter out null params (literals and modifiers that TS doesn't support)
            const params = node.typeParameters
                .map(genTypeParameter)
                .filter((p): p is string => p !== null);

            // Only add type params if we have any left after filtering
            if (params.length > 0) {
                result = `${base}<${params.join(', ')}>`;
            }
        }

        // Handle nullable: textus? -> string | null
        if (node.nullable) {
            result = `${result} | null`;
        }

        // Handle union types
        if (node.union && node.union.length > 0) {
            result = node.union.map(genType).join(' | ');
        }

        return result;
    }

    /**
     * Generate function declaration.
     *
     * TRANSFORMS:
     *   functio salve(nomen: textus): nihil -> function salve(nomen: string): null
     *   futura functio f(): numerus -> async function f(): number
     */
    function genFunctionDeclaration(node: FunctionDeclaration): string {
        const async = node.async ? 'async ' : '';
        const name = node.name.name;
        const params = node.params.map(genParameter).join(', ');
        const returnType = node.returnType ? `: ${genType(node.returnType)}` : '';
        const body = genBlockStatement(node.body);

        return `${ind()}${async}function ${name}(${params})${returnType} ${body}`;
    }

    /**
     * Generate function parameter.
     *
     * TRANSFORMS:
     *   nomen: textus -> nomen: string
     */
    function genParameter(node: Parameter): string {
        const name = node.name.name;
        const typeAnno = node.typeAnnotation ? `: ${genType(node.typeAnnotation)}` : '';

        return `${name}${typeAnno}`;
    }

    /**
     * Generate if statement.
     *
     * TRANSFORMS:
     *   si (conditio) { ... } -> if (conditio) { ... }
     *   si (conditio) { ... } aliter { ... } -> if (conditio) { ... } else { ... }
     *
     * WHY: Latin if-statements can have optional catch clauses for exception handling.
     *      When present, we wrap the entire if in a try-catch block.
     */
    function genIfStatement(node: IfStatement): string {
        let result = '';

        // WHY: Latin allows 'capta' (catch) clause on if-statements for brevity
        if (node.catchClause) {
            result += `${ind()}try {\n`;
            depth++;
            result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;
        } else {
            result += `${ind()}if (${genExpression(node.test)}) ${genBlockStatement(node.consequent)}`;
        }

        if (node.alternate) {
            if (node.alternate.type === 'IfStatement') {
                result += ` else ${genIfStatement(node.alternate).trim()}`;
            } else {
                result += ` else ${genBlockStatement(node.alternate)}`;
            }
        }

        return result;
    }

    function genWhileStatement(node: WhileStatement): string {
        const test = genExpression(node.test);
        const body = genBlockStatement(node.body);

        if (node.catchClause) {
            let result = `${ind()}try {\n`;

            depth++;
            result += `${ind()}while (${test}) ${body}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

            return result;
        }

        return `${ind()}while (${test}) ${body}`;
    }

    /**
     * Generate for statement.
     *
     * TRANSFORMS:
     *   ex 0..10 pro i { } -> for (let i = 0; i <= 10; i++) { }
     *   ex 0..10 per 2 pro i { } -> for (let i = 0; i <= 10; i += 2) { }
     *   ex items pro item { } -> for (const item of items) { }
     *
     * WHY: Range expressions compile to efficient traditional for loops
     *      instead of allocating arrays.
     */
    function genForStatement(node: ForStatement): string {
        const varName = node.variable.name;
        const body = genBlockStatement(node.body);

        // Check if iterable is a range expression for efficient loop generation
        if (node.iterable.type === 'RangeExpression') {
            const range = node.iterable;
            const start = genExpression(range.start);
            const end = genExpression(range.end);

            let forHeader: string;

            if (range.step) {
                const step = genExpression(range.step);

                // With step: need to handle positive/negative direction
                // For simplicity, assume positive step uses <=, negative uses >=
                forHeader = `for (let ${varName} = ${start}; ${varName} <= ${end}; ${varName} += ${step})`;
            } else {
                // Default step of 1
                forHeader = `for (let ${varName} = ${start}; ${varName} <= ${end}; ${varName}++)`;
            }

            if (node.catchClause) {
                let result = `${ind()}try {\n`;

                depth++;
                result += `${ind()}${forHeader} ${body}`;
                depth--;
                result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

                return result;
            }

            return `${ind()}${forHeader} ${body}`;
        }

        // Standard for-of/for-in loop
        const iterable = genExpression(node.iterable);
        const keyword = node.kind === 'in' ? 'in' : 'of';

        if (node.catchClause) {
            let result = `${ind()}try {\n`;

            depth++;
            result += `${ind()}for (const ${varName} ${keyword} ${iterable}) ${body}`;
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;

            return result;
        }

        return `${ind()}for (const ${varName} ${keyword} ${iterable}) ${body}`;
    }

    /**
     * Generate with statement (context block).
     *
     * TRANSFORMS:
     *   cum user { nomen = "Marcus" } -> user.nomen = "Marcus";
     *
     * WHY: Bare identifier assignments inside cum blocks become property
     *      assignments on the context object.
     */
    function genWithStatement(node: WithStatement): string {
        const context = genExpression(node.object);
        const lines: string[] = [];

        for (const stmt of node.body.body) {
            if (
                stmt.type === 'ExpressionStatement' &&
                stmt.expression.type === 'AssignmentExpression' &&
                stmt.expression.left.type === 'Identifier'
            ) {
                const prop = stmt.expression.left.name;
                const value = genExpression(stmt.expression.right);
                const op = stmt.expression.operator;

                lines.push(`${ind()}${context}.${prop} ${op} ${value}${semi ? ';' : ''}`);
            } else {
                lines.push(genStatement(stmt));
            }
        }

        return lines.join('\n');
    }

    /**
     * Generate switch statement.
     *
     * TRANSFORMS:
     *   elige x { si 1 { a() } si 2 { b() } aliter { c() } }
     *   -> switch (x) { case 1: a(); break; case 2: b(); break; default: c(); }
     */
    function genSwitchStatement(node: SwitchStatement): string {
        const discriminant = genExpression(node.discriminant);
        let result = '';

        if (node.catchClause) {
            result += `${ind()}try {\n`;
            depth++;
        }

        result += `${ind()}switch (${discriminant}) {\n`;
        depth++;

        for (const caseNode of node.cases) {
            const test = genExpression(caseNode.test);

            result += `${ind()}case ${test}: {\n`;
            depth++;

            for (const stmt of caseNode.consequent.body) {
                result += genStatement(stmt) + '\n';
            }

            result += `${ind()}break${semi ? ';' : ''}\n`;
            depth--;
            result += `${ind()}}\n`;
        }

        if (node.defaultCase) {
            result += `${ind()}default: {\n`;
            depth++;

            for (const stmt of node.defaultCase.body) {
                result += genStatement(stmt) + '\n';
            }

            depth--;
            result += `${ind()}}\n`;
        }

        depth--;
        result += `${ind()}}`;

        if (node.catchClause) {
            depth--;
            result += `\n${ind()}} catch (${node.catchClause.param.name}) ${genBlockStatement(node.catchClause.body)}`;
        }

        return result;
    }

    /**
     * Generate guard statement.
     *
     * TRANSFORMS:
     *   custodi { si x == nihil { redde } si y < 0 { iace "error" } }
     *   -> if (x == null) { return; } if (y < 0) { throw "error"; }
     *
     * WHY: Guard clauses are just sequential if statements with early exits.
     */
    function genGuardStatement(node: GuardStatement): string {
        const lines: string[] = [];

        for (const clause of node.clauses) {
            const test = genExpression(clause.test);
            const body = genBlockStatement(clause.consequent);

            lines.push(`${ind()}if (${test}) ${body}`);
        }

        return lines.join('\n');
    }

    /**
     * Generate assert statement.
     *
     * TRANSFORMS:
     *   adfirma x > 0 -> if (!(x > 0)) throw new Error("Assertion failed: x > 0");
     *   adfirma x > 0, "custom" -> if (!(x > 0)) throw new Error("custom");
     *
     * WHY: Always-on runtime checks that throw on failure.
     *      Auto-generates message from expression if not provided.
     */
    function genAssertStatement(node: AssertStatement): string {
        const test = genExpression(node.test);

        let message: string;

        if (node.message) {
            message = genExpression(node.message);
        } else {
            // Auto-generate message from the test expression
            message = `"Assertion failed: ${test.replace(/"/g, '\\"')}"`;
        }

        return `${ind()}if (!(${test})) { throw new Error(${message}); }`;
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (node.argument) {
            return `${ind()}return ${genExpression(node.argument)}${semi ? ';' : ''}`;
        }

        return `${ind()}return${semi ? ';' : ''}`;
    }

    function genThrowStatement(node: ThrowStatement): string {
        return `${ind()}throw ${genExpression(node.argument)}${semi ? ';' : ''}`;
    }

    function genScribeStatement(node: ScribeStatement): string {
        const args = node.arguments.map(genExpression).join(', ');
        return `${ind()}console.log(${args})${semi ? ';' : ''}`;
    }

    function genTryStatement(node: TryStatement): string {
        let result = `${ind()}try ${genBlockStatement(node.block)}`;

        if (node.handler) {
            result += ` catch (${node.handler.param.name}) ${genBlockStatement(node.handler.body)}`;
        }

        if (node.finalizer) {
            result += ` finally ${genBlockStatement(node.finalizer)}`;
        }

        return result;
    }

    function genBlockStatement(node: BlockStatement): string {
        if (node.body.length === 0) {
            return '{}';
        }

        depth++;
        const body = node.body.map(genStatement).join('\n');

        depth--;

        return `{\n${body}\n${ind()}}`;
    }

    function genExpressionStatement(node: ExpressionStatement): string {
        return `${ind()}${genExpression(node.expression)}${semi ? ';' : ''}`;
    }

    // ---------------------------------------------------------------------------
    // Expressions
    // ---------------------------------------------------------------------------

    /**
     * Generate code for any expression type.
     * WHY: Exhaustive switch ensures all expression types are handled.
     */
    function genExpression(node: Expression): string {
        switch (node.type) {
            case 'Identifier':
                return node.name;
            case 'Literal':
                return genLiteral(node);
            case 'TemplateLiteral':
                return `\`${node.raw}\``;
            case 'ThisExpression':
                return 'this';
            case 'ArrayExpression':
                return genArrayExpression(node);
            case 'BinaryExpression':
                return genBinaryExpression(node);
            case 'UnaryExpression':
                return genUnaryExpression(node);
            case 'CallExpression':
                return genCallExpression(node);
            case 'MemberExpression':
                return genMemberExpression(node);
            case 'ArrowFunctionExpression':
                return genArrowFunction(node);
            case 'AssignmentExpression':
                return genAssignmentExpression(node);
            case 'AwaitExpression':
                return `await ${genExpression(node.argument)}`;
            case 'NewExpression':
                return genNewExpression(node);
            case 'ConditionalExpression':
                return `${genExpression(node.test)} ? ${genExpression(node.consequent)} : ${genExpression(node.alternate)}`;
            case 'RangeExpression':
                return genRangeExpression(node);
            case 'ObjectExpression':
                return genObjectExpression(node);
            default:
                throw new Error(`Unknown expression type: ${(node as any).type}`);
        }
    }

    /**
     * Generate literal value.
     *
     * TRANSFORMS:
     *   "hello" -> "hello" (JSON-escaped)
     *   42 -> 42
     *   verum -> true
     *   nihil -> null
     *
     * WHY: JSON.stringify ensures proper escaping of string literals.
     */
    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return 'null';
        }

        if (typeof node.value === 'string') {
            return JSON.stringify(node.value);
        }

        if (typeof node.value === 'boolean') {
            return node.value ? 'true' : 'false';
        }

        return String(node.value);
    }

    /**
     * Generate array literal.
     *
     * TRANSFORMS:
     *   [] -> []
     *   [1, 2, 3] -> [1, 2, 3]
     *   [[1], [2]] -> [[1], [2]]
     */
    function genArrayExpression(node: ArrayExpression): string {
        const elements = node.elements.map(genExpression).join(', ');

        return `[${elements}]`;
    }

    /**
     * Generate object literal.
     *
     * TRANSFORMS:
     *   { nomen: "Marcus" } -> { nomen: "Marcus" }
     *   { nomen: x, aetas: y } -> { nomen: x, aetas: y }
     */
    function genObjectExpression(node: ObjectExpression): string {
        if (node.properties.length === 0) {
            return '{}';
        }

        const props = node.properties.map(prop => {
            const key = prop.key.type === 'Identifier' ? prop.key.name : genLiteral(prop.key);
            const value = genExpression(prop.value);

            return `${key}: ${value}`;
        });

        return `{ ${props.join(', ')} }`;
    }

    /**
     * Generate range expression as array.
     *
     * TRANSFORMS:
     *   0..5 -> Array.from({length: 6}, (_, i) => i)
     *   2..5 -> Array.from({length: 4}, (_, i) => 2 + i)
     *   0..10 per 2 -> Array.from({length: 6}, (_, i) => i * 2)
     *
     * WHY: When used outside a for-loop, ranges become arrays.
     *      In for-loops, they compile to efficient traditional loops instead.
     */
    function genRangeExpression(node: RangeExpression): string {
        const start = genExpression(node.start);
        const end = genExpression(node.end);

        if (node.step) {
            const step = genExpression(node.step);

            // With step: more complex calculation
            return `Array.from({length: Math.floor((${end} - ${start}) / ${step}) + 1}, (_, i) => ${start} + i * ${step})`;
        }

        // Simple range: start to end inclusive
        return `Array.from({length: ${end} - ${start} + 1}, (_, i) => ${start} + i)`;
    }

    /**
     * Generate binary expression.
     *
     * TRANSFORMS:
     *   x + y -> (x + y)
     *   a && b -> (a && b)
     *
     * WHY: Parentheses ensure correct precedence in all contexts.
     */
    function genBinaryExpression(node: BinaryExpression): string {
        const left = genExpression(node.left);
        const right = genExpression(node.right);

        return `(${left} ${node.operator} ${right})`;
    }

    /**
     * Generate unary expression.
     *
     * TRANSFORMS:
     *   !x -> !x (prefix)
     *   x++ -> x++ (postfix)
     *   nulla x -> inline empty check
     *   nonnulla x -> inline non-empty check
     */
    function genUnaryExpression(node: UnaryExpression): string {
        const arg = genExpression(node.argument);

        // nulla: check if null/empty
        if (node.operator === 'nulla') {
            return `(${arg} == null || (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length === 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length === 0 : !${arg}))`;
        }

        // nonnulla: check if non-null and has content
        if (node.operator === 'nonnulla') {
            return `(${arg} != null && (Array.isArray(${arg}) || typeof ${arg} === 'string' ? ${arg}.length > 0 : typeof ${arg} === 'object' ? Object.keys(${arg}).length > 0 : Boolean(${arg})))`;
        }

        return node.prefix ? `${node.operator}${arg}` : `${arg}${node.operator}`;
    }

    /**
     * Generate function call.
     *
     * TRANSFORMS:
     *   scribe("hello") -> console.log("hello")
     *   _pavimentum(x) -> Math.floor(x)
     *   foo(x, y) -> foo(x, y)
     *
     * Intrinsics are mapped to target-specific implementations.
     */
    function genCallExpression(node: CallExpression): string {
        const args = node.arguments.map(genExpression).join(', ');

        // Check for intrinsics
        if (node.callee.type === 'Identifier') {
            const name = node.callee.name;
            const intrinsic = TS_INTRINSICS[name];

            if (intrinsic) {
                return intrinsic(args);
            }
        }

        const callee = genExpression(node.callee);

        return `${callee}(${args})`;
    }

    /**
     * TypeScript intrinsic mappings.
     *
     * Maps Latin intrinsic names to TypeScript/JavaScript equivalents.
     */
    const TS_INTRINSICS: Record<string, (args: string) => string> = {
        // I/O (internal intrinsics used by norma.fab)
        _scribe: args => `console.log(${args})`,
        _vide: args => `console.debug(${args})`,
        _mone: args => `console.warn(${args})`,
        _lege: () => `prompt() ?? ""`,

        // Math (internal intrinsics used by norma.fab)
        _fortuitus: () => `Math.random()`,
        _pavimentum: args => `Math.floor(${args})`,
        _tectum: args => `Math.ceil(${args})`,
        _radix: args => `Math.sqrt(${args})`,
        _potentia: args => `Math.pow(${args})`,
    };

    /**
     * Generate member access.
     *
     * TRANSFORMS:
     *   obj.prop -> obj.prop
     *   obj[key] -> obj[key]
     */
    function genMemberExpression(node: MemberExpression): string {
        const obj = genExpression(node.object);

        if (node.computed) {
            return `${obj}[${genExpression(node.property)}]`;
        }

        return `${obj}.${node.property.name}`;
    }

    /**
     * Generate arrow function.
     *
     * TRANSFORMS:
     *   (x) => x + 1 -> (x) => x + 1
     *   (x) => { redde x + 1; } -> (x) => { return x + 1; }
     */
    function genArrowFunction(node: ArrowFunctionExpression): string {
        const params = node.params.map(genParameter).join(', ');

        if (node.body.type === 'BlockStatement') {
            const body = genBlockStatement(node.body);

            return `(${params}) => ${body}`;
        }

        const body = genExpression(node.body as Expression);

        return `(${params}) => ${body}`;
    }

    function genAssignmentExpression(node: AssignmentExpression): string {
        const left = node.left.type === 'Identifier' ? node.left.name : genExpression(node.left);

        return `${left} ${node.operator} ${genExpression(node.right)}`;
    }

    function genNewExpression(node: NewExpression): string {
        const callee = node.callee.name;
        const args: string[] = node.arguments.map(genExpression);

        if (node.withExpression) {
            args.push(genObjectExpression(node.withExpression));
        }

        const argsText = args.join(', ');

        return `new ${callee}(${argsText})`;
    }

    return genProgram(program);
}
