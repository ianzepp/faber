// Faber Romanus - Zig Standard Library
// Re-exports all collection types for single import
//
// Usage:
//   const faber = @import("faber");
//   var items = faber.Lista(i64).init(alloc);

pub const Lista = @import("lista.zig").Lista;
pub const Tabula = @import("tabula.zig").Tabula;
pub const Copia = @import("copia.zig").Copia;
