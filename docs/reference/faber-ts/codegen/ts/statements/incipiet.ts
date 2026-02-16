/**
 * TypeScript Code Generator - IncipietStatement (async entry point)
 *
 * TRANSFORMS:
 *   incipiet { body } -> (async () => { body })()
 *   incipiet ergo stmt -> (async () => { stmt })()
 *   @ cli incipiet {} -> CLI argument parser with async body
 *
 * TARGET: TypeScript/JavaScript needs an async IIFE for top-level await.
 */

import type { IncipietStatement } from '../../../parser/ast';
import type { TsGenerator, CliProgram, CliSingleCommand } from '../generator';

// Type mapping for TypeScript
const TYPE_MAP: Record<string, string> = {
    bivalens: 'boolean',
    textus: 'string',
    numerus: 'number',
    fractus: 'number',
};

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

    // Handle positional arguments
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
 * Generate single-command CLI code with async wrapper.
 */
function genSingleCommandCliAsync(cli: CliProgram, cmd: CliSingleCommand, node: IncipietStatement, g: TsGenerator): string {
    const ind = g.ind();
    const lines: string[] = [];
    const semi = g.semi ? ';' : '';

    // Get argumenta binding name (default to 'args' if not specified)
    const argsVar = node.argumentaBinding?.name ?? 'args';

    // Generate Argumenta interface (outside the async IIFE)
    lines.push(genArgumentaInterface(cmd, ind));
    lines.push(``);

    // Generate argument parsing (outside the async IIFE, sync operations)
    lines.push(genSingleCommandParser(cli, cmd, argsVar, ind, g));
    lines.push(``);

    // Wrap async body in IIFE
    lines.push(`${ind}(async () => {`);
    g.depth++;

    // Handle exit code modifier
    if (node.exitusModifier) {
        const code = node.exitusModifier.code;
        if (code.type === 'Literal') {
            // Fixed exit code: just emit body and exit
            lines.push(g.genBlockStatementContent(node.body));
            lines.push(`${g.ind()}process.exit(${code.value});`);
        }
        else {
            // Mutable exit code variable
            lines.push(`${g.ind()}let ${code.name} = 0;`);
            lines.push(g.genBlockStatementContent(node.body));
            lines.push(`${g.ind()}process.exit(${code.name});`);
        }
    }
    else {
        lines.push(g.genBlockStatementContent(node.body));
    }

    g.depth--;
    lines.push(`${ind}})()${semi}`);

    return lines.join('\n');
}

export function genIncipietStatement(node: IncipietStatement, g: TsGenerator): string {
    // CLI mode: generate argument parser and async dispatcher
    if (g.cli) {
        // Single-command mode
        if (g.cli.singleCommand) {
            return genSingleCommandCliAsync(g.cli, g.cli.singleCommand, node, g);
        }
        // Subcommand mode - not yet supported for async
        // Fall through to basic async IIFE
    }

    // Basic async IIFE wrapper (non-CLI mode)
    const lines: string[] = [];
    const semi = g.semi ? ';' : '';
    lines.push(`${g.ind()}(async () => {`);
    g.depth++;
    lines.push(g.genBlockStatementContent(node.body));
    g.depth--;
    lines.push(`${g.ind()}})()${semi}`);
    return lines.join('\n');
}
