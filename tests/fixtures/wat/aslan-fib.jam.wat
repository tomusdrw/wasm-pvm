(module
 (type $0 (func (param i32) (result i32)))
 (type $1 (func (param i32 i32) (result i32)))
 (type $2 (func (param i32)))
 (type $3 (func))
 (type $4 (func (param i32 i32 i32 i32)))
 (type $5 (func (param i32 i32 i32)))
 (type $6 (func (param i32) (result i64)))
 (type $7 (func (param i32 i32 i32 i32 i32) (result i32)))
 (type $8 (func (param i32 i32)))
 (type $9 (func (param i64) (result i32)))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (import "env" "console.log" (func $~lib/bindings/dom/console.log (param i32)))
 (import "imports" "log" (func $assembly/sdk/imports/log (param i32 i32 i32 i32 i32) (result i32)))
 (global $~lib/rt/stub/offset (mut i32) (i32.const 0))
 (global $assembly/sdk/service/logger (mut i32) (i32.const 0))
 (global $~argumentsLength (mut i32) (i32.const 0))
 (global $~lib/rt/__rtti_base i32 (i32.const 4832))
 (memory $0 1)
 (data $0 (i32.const 1036) "\1c")
 (data $0.1 (i32.const 1048) "\02\00\00\00\02\00\00\000")
 (data $1 (i32.const 1068) "\1c")
 (data $1.1 (i32.const 1080) "\02\00\00\00\02\00\00\009")
 (data $2 (i32.const 1100) "\1c")
 (data $2.1 (i32.const 1112) "\02\00\00\00\02\00\00\00a")
 (data $3 (i32.const 1132) "\1c")
 (data $3.1 (i32.const 1144) "\02\00\00\00\02\00\00\00f")
 (data $4 (i32.const 1164) "\1c")
 (data $4.1 (i32.const 1176) "\02\00\00\00\02\00\00\00A")
 (data $5 (i32.const 1196) "\1c")
 (data $5.1 (i32.const 1208) "\02\00\00\00\02\00\00\00F")
 (data $6 (i32.const 1228) "\1c")
 (data $6.1 (i32.const 1240) "\01\00\00\00\08\00\00\00\ff\fe\fc\f8\f0\e0\c0\80")
 (data $7 (i32.const 1260) ",")
 (data $7.1 (i32.const 1272) "\04\00\00\00\10\00\00\00\e0\04\00\00\e0\04\00\00\08\00\00\00\08")
 (data $8 (i32.const 1308) "<")
 (data $8.1 (i32.const 1320) "\02\00\00\00(\00\00\00A\00l\00l\00o\00c\00a\00t\00i\00o\00n\00 \00t\00o\00o\00 \00l\00a\00r\00g\00e")
 (data $9 (i32.const 1372) "<")
 (data $9.1 (i32.const 1384) "\02\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00s\00t\00u\00b\00.\00t\00s")
 (data $10 (i32.const 1436) "\1c")
 (data $10.1 (i32.const 1448) "\02\00\00\00\06\00\00\00f\00i\00b")
 (data $11 (i32.const 1468) ",")
 (data $11.1 (i32.const 1480) "\02\00\00\00\1c\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00l\00e\00n\00g\00t\00h")
 (data $12 (i32.const 1516) "<")
 (data $12.1 (i32.const 1528) "\02\00\00\00 \00\00\00~\00l\00i\00b\00/\00d\00a\00t\00a\00v\00i\00e\00w\00.\00t\00s")
 (data $13 (i32.const 1580) "\8c")
 (data $13.1 (i32.const 1592) "\02\00\00\00p\00\00\00A\00t\00t\00e\00m\00p\00t\00i\00n\00g\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00m\00o\00r\00e\00 \00d\00a\00t\00a\00 \00t\00h\00a\00n\00 \00t\00h\00e\00r\00e\00 \00i\00s\00 \00l\00e\00f\00t\00.\00 \00N\00e\00e\00d\00 ")
 (data $14 (i32.const 1724) ",")
 (data $14.1 (i32.const 1736) "\02\00\00\00\10\00\00\00,\00 \00l\00e\00f\00t\00:\00 ")
 (data $15 (i32.const 1772) "\1c")
 (data $15.1 (i32.const 1784) "\02\00\00\00\02\00\00\00.")
 (data $16 (i32.const 1804) ",\00\00\00\03\00\00\00\00\00\00\00\t\00\00\00\14\00\00\00@\06\00\00\00\00\00\00\d0\06\00\00\00\00\00\00\00\07")
 (data $17 (i32.const 1852) "|")
 (data $17.1 (i32.const 1864) "\02\00\00\00d\00\00\00t\00o\00S\00t\00r\00i\00n\00g\00(\00)\00 \00r\00a\00d\00i\00x\00 \00a\00r\00g\00u\00m\00e\00n\00t\00 \00m\00u\00s\00t\00 \00b\00e\00 \00b\00e\00t\00w\00e\00e\00n\00 \002\00 \00a\00n\00d\00 \003\006")
 (data $18 (i32.const 1980) "<")
 (data $18.1 (i32.const 1992) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00u\00t\00i\00l\00/\00n\00u\00m\00b\00e\00r\00.\00t\00s")
 (data $19 (i32.const 2044) "0\000\000\001\000\002\000\003\000\004\000\005\000\006\000\007\000\008\000\009\001\000\001\001\001\002\001\003\001\004\001\005\001\006\001\007\001\008\001\009\002\000\002\001\002\002\002\003\002\004\002\005\002\006\002\007\002\008\002\009\003\000\003\001\003\002\003\003\003\004\003\005\003\006\003\007\003\008\003\009\004\000\004\001\004\002\004\003\004\004\004\005\004\006\004\007\004\008\004\009\005\000\005\001\005\002\005\003\005\004\005\005\005\006\005\007\005\008\005\009\006\000\006\001\006\002\006\003\006\004\006\005\006\006\006\007\006\008\006\009\007\000\007\001\007\002\007\003\007\004\007\005\007\006\007\007\007\008\007\009\008\000\008\001\008\002\008\003\008\004\008\005\008\006\008\007\008\008\008\009\009\000\009\001\009\002\009\003\009\004\009\005\009\006\009\007\009\008\009\009")
 (data $20 (i32.const 2444) "\1c\04")
 (data $20.1 (i32.const 2456) "\02\00\00\00\00\04\00\000\000\000\001\000\002\000\003\000\004\000\005\000\006\000\007\000\008\000\009\000\00a\000\00b\000\00c\000\00d\000\00e\000\00f\001\000\001\001\001\002\001\003\001\004\001\005\001\006\001\007\001\008\001\009\001\00a\001\00b\001\00c\001\00d\001\00e\001\00f\002\000\002\001\002\002\002\003\002\004\002\005\002\006\002\007\002\008\002\009\002\00a\002\00b\002\00c\002\00d\002\00e\002\00f\003\000\003\001\003\002\003\003\003\004\003\005\003\006\003\007\003\008\003\009\003\00a\003\00b\003\00c\003\00d\003\00e\003\00f\004\000\004\001\004\002\004\003\004\004\004\005\004\006\004\007\004\008\004\009\004\00a\004\00b\004\00c\004\00d\004\00e\004\00f\005\000\005\001\005\002\005\003\005\004\005\005\005\006\005\007\005\008\005\009\005\00a\005\00b\005\00c\005\00d\005\00e\005\00f\006\000\006\001\006\002\006\003\006\004\006\005\006\006\006\007\006\008\006\009\006\00a\006\00b\006\00c\006\00d\006\00e\006\00f\007\000\007\001\007\002\007\003\007\004\007\005\007\006\007\007\007\008\007\009\007\00a\007\00b\007\00c\007\00d\007\00e\007\00f\008\000\008\001\008\002\008\003\008\004\008\005\008\006\008\007\008\008\008\009\008\00a\008\00b\008\00c\008\00d\008\00e\008\00f\009\000\009\001\009\002\009\003\009\004\009\005\009\006\009\007\009\008\009\009\009\00a\009\00b\009\00c\009\00d\009\00e\009\00f\00a\000\00a\001\00a\002\00a\003\00a\004\00a\005\00a\006\00a\007\00a\008\00a\009\00a\00a\00a\00b\00a\00c\00a\00d\00a\00e\00a\00f\00b\000\00b\001\00b\002\00b\003\00b\004\00b\005\00b\006\00b\007\00b\008\00b\009\00b\00a\00b\00b\00b\00c\00b\00d\00b\00e\00b\00f\00c\000\00c\001\00c\002\00c\003\00c\004\00c\005\00c\006\00c\007\00c\008\00c\009\00c\00a\00c\00b\00c\00c\00c\00d\00c\00e\00c\00f\00d\000\00d\001\00d\002\00d\003\00d\004\00d\005\00d\006\00d\007\00d\008\00d\009\00d\00a\00d\00b\00d\00c\00d\00d\00d\00e\00d\00f\00e\000\00e\001\00e\002\00e\003\00e\004\00e\005\00e\006\00e\007\00e\008\00e\009\00e\00a\00e\00b\00e\00c\00e\00d\00e\00e\00e\00f\00f\000\00f\001\00f\002\00f\003\00f\004\00f\005\00f\006\00f\007\00f\008\00f\009\00f\00a\00f\00b\00f\00c\00f\00d\00f\00e\00f\00f")
 (data $21 (i32.const 3500) "\\")
 (data $21.1 (i32.const 3512) "\02\00\00\00H\00\00\000\001\002\003\004\005\006\007\008\009\00a\00b\00c\00d\00e\00f\00g\00h\00i\00j\00k\00l\00m\00n\00o\00p\00q\00r\00s\00t\00u\00v\00w\00x\00y\00z")
 (data $22 (i32.const 3596) "\1c")
 (data $22.1 (i32.const 3608) "\02")
 (data $23 (i32.const 3628) "<")
 (data $23.1 (i32.const 3640) "\02\00\00\00$\00\00\00I\00n\00d\00e\00x\00 \00o\00u\00t\00 \00o\00f\00 \00r\00a\00n\00g\00e")
 (data $24 (i32.const 3692) "<")
 (data $24.1 (i32.const 3704) "\02\00\00\00$\00\00\00~\00l\00i\00b\00/\00t\00y\00p\00e\00d\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $25 (i32.const 3756) ",")
 (data $25.1 (i32.const 3768) "\02\00\00\00\1a\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $26 (i32.const 3804) "<")
 (data $26.1 (i32.const 3816) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00b\00u\00f\00f\00e\00r\00.\00t\00s")
 (data $27 (i32.const 3868) "\\")
 (data $27.1 (i32.const 3880) "\02\00\00\00@\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00l\00e\00n\00g\00t\00h\00 \00o\00f\00 \00b\00y\00t\00e\00s\003\002\00 \00(\00g\00o\00t\00:\00 ")
 (data $28 (i32.const 3964) "\1c")
 (data $28.1 (i32.const 3976) "\02\00\00\00\02\00\00\00)")
 (data $29 (i32.const 3996) "\1c\00\00\00\03\00\00\00\00\00\00\00\t\00\00\00\0c\00\00\000\0f\00\00\00\00\00\00\90\0f")
 (data $30 (i32.const 4028) "<")
 (data $30.1 (i32.const 4040) "\02\00\00\00,\00\00\00a\00s\00s\00e\00m\00b\00l\00y\00/\00c\00o\00r\00e\00/\00b\00y\00t\00e\00s\00.\00t\00s")
 (data $31 (i32.const 4092) "<")
 (data $31.1 (i32.const 4104) "\02\00\00\00&\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00A\00B\00I\00 \00p\00a\00y\00l\00o\00a\00d")
 (data $32 (i32.const 4156) "<")
 (data $32.1 (i32.const 4168) "\02\00\00\00*\00\00\00a\00s\00s\00e\00m\00b\00l\00y\00/\00s\00d\00k\00/\00i\00n\00d\00e\00x\00.\00t\00s")
 (data $33 (i32.const 4220) "L")
 (data $33.1 (i32.const 4232) "\02\00\00\004\00\00\00F\00i\00b\00o\00n\00a\00c\00c\00i\00 \00S\00e\00r\00v\00i\00c\00e\00 \00R\00e\00f\00i\00n\00e\00,\00 ")
 (data $34 (i32.const 4300) "<")
 (data $34.1 (i32.const 4312) "\02\00\00\00$\00\00\00U\00n\00p\00a\00i\00r\00e\00d\00 \00s\00u\00r\00r\00o\00g\00a\00t\00e")
 (data $35 (i32.const 4364) ",")
 (data $35.1 (i32.const 4376) "\02\00\00\00\1c\00\00\00~\00l\00i\00b\00/\00s\00t\00r\00i\00n\00g\00.\00t\00s")
 (data $36 (i32.const 4412) "L")
 (data $36.1 (i32.const 4424) "\02\00\00\00<\00\00\00F\00i\00b\00o\00n\00a\00c\00c\00i\00 \00S\00e\00r\00v\00i\00c\00e\00 \00A\00c\00c\00u\00m\00u\00l\00a\00t\00e\00,\00 ")
 (data $37 (i32.const 4492) "\1c")
 (data $37.1 (i32.const 4504) "\02\00\00\00\04\00\00\00 \00@")
 (data $38 (i32.const 4524) ",\00\00\00\03\00\00\00\00\00\00\00\t\00\00\00\10\00\00\00P\11\00\00\00\00\00\00\a0\11")
 (data $39 (i32.const 4572) ",")
 (data $39.1 (i32.const 4584) "\02\00\00\00\14\00\00\00f\00i\00b\00o\00n\00a\00c\00c\00i\00(")
 (data $40 (i32.const 4620) "\1c")
 (data $40.1 (i32.const 4632) "\02\00\00\00\08\00\00\00)\00 \00=\00 ")
 (data $41 (i32.const 4652) ",\00\00\00\03\00\00\00\00\00\00\00\t\00\00\00\10\00\00\00\f0\11\00\00\00\00\00\00 \12")
 (data $42 (i32.const 4700) "|")
 (data $42.1 (i32.const 4712) "\02\00\00\00^\00\00\00U\00n\00e\00x\00p\00e\00c\00t\00e\00d\00 \00\'\00n\00u\00l\00l\00\'\00 \00(\00n\00o\00t\00 \00a\00s\00s\00i\00g\00n\00e\00d\00 \00o\00r\00 \00f\00a\00i\00l\00e\00d\00 \00c\00a\00s\00t\00)")
 (data $43 (i32.const 4832) "\r\00\00\00 \00\00\00 \00\00\00 \00\00\00\00\00\00\00B\00\00\00\00\00\00\00A")
 (data $43.1 (i32.const 4872) "\04A")
 (export "refine_ext" (func $assembly/sdk/index/refine_ext))
 (export "accumulate_ext" (func $assembly/sdk/index/accumulate_ext))
 (export "__new" (func $~lib/rt/stub/__new))
 (export "__pin" (func $~lib/rt/stub/__pin))
 (export "__unpin" (func $~lib/rt/stub/__unpin))
 (export "__collect" (func $~lib/rt/stub/__collect))
 (export "__rtti_base" (global $~lib/rt/__rtti_base))
 (export "memory" (memory $0))
 (start $~start)
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
   i32.const 1328
   i32.const 1392
   i32.const 86
   i32.const 30
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 16
  i32.add
  local.tee $4
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1328
   i32.const 1392
   i32.const 33
   i32.const 29
   call $~lib/builtins/abort
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
 (func $assembly/core/codec/Decoder#constructor (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  i32.const 16
  i32.const 7
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  i32.store offset=8
  local.get $1
  i32.const 0
  i32.store offset=12
  local.get $1
  i32.const 0
  i32.store
  local.get $1
  i32.const 0
  i32.store8 offset=4
  local.get $0
  i32.load offset=4
  local.get $0
  i32.load
  local.tee $2
  i32.sub
  local.set $3
  local.get $0
  i32.load offset=8
  local.set $0
  i32.const 12
  i32.const 8
  call $~lib/rt/stub/__new
  local.tee $4
  i32.const 0
  i32.store
  local.get $4
  i32.const 0
  i32.store offset=4
  local.get $4
  i32.const 0
  i32.store offset=8
  local.get $2
  i32.const 20
  i32.sub
  i32.load offset=16
  local.get $0
  local.get $3
  i32.add
  i32.lt_u
  local.get $0
  i32.const 1073741820
  i32.gt_u
  i32.or
  if
   i32.const 1488
   i32.const 1536
   i32.const 25
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $4
  local.get $2
  i32.store
  local.get $4
  local.get $2
  local.get $3
  i32.add
  i32.store offset=4
  local.get $4
  local.get $0
  i32.store offset=8
  local.get $1
  local.get $4
  i32.store
  local.get $1
 )
 (func $~lib/util/number/utoa32_dec_lut (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  loop $while-continue|0
   local.get $1
   i32.const 10000
   i32.ge_u
   if
    local.get $1
    i32.const 10000
    i32.rem_u
    local.set $3
    local.get $1
    i32.const 10000
    i32.div_u
    local.set $1
    local.get $0
    local.get $2
    i32.const 4
    i32.sub
    local.tee $2
    i32.const 1
    i32.shl
    i32.add
    local.get $3
    i32.const 100
    i32.div_u
    i32.const 2
    i32.shl
    i32.const 2044
    i32.add
    i64.load32_u
    local.get $3
    i32.const 100
    i32.rem_u
    i32.const 2
    i32.shl
    i32.const 2044
    i32.add
    i64.load32_u
    i64.const 32
    i64.shl
    i64.or
    i64.store
    br $while-continue|0
   end
  end
  local.get $1
  i32.const 100
  i32.ge_u
  if
   local.get $0
   local.get $2
   i32.const 2
   i32.sub
   local.tee $2
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 100
   i32.rem_u
   i32.const 2
   i32.shl
   i32.const 2044
   i32.add
   i32.load
   i32.store
   local.get $1
   i32.const 100
   i32.div_u
   local.set $1
  end
  local.get $1
  i32.const 10
  i32.ge_u
  if
   local.get $0
   local.get $2
   i32.const 2
   i32.sub
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 2
   i32.shl
   i32.const 2044
   i32.add
   i32.load
   i32.store
  else
   local.get $0
   local.get $2
   i32.const 1
   i32.sub
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 48
   i32.add
   i32.store16
  end
 )
 (func $~lib/util/number/utoa32 (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  local.get $0
  i32.eqz
  if
   i32.const 1056
   return
  end
  local.get $0
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $0
   i32.const 100
   i32.lt_u
   if (result i32)
    local.get $0
    i32.const 10
    i32.ge_u
    i32.const 1
    i32.add
   else
    local.get $0
    i32.const 10000
    i32.ge_u
    i32.const 3
    i32.add
    local.get $0
    i32.const 1000
    i32.ge_u
    i32.add
   end
  else
   local.get $0
   i32.const 10000000
   i32.lt_u
   if (result i32)
    local.get $0
    i32.const 1000000
    i32.ge_u
    i32.const 6
    i32.add
   else
    local.get $0
    i32.const 1000000000
    i32.ge_u
    i32.const 8
    i32.add
    local.get $0
    i32.const 100000000
    i32.ge_u
    i32.add
   end
  end
  local.tee $2
  i32.const 1
  i32.shl
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  local.get $2
  call $~lib/util/number/utoa32_dec_lut
  local.get $1
 )
 (func $~lib/util/number/itoa32 (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.eqz
  if
   i32.const 1056
   return
  end
  i32.const 0
  local.get $0
  i32.sub
  local.get $0
  local.get $0
  i32.const 31
  i32.shr_u
  i32.const 1
  i32.shl
  local.tee $0
  select
  local.tee $3
  i32.const 100000
  i32.lt_u
  if (result i32)
   local.get $3
   i32.const 100
   i32.lt_u
   if (result i32)
    local.get $3
    i32.const 10
    i32.ge_u
    i32.const 1
    i32.add
   else
    local.get $3
    i32.const 10000
    i32.ge_u
    i32.const 3
    i32.add
    local.get $3
    i32.const 1000
    i32.ge_u
    i32.add
   end
  else
   local.get $3
   i32.const 10000000
   i32.lt_u
   if (result i32)
    local.get $3
    i32.const 1000000
    i32.ge_u
    i32.const 6
    i32.add
   else
    local.get $3
    i32.const 1000000000
    i32.ge_u
    i32.const 8
    i32.add
    local.get $3
    i32.const 100000000
    i32.ge_u
    i32.add
   end
  end
  local.tee $2
  i32.const 1
  i32.shl
  local.get $0
  i32.add
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  i32.add
  local.get $3
  local.get $2
  call $~lib/util/number/utoa32_dec_lut
  local.get $0
  if
   local.get $1
   i32.const 45
   i32.store16
  end
  local.get $1
 )
 (func $~lib/staticarray/StaticArray<~lib/string/String>#join (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  i32.const 3616
  local.set $3
  block $__inlined_func$~lib/util/string/joinStringArray$32
   local.get $0
   i32.const 20
   i32.sub
   i32.load offset=16
   i32.const 2
   i32.shr_u
   local.tee $7
   i32.const 1
   i32.sub
   local.tee $5
   i32.const 0
   i32.lt_s
   br_if $__inlined_func$~lib/util/string/joinStringArray$32
   local.get $5
   i32.eqz
   if
    local.get $0
    i32.load
    local.tee $3
    i32.eqz
    if
     i32.const 3616
     local.set $3
    end
    br $__inlined_func$~lib/util/string/joinStringArray$32
   end
   loop $for-loop|0
    local.get $4
    local.get $7
    i32.lt_s
    if
     local.get $0
     local.get $4
     i32.const 2
     i32.shl
     i32.add
     i32.load
     local.tee $3
     if
      local.get $1
      local.get $3
      i32.const 20
      i32.sub
      i32.load offset=16
      i32.const 1
      i32.shr_u
      i32.add
      local.set $1
     end
     local.get $4
     i32.const 1
     i32.add
     local.set $4
     br $for-loop|0
    end
   end
   local.get $1
   local.get $5
   i32.const 3612
   i32.load
   i32.const 1
   i32.shr_u
   local.tee $1
   i32.mul
   i32.add
   i32.const 1
   i32.shl
   i32.const 2
   call $~lib/rt/stub/__new
   local.set $3
   loop $for-loop|1
    local.get $5
    local.get $6
    i32.gt_s
    if
     local.get $0
     local.get $6
     i32.const 2
     i32.shl
     i32.add
     i32.load
     local.tee $4
     if
      local.get $3
      local.get $2
      i32.const 1
      i32.shl
      i32.add
      local.get $4
      local.get $4
      i32.const 20
      i32.sub
      i32.load offset=16
      i32.const 1
      i32.shr_u
      local.tee $4
      i32.const 1
      i32.shl
      memory.copy
      local.get $2
      local.get $4
      i32.add
      local.set $2
     end
     local.get $1
     if
      local.get $3
      local.get $2
      i32.const 1
      i32.shl
      i32.add
      i32.const 3616
      local.get $1
      i32.const 1
      i32.shl
      memory.copy
      local.get $1
      local.get $2
      i32.add
      local.set $2
     end
     local.get $6
     i32.const 1
     i32.add
     local.set $6
     br $for-loop|1
    end
   end
   local.get $0
   local.get $5
   i32.const 2
   i32.shl
   i32.add
   i32.load
   local.tee $0
   if
    local.get $3
    local.get $2
    i32.const 1
    i32.shl
    i32.add
    local.get $0
    local.get $0
    i32.const 20
    i32.sub
    i32.load offset=16
    i32.const -2
    i32.and
    memory.copy
   end
  end
  local.get $3
 )
 (func $assembly/core/codec/Decoder#moveOffset (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  block $__inlined_func$assembly/core/codec/Decoder#hasBytes$162 (result i32)
   local.get $0
   i32.load offset=8
   i32.load offset=8
   local.get $0
   i32.load offset=12
   local.get $1
   i32.add
   i32.lt_u
   if
    local.get $1
    call $~lib/util/number/utoa32
    local.set $3
    local.get $0
    i32.load offset=8
    i32.load offset=8
    local.get $0
    i32.load offset=12
    i32.sub
    call $~lib/util/number/itoa32
    local.set $2
    i32.const 1828
    local.get $3
    i32.store
    i32.const 1836
    local.get $2
    i32.store
    i32.const 1824
    call $~lib/staticarray/StaticArray<~lib/string/String>#join
    call $~lib/bindings/dom/console.log
    i32.const 0
    br $__inlined_func$assembly/core/codec/Decoder#hasBytes$162
   end
   i32.const 1
  end
  if
   local.get $0
   local.get $0
   i32.load offset=12
   local.tee $0
   local.get $1
   i32.add
   i32.store offset=12
   local.get $0
   return
  end
  local.get $0
  i32.const 1
  i32.store8 offset=4
  i32.const -1
 )
 (func $~lib/typedarray/Uint8Array#__get (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  local.get $0
  i32.load offset=8
  i32.ge_u
  if
   i32.const 3648
   i32.const 3712
   i32.const 167
   i32.const 45
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.add
  i32.load8_u
 )
 (func $assembly/core/codec/Decoder#varU64 (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i32)
  (local $3 i64)
  (local $4 i32)
  (local $5 i32)
  (local $6 i64)
  local.get $0
  i32.const 1
  call $assembly/core/codec/Decoder#moveOffset
  local.tee $4
  i32.const -1
  i32.eq
  if
   i64.const 0
   return
  end
  block $__inlined_func$assembly/core/codec/decodeVariableLengthExtraBytes$53 (result i32)
   local.get $0
   i32.load offset=8
   local.get $4
   call $~lib/typedarray/Uint8Array#__get
   local.set $4
   loop $for-loop|0
    local.get $1
    i32.const 1292
    i32.load
    local.tee $5
    i32.const 255
    i32.and
    i32.lt_u
    if
     local.get $1
     local.get $5
     i32.ge_u
     if
      i32.const 3648
      i32.const 3776
      i32.const 114
      i32.const 42
      call $~lib/builtins/abort
      unreachable
     end
     i32.const 8
     local.get $1
     i32.sub
     local.get $1
     i32.const 1284
     i32.load
     i32.add
     i32.load8_u
     local.get $4
     i32.const 255
     i32.and
     i32.le_u
     br_if $__inlined_func$assembly/core/codec/decodeVariableLengthExtraBytes$53
     drop
     local.get $1
     i32.const 1
     i32.add
     local.set $1
     br $for-loop|0
    end
   end
   i32.const 0
  end
  local.tee $1
  i32.eqz
  if
   local.get $4
   i64.extend_i32_u
   return
  end
  local.get $0
  local.get $1
  call $assembly/core/codec/Decoder#moveOffset
  local.set $5
  local.get $1
  i32.const 8
  i32.eq
  if
   local.get $5
   i32.const 31
   i32.shr_u
   local.get $0
   i32.load
   local.tee $0
   i32.load offset=8
   local.get $5
   i32.const 8
   i32.add
   i32.lt_s
   i32.or
   if
    i32.const 3648
    i32.const 1536
    i32.const 159
    i32.const 7
    call $~lib/builtins/abort
    unreachable
   end
   local.get $5
   local.get $0
   i32.load offset=4
   i32.add
   i64.load
   return
  end
  local.get $4
  i64.extend_i32_u
  i64.const 1
  i64.const 8
  local.get $1
  i64.extend_i32_u
  local.tee $6
  i64.sub
  local.tee $3
  i64.shl
  i64.const 0
  local.get $3
  i64.const 64
  i64.lt_u
  select
  i64.add
  i64.const 256
  i64.sub
  local.get $6
  i64.const 3
  i64.shl
  i64.shl
  local.set $3
  loop $for-loop|00
   local.get $1
   local.get $2
   i32.gt_s
   if
    local.get $3
    local.get $0
    i32.load offset=8
    local.get $2
    local.get $5
    i32.add
    call $~lib/typedarray/Uint8Array#__get
    i64.extend_i32_u
    local.get $2
    i64.extend_i32_s
    i64.const 3
    i64.shl
    i64.shl
    i64.or
    local.set $3
    local.get $2
    i32.const 1
    i32.add
    local.set $2
    br $for-loop|00
   end
  end
  local.get $3
 )
 (func $~lib/typedarray/Uint8Array#constructor (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  i32.const 12
  i32.const 6
  call $~lib/rt/stub/__new
  local.tee $1
  i32.eqz
  if
   i32.const 12
   i32.const 3
   call $~lib/rt/stub/__new
   local.set $1
  end
  local.get $1
  i32.const 0
  i32.store
  local.get $1
  i32.const 0
  i32.store offset=4
  local.get $1
  i32.const 0
  i32.store offset=8
  local.get $0
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1488
   i32.const 3824
   i32.const 19
   i32.const 57
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.const 1
  call $~lib/rt/stub/__new
  local.tee $2
  i32.const 0
  local.get $0
  memory.fill
  local.get $1
  local.get $2
  i32.store
  local.get $1
  local.get $2
  i32.store offset=4
  local.get $1
  local.get $0
  i32.store offset=8
  local.get $1
 )
 (func $assembly/core/codec/Decoder#bytesFixLen (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  block $folding-inner0
   local.get $1
   i32.eqz
   if
    i32.const 0
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    br $folding-inner0
   end
   local.get $0
   local.get $1
   call $assembly/core/codec/Decoder#moveOffset
   local.tee $2
   i32.const -1
   i32.eq
   if
    local.get $1
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    br $folding-inner0
   end
   local.get $1
   local.get $2
   i32.add
   local.set $1
   local.get $0
   i32.load offset=8
   local.tee $3
   i32.load offset=8
   local.set $4
   i32.const 12
   i32.const 6
   call $~lib/rt/stub/__new
   local.tee $0
   local.get $3
   i32.load
   i32.store
   local.get $0
   local.get $2
   i32.const 0
   i32.lt_s
   if (result i32)
    local.get $2
    local.get $4
    i32.add
    local.tee $2
    i32.const 0
    local.get $2
    i32.const 0
    i32.gt_s
    select
   else
    local.get $2
    local.get $4
    local.get $2
    local.get $4
    i32.lt_s
    select
   end
   local.tee $2
   local.get $3
   i32.load offset=4
   i32.add
   i32.store offset=4
   local.get $0
   local.get $1
   i32.const 0
   i32.lt_s
   if (result i32)
    local.get $1
    local.get $4
    i32.add
    local.tee $1
    i32.const 0
    local.get $1
    i32.const 0
    i32.gt_s
    select
   else
    local.get $1
    local.get $4
    local.get $1
    local.get $4
    i32.lt_s
    select
   end
   local.tee $1
   local.get $2
   local.get $1
   local.get $2
   i32.gt_s
   select
   local.get $2
   i32.sub
   i32.store offset=8
   i32.const 4
   i32.const 10
   call $~lib/rt/stub/__new
   local.tee $1
   local.get $0
   i32.store
   local.get $1
   return
  end
  i32.const 4
  i32.const 10
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  i32.store
  local.get $1
 )
 (func $assembly/sdk/index/ensureDecodeOk (param $0 i32)
  local.get $0
  i32.load8_u offset=4
  if (result i32)
   i32.const 1
  else
   local.get $0
   i32.load offset=12
   local.get $0
   i32.load offset=8
   i32.load offset=8
   i32.ne
  end
  if
   i32.const 4112
   i32.const 4176
   i32.const 8
   i32.const 5
   call $~lib/builtins/abort
   unreachable
  end
 )
 (func $~lib/string/String.UTF8.encode@varargs (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  block $2of2
   block $outOfRange
    global.get $~argumentsLength
    i32.const 1
    i32.sub
    br_table $2of2 $2of2 $2of2 $outOfRange
   end
   unreachable
  end
  local.get $0
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  local.get $1
  i32.add
  local.set $3
  loop $while-continue|0
   local.get $1
   local.get $3
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $4
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $2
     i32.const 1
     i32.add
    else
     local.get $4
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $2
      i32.const 2
      i32.add
     else
      local.get $4
      i32.const 64512
      i32.and
      i32.const 55296
      i32.eq
      local.get $1
      i32.const 2
      i32.add
      local.get $3
      i32.lt_u
      i32.and
      if
       local.get $1
       i32.load16_u offset=2
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $2
        i32.const 4
        i32.add
        local.set $2
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        br $while-continue|0
       end
      end
      local.get $2
      i32.const 3
      i32.add
     end
    end
    local.set $2
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|0
   end
  end
  local.get $2
  i32.const 1
  call $~lib/rt/stub/__new
  local.set $2
  local.get $0
  local.tee $1
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.get $1
  i32.add
  local.set $4
  local.get $2
  local.set $0
  loop $while-continue|00
   local.get $1
   local.get $4
   i32.lt_u
   if
    local.get $1
    i32.load16_u
    local.tee $3
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $0
     local.get $3
     i32.store8
     local.get $0
     i32.const 1
     i32.add
    else
     local.get $3
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $0
      local.get $3
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      local.get $3
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $0
      i32.const 2
      i32.add
     else
      local.get $3
      i32.const 63488
      i32.and
      i32.const 55296
      i32.eq
      if
       local.get $3
       i32.const 56320
       i32.lt_u
       local.get $1
       i32.const 2
       i32.add
       local.get $4
       i32.lt_u
       i32.and
       if
        local.get $1
        i32.load16_u offset=2
        local.tee $5
        i32.const 64512
        i32.and
        i32.const 56320
        i32.eq
        if
         local.get $0
         local.get $3
         i32.const 1023
         i32.and
         i32.const 10
         i32.shl
         i32.const 65536
         i32.add
         local.get $5
         i32.const 1023
         i32.and
         i32.or
         local.tee $3
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 24
         i32.shl
         local.get $3
         i32.const 6
         i32.shr_u
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 16
         i32.shl
         i32.or
         local.get $3
         i32.const 12
         i32.shr_u
         i32.const 63
         i32.and
         i32.const 128
         i32.or
         i32.const 8
         i32.shl
         i32.or
         local.get $3
         i32.const 18
         i32.shr_u
         i32.const 240
         i32.or
         i32.or
         i32.store
         local.get $0
         i32.const 4
         i32.add
         local.set $0
         local.get $1
         i32.const 4
         i32.add
         local.set $1
         br $while-continue|00
        end
       end
      end
      local.get $0
      local.get $3
      i32.const 12
      i32.shr_u
      i32.const 224
      i32.or
      local.get $3
      i32.const 6
      i32.shr_u
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $0
      local.get $3
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=2
      local.get $0
      i32.const 3
      i32.add
     end
    end
    local.set $0
    local.get $1
    i32.const 2
    i32.add
    local.set $1
    br $while-continue|00
   end
  end
  local.get $2
 )
 (func $assembly/sdk/logger/Logger#log (param $0 i32) (param $1 i32)
  local.get $0
  i32.load
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  call $~lib/string/String.UTF8.encode@varargs
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $1
  call $~lib/string/String.UTF8.encode@varargs
  local.set $1
  i32.const 2
  local.get $0
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  local.get $1
  local.get $1
  i32.const 20
  i32.sub
  i32.load offset=16
  call $assembly/sdk/imports/log
  drop
 )
 (func $assembly/sdk/index/refine_ext (param $0 i32) (result i32)
  (local $1 i64)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $0
  call $assembly/core/codec/Decoder#constructor
  local.tee $0
  call $assembly/core/codec/Decoder#varU64
  drop
  local.get $0
  call $assembly/core/codec/Decoder#varU64
  drop
  local.get $0
  call $assembly/core/codec/Decoder#varU64
  i32.wrap_i64
  local.set $3
  local.get $0
  call $assembly/core/codec/Decoder#varU64
  local.tee $1
  i64.const 4294967295
  i64.gt_u
  if
   local.get $0
   i32.const 1
   i32.store8 offset=4
  end
  local.get $0
  local.get $1
  i32.wrap_i64
  call $assembly/core/codec/Decoder#bytesFixLen
  local.set $2
  local.get $0
  i32.const 32
  call $assembly/core/codec/Decoder#bytesFixLen
  i32.load
  local.set $4
  i32.const 4
  i32.const 11
  call $~lib/rt/stub/__new
  local.set $5
  local.get $4
  i32.load offset=8
  local.tee $6
  i32.const 32
  i32.ne
  if
   i32.const 4020
   local.get $6
   call $~lib/util/number/itoa32
   i32.store
   i32.const 4016
   call $~lib/staticarray/StaticArray<~lib/string/String>#join
   i32.const 4048
   i32.const 70
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $5
  if (result i32)
   local.get $5
  else
   i32.const 4
   i32.const 10
   call $~lib/rt/stub/__new
  end
  local.get $4
  i32.store
  local.get $0
  call $assembly/sdk/index/ensureDecodeOk
  global.get $assembly/sdk/service/logger
  local.set $4
  local.get $3
  call $~lib/util/number/utoa32
  local.set $5
  i32.const 3616
  local.set $0
  i32.const 4236
  i32.load
  i32.const -2
  i32.and
  local.tee $6
  local.get $5
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const -2
  i32.and
  local.tee $7
  i32.add
  local.tee $3
  if
   local.get $3
   i32.const 2
   call $~lib/rt/stub/__new
   local.tee $0
   i32.const 4240
   local.get $6
   memory.copy
   local.get $0
   local.get $6
   i32.add
   local.get $5
   local.get $7
   memory.copy
  end
  local.get $4
  local.get $0
  call $assembly/sdk/logger/Logger#log
  local.get $2
  i32.load
 )
 (func $~lib/util/number/utoa64 (param $0 i64) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  i64.eqz
  if
   i32.const 1056
   return
  end
  local.get $0
  i64.const 4294967295
  i64.le_u
  if
   local.get $0
   i32.wrap_i64
   local.tee $1
   i32.const 100000
   i32.lt_u
   if (result i32)
    local.get $1
    i32.const 100
    i32.lt_u
    if (result i32)
     local.get $1
     i32.const 10
     i32.ge_u
     i32.const 1
     i32.add
    else
     local.get $1
     i32.const 10000
     i32.ge_u
     i32.const 3
     i32.add
     local.get $1
     i32.const 1000
     i32.ge_u
     i32.add
    end
   else
    local.get $1
    i32.const 10000000
    i32.lt_u
    if (result i32)
     local.get $1
     i32.const 1000000
     i32.ge_u
     i32.const 6
     i32.add
    else
     local.get $1
     i32.const 1000000000
     i32.ge_u
     i32.const 8
     i32.add
     local.get $1
     i32.const 100000000
     i32.ge_u
     i32.add
    end
   end
   local.tee $3
   i32.const 1
   i32.shl
   i32.const 2
   call $~lib/rt/stub/__new
   local.tee $2
   local.get $1
   local.get $3
   call $~lib/util/number/utoa32_dec_lut
  else
   local.get $0
   i64.const 1000000000000000
   i64.lt_u
   if (result i32)
    local.get $0
    i64.const 1000000000000
    i64.lt_u
    if (result i32)
     local.get $0
     i64.const 100000000000
     i64.ge_u
     i32.const 10
     i32.add
     local.get $0
     i64.const 10000000000
     i64.ge_u
     i32.add
    else
     local.get $0
     i64.const 100000000000000
     i64.ge_u
     i32.const 13
     i32.add
     local.get $0
     i64.const 10000000000000
     i64.ge_u
     i32.add
    end
   else
    local.get $0
    i64.const 100000000000000000
    i64.lt_u
    if (result i32)
     local.get $0
     i64.const 10000000000000000
     i64.ge_u
     i32.const 16
     i32.add
    else
     local.get $0
     i64.const -8446744073709551616
     i64.ge_u
     i32.const 18
     i32.add
     local.get $0
     i64.const 1000000000000000000
     i64.ge_u
     i32.add
    end
   end
   local.tee $1
   i32.const 1
   i32.shl
   i32.const 2
   call $~lib/rt/stub/__new
   local.set $2
   loop $while-continue|0
    local.get $0
    i64.const 100000000
    i64.ge_u
    if
     local.get $2
     local.get $1
     i32.const 4
     i32.sub
     local.tee $1
     i32.const 1
     i32.shl
     i32.add
     local.get $0
     local.get $0
     i64.const 100000000
     i64.div_u
     local.tee $0
     i64.const 100000000
     i64.mul
     i64.sub
     i32.wrap_i64
     local.tee $3
     i32.const 10000
     i32.rem_u
     local.tee $4
     i32.const 100
     i32.div_u
     i32.const 2
     i32.shl
     i32.const 2044
     i32.add
     i64.load32_u
     local.get $4
     i32.const 100
     i32.rem_u
     i32.const 2
     i32.shl
     i32.const 2044
     i32.add
     i64.load32_u
     i64.const 32
     i64.shl
     i64.or
     i64.store
     local.get $2
     local.get $1
     i32.const 4
     i32.sub
     local.tee $1
     i32.const 1
     i32.shl
     i32.add
     local.get $3
     i32.const 10000
     i32.div_u
     local.tee $3
     i32.const 100
     i32.div_u
     i32.const 2
     i32.shl
     i32.const 2044
     i32.add
     i64.load32_u
     local.get $3
     i32.const 100
     i32.rem_u
     i32.const 2
     i32.shl
     i32.const 2044
     i32.add
     i64.load32_u
     i64.const 32
     i64.shl
     i64.or
     i64.store
     br $while-continue|0
    end
   end
   local.get $2
   local.get $0
   i32.wrap_i64
   local.get $1
   call $~lib/util/number/utoa32_dec_lut
  end
  local.get $2
 )
 (func $assembly/sdk/index/accumulate_ext (param $0 i32) (result i32)
  (local $1 i64)
  (local $2 i64)
  (local $3 i64)
  (local $4 i64)
  (local $5 i32)
  (local $6 i32)
  local.get $0
  call $assembly/core/codec/Decoder#constructor
  local.tee $5
  call $assembly/core/codec/Decoder#varU64
  i32.wrap_i64
  local.set $0
  local.get $5
  call $assembly/core/codec/Decoder#varU64
  i32.wrap_i64
  local.set $6
  local.get $5
  call $assembly/core/codec/Decoder#varU64
  drop
  local.get $5
  call $assembly/sdk/index/ensureDecodeOk
  global.get $assembly/sdk/service/logger
  local.set $5
  local.get $6
  call $~lib/util/number/utoa32
  local.set $6
  local.get $0
  call $~lib/util/number/utoa32
  local.set $0
  i32.const 4548
  local.get $6
  i32.store
  i32.const 4556
  local.get $0
  i32.store
  local.get $5
  i32.const 4544
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $assembly/sdk/logger/Logger#log
  i64.const 1
  local.set $1
  loop $for-loop|0
   local.get $4
   i64.const 10
   i64.lt_u
   if
    local.get $1
    local.get $3
    i64.add
    local.set $2
    local.get $1
    local.set $3
    local.get $2
    local.set $1
    local.get $4
    i64.const 1
    i64.add
    local.set $4
    br $for-loop|0
   end
  end
  global.get $assembly/sdk/service/logger
  local.set $0
  i64.const 10
  call $~lib/util/number/utoa64
  local.set $5
  local.get $3
  call $~lib/util/number/utoa64
  local.set $6
  i32.const 4676
  local.get $5
  i32.store
  i32.const 4684
  local.get $6
  i32.store
  local.get $0
  i32.const 4672
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $assembly/sdk/logger/Logger#log
  i32.const 8
  i32.const 12
  call $~lib/rt/stub/__new
  local.tee $5
  i32.const 0
  i32.store8
  local.get $5
  i32.const 0
  i32.store offset=4
  block $__inlined_func$assembly/sdk/index/encodeOptionalCodeHash$170
   local.get $5
   i32.load8_u
   i32.eqz
   if
    i32.const 1
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    br $__inlined_func$assembly/sdk/index/encodeOptionalCodeHash$170
   end
   i32.const 33
   call $~lib/typedarray/Uint8Array#constructor
   local.tee $0
   i32.load offset=8
   i32.eqz
   if
    i32.const 3648
    i32.const 3712
    i32.const 178
    i32.const 45
    call $~lib/builtins/abort
    unreachable
   end
   local.get $0
   i32.load offset=4
   i32.const 1
   i32.store8
   local.get $5
   i32.load offset=4
   local.tee $5
   i32.eqz
   if
    i32.const 4720
    i32.const 4176
    i32.const 19
    i32.const 11
    call $~lib/builtins/abort
    unreachable
   end
   local.get $0
   i32.load offset=8
   local.get $5
   i32.load
   local.tee $5
   i32.load offset=8
   local.tee $6
   i32.const 1
   i32.add
   i32.lt_s
   if
    i32.const 3648
    i32.const 3712
    i32.const 1902
    i32.const 5
    call $~lib/builtins/abort
    unreachable
   end
   local.get $0
   i32.load offset=4
   i32.const 1
   i32.add
   local.get $5
   i32.load offset=4
   local.get $6
   memory.copy
  end
  local.get $0
 )
 (func $~lib/rt/stub/__pin (param $0 i32) (result i32)
  local.get $0
 )
 (func $~lib/rt/stub/__unpin (param $0 i32)
 )
 (func $~lib/rt/stub/__collect
 )
 (func $~start
  (local $0 i32)
  i32.const 1052
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1056
   i32.load16_u
   drop
  end
  i32.const 1084
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1088
   i32.load16_u
   drop
  end
  i32.const 1116
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1120
   i32.load16_u
   drop
  end
  i32.const 1148
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1152
   i32.load16_u
   drop
  end
  i32.const 1180
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1184
   i32.load16_u
   drop
  end
  i32.const 1212
  i32.load
  i32.const 1
  i32.shr_u
  if
   i32.const 1216
   i32.load16_u
   drop
  end
  i32.const 4892
  global.set $~lib/rt/stub/offset
  i32.const 4
  i32.const 5
  call $~lib/rt/stub/__new
  local.tee $0
  i32.const 0
  i32.store
  local.get $0
  i32.const 1456
  i32.store
  local.get $0
  global.set $assembly/sdk/service/logger
 )
)
