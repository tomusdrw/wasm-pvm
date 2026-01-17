(module
  ;; Add two i32 arguments passed via SPI args
  ;; Args: 8 bytes (two little-endian U32 values)
  ;; Returns: 4 bytes (one little-endian U32 value) via r7/r8
  (func (export "add") (param i32 i32) (result i32)
    local.get 0
    local.get 1
    i32.add
  )
)
