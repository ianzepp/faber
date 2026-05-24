(module
  (import "" "capability-call" (func $capability-call (param i32) (result i32)))
  (func (export "route") (param i32) (result i32)
    local.get 0
    call $capability-call)
)
