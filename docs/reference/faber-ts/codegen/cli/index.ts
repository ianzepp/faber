/**
 * CLI Code Generation Module
 *
 * Standalone CLI detection and command tree building.
 * Isolated from core compiler - uses parser but not semantic analyzer.
 *
 * @module codegen/cli
 */

export { detectCliProgram, type CliDetectionResult } from './detector';
export {
    loadCliModule,
    resolveModulePath,
    isLocalImport,
    createResolverContext,
    resolveImportedModule,
    type CliModuleInfo,
    type CliFunctionInfo,
    type CliParamInfo,
    type CliIncipitInfo,
    type CliImportInfo,
    type CliResolverContext,
} from './resolver';
