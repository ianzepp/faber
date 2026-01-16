/**
 * TypeScript Code Generator - IncipitStatement (entry point)
 *
 * TRANSFORMS:
 *   incipit { body } -> body (top-level statements)
 *   incipit ergo stmt -> stmt (the ergo statement)
 *   @ cli incipit {} -> CLI argument parser and dispatcher
 *
 * TARGET: TypeScript/JavaScript executes top-level code directly.
 *         No wrapper function needed - just emit the body statements.
 */

import type { IncipitStatement } from '../../../parser/ast';
import type { TsGenerator, CliProgram, CliCommandNode, CliParam, CliSingleCommand, CliOption, CliOperand } from '../generator';

// Type mapping for TypeScript
const TYPE_MAP: Record<string, string> = {
    bivalens: 'boolean',
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
};

/**
 * Generate help text for a command node (shows its children).
 */
function genNodeHelp(
    cli: CliProgram,
    node: CliCommandNode,
    pathPrefix: string,
    ind: string,
    g: TsGenerator
): string {
    const lines: string[] = [];
    const fullCommand = pathPrefix ? `${cli.name} ${pathPrefix}` : cli.name;

    // Header (only at root level)
    if (!pathPrefix) {
        if (cli.version) {
            lines.push(`${ind}console.log("${cli.name} v${cli.version}");`);
        }
        else {
            lines.push(`${ind}console.log("${cli.name}");`);
        }
        if (cli.description) {
            lines.push(`${ind}console.log("${cli.description}");`);
        }
        lines.push(`${ind}console.log("");`);
    }

    lines.push(`${ind}console.log("Usage: ${fullCommand} <command> [options]");`);
    lines.push(`${ind}console.log("");`);
    lines.push(`${ind}console.log("Commands:");`);

    // List children with descriptions
    // WHY: Calculate max width at codegen time for aligned output
    const cmdWidths: number[] = [];
    for (const [name, child] of node.children) {
        const aliasStr = child.alias ? `, ${child.alias}` : '';
        const isGroup = child.children.size > 0 && !child.functionName;
        const suffix = isGroup ? ' ...' : '';
        cmdWidths.push((name + aliasStr + suffix).length);
    }
    const maxCmdWidth = Math.max(12, ...cmdWidths); // Minimum 12 chars

    for (const [name, child] of node.children) {
        const aliasStr = child.alias ? `, ${child.alias}` : '';
        const isGroup = child.children.size > 0 && !child.functionName;
        const suffix = isGroup ? ' ...' : '';
        const cmdPart = `${name}${aliasStr}${suffix}`;

        if (child.description) {
            const padding = ' '.repeat(maxCmdWidth - cmdPart.length + 2);
            lines.push(`${ind}console.log("  ${cmdPart}${padding}${child.description}");`);
        }
        else {
            lines.push(`${ind}console.log("  ${cmdPart}");`);
        }
    }

    lines.push(`${ind}console.log("");`);
    lines.push(`${ind}console.log("Options:");`);
    lines.push(`${ind}console.log("  --help, -h     Show this help message");`);

    // Version only at root
    if (!pathPrefix && cli.version) {
        lines.push(`${ind}console.log("  --version, -v  Show version number");`);
    }

    return lines.join('\n');
}

/**
 * Generate help text for a leaf command (shows its options).
 */
function genLeafCommandHelp(
    cli: CliProgram,
    node: CliCommandNode,
    commandPath: string,
    ind: string
): string {
    const lines: string[] = [];
    const params = node.params ?? [];
    const fullCommand = `${cli.name} ${commandPath}`;

    const positionalParams = params.filter(p => !p.optional);
    const optionalParams = params.filter(p => p.optional);

    // Command description
    if (node.description) {
        lines.push(`${ind}console.log("${node.description}");`);
        lines.push(`${ind}console.log("");`);
    }

    // Usage line
    let usage = `Usage: ${fullCommand}`;
    if (optionalParams.length > 0) {
        usage += ' [options]';
    }
    for (const p of positionalParams) {
        usage += ` <${p.name}>`;
    }
    lines.push(`${ind}console.log("${usage}");`);
    lines.push(`${ind}console.log("");`);

    // Arguments section (positional params)
    if (positionalParams.length > 0) {
        lines.push(`${ind}console.log("Arguments:");`);

        const argWidths = positionalParams.map(p => p.name.length);
        const maxArgWidth = Math.max(12, ...argWidths);

        for (const p of positionalParams) {
            const padding = ' '.repeat(maxArgWidth - p.name.length + 2);
            if (p.description) {
                lines.push(`${ind}console.log("  ${p.name}${padding}${p.description}");`);
            }
            else {
                lines.push(`${ind}console.log("  ${p.name}");`);
            }
        }
        lines.push(`${ind}console.log("");`);
    }

    // Options section
    if (optionalParams.length > 0) {
        lines.push(`${ind}console.log("Options:");`);

        // Calculate max width for alignment
        const optWidths = optionalParams.map(p => {
            const longFlag = p.longFlag ?? p.name;
            if (p.shortFlag) {
                return `-${p.shortFlag}, --${longFlag}`.length;
            }
            return `--${longFlag}`.length;
        });
        const maxOptWidth = Math.max(16, ...optWidths);

        for (const p of optionalParams) {
            const longFlag = p.longFlag ?? p.name;
            let flagPart: string;
            if (p.shortFlag) {
                flagPart = `-${p.shortFlag}, --${longFlag}`;
            }
            else {
                flagPart = `--${longFlag}`;
            }
            const padding = ' '.repeat(maxOptWidth - flagPart.length + 2);

            if (p.description) {
                lines.push(`${ind}console.log("  ${flagPart}${padding}${p.description}");`);
            }
            else {
                lines.push(`${ind}console.log("  ${flagPart}");`);
            }
        }
        lines.push(`${ind}console.log("");`);
    }

    // Standard help option
    lines.push(`${ind}console.log("  --help, -h      Show this help message");`);

    return lines.join('\n');
}

/**
 * Generate argument parsing and function call for a leaf command.
 */
function genLeafCommand(
    cli: CliProgram,
    node: CliCommandNode,
    commandPath: string,
    argsVar: string,
    startIdx: number,
    ind: string,
    g: TsGenerator
): string {
    const lines: string[] = [];
    const params = node.params ?? [];

    const positionalParams = params.filter(p => !p.optional);
    const optionalParams = params.filter(p => p.optional);

    // Help flag check - must come before argument parsing
    lines.push(`${ind}if (${argsVar}[${startIdx}] === "--help" || ${argsVar}[${startIdx}] === "-h") {`);
    lines.push(genLeafCommandHelp(cli, node, commandPath, ind + g.indent));
    lines.push(`${ind}  process.exit(0);`);
    lines.push(`${ind}}`);

    // Declare variables for parsed args
    for (const p of params) {
        let defaultVal: string;
        if (p.type === 'bivalens') {
            defaultVal = 'false';
        }
        else if (p.defaultValue !== undefined) {
            defaultVal = `"${p.defaultValue}"`;
        }
        else if (p.optional) {
            defaultVal = 'undefined';
        }
        else {
            defaultVal = '""';
        }

        const tsType = p.type === 'bivalens' ? 'boolean' : 'string';
        const nullableType = p.optional && p.type !== 'bivalens' ? ' | undefined' : '';
        lines.push(`${ind}let ${p.name}: ${tsType}${nullableType} = ${defaultVal};`);
    }

    lines.push(`${ind}for (let _i = ${startIdx}; _i < ${argsVar}.length; _i++) {`);
    lines.push(`${ind}  const _arg = ${argsVar}[_i]!;`);

    // Handle optional flags (use longFlag if available, otherwise fall back to name)
    for (const p of optionalParams) {
        const longFlag = `--${p.longFlag ?? p.name}`;
        const shortFlag = p.shortFlag ? `-${p.shortFlag}` : null;
        const flagCheck = shortFlag
            ? `_arg === "${longFlag}" || _arg === "${shortFlag}"`
            : `_arg === "${longFlag}"`;

        if (p.type === 'bivalens') {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = true; continue; }`);
        }
        else {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = ${argsVar}[++_i]; continue; }`);
        }
    }

    // Handle positional args
    if (positionalParams.length > 0) {
        lines.push(`${ind}  if (!_arg.startsWith("-")) {`);
        for (const p of positionalParams) {
            lines.push(`${ind}    if (${p.name} === "") { ${p.name} = _arg; continue; }`);
        }
        lines.push(`${ind}  }`);
    }

    lines.push(`${ind}}`);

    // Validate required args
    for (const p of positionalParams) {
        lines.push(`${ind}if (${p.name} === "") {`);
        lines.push(`${ind}  console.error("Missing required argument: ${p.name}");`);
        lines.push(`${ind}  process.exit(1);`);
        lines.push(`${ind}}`);
    }

    // Call the function (with module prefix if from imported module)
    const argList = params.map(p => p.name).join(', ');
    const funcCall = node.modulePrefix
        ? `${node.modulePrefix}.${node.functionName}`
        : node.functionName;
    lines.push(`${ind}${funcCall}(${argList});`);

    return lines.join('\n');
}

/**
 * Generate dispatcher for a command node (recursive).
 */
function genNodeDispatcher(
    cli: CliProgram,
    node: CliCommandNode,
    pathPrefix: string,
    argsVar: string,
    argIdx: number,
    ind: string,
    g: TsGenerator
): string {
    const lines: string[] = [];
    const cmdVar = `_cmd${argIdx}`;

    lines.push(`${ind}const ${cmdVar} = ${argsVar}[${argIdx}];`);

    // Help flag or no command
    lines.push(`${ind}if (${cmdVar} === "--help" || ${cmdVar} === "-h" || ${cmdVar} === undefined) {`);
    lines.push(genNodeHelp(cli, node, pathPrefix, ind + g.indent, g));
    lines.push(`${ind}  process.exit(0);`);
    lines.push(`${ind}}`);

    // Version flag (only at root)
    if (!pathPrefix && cli.version) {
        lines.push(`${ind}if (${cmdVar} === "--version" || ${cmdVar} === "-v") {`);
        lines.push(`${ind}  console.log("${cli.version}");`);
        lines.push(`${ind}  process.exit(0);`);
        lines.push(`${ind}}`);
    }

    // Command dispatch
    lines.push(`${ind}switch (${cmdVar}) {`);

    for (const [name, child] of node.children) {
        // Case labels (include alias if present)
        if (child.alias) {
            lines.push(`${ind}  case "${name}":`);
            lines.push(`${ind}  case "${child.alias}": {`);
        }
        else {
            lines.push(`${ind}  case "${name}": {`);
        }

        const childPath = pathPrefix ? `${pathPrefix} ${name}` : name;

        if (child.functionName && child.children.size === 0) {
            // Leaf node - call the function
            lines.push(genLeafCommand(cli, child, childPath, argsVar, argIdx + 1, ind + g.indent + g.indent, g));
        }
        else if (child.children.size > 0) {
            // Branch node - recurse
            lines.push(genNodeDispatcher(cli, child, childPath, argsVar, argIdx + 1, ind + g.indent + g.indent, g));
        }

        lines.push(`${ind}    break;`);
        lines.push(`${ind}  }`);
    }

    // Default case
    const errorContext = pathPrefix ? `${cli.name} ${pathPrefix}` : cli.name;
    lines.push(`${ind}  default: {`);
    lines.push(`${ind}    console.error(\`Unknown command: \${${cmdVar}}\`);`);
    lines.push(`${ind}    console.error("Run '${errorContext} --help' for usage.");`);
    lines.push(`${ind}    process.exit(1);`);
    lines.push(`${ind}  }`);
    lines.push(`${ind}}`);

    return lines.join('\n');
}

/**
 * Generate CLI dispatcher code (entry point).
 */
function genCliDispatcher(cli: CliProgram, g: TsGenerator): string {
    const ind = g.ind();
    const lines: string[] = [];

    // WHY: CLI module imports are now hoisted to top-of-file in ts/index.ts
    // to ensure they appear before any non-import statements (valid ESM)

    lines.push(`${ind}const _args = process.argv.slice(2);`);
    lines.push(``);
    lines.push(genNodeDispatcher(cli, cli.root, '', '_args', 0, ind, g));

    return lines.join('\n');
}

// =============================================================================
// SINGLE-COMMAND MODE
// =============================================================================

/**
 * Generate Argumenta interface for single-command CLI.
 */
function genArgumentaInterface(cmd: CliSingleCommand, ind: string): string {
    const lines: string[] = [];
    lines.push(`${ind}interface Argumenta {`);

    for (const opt of cmd.options) {
        const tsType = TYPE_MAP[opt.type] ?? 'string';
        const optional = opt.defaultValue !== undefined || opt.type === 'bivalens' ? '' : '?';
        lines.push(`${ind}  ${opt.internal}${optional}: ${tsType};`);
    }

    for (const op of cmd.operands) {
        const tsType = TYPE_MAP[op.type] ?? 'string';
        const optional = op.defaultValue !== undefined || op.rest ? '' : '?';
        if (op.rest) {
            lines.push(`${ind}  ${op.name}: ${tsType}[];`);
        }
        else {
            lines.push(`${ind}  ${op.name}${optional}: ${tsType};`);
        }
    }

    lines.push(`${ind}}`);
    return lines.join('\n');
}

/**
 * Generate help text for single-command CLI.
 */
function genSingleCommandHelp(cli: CliProgram, cmd: CliSingleCommand, ind: string): string {
    const lines: string[] = [];

    // Header
    if (cli.version) {
        lines.push(`${ind}console.log("${cli.name} v${cli.version}");`);
    }
    else {
        lines.push(`${ind}console.log("${cli.name}");`);
    }
    if (cli.description) {
        lines.push(`${ind}console.log("${cli.description}");`);
    }
    lines.push(`${ind}console.log("");`);

    // Usage line
    let usage = `Usage: ${cli.name}`;
    if (cmd.options.length > 0) {
        usage += ' [options]';
    }
    for (const op of cmd.operands) {
        if (op.rest) {
            usage += ` [${op.name}...]`;
        }
        else if (op.defaultValue !== undefined) {
            usage += ` [${op.name}]`;
        }
        else {
            usage += ` <${op.name}>`;
        }
    }
    lines.push(`${ind}console.log("${usage}");`);
    lines.push(`${ind}console.log("");`);

    // Options section
    if (cmd.options.length > 0) {
        lines.push(`${ind}console.log("Options:");`);

        // Calculate max width for alignment
        // Handle three cases: short-only, long-only, both
        const optWidths = cmd.options.map(opt => {
            if (opt.short && opt.external) {
                return `-${opt.short}, --${opt.external}`.length;
            }
            else if (opt.short) {
                return `-${opt.short}`.length;
            }
            else {
                return `--${opt.external}`.length;
            }
        });
        const maxOptWidth = Math.max(16, ...optWidths);

        for (const opt of cmd.options) {
            // Build flag part: short-only, long-only, or both
            let flagPart: string;
            if (opt.short && opt.external) {
                flagPart = `-${opt.short}, --${opt.external}`;
            }
            else if (opt.short) {
                flagPart = `-${opt.short}`;
            }
            else {
                flagPart = `--${opt.external}`;
            }
            const padding = ' '.repeat(maxOptWidth - flagPart.length + 2);

            if (opt.description) {
                lines.push(`${ind}console.log("  ${flagPart}${padding}${opt.description}");`);
            }
            else {
                lines.push(`${ind}console.log("  ${flagPart}");`);
            }
        }
        lines.push(`${ind}console.log("");`);
    }

    // Operands section
    if (cmd.operands.length > 0) {
        lines.push(`${ind}console.log("Arguments:");`);

        const opWidths = cmd.operands.map(op => op.name.length);
        const maxOpWidth = Math.max(12, ...opWidths);

        for (const op of cmd.operands) {
            const padding = ' '.repeat(maxOpWidth - op.name.length + 2);
            if (op.description) {
                lines.push(`${ind}console.log("  ${op.name}${padding}${op.description}");`);
            }
            else {
                lines.push(`${ind}console.log("  ${op.name}");`);
            }
        }
        lines.push(`${ind}console.log("");`);
    }

    // Standard help/version
    lines.push(`${ind}console.log("  --help, -h     Show this help message");`);
    if (cli.version) {
        lines.push(`${ind}console.log("  --version, -v  Show version number");`);
    }

    return lines.join('\n');
}

/**
 * Generate argument parsing for single-command CLI.
 */
function genSingleCommandParser(cli: CliProgram, cmd: CliSingleCommand, argsVar: string, ind: string, g: TsGenerator): string {
    const lines: string[] = [];

    lines.push(`${ind}const _args = process.argv.slice(2);`);
    lines.push(``);

    // Help flag check
    lines.push(`${ind}if (_args.includes("--help") || _args.includes("-h")) {`);
    lines.push(genSingleCommandHelp(cli, cmd, ind + g.indent));
    lines.push(`${ind}  process.exit(0);`);
    lines.push(`${ind}}`);

    // Version flag check
    if (cli.version) {
        lines.push(`${ind}if (_args.includes("--version") || _args.includes("-v")) {`);
        lines.push(`${ind}  console.log("${cli.version}");`);
        lines.push(`${ind}  process.exit(0);`);
        lines.push(`${ind}}`);
    }

    lines.push(``);

    // Initialize argumenta object
    lines.push(`${ind}const ${argsVar}: Argumenta = {`);

    // Initialize options with defaults
    for (const opt of cmd.options) {
        let defaultVal: string;
        if (opt.type === 'bivalens') {
            defaultVal = 'false';
        }
        else if (opt.defaultValue !== undefined) {
            defaultVal = JSON.stringify(opt.defaultValue);
        }
        else {
            defaultVal = 'undefined as any';
        }
        lines.push(`${ind}  ${opt.internal}: ${defaultVal},`);
    }

    // Initialize operands with defaults
    for (const op of cmd.operands) {
        let defaultVal: string;
        if (op.rest) {
            defaultVal = '[]';
        }
        else if (op.defaultValue !== undefined) {
            defaultVal = JSON.stringify(op.defaultValue);
        }
        else {
            defaultVal = 'undefined as any';
        }
        lines.push(`${ind}  ${op.name}: ${defaultVal},`);
    }

    lines.push(`${ind}};`);
    lines.push(``);

    // Parse arguments
    lines.push(`${ind}let _positionalIdx = 0;`);
    lines.push(`${ind}for (let _i = 0; _i < _args.length; _i++) {`);
    lines.push(`${ind}  const _arg = _args[_i]!;`);

    // Handle option flags
    // Three cases: short-only, long-only, or both
    for (const opt of cmd.options) {
        const hasLong = opt.external && opt.external.length > 0;
        const hasShort = opt.short && opt.short.length > 0;

        let flagCheck: string;
        if (hasLong && hasShort) {
            flagCheck = `_arg === "--${opt.external}" || _arg === "-${opt.short}"`;
        }
        else if (hasShort) {
            flagCheck = `_arg === "-${opt.short}"`;
        }
        else {
            flagCheck = `_arg === "--${opt.external}"`;
        }

        if (opt.type === 'bivalens') {
            lines.push(`${ind}  if (${flagCheck}) { ${argsVar}.${opt.internal} = true; continue; }`);
        }
        else {
            lines.push(`${ind}  if (${flagCheck}) { ${argsVar}.${opt.internal} = _args[++_i]!; continue; }`);
        }
    }

    // Handle positional arguments (non-rest operands first, then rest)
    const nonRestOperands = cmd.operands.filter(op => !op.rest);
    const restOperand = cmd.operands.find(op => op.rest);

    if (nonRestOperands.length > 0 || restOperand) {
        lines.push(`${ind}  if (!_arg.startsWith("-")) {`);

        for (let i = 0; i < nonRestOperands.length; i++) {
            const op = nonRestOperands[i]!;
            lines.push(`${ind}    if (_positionalIdx === ${i}) { ${argsVar}.${op.name} = _arg; _positionalIdx++; continue; }`);
        }

        if (restOperand) {
            lines.push(`${ind}    ${argsVar}.${restOperand.name}.push(_arg);`);
            lines.push(`${ind}    continue;`);
        }

        lines.push(`${ind}  }`);
    }

    lines.push(`${ind}}`);
    lines.push(``);

    // Validate required operands
    for (const op of nonRestOperands) {
        if (op.defaultValue === undefined) {
            lines.push(`${ind}if (${argsVar}.${op.name} === undefined) {`);
            lines.push(`${ind}  console.error("Missing required argument: ${op.name}");`);
            lines.push(`${ind}  process.exit(1);`);
            lines.push(`${ind}}`);
        }
    }

    return lines.join('\n');
}

/**
 * Generate single-command CLI code.
 */
function genSingleCommandCli(cli: CliProgram, cmd: CliSingleCommand, node: IncipitStatement, g: TsGenerator): string {
    const ind = g.ind();
    const lines: string[] = [];

    // Get argumenta binding name (default to 'args' if not specified)
    const argsVar = node.argumentaBinding?.name ?? 'args';

    // Generate Argumenta interface
    lines.push(genArgumentaInterface(cmd, ind));
    lines.push(``);

    // Generate argument parsing
    lines.push(genSingleCommandParser(cli, cmd, argsVar, ind, g));
    lines.push(``);

    // Handle exit code modifier
    if (node.exitusModifier) {
        const code = node.exitusModifier.code;
        if (code.type === 'Literal') {
            // Fixed exit code: just emit body and exit
            if (node.ergoStatement) {
                lines.push(g.genStatement(node.ergoStatement));
            }
            else if (node.body) {
                lines.push(g.genBlockStatementContent(node.body));
            }
            lines.push(`${ind}process.exit(${code.value});`);
        }
        else {
            // Mutable exit code variable
            lines.push(`${ind}let ${code.name} = 0;`);
            if (node.ergoStatement) {
                lines.push(g.genStatement(node.ergoStatement));
            }
            else if (node.body) {
                lines.push(g.genBlockStatementContent(node.body));
            }
            lines.push(`${ind}process.exit(${code.name});`);
        }
    }
    else {
        // No exit modifier - just emit body
        if (node.ergoStatement) {
            lines.push(g.genStatement(node.ergoStatement));
        }
        else if (node.body) {
            lines.push(g.genBlockStatementContent(node.body));
        }
    }

    return lines.join('\n');
}

export function genIncipitStatement(node: IncipitStatement, g: TsGenerator): string {
    // CLI mode: generate argument parser and dispatcher
    if (g.cli) {
        // Single-command mode
        if (g.cli.singleCommand) {
            return genSingleCommandCli(g.cli, g.cli.singleCommand, node, g);
        }
        // Subcommand mode
        return genCliDispatcher(g.cli, g);
    }

    // Handle ergo form: incipit ergo <statement>
    if (node.ergoStatement) {
        return g.genStatement(node.ergoStatement);
    }

    // Just emit the body statements - no wrapper needed for TS
    return g.genBlockStatementContent(node.body!);
}
