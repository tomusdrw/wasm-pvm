(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (type $int_to_int (func (param i32) (result i32)))
  
  (table 2 funcref)
  (elem (i32.const 0) $double $triple)
  
  (func $double (param $x i32) (result i32)
    (i32.mul (local.get $x) (i32.const 2))
  )
  
  (func $triple (param $x i32) (result i32)
    (i32.mul (local.get $x) (i32.const 3))
  )
  
  (func $call_fn (param $idx i32) (param $val i32) (result i32)
    (call_indirect (type $int_to_int) (local.get $val) (local.get $idx))
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $idx i32)
    (local $val i32)
    (local $result i32)
    
    ;; Read idx and val from args
    (local.set $idx (i32.load (local.get $args_ptr)))
    (local.set $val (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
    
    ;; Call the function via table
    (local.set $result (call $call_fn (local.get $idx) (local.get $val)))
    
    ;; Store result
    (i32.store (i32.const 0x30100) (local.get $result))
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
