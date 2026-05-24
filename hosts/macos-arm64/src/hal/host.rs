/// First HAL surface that should migrate toward host-owned syscalls.
///
/// `consolum` is outside-world console I/O, already split into Faber interface
/// metadata under `stdlib/norma/hal/consolum.fab` and temporary Rust support
/// under `crates/norma/hal/consolum.rs`. That makes it a good first migration
/// candidate after the frame router is proven: the compiler can keep the Faber
/// contract while the host owns the terminal effects.
pub const FIRST_HAL_MIGRATION_CANDIDATE: &str = "norma:hal/consolum";

pub const FIRST_HAL_MIGRATION_RATIONALE: &str =
    "Console I/O is host-owned outside-world behavior and should route through \
     frame-shaped syscalls instead of being linked as generated Rust support.";
