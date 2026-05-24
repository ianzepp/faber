(component
  (import "capability-call"
    (func $capability-call
      (param "route-code" s32)
      (result s32)))
  (core module $m
    (import "" "capability-call" (func $capability-call (param i32) (result i32)))
    (func (export "route") (param i32) (result i32)
      local.get 0
      call $capability-call)
  )
  (core func $capability-call-lowered (canon lower (func $capability-call)))
  (core instance $i (instantiate $m
    (with "" (instance
      (export "capability-call" (func $capability-call-lowered)))))
  )
  (func (export "route")
    (param "route-code" s32)
    (result s32)
    (canon lift (core func $i "route")))
)
