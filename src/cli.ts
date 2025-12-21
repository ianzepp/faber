#!/usr/bin/env bun

import { tokenize } from "./tokenizer"
import { parse } from "./parser"
import { generate, type CodegenTarget } from "./codegen"

const args = process.argv.slice(2)

function printUsage() {
  console.log(`
Faber Romanus - The Roman Craftsman
A Latin programming language

Usage:
  faber <command> [options] <file>

Commands:
  compile <file.la>     Compile .la file to target language
  run <file.la>         Compile and execute (TS target only)
  check <file.la>       Check for errors without compiling

Options:
  -t, --target <lang>    Target language: ts (default), zig
  -o, --output <file>    Output file (default: stdout)
  -h, --help             Show this help
  -v, --version          Show version

Examples:
  faber compile hello.la                    # Compile to TS (stdout)
  faber compile hello.la -o hello.ts        # Compile to TS file
  faber compile hello.la --target zig       # Compile to Zig (stdout)
  faber compile hello.la -t zig -o hello.zig
  faber run hello.la                        # Compile to TS and execute
`)
}

async function compile(inputFile: string, target: CodegenTarget, outputFile?: string): Promise<string> {
  const source = await Bun.file(inputFile).text()

  // Tokenize
  const { tokens, errors: tokenErrors } = tokenize(source)
  if (tokenErrors.length > 0) {
    console.error("Tokenizer errors:")
    for (const err of tokenErrors) {
      console.error(`  ${inputFile}:${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }

  // Parse
  const { program, errors: parseErrors } = parse(tokens)
  if (parseErrors.length > 0) {
    console.error("Parser errors:")
    for (const err of parseErrors) {
      console.error(`  ${inputFile}:${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }

  if (!program) {
    console.error("Failed to parse program")
    process.exit(1)
  }

  // Generate
  const output = generate(program, { target })

  if (outputFile) {
    await Bun.write(outputFile, output)
    console.log(`Compiled: ${inputFile} -> ${outputFile} (${target})`)
  }
  else {
    console.log(output)
  }

  return output
}

async function run(inputFile: string) {
  const ts = await compile(inputFile, "ts")

  // Execute the generated TS (Bun runs TS natively)
  try {
    const fn = new Function(ts)  // Bun can execute TS directly
    fn()
  }
  catch (err) {
    console.error("Runtime error:", err)
    process.exit(1)
  }
}

async function check(inputFile: string) {
  const source = await Bun.file(inputFile).text()

  const { tokens, errors: tokenErrors } = tokenize(source)
  const { program, errors: parseErrors } = parse(tokens)

  const allErrors = [...tokenErrors, ...parseErrors]

  if (allErrors.length === 0) {
    console.log(`${inputFile}: No errors`)
  }
  else {
    console.log(`${inputFile}: ${allErrors.length} error(s)`)
    for (const err of allErrors) {
      console.log(`  ${err.position.line}:${err.position.column} - ${err.message}`)
    }
    process.exit(1)
  }
}

// Main
const command = args[0]

if (!command || command === "-h" || command === "--help") {
  printUsage()
  process.exit(0)
}

if (command === "-v" || command === "--version") {
  console.log("Faber Romanus v0.2.0")
  process.exit(0)
}

const inputFile = args[1]
let outputFile: string | undefined
let target: CodegenTarget = "ts"

// Parse options
for (let i = 2; i < args.length; i++) {
  if (args[i] === "-o" || args[i] === "--output") {
    outputFile = args[++i]
  }
  else if (args[i] === "-t" || args[i] === "--target") {
    const t = args[++i]
    if (t !== "ts" && t !== "zig") {
      console.error(`Error: Unknown target '${t}'. Valid targets: ts, zig`)
      process.exit(1)
    }
    target = t
  }
}

if (!inputFile) {
  console.error("Error: No input file specified")
  printUsage()
  process.exit(1)
}

switch (command) {
  case "compile":
    await compile(inputFile, target, outputFile)
    break
  case "run":
    if (target !== "ts") {
      console.error("Error: 'run' command only works with TS target")
      process.exit(1)
    }
    await run(inputFile)
    break
  case "check":
    await check(inputFile)
    break
  default:
    console.error(`Unknown command: ${command}`)
    printUsage()
    process.exit(1)
}
