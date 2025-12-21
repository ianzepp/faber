export type CodegenTarget = 'ts' | 'zig'

export interface CodegenOptions {
  target?: CodegenTarget
  indent?: string
  semicolons?: boolean  // TS only
}
