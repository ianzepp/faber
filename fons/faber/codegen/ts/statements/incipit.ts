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
import type { TsGenerator, CliProgram, CliCommand } from '../generator';

/**
 * Generate help text for a CLI program.
 */
function genHelpText(cli: CliProgram, ind: string): string {
    const lines: string[] = [];

    // Header
    if (cli.version) {
        lines.push(`console.log("${cli.name} v${cli.version}");`);
    }
    else {
        lines.push(`console.log("${cli.name}");`);
    }

    if (cli.description) {
        lines.push(`console.log("${cli.description}");`);
    }

    lines.push(`console.log("");`);
    lines.push(`console.log("Usage: ${cli.name} <command> [options]");`);
    lines.push(`console.log("");`);
    lines.push(`console.log("Commands:");`);

    // Command list
    for (const cmd of cli.commands) {
        const aliasStr = cmd.alias ? `, ${cmd.alias}` : '';
        lines.push(`console.log("  ${cmd.name}${aliasStr}");`);
    }

    lines.push(`console.log("");`);
    lines.push(`console.log("Options:");`);
    lines.push(`console.log("  --help, -h     Show this help message");`);
    if (cli.version) {
        lines.push(`console.log("  --version, -v  Show version number");`);
    }

    return lines.map(l => ind + l).join('\n');
}

/**
 * Generate argument parsing for a command.
 */
function genCommandCall(cmd: CliCommand, ind: string): string {
    const lines: string[] = [];

    // Positional params: required, no `si` prefix
    // Optional params: have `si` prefix, become flags
    const positionalParams = cmd.params.filter(p => !p.optional);
    const optionalParams = cmd.params.filter(p => p.optional);

    // Declare variables for parsed args
    for (const p of cmd.params) {
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
        const nullableType = p.optional && p.type !== 'bivalens' ? ` | undefined` : '';
        lines.push(`${ind}let ${p.name}: ${tsType}${nullableType} = ${defaultVal};`);
    }

    lines.push(`${ind}let _argIdx = 1;`);
    lines.push(`${ind}while (_argIdx < args.length) {`);
    lines.push(`${ind}  const _arg = args[_argIdx]!;`);

    // Handle optional flags first (so they're consumed before positional handling)
    for (const p of optionalParams) {
        const longFlag = `--${p.name}`;
        const shortFlag = p.shortFlag ? `-${p.shortFlag}` : null;
        const flagCheck = shortFlag
            ? `_arg === "${longFlag}" || _arg === "${shortFlag}"`
            : `_arg === "${longFlag}"`;

        if (p.type === 'bivalens') {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = true; _argIdx++; continue; }`);
        }
        else {
            lines.push(`${ind}  if (${flagCheck}) { ${p.name} = args[++_argIdx]; _argIdx++; continue; }`);
        }
    }

    // Handle positional args (only non-optional params)
    if (positionalParams.length > 0) {
        lines.push(`${ind}  if (!_arg.startsWith("-")) {`);
        for (const p of positionalParams) {
            lines.push(`${ind}    if (${p.name} === "") { ${p.name} = _arg; _argIdx++; continue; }`);
        }
        lines.push(`${ind}  }`);
    }

    lines.push(`${ind}  _argIdx++;`);
    lines.push(`${ind}}`);

    // Validate required args (only positional/non-optional)
    for (const p of positionalParams) {
        lines.push(`${ind}if (${p.name} === "") {`);
        lines.push(`${ind}  console.error("Missing required argument: ${p.name}");`);
        lines.push(`${ind}  process.exit(1);`);
        lines.push(`${ind}}`);
    }

    // Call the function
    const argList = cmd.params.map(p => p.name).join(', ');
    lines.push(`${ind}${cmd.functionName}(${argList});`);

    return lines.join('\n');
}

/**
 * Generate CLI dispatcher code.
 */
function genCliDispatcher(cli: CliProgram, g: TsGenerator): string {
    const ind = g.ind();
    const lines: string[] = [];

    lines.push(`${ind}const args = process.argv.slice(2);`);
    lines.push(`${ind}const command = args[0];`);
    lines.push(``);

    // Help flag
    lines.push(`${ind}if (command === "--help" || command === "-h" || command === undefined) {`);
    lines.push(genHelpText(cli, ind + g.indent));
    lines.push(`${ind}  process.exit(0);`);
    lines.push(`${ind}}`);
    lines.push(``);

    // Version flag
    if (cli.version) {
        lines.push(`${ind}if (command === "--version" || command === "-v") {`);
        lines.push(`${ind}  console.log("${cli.version}");`);
        lines.push(`${ind}  process.exit(0);`);
        lines.push(`${ind}}`);
        lines.push(``);
    }

    // Command dispatch
    lines.push(`${ind}switch (command) {`);
    for (const cmd of cli.commands) {
        const cases = cmd.alias
            ? `case "${cmd.name}":\n${ind}case "${cmd.alias}":`
            : `case "${cmd.name}":`;
        lines.push(`${ind}  ${cases} {`);
        lines.push(genCommandCall(cmd, ind + g.indent + g.indent));
        lines.push(`${ind}    break;`);
        lines.push(`${ind}  }`);
    }
    lines.push(`${ind}  default: {`);
    lines.push(`${ind}    console.error(\`Unknown command: \${command}\`);`);
    lines.push(`${ind}    console.error("Run '${cli.name} --help' for usage.");`);
    lines.push(`${ind}    process.exit(1);`);
    lines.push(`${ind}  }`);
    lines.push(`${ind}}`);

    return lines.join('\n');
}

export function genIncipitStatement(node: IncipitStatement, g: TsGenerator): string {
    // CLI mode: generate argument parser and dispatcher
    if (g.cli) {
        return genCliDispatcher(g.cli, g);
    }

    // Handle ergo form: incipit ergo <statement>
    if (node.ergoStatement) {
        return g.genStatement(node.ergoStatement);
    }

    // Just emit the body statements - no wrapper needed for TS
    return g.genBlockStatementContent(node.body!);
}
