(module
 (type $0 (func))
 (type $1 (func (param i32 i32)))
 (type $2 (func (param i32 i32) (result i32)))
 (type $3 (func (param i32) (result i32)))
 (type $4 (func (param i32)))
 (global $assembly/control-flow/result_ptr (mut i32) (i32.const 0))
 (global $assembly/control-flow/result_len (mut i32) (i32.const 0))
 (global $~lib/rt/stub/offset (mut i32) (i32.const 0))
 (global $~lib/rt/__rtti_base i32 (i32.const 1024))
 (memory $0 1)
 (data $0 (i32.const 1024) "\04\00\00\00 \00\00\00 \00\00\00 ")
 (export "result_ptr" (global $assembly/control-flow/result_ptr))
 (export "result_len" (global $assembly/control-flow/result_len))
 (export "main" (func $assembly/control-flow/main))
 (export "__new" (func $~lib/rt/stub/__new))
 (export "__pin" (func $~lib/rt/stub/__pin))
 (export "__unpin" (func $~lib/rt/stub/__unpin))
 (export "__collect" (func $~lib/rt/stub/__collect))
 (export "__rtti_base" (global $~lib/rt/__rtti_base))
 (start $~start)
 (func $assembly/control-flow/main (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  i32.const 1
  i32.const 2
  local.get $0
  i32.load
  local.tee $3
  i32.const 10
  i32.gt_s
  select
  local.set $0
  i32.const 0
  local.set $1
  loop $while-continue|0
   local.get $1
   local.get $3
   i32.lt_s
   if
    local.get $0
    i32.const 1
    i32.add
    local.set $0
    local.get $1
    i32.const 1
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  loop $for-loop|1
   local.get $2
   i32.const 5
   i32.lt_s
   if
    i32.const 0
    local.set $1
    loop $for-loop|2
     local.get $1
     i32.const 5
     i32.lt_s
     local.get $1
     i32.const 2
     i32.le_s
     i32.and
     if
      local.get $0
      i32.const 1
      i32.add
      local.set $0
      local.get $1
      i32.const 1
      i32.add
      local.set $1
      br $for-loop|2
     end
    end
    local.get $2
    i32.const 1
    i32.add
    local.set $2
    br $for-loop|1
   end
  end
  i32.const 196864
  local.get $0
  i32.store
  i32.const 196864
  global.set $assembly/control-flow/result_ptr
  i32.const 4
  global.set $assembly/control-flow/result_len
 )
 (func $~lib/rt/stub/__new (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $0
  i32.const 1073741804
  i32.gt_u
  if
   unreachable
  end
  local.get $0
  i32.const 16
  i32.add
  local.tee $4
  i32.const 1073741820
  i32.gt_u
  if
   unreachable
  end
  global.get $~lib/rt/stub/offset
  local.set $3
  global.get $~lib/rt/stub/offset
  i32.const 4
  i32.add
  local.tee $2
  local.get $4
  i32.const 19
  i32.add
  i32.const -16
  i32.and
  i32.const 4
  i32.sub
  local.tee $4
  i32.add
  local.tee $5
  memory.size
  local.tee $6
  i32.const 16
  i32.shl
  i32.const 15
  i32.add
  i32.const -16
  i32.and
  local.tee $7
  i32.gt_u
  if
   local.get $6
   local.get $5
   local.get $7
   i32.sub
   i32.const 65535
   i32.add
   i32.const -65536
   i32.and
   i32.const 16
   i32.shr_u
   local.tee $7
   local.get $6
   local.get $7
   i32.gt_s
   select
   memory.grow
   i32.const 0
   i32.lt_s
   if
    local.get $7
    memory.grow
    i32.const 0
    i32.lt_s
    if
     unreachable
    end
   end
  end
  local.get $5
  global.set $~lib/rt/stub/offset
  local.get $3
  local.get $4
  i32.store
  local.get $2
  i32.const 4
  i32.sub
  local.tee $3
  i32.const 0
  i32.store offset=4
  local.get $3
  i32.const 0
  i32.store offset=8
  local.get $3
  local.get $1
  i32.store offset=12
  local.get $3
  local.get $0
  i32.store offset=16
  local.get $2
  i32.const 16
  i32.add
 )
 (func $~lib/rt/stub/__pin (param $0 i32) (result i32)
  local.get $0
 )
 (func $~lib/rt/stub/__unpin (param $0 i32)
 )
 (func $~lib/rt/stub/__collect
 )
 (func $~start
  i32.const 1052
  global.set $~lib/rt/stub/offset
 )
)
