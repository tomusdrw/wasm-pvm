(module
  ;; Add two hardcoded numbers: 5 + 7
  (func (export "add") (result i32)
    i32.const 5
    i32.const 7
    i32.add
  )
)
