(module
  (import "host" "readdir" (func $readdir (param i32 i32)))
  (memory (export "memory") 1)
  (data (i32.const 0) ".")
  (func (export "run")
    i32.const 0
    i32.const 1
    call $readdir
  )
)
