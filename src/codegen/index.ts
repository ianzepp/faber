import type { Program } from "../parser/ast"
import type { CodegenOptions, CodegenTarget } from "./types"
import { generateTs } from "./ts"
import { generateZig } from "./zig"

export type { CodegenOptions, CodegenTarget } from "./types"
export { generateTs } from "./ts"
export { generateZig } from "./zig"

export function generate(program: Program, options: CodegenOptions = {}): string {
  const target = options.target ?? "ts"

  switch (target) {
    case "ts":
      return generateTs(program, options)
    case "zig":
      return generateZig(program, options)
    default:
      throw new Error(`Unknown codegen target: ${target}`)
  }
}
