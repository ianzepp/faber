#!/usr/bin/env bun

import { tokenize } from "./tokenizer"
import { parse } from "./parser"
import { generate } from "./codegen"

const args = process.argv.slice(2)

function printUsage() {
  console.log(`
Faber Romanus - The Roman Craftsman
A Latin-to-JavaScript transpiler

Usage:
  faber <command> [options] <file>

Commands:
  compile <file.fab>     Compile .fab file to JavaScript
  run <file.fab>         Compile and execute
  check <file.fab>       Check for errors without compiling

Options:
  -o, --output <file>    Output file (default: stdout)
  -h, --help             Show this help
  -v, --version          Show version

Example:
  faber compile hello.fab -o hello.js
  faber run hello.fab
`)
}

async function compile(inputFile: string, outputFile?: string) {
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
  const js = generate(program)

  if (outputFile) {
    await Bun.write(outputFile, js)
    console.log(`Compiled: ${inputFile} -> ${outputFile}`)
  }
  else {
    console.log(js)
  }

  return js
}

async function run(inputFile: string) {
  const js = await compile(inputFile)

  // Execute the generated JS
  try {
    const fn = new Function(js)
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
    console.log(`✓ ${inputFile}: No errors`)
  }
  else {
    console.log(`✗ ${inputFile}: ${allErrors.length} error(s)`)
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
  console.log("Faber Romanus v0.1.0")
  process.exit(0)
}

const inputFile = args[1]
let outputFile: string | undefined

// Parse options
for (let i = 2; i < args.length; i++) {
  if (args[i] === "-o" || args[i] === "--output") {
    outputFile = args[++i]
  }
}

if (!inputFile) {
  console.error("Error: No input file specified")
  printUsage()
  process.exit(1)
}

switch (command) {
  case "compile":
    await compile(inputFile, outputFile)
    break
  case "run":
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
