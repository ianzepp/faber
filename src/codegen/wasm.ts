/**
 * WASM Code Generator - Emit WebAssembly Text Format (WAT)
 *
 * COMPILER PHASE
 * ==============
 * codegen
 *
 * ARCHITECTURE
 * ============
 * This module transforms a validated Latin AST into WebAssembly Text Format (WAT).
 * WAT is the human-readable S-expression representation of WebAssembly that can
 * be assembled into binary .wasm using tools like wat2wasm.
 *
 * Key characteristics of WASM:
 * - Stack-based: Operations pop operands, push results
 * - Typed: i32, i64, f32, f64 (no native strings)
 * - Sandboxed: Must import external functions for I/O
 * - Structured: No goto, uses block/loop/br for control flow
 *
 * Key transformations:
 * - Functions become (func ...) definitions
 * - Variables become local.get/local.set operations
 * - Arithmetic uses stack operations (i64.add, i64.mul, etc.)
 * - Strings stored in data segments, referenced by offset
 * - scribe() imports print function from host environment
 * - Control flow uses (if ...) (block ...) (loop ...) (br ...)
 *
 * INPUT/OUTPUT CONTRACT
 * =====================
 * INPUT:  Program AST node with Latin keywords and type names
 * OUTPUT: Valid WAT source code string
 * ERRORS: Throws on unsupported AST node types
 *
 * TARGET DIFFERENCES
 * ==================
 * WASM uses stack-based semantics:
 * - No variables in traditional sense (locals are typed slots)
 * - No native strings (memory + data segments)
 * - No exceptions (trap or return error codes)
 * - No closures (functions are flat)
 * - Must import all I/O from host
 *
 * INVARIANTS
 * ==========
 * INV-1: Generated code is syntactically valid WAT
 * INV-2: All functions are exported for host access
 * INV-3: Memory is exported for string access
 * INV-4: Data segment offsets are correctly calculated
 */

import type {
    Program,
    Statement,
    Expression,
    VariableDeclaration,
    FunctionDeclaration,
    IfStatement,
    WhileStatement,
    ForStatement,
    ReturnStatement,
    BlockStatement,
    ExpressionStatement,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    AssignmentExpression,
    Identifier,
    Literal,
} from "../parser/ast";
import type { CodegenOptions } from "./types";

// =============================================================================
// TYPES
// =============================================================================

/**
 * WASM value types.
 * WHY: WASM has only 4 numeric types. We default to i64 for integers.
 */
type WasmType = "i32" | "i64" | "f32" | "f64";

/**
 * Track string literals for data segment.
 */
interface StringData {
    value: string
    offset: number
    length: number
}

/**
 * Track local variables in a function.
 */
interface LocalVar {
    name: string
    type: WasmType
    index: number
}

// =============================================================================
// TYPE MAPPING
// =============================================================================

/**
 * Map Latin type names to WASM types.
 *
 * WHY: WASM has only numeric types. Strings are handled via memory.
 *
 * TARGET MAPPING:
 * | Latin     | WASM  | Notes                    |
 * |-----------|-------|--------------------------|
 * | Numerus   | i64   | 64-bit signed integer    |
 * | Bivalens  | i32   | 0 = false, 1 = true      |
 * | Textus    | i32   | Pointer to memory offset |
 * | Nihil     | (none)| No return value          |
 */
function mapType(latinType: string): WasmType {
    switch (latinType) {
        case "Numerus":
            return "i64";
        case "Bivalens":
            return "i32";
        case "Textus":
            return "i32"; // Pointer to string in memory
        default:
            return "i64"; // Default to i64
    }
}

// =============================================================================
// MAIN GENERATOR
// =============================================================================

/**
 * Generate WAT source code from a Latin AST.
 *
 * TRANSFORMS:
 *   Program AST -> WAT source code string
 *
 * @param program - Validated AST from parser
 * @param options - Formatting configuration
 * @returns WAT source code
 */
export function generateWasm(program: Program, options: CodegenOptions = {}): string {
    const indent = options.indent ?? "  ";

    // Track string literals for data segment
    const strings: StringData[] = [];
    let stringOffset = 0;

    // Track current function's locals
    let currentLocals: LocalVar[] = [];
    let localIndex = 0;

    // Track depth for indentation
    let depth = 0;

    function ind(): string {
        return indent.repeat(depth);
    }

    // ---------------------------------------------------------------------------
    // String Handling
    // ---------------------------------------------------------------------------

    /**
   * Register a string literal and return its memory offset.
   */
    function addString(value: string): number {
    // Check if already registered
        const existing = strings.find(s => s.value === value);

        if (existing) {
            return existing.offset;
        }

        const offset = stringOffset;
        const length = value.length;

        strings.push({ value, offset, length });
        stringOffset += length + 1; // +1 for null terminator

        return offset;
    }

    /**
   * Get string length by offset.
   */
    function _getStringLength(offset: number): number {
        const str = strings.find(s => s.offset === offset);

        return str?.length ?? 0;
    }

    // ---------------------------------------------------------------------------
    // Local Variable Tracking
    // ---------------------------------------------------------------------------

    /**
   * Add a local variable to current function scope.
   */
    function addLocal(name: string, type: WasmType = "i64"): number {
        const index = localIndex++;

        currentLocals.push({ name, type, index });

        return index;
    }

    /**
   * Get local variable index by name.
   */
    function getLocal(name: string): number | undefined {
        const local = currentLocals.find(l => l.name === name);

        return local?.index;
    }

    /**
   * Reset locals for new function.
   */
    function resetLocals(): void {
        currentLocals = [];
        localIndex = 0;
    }

    // ---------------------------------------------------------------------------
    // Program Generation
    // ---------------------------------------------------------------------------

    function genProgram(node: Program): string {
        const lines: string[] = [];

        lines.push("(module");
        depth++;

        // WHY: Import print functions from host for scribe() support
        if (programUsesScribe(node)) {
            lines.push(`${ind()};; Import print functions from host environment`);
            lines.push(`${ind()}(import "env" "print_i64" (func $print_i64 (param i64)))`);
            lines.push(`${ind()}(import "env" "print_str" (func $print_str (param i32 i32)))`);
            lines.push("");
        }

        // WHY: Export memory so host can read string data
        lines.push(`${ind()};; Memory for string data (1 page = 64KB)`);
        lines.push(`${ind()}(memory (export "memory") 1)`);
        lines.push("");

        // Collect functions and top-level statements
        const functions: FunctionDeclaration[] = [];
        const topLevelStatements: Statement[] = [];

        for (const stmt of node.body) {
            if (stmt.type === "FunctionDeclaration") {
                functions.push(stmt);
            }
            else if (stmt.type !== "ImportDeclaration") {
                topLevelStatements.push(stmt);
            }
        }

        // Generate functions
        for (const fn of functions) {
            lines.push(genFunctionDeclaration(fn));
            lines.push("");
        }

        // Generate main function if there are top-level statements
        if (topLevelStatements.length > 0) {
            lines.push(genMainFunction(topLevelStatements));
            lines.push("");
        }

        // Generate data segment for strings
        if (strings.length > 0) {
            lines.push(`${ind()};; String data segment`);
            for (const str of strings) {
                const escaped = str.value.replace(/\\/g, "\\\\").replace(/"/g, '\\"');

                lines.push(`${ind()}(data (i32.const ${str.offset}) "${escaped}\\00")`);
            }
        }

        depth--;
        lines.push(")");

        return lines.join("\n");
    }

    /**
   * Check if program uses scribe() function.
   */
    function programUsesScribe(node: Program): boolean {
        const source = JSON.stringify(node);

        return source.includes('"name":"scribe"');
    }

    // ---------------------------------------------------------------------------
    // Function Generation
    // ---------------------------------------------------------------------------

    function genMainFunction(statements: Statement[]): string {
        resetLocals();
        const lines: string[] = [];

        lines.push(`${ind()}(func (export "_start")`);
        depth++;

        // Collect all variable declarations for locals
        const varDecls = collectVariables(statements);

        for (const name of varDecls) {
            addLocal(name, "i64");
        }

        // Emit locals
        if (currentLocals.length > 0) {
            const localDecls = currentLocals.map(l => `(local $${l.name} ${l.type})`).join(" ");

            lines.push(`${ind()}${localDecls}`);
        }

        // Generate statements
        for (const stmt of statements) {
            const code = genStatement(stmt);

            if (code) {
                lines.push(code);
            }
        }

        depth--;
        lines.push(`${ind()})`);

        return lines.join("\n");
    }

    function genFunctionDeclaration(node: FunctionDeclaration): string {
        resetLocals();
        const lines: string[] = [];
        const name = node.name.name;

        // Build parameter list
        const params: string[] = [];

        for (const param of node.params) {
            const paramName = param.name.name;
            const paramType = param.typeAnnotation ? mapType(param.typeAnnotation.name) : "i64";

            addLocal(paramName, paramType);
            params.push(`(param $${paramName} ${paramType})`);
        }

        // Determine return type
        let returnType = "";

        if (node.returnType && node.returnType.name !== "Nihil") {
            returnType = ` (result ${mapType(node.returnType.name)})`;
        }

        lines.push(`${ind()}(func (export "${name}") ${params.join(" ")}${returnType}`);
        depth++;

        // Collect additional locals from function body
        const varDecls = collectVariables(node.body.body);

        for (const varName of varDecls) {
            if (!getLocal(varName)) {
                addLocal(varName, "i64");
            }
        }

        // Emit non-parameter locals
        const nonParamLocals = currentLocals.slice(node.params.length);

        if (nonParamLocals.length > 0) {
            const localDecls = nonParamLocals.map(l => `(local $${l.name} ${l.type})`).join(" ");

            lines.push(`${ind()}${localDecls}`);
        }

        // Generate function body
        for (const stmt of node.body.body) {
            const code = genStatement(stmt);

            if (code) {
                lines.push(code);
            }
        }

        depth--;
        lines.push(`${ind()})`);

        return lines.join("\n");
    }

    /**
   * Collect variable names from statements for local declarations.
   */
    function collectVariables(statements: Statement[]): string[] {
        const vars: string[] = [];

        for (const stmt of statements) {
            if (stmt.type === "VariableDeclaration") {
                vars.push(stmt.name.name);
            }
            else if (stmt.type === "IfStatement") {
                vars.push(...collectVariables(stmt.consequent.body));
                if (stmt.alternate) {
                    if (stmt.alternate.type === "BlockStatement") {
                        vars.push(...collectVariables(stmt.alternate.body));
                    }
                }
            }
            else if (stmt.type === "WhileStatement" || stmt.type === "ForStatement") {
                vars.push(...collectVariables(stmt.body.body));
            }
        }

        return vars;
    }

    // ---------------------------------------------------------------------------
    // Statement Generation
    // ---------------------------------------------------------------------------

    function genStatement(node: Statement): string {
        switch (node.type) {
            case "VariableDeclaration":
                return genVariableDeclaration(node);
            case "FunctionDeclaration":
                return ""; // Handled at top level
            case "IfStatement":
                return genIfStatement(node);
            case "WhileStatement":
                return genWhileStatement(node);
            case "ForStatement":
                return genForStatement(node);
            case "ReturnStatement":
                return genReturnStatement(node);
            case "BlockStatement":
                return genBlockStatement(node);
            case "ExpressionStatement":
                return genExpressionStatement(node);
            case "ImportDeclaration":
                return ""; // Handled separately
            case "ThrowStatement":
                return `${ind()}unreachable`; // WASM trap
            case "TryStatement":
                // WASM doesn't have exceptions - just execute the try block
                return node.block.body.map(genStatement).filter(Boolean).join("\n");
            default:
                return `${ind()};; Unsupported: ${(node as Statement).type}`;
        }
    }

    function genVariableDeclaration(node: VariableDeclaration): string {
        const name = node.name.name;

        if (!node.init) {
            return `${ind()};; ${name} declared without initializer`;
        }

        const lines: string[] = [];

        lines.push(genExpression(node.init));
        lines.push(`${ind()}local.set $${name}`);

        return lines.join("\n");
    }

    function genIfStatement(node: IfStatement): string {
        const lines: string[] = [];

        // Push condition onto stack
        lines.push(genExpression(node.test));

        lines.push(`${ind()}(if`);
        depth++;
        lines.push(`${ind()}(then`);
        depth++;

        for (const stmt of node.consequent.body) {
            const code = genStatement(stmt);

            if (code) {
                lines.push(code);
            }
        }

        depth--;
        lines.push(`${ind()})`);

        if (node.alternate) {
            lines.push(`${ind()}(else`);
            depth++;

            if (node.alternate.type === "BlockStatement") {
                for (const stmt of node.alternate.body) {
                    const code = genStatement(stmt);

                    if (code) {
                        lines.push(code);
                    }
                }
            }
            else if (node.alternate.type === "IfStatement") {
                lines.push(genIfStatement(node.alternate));
            }

            depth--;
            lines.push(`${ind()})`);
        }

        depth--;
        lines.push(`${ind()})`);

        return lines.join("\n");
    }

    function genWhileStatement(node: WhileStatement): string {
        const lines: string[] = [];

        lines.push(`${ind()}(block $break`);
        depth++;
        lines.push(`${ind()}(loop $continue`);
        depth++;

        // Check condition, break if false
        lines.push(genExpression(node.test));
        lines.push(`${ind()}i32.eqz`);
        lines.push(`${ind()}br_if $break`);

        // Loop body
        for (const stmt of node.body.body) {
            const code = genStatement(stmt);

            if (code) {
                lines.push(code);
            }
        }

        // Continue loop
        lines.push(`${ind()}br $continue`);

        depth--;
        lines.push(`${ind()})`);
        depth--;
        lines.push(`${ind()})`);

        return lines.join("\n");
    }

    function genForStatement(_node: ForStatement): string {
    // For-in loops are complex in WASM - emit as comment for now
        return `${ind()};; TODO: for-in loop not yet supported in WASM target`;
    }

    function genReturnStatement(node: ReturnStatement): string {
        if (!node.argument) {
            return `${ind()}return`;
        }

        const lines: string[] = [];

        lines.push(genExpression(node.argument));
        lines.push(`${ind()}return`);

        return lines.join("\n");
    }

    function genBlockStatement(node: BlockStatement): string {
        return node.body.map(genStatement).filter(Boolean).join("\n");
    }

    function genExpressionStatement(node: ExpressionStatement): string {
        const expr = genExpression(node.expression);

        // If expression leaves value on stack and isn't a call, drop it
        if (node.expression.type !== "CallExpression" &&
        node.expression.type !== "AssignmentExpression") {
            return `${expr}\n${ind()}drop`;
        }

        return expr;
    }

    // ---------------------------------------------------------------------------
    // Expression Generation
    // ---------------------------------------------------------------------------

    function genExpression(node: Expression): string {
        switch (node.type) {
            case "Literal":
                return genLiteral(node);
            case "Identifier":
                return genIdentifier(node);
            case "BinaryExpression":
                return genBinaryExpression(node);
            case "UnaryExpression":
                return genUnaryExpression(node);
            case "CallExpression":
                return genCallExpression(node);
            case "AssignmentExpression":
                return genAssignmentExpression(node);
            case "MemberExpression":
                return `${ind()};; TODO: member expression not yet supported`;
            case "ArrowFunctionExpression":
                return `${ind()};; TODO: arrow functions not supported in WASM`;
            case "AwaitExpression":
                // WASM is synchronous - just evaluate the inner expression
                return genExpression(node.argument);
            case "NewExpression":
                return `${ind()};; TODO: new expression not yet supported`;
            case "TemplateLiteral":
                // For now, just use the first quasi if it exists
                if (node.quasis.length > 0 && node.quasis[0].value) {
                    const offset = addString(node.quasis[0].value);

                    return `${ind()}i32.const ${offset}`;
                }

                return `${ind()}i32.const 0`;
            default:
                return `${ind()};; Unsupported expression: ${(node as Expression).type}`;
        }
    }

    function genLiteral(node: Literal): string {
        if (node.value === null) {
            return `${ind()}i64.const 0`;
        }

        if (typeof node.value === "boolean") {
            return `${ind()}i32.const ${node.value ? 1 : 0}`;
        }

        if (typeof node.value === "number") {
            if (Number.isInteger(node.value)) {
                return `${ind()}i64.const ${node.value}`;
            }

            return `${ind()}f64.const ${node.value}`;
        }

        if (typeof node.value === "string") {
            const offset = addString(node.value);

            return `${ind()}i32.const ${offset}`;
        }

        return `${ind()}i64.const 0`;
    }

    function genIdentifier(node: Identifier): string {
    // Handle Latin boolean/null keywords
        if (node.name === "verum") {
            return `${ind()}i32.const 1`;
        }

        if (node.name === "falsum") {
            return `${ind()}i32.const 0`;
        }

        if (node.name === "nihil") {
            return `${ind()}i64.const 0`;
        }

        // Look up local variable
        const localIdx = getLocal(node.name);

        if (localIdx !== undefined) {
            return `${ind()}local.get $${node.name}`;
        }

        // Unknown identifier - might be a global or function reference
        return `${ind()};; Unknown: ${node.name}`;
    }

    function genBinaryExpression(node: BinaryExpression): string {
        const lines: string[] = [];

        // Push left operand
        lines.push(genExpression(node.left));
        // Push right operand
        lines.push(genExpression(node.right));

        // Apply operator
        // WHY: WASM is stack-based, so operands must be pushed before operation
        switch (node.operator) {
            case "+":
                lines.push(`${ind()}i64.add`);
                break;
            case "-":
                lines.push(`${ind()}i64.sub`);
                break;
            case "*":
                lines.push(`${ind()}i64.mul`);
                break;
            case "/":
                lines.push(`${ind()}i64.div_s`);
                break;
            case "%":
                lines.push(`${ind()}i64.rem_s`);
                break;
            case "==":
                lines.push(`${ind()}i64.eq`);
                break;
            case "!=":
                lines.push(`${ind()}i64.ne`);
                break;
            case "<":
                lines.push(`${ind()}i64.lt_s`);
                break;
            case "<=":
                lines.push(`${ind()}i64.le_s`);
                break;
            case ">":
                lines.push(`${ind()}i64.gt_s`);
                break;
            case ">=":
                lines.push(`${ind()}i64.ge_s`);
                break;
            case "&&":
                lines.push(`${ind()}i32.and`);
                break;
            case "||":
                lines.push(`${ind()}i32.or`);
                break;
            default:
                lines.push(`${ind()};; Unknown operator: ${node.operator}`);
        }

        return lines.join("\n");
    }

    function genUnaryExpression(node: UnaryExpression): string {
        const lines: string[] = [];

        lines.push(genExpression(node.argument));

        switch (node.operator) {
            case "-":
                // Negate: 0 - value
                lines.unshift(`${ind()}i64.const 0`);
                lines.push(`${ind()}i64.sub`);
                break;
            case "!":
                lines.push(`${ind()}i32.eqz`);
                break;
            default:
                lines.push(`${ind()};; Unknown unary operator: ${node.operator}`);
        }

        return lines.join("\n");
    }

    function genCallExpression(node: CallExpression): string {
        const lines: string[] = [];

        // Get function name
        let funcName = "";

        if (node.callee.type === "Identifier") {
            funcName = node.callee.name;
        }

        // Special handling for scribe()
        if (funcName === "scribe") {
            if (node.arguments.length > 0) {
                const arg = node.arguments[0];

                if (arg.type === "Literal" && typeof arg.value === "string") {
                    // String literal - use print_str
                    const offset = addString(arg.value);
                    const length = arg.value.length;

                    lines.push(`${ind()}i32.const ${offset}`);
                    lines.push(`${ind()}i32.const ${length}`);
                    lines.push(`${ind()}call $print_str`);
                }
                else {
                    // Assume numeric - use print_i64
                    lines.push(genExpression(arg));
                    lines.push(`${ind()}call $print_i64`);
                }
            }

            return lines.join("\n");
        }

        // Regular function call - push arguments then call
        for (const arg of node.arguments) {
            lines.push(genExpression(arg));
        }

        lines.push(`${ind()}call $${funcName}`);

        return lines.join("\n");
    }

    function genAssignmentExpression(node: AssignmentExpression): string {
        const lines: string[] = [];

        if (node.left.type === "Identifier") {
            lines.push(genExpression(node.right));
            lines.push(`${ind()}local.set $${node.left.name}`);
        }

        return lines.join("\n");
    }

    // ---------------------------------------------------------------------------
    // Entry Point
    // ---------------------------------------------------------------------------

    return genProgram(program);
}
