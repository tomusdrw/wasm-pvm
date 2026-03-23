(module
 (type $0 (func (param i32) (result i32)))
 (type $1 (func (param i32 i32)))
 (type $2 (func (param i32) (result i64)))
 (type $3 (func (param i32 i32) (result i32)))
 (type $4 (func (param i32 i32) (result i64)))
 (type $5 (func (param i32 i32 i32)))
 (type $6 (func (param i32 i32 i32 i32) (result i64)))
 (type $7 (func (param i32 i32 i32) (result i64)))
 (type $8 (func (param i32 i32 i32) (result i32)))
 (type $9 (func (param i32 i32 i32 i32 i32 i32) (result i64)))
 (type $10 (func (param i64) (result i32)))
 (type $11 (func (result i64)))
 (type $12 (func (result i32)))
 (type $13 (func (param i32 i64)))
 (type $14 (func (param i32 i32 i32 i32 i32) (result i64)))
 (type $15 (func (param i32 i32 i32 i32)))
 (type $16 (func (param i32 i32 i32 i32 i32) (result i32)))
 (type $17 (func (param i32 i64 i32)))
 (type $18 (func (param i32 i32 i64)))
 (type $19 (func (param i64 i32) (result i64)))
 (type $20 (func (param i64 i32 i32) (result i32)))
 (type $21 (func (param i32 i32 i64 i64 i32 i32) (result i64)))
 (type $22 (func (param i32 i64 i64) (result i64)))
 (type $23 (func (param i32 i64 i64 i32) (result i64)))
 (type $24 (func))
 (type $25 (func (param i32 i32 i32 i32) (result i32)))
 (import "env" "abort" (func $~lib/builtins/abort (param i32 i32 i32 i32)))
 (import "ecalli" "log" (func $~lib/@fluffylabs/as-lan/ecalli/general/log/log (param i32 i32 i32 i32 i32) (result i32)))
 (import "ecalli" "fetch" (func $~lib/@fluffylabs/as-lan/ecalli/general/fetch/fetch (param i32 i32 i32 i32 i32 i32) (result i64)))
 (import "ecalli" "gas" (func $~lib/@fluffylabs/as-lan/ecalli/general/gas/gas (result i64)))
 (import "ecalli" "lookup" (func $~lib/@fluffylabs/as-lan/ecalli/general/lookup/lookup (param i32 i32 i32 i32 i32) (result i64)))
 (import "ecalli" "read" (func $~lib/@fluffylabs/as-lan/ecalli/general/read/read (param i32 i32 i32 i32 i32 i32) (result i64)))
 (import "ecalli" "write" (func $~lib/@fluffylabs/as-lan/ecalli/general/write/write (param i32 i32 i32 i32) (result i64)))
 (import "ecalli" "info" (func $~lib/@fluffylabs/as-lan/ecalli/general/info/info (param i32 i32 i32 i32) (result i64)))
 (import "ecalli" "bless" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/bless/bless (param i32 i32 i32 i32 i32 i32) (result i64)))
 (import "ecalli" "assign" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/assign/assign (param i32 i32 i32) (result i64)))
 (import "ecalli" "designate" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/designate/designate (param i32) (result i64)))
 (import "ecalli" "checkpoint" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/checkpoint/checkpoint (result i64)))
 (import "ecalli" "new_service" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/new_service/new_service (param i32 i32 i64 i64 i32 i32) (result i64)))
 (import "ecalli" "upgrade" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/upgrade/upgrade (param i32 i64 i64) (result i64)))
 (import "ecalli" "transfer" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/transfer/transfer (param i32 i64 i64 i32) (result i64)))
 (import "ecalli" "eject" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/eject/eject (param i32 i32) (result i64)))
 (import "ecalli" "query" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/query/query (param i32 i32 i32) (result i64)))
 (import "ecalli" "solicit" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/solicit/solicit (param i32 i32) (result i64)))
 (import "ecalli" "forget" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/forget/forget (param i32 i32) (result i64)))
 (import "ecalli" "yield_result" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/yield_result/yield_result (param i32) (result i64)))
 (import "ecalli" "provide" (func $~lib/@fluffylabs/as-lan/ecalli/accumulate/provide/provide (param i32 i32 i32) (result i64)))
 (import "ecalli" "historical_lookup" (func $~lib/@fluffylabs/as-lan/ecalli/refine/historical_lookup/historical_lookup (param i32 i32 i32 i32 i32) (result i64)))
 (import "ecalli" "export" (func $~lib/@fluffylabs/as-lan/ecalli/refine/export/export_ (param i32 i32) (result i64)))
 (import "ecalli" "machine" (func $~lib/@fluffylabs/as-lan/ecalli/refine/machine/machine (param i32 i32 i32) (result i64)))
 (import "ecalli" "peek" (func $~lib/@fluffylabs/as-lan/ecalli/refine/peek/peek (param i32 i32 i32 i32) (result i64)))
 (import "ecalli" "poke" (func $~lib/@fluffylabs/as-lan/ecalli/refine/poke/poke (param i32 i32 i32 i32) (result i64)))
 (import "ecalli" "pages" (func $~lib/@fluffylabs/as-lan/ecalli/refine/pages/pages (param i32 i32 i32 i32) (result i64)))
 (import "ecalli" "invoke" (func $~lib/@fluffylabs/as-lan/ecalli/refine/invoke/invoke (param i32 i32 i32) (result i64)))
 (import "ecalli" "expunge" (func $~lib/@fluffylabs/as-lan/ecalli/refine/expunge/expunge (param i32) (result i64)))
 (global $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_0 (mut i32) (i32.const 0))
 (global $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_a (mut i32) (i32.const 0))
 (global $~lib/rt/stub/offset (mut i32) (i32.const 0))
 (global $assembly/dispatch/common/logger (mut i32) (i32.const 0))
 (global $~argumentsLength (mut i32) (i32.const 0))
 (memory $0 1 16)
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
 (data $8 (i32.const 1308) ",")
 (data $8.1 (i32.const 1320) "\02\00\00\00\16\00\00\00e\00c\00a\00l\00l\00i\00-\00t\00e\00s\00t")
 (data $9 (i32.const 1356) "<")
 (data $9.1 (i32.const 1368) "\02\00\00\00(\00\00\00A\00l\00l\00o\00c\00a\00t\00i\00o\00n\00 \00t\00o\00o\00 \00l\00a\00r\00g\00e")
 (data $10 (i32.const 1420) "<")
 (data $10.1 (i32.const 1432) "\02\00\00\00\1e\00\00\00~\00l\00i\00b\00/\00r\00t\00/\00s\00t\00u\00b\00.\00t\00s")
 (data $11 (i32.const 1484) ",")
 (data $11.1 (i32.const 1496) "\02\00\00\00\1c\00\00\00I\00n\00v\00a\00l\00i\00d\00 \00l\00e\00n\00g\00t\00h")
 (data $12 (i32.const 1532) "<")
 (data $12.1 (i32.const 1544) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00b\00u\00f\00f\00e\00r\00.\00t\00s")
 (data $13 (i32.const 1596) "<")
 (data $13.1 (i32.const 1608) "\02\00\00\00 \00\00\00~\00l\00i\00b\00/\00d\00a\00t\00a\00v\00i\00e\00w\00.\00t\00s")
 (data $14 (i32.const 1660) "<")
 (data $14.1 (i32.const 1672) "\02\00\00\00$\00\00\00I\00n\00d\00e\00x\00 \00o\00u\00t\00 \00o\00f\00 \00r\00a\00n\00g\00e")
 (data $15 (i32.const 1724) "<")
 (data $15.1 (i32.const 1736) "\02\00\00\00$\00\00\00~\00l\00i\00b\00/\00t\00y\00p\00e\00d\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $16 (i32.const 1788) ",")
 (data $16.1 (i32.const 1800) "\02\00\00\00\1a\00\00\00~\00l\00i\00b\00/\00a\00r\00r\00a\00y\00.\00t\00s")
 (data $17 (i32.const 1836) "|")
 (data $17.1 (i32.const 1848) "\02\00\00\00d\00\00\00t\00o\00S\00t\00r\00i\00n\00g\00(\00)\00 \00r\00a\00d\00i\00x\00 \00a\00r\00g\00u\00m\00e\00n\00t\00 \00m\00u\00s\00t\00 \00b\00e\00 \00b\00e\00t\00w\00e\00e\00n\00 \002\00 \00a\00n\00d\00 \003\006")
 (data $18 (i32.const 1964) "<")
 (data $18.1 (i32.const 1976) "\02\00\00\00&\00\00\00~\00l\00i\00b\00/\00u\00t\00i\00l\00/\00n\00u\00m\00b\00e\00r\00.\00t\00s")
 (data $19 (i32.const 2028) "\\")
 (data $19.1 (i32.const 2040) "\02\00\00\00H\00\00\000\001\002\003\004\005\006\007\008\009\00a\00b\00c\00d\00e\00f\00g\00h\00i\00j\00k\00l\00m\00n\00o\00p\00q\00r\00s\00t\00u\00v\00w\00x\00y\00z")
 (data $20 (i32.const 2124) "\\")
 (data $20.1 (i32.const 2136) "\02\00\00\00B\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00p\00a\00r\00s\00e\00 \00a\00c\00c\00u\00m\00u\00l\00a\00t\00e\00 \00a\00r\00g\00s\00:\00 ")
 (data $21 (i32.const 2220) "\1c")
 (data $21.1 (i32.const 2232) "\02")
 (data $22 (i32.const 2252) "<")
 (data $22.1 (i32.const 2264) "\02\00\00\00$\00\00\00U\00n\00p\00a\00i\00r\00e\00d\00 \00s\00u\00r\00r\00o\00g\00a\00t\00e")
 (data $23 (i32.const 2316) ",")
 (data $23.1 (i32.const 2328) "\02\00\00\00\1c\00\00\00~\00l\00i\00b\00/\00s\00t\00r\00i\00n\00g\00.\00t\00s")
 (data $24 (i32.const 2364) "<")
 (data $24.1 (i32.const 2376) "\02\00\00\00\"\00\00\00a\00c\00c\00u\00m\00u\00l\00a\00t\00e\00:\00 \00s\00l\00o\00t\00=")
 (data $25 (i32.const 2428) ",")
 (data $25.1 (i32.const 2440) "\02\00\00\00\12\00\00\00 \00s\00e\00r\00v\00i\00c\00e\00=")
 (data $26 (i32.const 2476) ",")
 (data $26.1 (i32.const 2488) "\02\00\00\00\18\00\00\00 \00a\00r\00g\00s\00L\00e\00n\00g\00t\00h\00=")
 (data $27 (i32.const 2524) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\18\00\00\00P\t\00\00\00\00\00\00\90\t\00\00\00\00\00\00\c0\t")
 (data $28 (i32.const 2572) "\1c")
 (data $28.1 (i32.const 2584) "\02\00\00\00\0c\00\00\00f\00e\00t\00c\00h\00(")
 (data $29 (i32.const 2604) "\1c")
 (data $29.1 (i32.const 2616) "\02\00\00\00\04\00\00\00,\00 ")
 (data $30 (i32.const 2636) ",")
 (data $30.1 (i32.const 2648) "\02\00\00\00\16\00\00\00)\00 \00r\00e\00t\00u\00r\00n\00e\00d\00 ")
 (data $31 (i32.const 2684) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\18\00\00\00 \n\00\00\00\00\00\00@\n\00\00\00\00\00\00`\n")
 (data $32 (i32.const 2732) "<")
 (data $32.1 (i32.const 2744) "\02\00\00\00,\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00i\00t\00e\00m\00 ")
 (data $33 (i32.const 2796) "\1c")
 (data $33.1 (i32.const 2808) "\02\00\00\00\08\00\00\00 \00t\00a\00g")
 (data $34 (i32.const 2828) "\1c\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\0c\00\00\00\c0\n\00\00\00\00\00\00\00\0b")
 (data $35 (i32.const 2860) "\\")
 (data $35.1 (i32.const 2872) "\02\00\00\00D\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00o\00p\00e\00r\00a\00n\00d\00 \00a\00t\00 \00i\00n\00d\00e\00x\00 ")
 (data $36 (i32.const 2956) ",")
 (data $36.1 (i32.const 2968) "\02\00\00\00\10\00\00\00o\00p\00e\00r\00a\00n\00d\00[")
 (data $37 (i32.const 3004) ",")
 (data $37.1 (i32.const 3016) "\02\00\00\00\10\00\00\00]\00:\00 \00h\00a\00s\00h\00=")
 (data $38 (i32.const 3052) "\1c")
 (data $38.1 (i32.const 3064) "\02\00\00\00\n\00\00\00 \00g\00a\00s\00=")
 (data $39 (i32.const 3084) ",")
 (data $39.1 (i32.const 3096) "\02\00\00\00\18\00\00\00 \00r\00e\00s\00u\00l\00t\00K\00i\00n\00d\00=")
 (data $40 (i32.const 3132) "<\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00 \00\00\00\a0\0b\00\00\00\00\00\00\d0\0b\00\00\00\00\00\00\00\0c\00\00\00\00\00\00 \0c")
 (data $41 (i32.const 3196) "\1c")
 (data $41.1 (i32.const 3208) "\10\00\00\00\08\00\00\00\01")
 (data $42 (i32.const 3228) "\1c")
 (data $42.1 (i32.const 3240) "\02\00\00\00\04\00\00\000\00x")
 (data $43 (i32.const 3260) "<")
 (data $43.1 (i32.const 3272) "\02\00\00\00,\00\00\00]\00:\00 \00n\00o\00n\00-\00o\00k\00 \00r\00e\00s\00u\00l\00t\00 \00k\00i\00n\00d\00=")
 (data $44 (i32.const 3324) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00\a0\0b\00\00\00\00\00\00\d0\0c")
 (data $45 (i32.const 3372) "\\")
 (data $45.1 (i32.const 3384) "\02\00\00\00H\00\00\00]\00:\00 \00e\00m\00p\00t\00y\00 \00o\00k\00B\00l\00o\00b\00,\00 \00n\00o\00t\00h\00i\00n\00g\00 \00t\00o\00 \00d\00i\00s\00p\00a\00t\00c\00h")
 (data $46 (i32.const 3468) "\1c\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\0c\00\00\00\a0\0b\00\00\00\00\00\00@\r")
 (data $47 (i32.const 3500) "l")
 (data $47.1 (i32.const 3512) "\02\00\00\00X\00\00\00]\00:\00 \00f\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00e\00c\00a\00l\00l\00i\00 \00i\00n\00d\00e\00x\00 \00f\00r\00o\00m\00 \00o\00k\00B\00l\00o\00b")
 (data $48 (i32.const 3612) "\1c\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\0c\00\00\00\a0\0b\00\00\00\00\00\00\c0\r")
 (data $49 (i32.const 3644) "<")
 (data $49.1 (i32.const 3656) "\02\00\00\00&\00\00\00]\00:\00 \00d\00i\00s\00p\00a\00t\00c\00h\00 \00e\00c\00a\00l\00l\00i\00 ")
 (data $50 (i32.const 3708) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00\a0\0b\00\00\00\00\00\00P\0e")
 (data $51 (i32.const 3756) ",")
 (data $51.1 (i32.const 3768) "\02\00\00\00\10\00\00\00g\00a\00s\00(\00)\00 \00=\00 ")
 (data $52 (i32.const 3804) "L")
 (data $52.1 (i32.const 3816) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00f\00e\00t\00c\00h\00 \00p\00a\00r\00a\00m\00s")
 (data $53 (i32.const 3884) ",")
 (data $53.1 (i32.const 3896) "\02\00\00\00\16\00\00\00f\00e\00t\00c\00h\00(\00k\00i\00n\00d\00=")
 (data $54 (i32.const 3932) "\1c")
 (data $54.1 (i32.const 3944) "\02\00\00\00\08\00\00\00)\00 \00=\00 ")
 (data $55 (i32.const 3964) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00@\0f\00\00\00\00\00\00p\0f")
 (data $56 (i32.const 4012) "L")
 (data $56.1 (i32.const 4024) "\02\00\00\00<\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00l\00o\00o\00k\00u\00p\00 \00p\00a\00r\00a\00m\00s")
 (data $57 (i32.const 4092) ",")
 (data $57.1 (i32.const 4104) "\02\00\00\00\16\00\00\00l\00o\00o\00k\00u\00p\00(\00)\00 \00=\00 ")
 (data $58 (i32.const 4140) "L")
 (data $58.1 (i32.const 4152) "\02\00\00\008\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00r\00e\00a\00d\00 \00p\00a\00r\00a\00m\00s")
 (data $59 (i32.const 4220) ",")
 (data $59.1 (i32.const 4232) "\02\00\00\00\12\00\00\00r\00e\00a\00d\00(\00)\00 \00=\00 ")
 (data $60 (i32.const 4268) "L")
 (data $60.1 (i32.const 4280) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00w\00r\00i\00t\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $61 (i32.const 4348) ",")
 (data $61.1 (i32.const 4360) "\02\00\00\00\14\00\00\00w\00r\00i\00t\00e\00(\00)\00 \00=\00 ")
 (data $62 (i32.const 4396) "L")
 (data $62.1 (i32.const 4408) "\02\00\00\008\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00i\00n\00f\00o\00 \00p\00a\00r\00a\00m\00s")
 (data $63 (i32.const 4476) ",")
 (data $63.1 (i32.const 4488) "\02\00\00\00\12\00\00\00i\00n\00f\00o\00(\00)\00 \00=\00 ")
 (data $64 (i32.const 4524) "L")
 (data $64.1 (i32.const 4536) "\02\00\00\006\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00l\00o\00g\00 \00p\00a\00r\00a\00m\00s")
 (data $65 (i32.const 4604) ",")
 (data $65.1 (i32.const 4616) "\02\00\00\00\14\00\00\00l\00o\00g\00(\00l\00e\00v\00e\00l\00=")
 (data $66 (i32.const 4652) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00\10\12\00\00\00\00\00\00p\0f")
 (data $67 (i32.const 4700) "L")
 (data $67.1 (i32.const 4712) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00b\00l\00e\00s\00s\00 \00p\00a\00r\00a\00m\00s")
 (data $68 (i32.const 4780) ",")
 (data $68.1 (i32.const 4792) "\02\00\00\00\14\00\00\00b\00l\00e\00s\00s\00(\00)\00 \00=\00 ")
 (data $69 (i32.const 4828) "L")
 (data $69.1 (i32.const 4840) "\02\00\00\00<\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00a\00s\00s\00i\00g\00n\00 \00p\00a\00r\00a\00m\00s")
 (data $70 (i32.const 4908) ",")
 (data $70.1 (i32.const 4920) "\02\00\00\00\16\00\00\00a\00s\00s\00i\00g\00n\00(\00)\00 \00=\00 ")
 (data $71 (i32.const 4956) "\\")
 (data $71.1 (i32.const 4968) "\02\00\00\00B\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00d\00e\00s\00i\00g\00n\00a\00t\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $72 (i32.const 5052) ",")
 (data $72.1 (i32.const 5064) "\02\00\00\00\1c\00\00\00d\00e\00s\00i\00g\00n\00a\00t\00e\00(\00)\00 \00=\00 ")
 (data $73 (i32.const 5100) "<")
 (data $73.1 (i32.const 5112) "\02\00\00\00\1e\00\00\00c\00h\00e\00c\00k\00p\00o\00i\00n\00t\00(\00)\00 \00=\00 ")
 (data $74 (i32.const 5164) "\\")
 (data $74.1 (i32.const 5176) "\02\00\00\00F\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00n\00e\00w\00_\00s\00e\00r\00v\00i\00c\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $75 (i32.const 5260) "<")
 (data $75.1 (i32.const 5272) "\02\00\00\00 \00\00\00n\00e\00w\00_\00s\00e\00r\00v\00i\00c\00e\00(\00)\00 \00=\00 ")
 (data $76 (i32.const 5324) "\\")
 (data $76.1 (i32.const 5336) "\02\00\00\00>\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00u\00p\00g\00r\00a\00d\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $77 (i32.const 5420) ",")
 (data $77.1 (i32.const 5432) "\02\00\00\00\18\00\00\00u\00p\00g\00r\00a\00d\00e\00(\00)\00 \00=\00 ")
 (data $78 (i32.const 5468) "\\")
 (data $78.1 (i32.const 5480) "\02\00\00\00@\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00t\00r\00a\00n\00s\00f\00e\00r\00 \00p\00a\00r\00a\00m\00s")
 (data $79 (i32.const 5564) ",")
 (data $79.1 (i32.const 5576) "\02\00\00\00\1a\00\00\00t\00r\00a\00n\00s\00f\00e\00r\00(\00)\00 \00=\00 ")
 (data $80 (i32.const 5612) "L")
 (data $80.1 (i32.const 5624) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00e\00j\00e\00c\00t\00 \00p\00a\00r\00a\00m\00s")
 (data $81 (i32.const 5692) ",")
 (data $81.1 (i32.const 5704) "\02\00\00\00\14\00\00\00e\00j\00e\00c\00t\00(\00)\00 \00=\00 ")
 (data $82 (i32.const 5740) "L")
 (data $82.1 (i32.const 5752) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00q\00u\00e\00r\00y\00 \00p\00a\00r\00a\00m\00s")
 (data $83 (i32.const 5820) ",")
 (data $83.1 (i32.const 5832) "\02\00\00\00\14\00\00\00q\00u\00e\00r\00y\00(\00)\00 \00=\00 ")
 (data $84 (i32.const 5868) "\\")
 (data $84.1 (i32.const 5880) "\02\00\00\00>\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00s\00o\00l\00i\00c\00i\00t\00 \00p\00a\00r\00a\00m\00s")
 (data $85 (i32.const 5964) ",")
 (data $85.1 (i32.const 5976) "\02\00\00\00\18\00\00\00s\00o\00l\00i\00c\00i\00t\00(\00)\00 \00=\00 ")
 (data $86 (i32.const 6012) "L")
 (data $86.1 (i32.const 6024) "\02\00\00\00<\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00f\00o\00r\00g\00e\00t\00 \00p\00a\00r\00a\00m\00s")
 (data $87 (i32.const 6092) ",")
 (data $87.1 (i32.const 6104) "\02\00\00\00\16\00\00\00f\00o\00r\00g\00e\00t\00(\00)\00 \00=\00 ")
 (data $88 (i32.const 6140) "\\")
 (data $88.1 (i32.const 6152) "\02\00\00\00H\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00y\00i\00e\00l\00d\00_\00r\00e\00s\00u\00l\00t\00 \00p\00a\00r\00a\00m\00s")
 (data $89 (i32.const 6236) "<")
 (data $89.1 (i32.const 6248) "\02\00\00\00\"\00\00\00y\00i\00e\00l\00d\00_\00r\00e\00s\00u\00l\00t\00(\00)\00 \00=\00 ")
 (data $90 (i32.const 6300) "\\")
 (data $90.1 (i32.const 6312) "\02\00\00\00>\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00p\00r\00o\00v\00i\00d\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $91 (i32.const 6396) ",")
 (data $91.1 (i32.const 6408) "\02\00\00\00\18\00\00\00p\00r\00o\00v\00i\00d\00e\00(\00)\00 \00=\00 ")
 (data $92 (i32.const 6444) "<")
 (data $92.1 (i32.const 6456) "\02\00\00\00$\00\00\00]\00:\00 \00u\00n\00k\00n\00o\00w\00n\00 \00e\00c\00a\00l\00l\00i\00 ")
 (data $93 (i32.const 6508) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00\a0\0b\00\00\00\00\00\00@\19")
 (data $94 (i32.const 6556) "\\")
 (data $94.1 (i32.const 6568) "\02\00\00\00F\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00t\00r\00a\00n\00s\00f\00e\00r\00 \00a\00t\00 \00i\00n\00d\00e\00x\00 ")
 (data $95 (i32.const 6652) ",")
 (data $95.1 (i32.const 6664) "\02\00\00\00\12\00\00\00t\00r\00a\00n\00s\00f\00e\00r\00[")
 (data $96 (i32.const 6700) ",")
 (data $96.1 (i32.const 6712) "\02\00\00\00\14\00\00\00]\00:\00 \00s\00o\00u\00r\00c\00e\00=")
 (data $97 (i32.const 6748) "\1c")
 (data $97.1 (i32.const 6760) "\02\00\00\00\0c\00\00\00 \00d\00e\00s\00t\00=")
 (data $98 (i32.const 6780) ",")
 (data $98.1 (i32.const 6792) "\02\00\00\00\10\00\00\00 \00a\00m\00o\00u\00n\00t\00=")
 (data $99 (i32.const 6828) "<\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00(\00\00\00\10\1a\00\00\00\00\00\00@\1a\00\00\00\00\00\00p\1a\00\00\00\00\00\00\90\1a\00\00\00\00\00\00\00\0c")
 (data $100 (i32.const 6892) "<")
 (data $100.1 (i32.const 6904) "\02\00\00\00$\00\00\00U\00n\00k\00n\00o\00w\00n\00 \00i\00t\00e\00m\00 \00k\00i\00n\00d\00 ")
 (data $101 (i32.const 6956) ",")
 (data $101.1 (i32.const 6968) "\02\00\00\00\14\00\00\00 \00a\00t\00 \00i\00n\00d\00e\00x\00 ")
 (data $102 (i32.const 7004) ",\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00\10\00\00\00\00\1b\00\00\00\00\00\00@\1b")
 (data $103 (i32.const 7052) "L")
 (data $103.1 (i32.const 7064) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00p\00a\00r\00s\00e\00 \00r\00e\00f\00i\00n\00e\00 \00a\00r\00g\00s\00:\00 ")
 (data $104 (i32.const 7132) ",")
 (data $104.1 (i32.const 7144) "\02\00\00\00\1a\00\00\00r\00e\00f\00i\00n\00e\00:\00 \00c\00o\00r\00e\00=")
 (data $105 (i32.const 7180) "\1c")
 (data $105.1 (i32.const 7192) "\02\00\00\00\0c\00\00\00 \00i\00t\00e\00m\00=")
 (data $106 (i32.const 7212) ",")
 (data $106.1 (i32.const 7224) "\02\00\00\00\10\00\00\00 \00w\00p\00H\00a\00s\00h\00=")
 (data $107 (i32.const 7260) "<\00\00\00\03\00\00\00\00\00\00\00\0b\00\00\00 \00\00\00\f0\1b\00\00\00\00\00\00 \1c\00\00\00\00\00\00\90\t\00\00\00\00\00\00@\1c")
 (data $108 (i32.const 7324) "\\")
 (data $108.1 (i32.const 7336) "\02\00\00\00>\00\00\00M\00i\00s\00s\00i\00n\00g\00 \00e\00c\00a\00l\00l\00i\00 \00i\00n\00d\00e\00x\00 \00i\00n\00 \00p\00a\00y\00l\00o\00a\00d")
 (data $109 (i32.const 7420) "<")
 (data $109.1 (i32.const 7432) "\02\00\00\00 \00\00\00d\00i\00s\00p\00a\00t\00c\00h\00 \00e\00c\00a\00l\00l\00i\00 ")
 (data $110 (i32.const 7484) "l")
 (data $110.1 (i32.const 7496) "\02\00\00\00R\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00h\00i\00s\00t\00o\00r\00i\00c\00a\00l\00_\00l\00o\00o\00k\00u\00p\00 \00p\00a\00r\00a\00m\00s")
 (data $111 (i32.const 7596) "<")
 (data $111.1 (i32.const 7608) "\02\00\00\00,\00\00\00h\00i\00s\00t\00o\00r\00i\00c\00a\00l\00_\00l\00o\00o\00k\00u\00p\00(\00)\00 \00=\00 ")
 (data $112 (i32.const 7660) "L")
 (data $112.1 (i32.const 7672) "\02\00\00\00<\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00e\00x\00p\00o\00r\00t\00 \00p\00a\00r\00a\00m\00s")
 (data $113 (i32.const 7740) ",")
 (data $113.1 (i32.const 7752) "\02\00\00\00\16\00\00\00e\00x\00p\00o\00r\00t\00(\00)\00 \00=\00 ")
 (data $114 (i32.const 7788) "\\")
 (data $114.1 (i32.const 7800) "\02\00\00\00>\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00m\00a\00c\00h\00i\00n\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $115 (i32.const 7884) ",")
 (data $115.1 (i32.const 7896) "\02\00\00\00\18\00\00\00m\00a\00c\00h\00i\00n\00e\00(\00)\00 \00=\00 ")
 (data $116 (i32.const 7932) "L")
 (data $116.1 (i32.const 7944) "\02\00\00\008\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00p\00e\00e\00k\00 \00p\00a\00r\00a\00m\00s")
 (data $117 (i32.const 8012) ",")
 (data $117.1 (i32.const 8024) "\02\00\00\00\12\00\00\00p\00e\00e\00k\00(\00)\00 \00=\00 ")
 (data $118 (i32.const 8060) "L")
 (data $118.1 (i32.const 8072) "\02\00\00\008\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00p\00o\00k\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $119 (i32.const 8140) ",")
 (data $119.1 (i32.const 8152) "\02\00\00\00\12\00\00\00p\00o\00k\00e\00(\00)\00 \00=\00 ")
 (data $120 (i32.const 8188) "L")
 (data $120.1 (i32.const 8200) "\02\00\00\00:\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00p\00a\00g\00e\00s\00 \00p\00a\00r\00a\00m\00s")
 (data $121 (i32.const 8268) ",")
 (data $121.1 (i32.const 8280) "\02\00\00\00\14\00\00\00p\00a\00g\00e\00s\00(\00)\00 \00=\00 ")
 (data $122 (i32.const 8316) "L")
 (data $122.1 (i32.const 8328) "\02\00\00\00<\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00i\00n\00v\00o\00k\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $123 (i32.const 8396) ",")
 (data $123.1 (i32.const 8408) "\02\00\00\00\16\00\00\00i\00n\00v\00o\00k\00e\00(\00)\00 \00=\00 ")
 (data $124 (i32.const 8444) "\\")
 (data $124.1 (i32.const 8456) "\02\00\00\00>\00\00\00F\00a\00i\00l\00e\00d\00 \00t\00o\00 \00d\00e\00c\00o\00d\00e\00 \00e\00x\00p\00u\00n\00g\00e\00 \00p\00a\00r\00a\00m\00s")
 (data $125 (i32.const 8540) ",")
 (data $125.1 (i32.const 8552) "\02\00\00\00\18\00\00\00e\00x\00p\00u\00n\00g\00e\00(\00)\00 \00=\00 ")
 (data $126 (i32.const 8588) "<")
 (data $126.1 (i32.const 8600) "\02\00\00\00,\00\00\00U\00n\00k\00n\00o\00w\00n\00 \00e\00c\00a\00l\00l\00i\00 \00i\00n\00d\00e\00x\00:\00 ")
 (table $0 2 2 funcref)
 (elem $0 (i32.const 1) $~lib/@fluffylabs/as-lan/core/bytes/bytesToHexString~anonymous|0)
 (export "accumulate" (func $assembly/accumulate/accumulate))
 (export "refine" (func $assembly/refine/refine))
 (export "memory" (memory $0))
 (start $~start)
 (func $~lib/string/String#get:length (param $0 i32) (result i32)
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
  i32.const 1
  i32.shr_u
 )
 (func $~lib/string/String#charCodeAt (param $0 i32) (result i32)
  local.get $0
  call $~lib/string/String#get:length
  i32.eqz
  if
   i32.const -1
   return
  end
  local.get $0
  i32.load16_u
 )
 (func $~lib/@fluffylabs/as-lan/logger/Logger#set:target (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store
 )
 (func $~lib/rt/common/OBJECT#set:gcInfo (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store offset=4
 )
 (func $~lib/rt/common/OBJECT#set:gcInfo2 (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store offset=8
 )
 (func $~lib/rt/common/OBJECT#set:rtId (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store offset=12
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
   i32.const 1376
   i32.const 1440
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
   i32.const 1376
   i32.const 1440
   i32.const 33
   i32.const 29
   call $~lib/builtins/abort
   unreachable
  end
  global.get $~lib/rt/stub/offset
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
  local.get $4
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $2
  i32.const 4
  i32.sub
  local.tee $3
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $3
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $3
  local.get $1
  call $~lib/rt/common/OBJECT#set:rtId
  local.get $3
  local.get $0
  i32.store offset=16
  local.get $2
  i32.const 16
  i32.add
 )
 (func $~lib/typedarray/Uint8Array#constructor (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  i32.const 12
  i32.const 8
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
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $1
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $1
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $0
  i32.const 1073741820
  i32.gt_u
  if
   i32.const 1504
   i32.const 1552
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
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $1
  local.get $2
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $1
  local.get $0
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $1
 )
 (func $~lib/@fluffylabs/as-lan/core/mem/readFromMemory (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.tee $2
  i32.load offset=4
  local.get $0
  local.get $1
  memory.copy
  local.get $2
 )
 (func $~lib/arraybuffer/ArrayBuffer#get:byteLength (param $0 i32) (result i32)
  local.get $0
  i32.const 20
  i32.sub
  i32.load offset=16
 )
 (func $~lib/dataview/DataView#constructor (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  i32.const 12
  i32.const 10
  call $~lib/rt/stub/__new
  local.tee $3
  i32.const 0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $3
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $3
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $0
  call $~lib/arraybuffer/ArrayBuffer#get:byteLength
  local.get $1
  local.get $2
  i32.add
  i32.lt_u
  local.get $2
  i32.const 1073741820
  i32.gt_u
  i32.or
  if
   i32.const 1504
   i32.const 1616
   i32.const 25
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $3
  local.get $0
  local.get $1
  i32.add
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $3
  local.get $2
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $3
 )
 (func $~lib/arraybuffer/ArrayBufferView#get:byteOffset (param $0 i32) (result i32)
  local.get $0
  i32.load offset=4
  local.get $0
  i32.load
  i32.sub
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#set:_isError (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store8 offset=4
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob (param $0 i32) (result i32)
  (local $1 i32)
  i32.const 16
  i32.const 9
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $1
  i32.const 0
  call $~lib/rt/common/OBJECT#set:rtId
  local.get $1
  i32.const 0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $1
  i32.const 0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#set:_isError
  local.get $1
  local.get $0
  i32.load
  local.get $0
  call $~lib/arraybuffer/ArrayBufferView#get:byteOffset
  local.get $0
  i32.load offset=8
  call $~lib/dataview/DataView#constructor
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $1
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $1
  local.get $0
  i32.load offset=8
  i32.load offset=8
  local.get $0
  i32.load offset=12
  i32.sub
  i32.le_u
  if
   local.get $0
   i32.load offset=12
   local.get $0
   local.get $0
   i32.load offset=12
   local.get $1
   i32.add
   call $~lib/rt/common/OBJECT#set:rtId
   return
  end
  local.get $0
  i32.const 1
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#set:_isError
  i32.const -1
 )
 (func $~lib/typedarray/Uint8Array#__get (param $0 i32) (param $1 i32) (result i32)
  local.get $1
  local.get $0
  i32.load offset=8
  i32.ge_u
  if
   i32.const 1680
   i32.const 1744
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
 (func $~lib/dataview/DataView#getUint64 (param $0 i32) (param $1 i32) (result i64)
  local.get $1
  i32.const 31
  i32.shr_u
  local.get $0
  i32.load offset=8
  local.get $1
  i32.const 8
  i32.add
  i32.lt_s
  i32.or
  if
   i32.const 1680
   i32.const 1616
   i32.const 159
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.add
  i64.load
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64 (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i64)
  (local $3 i64)
  (local $4 i64)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $0
  i32.const 1
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset
  local.tee $6
  i32.const -1
  i32.eq
  if
   i64.const 0
   return
  end
  block $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/decode/decodeVariableLengthExtraBytes$29 (result i32)
   local.get $0
   i32.load offset=8
   local.get $6
   call $~lib/typedarray/Uint8Array#__get
   local.set $6
   loop $for-loop|0
    local.get $1
    i32.const 1292
    i32.load
    i32.const 255
    i32.and
    i32.lt_u
    if
     local.get $1
     i32.const 1292
     i32.load
     i32.ge_u
     if
      i32.const 1680
      i32.const 1808
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
     local.get $6
     i32.const 255
     i32.and
     i32.le_u
     br_if $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/decode/decodeVariableLengthExtraBytes$29
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
  local.tee $7
  i32.eqz
  if
   local.get $6
   i64.extend_i32_u
   return
  end
  local.get $0
  local.get $7
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset
  local.tee $1
  i32.const -1
  i32.eq
  if
   i64.const 0
   return
  end
  local.get $7
  i32.const 8
  i32.eq
  if
   local.get $0
   i32.load
   local.get $1
   call $~lib/dataview/DataView#getUint64
   return
  end
  i64.const 2
  local.set $3
  i64.const 8
  local.get $7
  i64.extend_i32_u
  i64.sub
  local.set $4
  i64.const 1
  local.set $2
  loop $while-continue|0
   local.get $4
   i64.const 0
   i64.ne
   if
    local.get $2
    local.get $3
    i64.mul
    local.get $2
    local.get $4
    i64.const 1
    i64.and
    i32.wrap_i64
    select
    local.set $2
    local.get $4
    i64.const 1
    i64.shr_u
    local.set $4
    local.get $3
    local.get $3
    i64.mul
    local.set $3
    br $while-continue|0
   end
  end
  local.get $6
  i64.extend_i32_u
  local.get $2
  i64.add
  i64.const 256
  i64.sub
  local.get $7
  i64.extend_i32_u
  i64.const 3
  i64.shl
  i64.shl
  local.set $2
  loop $for-loop|00
   local.get $5
   local.get $7
   i32.lt_s
   if
    local.get $2
    local.get $0
    i32.load offset=8
    local.get $1
    local.get $5
    i32.add
    call $~lib/typedarray/Uint8Array#__get
    i64.extend_i32_u
    local.get $5
    i64.extend_i32_s
    i64.const 3
    i64.shl
    i64.shl
    i64.or
    local.set $2
    local.get $5
    i32.const 1
    i32.add
    local.set $5
    br $for-loop|00
   end
  end
  local.get $2
 )
 (func $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#set:isError" (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store8
 )
 (func $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor" (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  local.get $1
  local.get $2
  i32.const 7
  call $"byn$mgfn-shared$~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor"
 )
 (func $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>" (param $0 i32) (result i32)
  i32.const 0
  i32.const 0
  local.get $0
  call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor"
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#isFinished (param $0 i32) (result i32)
  local.get $0
  i32.load offset=12
  local.get $0
  i32.load offset=8
  i32.load offset=8
  i32.eq
 )
 (func $~lib/util/number/decimalCount32 (param $0 i32) (result i32)
  local.get $0
  i32.const 10
  i32.ge_u
  i32.const 1
  i32.add
  local.get $0
  i32.const 10000
  i32.ge_u
  i32.const 3
  i32.add
  local.get $0
  i32.const 1000
  i32.ge_u
  i32.add
  local.get $0
  i32.const 100
  i32.lt_u
  select
  local.get $0
  i32.const 1000000
  i32.ge_u
  i32.const 6
  i32.add
  local.get $0
  i32.const 1000000000
  i32.ge_u
  i32.const 8
  i32.add
  local.get $0
  i32.const 100000000
  i32.ge_u
  i32.add
  local.get $0
  i32.const 10000000
  i32.lt_u
  select
  local.get $0
  i32.const 100000
  i32.lt_u
  select
 )
 (func $~lib/util/number/utoa_dec_simple<u32> (param $0 i32) (param $1 i32) (param $2 i32)
  loop $do-loop|0
   local.get $0
   local.get $2
   i32.const 1
   i32.sub
   local.tee $2
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i32.const 10
   i32.rem_u
   i32.const 48
   i32.add
   i32.store16
   local.get $1
   i32.const 10
   i32.div_u
   local.tee $1
   br_if $do-loop|0
  end
 )
 (func $~lib/number/I32#toString (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  if
   i32.const 0
   local.get $0
   i32.sub
   local.get $0
   local.get $0
   i32.const 31
   i32.shr_u
   i32.const 1
   i32.shl
   local.tee $1
   select
   local.tee $3
   call $~lib/util/number/decimalCount32
   local.set $2
   local.get $1
   local.get $2
   i32.const 1
   i32.shl
   local.get $1
   i32.add
   i32.const 2
   call $~lib/rt/stub/__new
   local.tee $0
   i32.add
   local.get $3
   local.get $2
   call $~lib/util/number/utoa_dec_simple<u32>
   local.get $1
   if
    local.get $0
    i32.const 45
    i32.store16
   end
  else
   i32.const 1056
   local.set $0
  end
  local.get $0
 )
 (func $~lib/string/String#concat (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  call $~lib/string/String#get:length
  i32.const 1
  i32.shl
  local.tee $2
  local.get $1
  call $~lib/string/String#get:length
  i32.const 1
  i32.shl
  local.tee $3
  i32.add
  local.tee $4
  i32.eqz
  if
   i32.const 2240
   return
  end
  local.get $4
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $4
  local.get $0
  local.get $2
  memory.copy
  local.get $2
  local.get $4
  i32.add
  local.get $1
  local.get $3
  memory.copy
  local.get $4
 )
 (func $~lib/string/String.UTF8.encodeUnsafe (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  local.get $1
  i32.const 1
  i32.shl
  i32.add
  local.set $3
  local.get $2
  local.set $1
  loop $while-continue|0
   local.get $0
   local.get $3
   i32.lt_u
   if
    local.get $0
    i32.load16_u
    local.tee $2
    i32.const 128
    i32.lt_u
    if (result i32)
     local.get $1
     local.get $2
     i32.store8
     local.get $1
     i32.const 1
     i32.add
    else
     local.get $2
     i32.const 2048
     i32.lt_u
     if (result i32)
      local.get $1
      local.get $2
      i32.const 6
      i32.shr_u
      i32.const 192
      i32.or
      local.get $2
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.const 8
      i32.shl
      i32.or
      i32.store16
      local.get $1
      i32.const 2
      i32.add
     else
      local.get $2
      i32.const 56320
      i32.lt_u
      local.get $0
      i32.const 2
      i32.add
      local.get $3
      i32.lt_u
      i32.and
      local.get $2
      i32.const 63488
      i32.and
      i32.const 55296
      i32.eq
      i32.and
      if
       local.get $0
       i32.load16_u offset=2
       local.tee $4
       i32.const 64512
       i32.and
       i32.const 56320
       i32.eq
       if
        local.get $1
        local.get $2
        i32.const 1023
        i32.and
        i32.const 10
        i32.shl
        i32.const 65536
        i32.add
        local.get $4
        i32.const 1023
        i32.and
        i32.or
        local.tee $2
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 24
        i32.shl
        local.get $2
        i32.const 6
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 16
        i32.shl
        i32.or
        local.get $2
        i32.const 12
        i32.shr_u
        i32.const 63
        i32.and
        i32.const 128
        i32.or
        i32.const 8
        i32.shl
        i32.or
        local.get $2
        i32.const 18
        i32.shr_u
        i32.const 240
        i32.or
        i32.or
        i32.store
        local.get $1
        i32.const 4
        i32.add
        local.set $1
        local.get $0
        i32.const 4
        i32.add
        local.set $0
        br $while-continue|0
       end
      end
      local.get $1
      local.get $2
      i32.const 12
      i32.shr_u
      i32.const 224
      i32.or
      local.get $2
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
      local.get $1
      local.get $2
      i32.const 63
      i32.and
      i32.const 128
      i32.or
      i32.store8 offset=2
      local.get $1
      i32.const 3
      i32.add
     end
    end
    local.set $1
    local.get $0
    i32.const 2
    i32.add
    local.set $0
    br $while-continue|0
   end
  end
 )
 (func $~lib/string/String.UTF8.encode@varargs (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
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
  local.set $1
  local.get $0
  local.get $0
  call $~lib/string/String#get:length
  local.get $1
  call $~lib/string/String.UTF8.encodeUnsafe
  local.get $1
 )
 (func $~lib/@fluffylabs/as-lan/logger/Logger#_log (param $0 i32) (param $1 i32) (param $2 i32)
  local.get $0
  i32.load
  i32.const 1
  global.set $~argumentsLength
  call $~lib/string/String.UTF8.encode@varargs
  local.set $0
  i32.const 1
  global.set $~argumentsLength
  local.get $2
  call $~lib/string/String.UTF8.encode@varargs
  local.set $2
  local.get $1
  local.get $0
  local.get $0
  call $~lib/arraybuffer/ArrayBuffer#get:byteLength
  local.get $2
  local.get $2
  call $~lib/arraybuffer/ArrayBuffer#get:byteLength
  call $~lib/@fluffylabs/as-lan/ecalli/general/log/log
  drop
 )
 (func $~lib/@fluffylabs/as-lan/logger/Logger#warn (param $0 i32) (param $1 i32)
  local.get $0
  i32.const 1
  local.get $1
  call $~lib/@fluffylabs/as-lan/logger/Logger#_log
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
  call $~lib/util/number/decimalCount32
  local.tee $1
  i32.const 1
  i32.shl
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $2
  local.get $0
  local.get $1
  call $~lib/util/number/utoa_dec_simple<u32>
  local.get $2
 )
 (func $~lib/staticarray/StaticArray<~lib/string/String>#__uset (param $0 i32) (param $1 i32) (param $2 i32)
  local.get $0
  local.get $1
  i32.const 2
  i32.shl
  i32.add
  local.get $2
  i32.store
 )
 (func $~lib/string/String.__ne (param $0 i32) (result i32)
  local.get $0
  i32.eqz
  i32.eqz
 )
 (func $~lib/string/String.__concat (param $0 i32) (param $1 i32) (result i32)
  local.get $0
  local.get $1
  call $~lib/string/String#concat
 )
 (func $~lib/staticarray/StaticArray<~lib/string/String>#join (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  block $__inlined_func$~lib/util/string/joinReferenceArray<~lib/string/String> (result i32)
   i32.const 2240
   local.get $0
   local.tee $1
   i32.const 20
   i32.sub
   i32.load offset=16
   i32.const 2
   i32.shr_u
   i32.const 1
   i32.sub
   local.tee $2
   i32.const 0
   i32.lt_s
   br_if $__inlined_func$~lib/util/string/joinReferenceArray<~lib/string/String>
   drop
   local.get $2
   i32.eqz
   if
    local.get $0
    i32.load
    local.tee $0
    i32.const 2240
    local.get $0
    call $~lib/string/String.__ne
    select
    br $__inlined_func$~lib/util/string/joinReferenceArray<~lib/string/String>
   end
   i32.const 2240
   local.set $0
   i32.const 2240
   call $~lib/string/String#get:length
   local.set $4
   loop $for-loop|0
    local.get $2
    local.get $3
    i32.gt_s
    if
     local.get $1
     local.get $3
     i32.const 2
     i32.shl
     i32.add
     i32.load
     local.tee $5
     call $~lib/string/String.__ne
     if
      local.get $0
      local.get $5
      call $~lib/string/String.__concat
      local.set $0
     end
     local.get $4
     if
      local.get $0
      i32.const 2240
      call $~lib/string/String.__concat
      local.set $0
     end
     local.get $3
     i32.const 1
     i32.add
     local.set $3
     br $for-loop|0
    end
   end
   local.get $1
   local.get $2
   i32.const 2
   i32.shl
   i32.add
   i32.load
   local.tee $1
   call $~lib/string/String.__ne
   if (result i32)
    local.get $0
    local.get $1
    call $~lib/string/String.__concat
   else
    local.get $0
   end
  end
 )
 (func $~lib/@fluffylabs/as-lan/logger/Logger#info (param $0 i32) (param $1 i32)
  local.get $0
  i32.const 2
  local.get $1
  call $~lib/@fluffylabs/as-lan/logger/Logger#_log
 )
 (func $~lib/util/number/decimalCount64High (param $0 i64) (result i32)
  local.get $0
  i64.const 100000000000
  i64.ge_u
  i32.const 10
  i32.add
  local.get $0
  i64.const 10000000000
  i64.ge_u
  i32.add
  local.get $0
  i64.const 100000000000000
  i64.ge_u
  i32.const 13
  i32.add
  local.get $0
  i64.const 10000000000000
  i64.ge_u
  i32.add
  local.get $0
  i64.const 1000000000000
  i64.lt_u
  select
  local.get $0
  i64.const 10000000000000000
  i64.ge_u
  i32.const 16
  i32.add
  local.get $0
  i64.const -8446744073709551616
  i64.ge_u
  i32.const 18
  i32.add
  local.get $0
  i64.const 1000000000000000000
  i64.ge_u
  i32.add
  local.get $0
  i64.const 100000000000000000
  i64.lt_u
  select
  local.get $0
  i64.const 1000000000000000
  i64.lt_u
  select
 )
 (func $~lib/util/number/utoa_dec_simple<u64> (param $0 i32) (param $1 i64) (param $2 i32)
  loop $do-loop|0
   local.get $0
   local.get $2
   i32.const 1
   i32.sub
   local.tee $2
   i32.const 1
   i32.shl
   i32.add
   local.get $1
   i64.const 10
   i64.rem_u
   i32.wrap_i64
   i32.const 48
   i32.add
   i32.store16
   local.get $1
   i64.const 10
   i64.div_u
   local.tee $1
   i64.const 0
   i64.ne
   br_if $do-loop|0
  end
 )
 (func $~lib/number/I64#toString (param $0 i64) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  i64.eqz
  if
   i32.const 1056
   local.set $2
  else
   i64.const 0
   local.get $0
   i64.sub
   local.get $0
   local.get $0
   i64.const 63
   i64.shr_u
   i32.wrap_i64
   i32.const 1
   i32.shl
   local.tee $1
   select
   local.tee $0
   i64.const 4294967295
   i64.le_u
   if
    local.get $1
    local.get $0
    i32.wrap_i64
    local.tee $3
    call $~lib/util/number/decimalCount32
    local.tee $4
    i32.const 1
    i32.shl
    local.get $1
    i32.add
    i32.const 2
    call $~lib/rt/stub/__new
    local.tee $2
    i32.add
    local.get $3
    local.get $4
    call $~lib/util/number/utoa_dec_simple<u32>
   else
    local.get $1
    local.get $0
    call $~lib/util/number/decimalCount64High
    local.tee $3
    i32.const 1
    i32.shl
    local.get $1
    i32.add
    i32.const 2
    call $~lib/rt/stub/__new
    local.tee $2
    i32.add
    local.get $0
    local.get $3
    call $~lib/util/number/utoa_dec_simple<u64>
   end
   local.get $1
   if
    local.get $2
    i32.const 45
    i32.store16
   end
  end
  local.get $2
 )
 (func $~lib/typedarray/Uint8Array#subarray (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  i32.load offset=8
  local.set $3
  i32.const 12
  i32.const 8
  call $~lib/rt/stub/__new
  local.tee $4
  local.get $0
  i32.load
  i32.store
  local.get $4
  local.get $0
  i32.load offset=4
  local.get $1
  i32.const 0
  i32.lt_s
  if (result i32)
   local.get $1
   local.get $3
   i32.add
   local.tee $0
   i32.const 0
   local.get $0
   i32.const 0
   i32.gt_s
   select
  else
   local.get $1
   local.get $3
   local.get $1
   local.get $3
   i32.lt_s
   select
  end
  local.tee $0
  i32.add
  i32.store offset=4
  local.get $4
  local.get $2
  i32.const 0
  i32.lt_s
  if (result i32)
   local.get $2
   local.get $3
   i32.add
   local.tee $1
   i32.const 0
   local.get $1
   i32.const 0
   i32.gt_s
   select
  else
   local.get $2
   local.get $3
   local.get $2
   local.get $3
   i32.lt_s
   select
  end
  local.tee $1
  local.get $0
  local.get $0
  local.get $1
  i32.lt_s
  select
  local.get $0
  i32.sub
  i32.store offset=8
  local.get $4
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32 (param $0 i32) (result i32)
  (local $1 i64)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
  local.tee $1
  i64.const 4294967295
  i64.gt_u
  if
   local.get $0
   i32.const 1
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#set:_isError
   i32.const 0
   return
  end
  local.get $1
  i32.wrap_i64
 )
 (func $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor (param $0 i32) (result i32)
  (local $1 i32)
  i32.const 4
  i32.const 14
  call $~lib/rt/stub/__new
  local.tee $1
  local.get $0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $1
 )
 (func $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob.empty (result i32)
  i32.const 0
  call $~lib/typedarray/Uint8Array#constructor
  call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesFixLen (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $1
  i32.eqz
  if
   call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob.empty
   return
  end
  local.get $0
  local.get $1
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset
  local.tee $2
  i32.const -1
  i32.eq
  if
   i32.const 0
   call $~lib/typedarray/Uint8Array#constructor
   call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor
   return
  end
  local.get $0
  i32.load offset=8
  local.get $2
  local.get $1
  local.get $2
  i32.add
  call $~lib/typedarray/Uint8Array#subarray
  call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32 (param $0 i32) (result i32)
  (local $1 i32)
  local.get $0
  i32.const 32
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesFixLen
  i32.load
  local.set $1
  i32.const 8
  i32.const 13
  call $~lib/rt/stub/__new
  local.tee $0
  i32.const 0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $0
  local.get $1
  call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor
  local.tee $1
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
  local.get $1
  i32.load
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $0
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen (param $0 i32) (result i32)
  local.get $0
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesFixLen
 )
 (func $~lib/@fluffylabs/as-lan/jam/accumulate/item/WorkExecResult.create (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  i32.const 8
  i32.const 15
  call $~lib/rt/stub/__new
  local.tee $2
  local.get $0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $2
  local.get $1
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $2
 )
 (func $~lib/string/String.fromCharCode@varargs (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  block $1of1
   block $0of1
    block $outOfRange
     global.get $~argumentsLength
     i32.const 1
     i32.sub
     br_table $0of1 $1of1 $outOfRange
    end
    unreachable
   end
   i32.const -1
   local.set $1
  end
  i32.const 2
  local.get $1
  i32.const 0
  i32.gt_s
  local.tee $3
  i32.shl
  i32.const 2
  call $~lib/rt/stub/__new
  local.tee $2
  local.get $0
  i32.store16
  local.get $3
  if
   local.get $2
   local.get $1
   i32.store16 offset=2
  end
  local.get $2
 )
 (func $~lib/@fluffylabs/as-lan/core/bytes/bytesToHexString~anonymous|0 (param $0 i32) (result i32)
  local.get $0
  i32.const 10
  i32.ge_s
  if
   i32.const 1
   global.set $~argumentsLength
   local.get $0
   global.get $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_a
   i32.add
   i32.const 10
   i32.sub
   call $~lib/string/String.fromCharCode@varargs
   return
  end
  i32.const 1
  global.set $~argumentsLength
  local.get $0
  global.get $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_0
  i32.add
  call $~lib/string/String.fromCharCode@varargs
 )
 (func $~lib/@fluffylabs/as-lan/core/bytes/Bytes32#toString (param $0 i32) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.load
  i32.load
  local.set $2
  i32.const 3248
  local.set $0
  loop $for-loop|0
   local.get $1
   local.get $2
   i32.load offset=8
   i32.lt_s
   if
    local.get $2
    local.get $1
    call $~lib/typedarray/Uint8Array#__get
    local.set $3
    i32.const 1
    global.set $~argumentsLength
    local.get $0
    local.get $3
    i32.const 4
    i32.shr_u
    i32.const 3216
    i32.load
    call_indirect (type $0)
    call $~lib/string/String.__concat
    i32.const 1
    global.set $~argumentsLength
    local.get $3
    i32.const 15
    i32.and
    i32.const 3216
    i32.load
    call_indirect (type $0)
    call $~lib/string/String.__concat
    local.set $0
    local.get $1
    i32.const 1
    i32.add
    local.set $1
    br $for-loop|0
   end
  end
  local.get $0
 )
 (func $~lib/number/U64#toString (param $0 i64) (result i32)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i64.eqz
  if
   i32.const 1056
   local.set $1
  else
   local.get $0
   i64.const 4294967295
   i64.le_u
   if
    local.get $0
    i32.wrap_i64
    local.tee $2
    call $~lib/util/number/decimalCount32
    local.tee $3
    i32.const 1
    i32.shl
    i32.const 2
    call $~lib/rt/stub/__new
    local.tee $1
    local.get $2
    local.get $3
    call $~lib/util/number/utoa_dec_simple<u32>
   else
    local.get $0
    call $~lib/util/number/decimalCount64High
    local.tee $2
    i32.const 1
    i32.shl
    i32.const 2
    call $~lib/rt/stub/__new
    local.tee $1
    local.get $0
    local.get $2
    call $~lib/util/number/utoa_dec_simple<u64>
   end
  end
  local.get $1
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#set:_isError (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store8 offset=8
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder.create@varargs (result i32)
  (local $0 i32)
  (local $1 i32)
  block $1of1
   block $0of1
    block $outOfRange
     global.get $~argumentsLength
     br_table $0of1 $1of1 $outOfRange
    end
    unreachable
   end
   i32.const 32
   local.set $0
  end
  local.get $0
  call $~lib/typedarray/Uint8Array#constructor
  local.set $1
  i32.const 17
  i32.const 17
  call $~lib/rt/stub/__new
  local.tee $0
  local.get $1
  call $~lib/rt/common/OBJECT#set:rtId
  local.get $0
  i32.const 1
  i32.store8 offset=16
  local.get $0
  i32.const 0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
  i32.const 0
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $0
  i32.const 0
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#set:_isError
  local.get $0
  local.get $1
  i32.load
  local.get $1
  call $~lib/arraybuffer/ArrayBufferView#get:byteOffset
  local.get $1
  i32.load offset=8
  call $~lib/dataview/DataView#constructor
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
 )
 (func $~lib/typedarray/Uint8Array#set<~lib/typedarray/Uint8Array> (param $0 i32) (param $1 i32) (param $2 i32)
  (local $3 i32)
  local.get $1
  i32.load offset=8
  local.set $3
  local.get $2
  i32.const 0
  i32.lt_s
  if (result i32)
   i32.const 1
  else
   local.get $0
   i32.load offset=8
   local.get $2
   local.get $3
   i32.add
   i32.lt_s
  end
  if
   i32.const 1680
   i32.const 1744
   i32.const 1902
   i32.const 5
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $2
  i32.add
  local.get $1
  i32.load offset=4
  local.get $3
  memory.copy
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity (param $0 i32) (param $1 i32) (result i32)
  (local $2 i32)
  local.get $0
  i32.load8_u offset=8
  if
   i32.const 0
   return
  end
  local.get $0
  i32.load offset=12
  i32.load offset=8
  local.get $0
  i32.load offset=4
  i32.sub
  local.get $1
  i32.ge_u
  if
   i32.const 1
   return
  end
  local.get $0
  i32.load8_u offset=16
  i32.eqz
  if
   local.get $0
   i32.const 1
   call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#set:_isError
   i32.const 0
   return
  end
  local.get $1
  local.get $0
  i32.load offset=4
  i32.add
  local.set $2
  local.get $0
  i32.load offset=12
  i32.load offset=8
  local.tee $1
  i32.eqz
  if
   i32.const 32
   local.set $1
  end
  loop $while-continue|0
   local.get $1
   local.get $2
   i32.lt_u
   if
    local.get $1
    i32.const 1
    i32.shl
    local.set $1
    br $while-continue|0
   end
  end
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.tee $2
  local.get $0
  i32.load offset=12
  i32.const 0
  local.get $0
  i32.load offset=4
  call $~lib/typedarray/Uint8Array#subarray
  i32.const 0
  call $~lib/typedarray/Uint8Array#set<~lib/typedarray/Uint8Array>
  local.get $0
  local.get $2
  call $~lib/rt/common/OBJECT#set:rtId
  local.get $0
  local.get $2
  i32.load
  i32.const 0
  local.get $1
  call $~lib/dataview/DataView#constructor
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  i32.const 1
 )
 (func $~lib/dataview/DataView#setUint64 (param $0 i32) (param $1 i32) (param $2 i64)
  local.get $1
  i32.const 31
  i32.shr_u
  local.get $0
  i32.load offset=8
  local.get $1
  i32.const 8
  i32.add
  i32.lt_s
  i32.or
  if
   i32.const 1680
   i32.const 1616
   i32.const 174
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.add
  local.get $2
  i64.store
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u64 (param $0 i32) (param $1 i64)
  local.get $0
  i32.const 8
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
  i32.eqz
  if
   return
  end
  local.get $0
  i32.load
  local.get $0
  i32.load offset=4
  local.get $1
  call $~lib/dataview/DataView#setUint64
  local.get $0
  local.get $0
  i32.load offset=4
  i32.const 8
  i32.add
  call $~lib/rt/common/OBJECT#set:gcInfo
 )
 (func $~lib/dataview/DataView#setUint8 (param $0 i32) (param $1 i32) (param $2 i32)
  local.get $1
  local.get $0
  i32.load offset=8
  i32.ge_u
  if
   i32.const 1680
   i32.const 1616
   i32.const 128
   i32.const 50
   call $~lib/builtins/abort
   unreachable
  end
  local.get $0
  i32.load offset=4
  local.get $1
  i32.add
  local.get $2
  i32.store8
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#varU64 (param $0 i32) (param $1 i64)
  (local $2 i32)
  (local $3 i32)
  (local $4 i64)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  local.get $1
  i64.const 128
  i64.lt_u
  if
   local.get $0
   i32.const 1
   call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
   if
    local.get $0
    i32.load
    local.get $0
    i32.load offset=4
    local.get $1
    i32.wrap_i64
    call $~lib/dataview/DataView#setUint8
    local.get $0
    local.get $0
    i32.load offset=4
    i32.const 1
    i32.add
    call $~lib/rt/common/OBJECT#set:gcInfo
   end
   return
  end
  i32.const 1
  local.set $2
  block $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/encode/encodeVariableLengthExtraBytes$92
   loop $for-loop|0
    local.get $2
    i32.const 7
    i32.le_u
    if
     local.get $1
     i64.const 1
     local.get $2
     i32.const 1
     i32.add
     local.tee $3
     i32.const 255
     i32.and
     i64.extend_i32_u
     i64.const 7
     i64.mul
     i64.shl
     i64.lt_u
     br_if $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/encode/encodeVariableLengthExtraBytes$92
     local.get $3
     local.set $2
     br $for-loop|0
    end
   end
   i32.const 8
   local.set $2
  end
  local.get $2
  i32.const 8
  i32.eq
  if
   local.get $0
   i32.const 9
   call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
   i32.eqz
   if
    return
   end
   local.get $0
   i32.load
   local.get $0
   i32.load offset=4
   i32.const 255
   call $~lib/dataview/DataView#setUint8
   local.get $0
   local.get $0
   i32.load offset=4
   i32.const 1
   i32.add
   call $~lib/rt/common/OBJECT#set:gcInfo
   local.get $0
   i32.load
   local.get $0
   i32.load offset=4
   local.get $1
   call $~lib/dataview/DataView#setUint64
   local.get $0
   local.get $0
   i32.load offset=4
   i32.const 8
   i32.add
   call $~lib/rt/common/OBJECT#set:gcInfo
   return
  end
  local.get $0
  local.get $2
  i32.const 1
  i32.add
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
  i32.eqz
  if
   return
  end
  local.get $1
  local.get $2
  i64.extend_i32_u
  i64.const 3
  i64.shl
  i64.shr_u
  local.set $4
  i32.const 2
  local.set $6
  i32.const 8
  local.get $2
  i32.sub
  i32.const 255
  i32.and
  local.set $3
  i32.const 1
  local.set $7
  loop $while-continue|0
   local.get $3
   if
    local.get $6
    local.get $7
    i32.mul
    local.get $7
    local.get $3
    i32.const 1
    i32.and
    select
    local.set $7
    local.get $3
    i32.const 1
    i32.shr_u
    local.set $3
    local.get $6
    local.get $6
    i32.mul
    local.set $6
    br $while-continue|0
   end
  end
  local.get $0
  i32.load
  local.get $0
  i32.load offset=4
  local.get $4
  i32.wrap_i64
  i32.const 0
  local.get $7
  i32.const 255
  i32.and
  i32.sub
  i32.or
  call $~lib/dataview/DataView#setUint8
  local.get $0
  local.get $0
  i32.load offset=4
  i32.const 1
  i32.add
  call $~lib/rt/common/OBJECT#set:gcInfo
  loop $for-loop|00
   local.get $2
   local.get $5
   i32.gt_u
   if
    local.get $0
    i32.load
    local.get $0
    i32.load offset=4
    local.get $1
    local.get $5
    i64.extend_i32_u
    i64.const 3
    i64.shl
    i64.shr_u
    i32.wrap_i64
    call $~lib/dataview/DataView#setUint8
    local.get $0
    local.get $0
    i32.load offset=4
    i32.const 1
    i32.add
    call $~lib/rt/common/OBJECT#set:gcInfo
    local.get $5
    i32.const 1
    i32.add
    local.set $5
    br $for-loop|00
   end
  end
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#finish (param $0 i32) (result i32)
  local.get $0
  i32.load offset=12
  i32.const 0
  local.get $0
  i32.load offset=4
  call $~lib/typedarray/Uint8Array#subarray
 )
 (func $~lib/@fluffylabs/as-lan/jam/service/Response.with (param $0 i64) (param $1 i32) (result i64)
  (local $2 i32)
  (local $3 i32)
  i32.const 0
  global.set $~argumentsLength
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder.create@varargs
  local.tee $2
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u64
  local.get $1
  if
   local.get $2
   local.get $1
   call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob#constructor
   local.tee $1
   i32.load
   i64.load32_s offset=8
   call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#varU64
   block $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#bytesFixLen$262
    local.get $1
    i32.load
    local.tee $3
    i32.load offset=8
    local.tee $1
    i32.eqz
    br_if $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#bytesFixLen$262
    local.get $2
    local.get $1
    call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
    i32.eqz
    br_if $__inlined_func$~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#bytesFixLen$262
    local.get $2
    i32.load offset=12
    local.get $3
    local.get $2
    i32.load offset=4
    call $~lib/typedarray/Uint8Array#set<~lib/typedarray/Uint8Array>
    local.get $2
    local.get $2
    i32.load offset=4
    local.get $1
    i32.add
    call $~lib/rt/common/OBJECT#set:gcInfo
   end
  else
   local.get $2
   i64.const 0
   call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#varU64
  end
  local.get $2
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#finish
  local.tee $1
  i64.load32_u offset=4
  local.get $1
  i64.load32_s offset=8
  i64.const 32
  i64.shl
  i64.or
 )
 (func $assembly/dispatch/general/dispatchGas (result i64)
  (local $0 i64)
  call $~lib/@fluffylabs/as-lan/ecalli/general/gas/gas
  local.set $0
  global.get $assembly/dispatch/common/logger
  i32.const 3776
  local.get $0
  call $~lib/number/I64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $0
  i32.const 0
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/common/outputLen (param $0 i64) (param $1 i32) (param $2 i32) (result i32)
  (local $3 i32)
  local.get $0
  i64.const 0
  i64.lt_s
  if
   i32.const 0
   return
  end
  local.get $0
  i32.wrap_i64
  local.tee $3
  local.get $1
  i32.le_u
  if
   i32.const 0
   return
  end
  local.get $2
  local.get $3
  local.get $1
  i32.sub
  local.tee $1
  local.get $1
  local.get $2
  i32.gt_u
  select
 )
 (func $assembly/dispatch/general/dispatchFetch (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i64)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $4
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $5
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $6
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $1
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 3824
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.tee $0
  i32.load offset=4
  local.get $3
  local.get $1
  local.get $4
  local.get $5
  local.get $6
  call $~lib/@fluffylabs/as-lan/ecalli/general/fetch/fetch
  local.set $2
  global.get $assembly/dispatch/common/logger
  local.get $4
  call $~lib/util/number/utoa32
  local.set $4
  local.get $2
  call $~lib/number/I64#toString
  local.set $6
  i32.const 3984
  i32.const 1
  local.get $4
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3984
  i32.const 3
  local.get $6
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3984
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  local.get $0
  i32.const 0
  local.get $2
  local.get $3
  local.get $1
  call $assembly/dispatch/common/outputLen
  call $~lib/typedarray/Uint8Array#subarray
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/general/dispatchLookup (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i64)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $1
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 4032
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.set $0
  i32.load offset=4
  i32.load offset=4
  local.get $0
  i32.load offset=4
  local.get $3
  local.get $1
  call $~lib/@fluffylabs/as-lan/ecalli/general/lookup/lookup
  local.set $2
  global.get $assembly/dispatch/common/logger
  i32.const 4112
  local.get $2
  call $~lib/number/I64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  local.get $0
  i32.const 0
  local.get $2
  local.get $3
  local.get $1
  call $assembly/dispatch/common/outputLen
  call $~lib/typedarray/Uint8Array#subarray
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/general/dispatchRead (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i64)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $4
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $1
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 4160
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.set $0
  local.get $3
  i32.load
  i32.load offset=4
  local.get $3
  i32.load
  i32.load offset=8
  local.get $0
  i32.load offset=4
  local.get $4
  local.get $1
  call $~lib/@fluffylabs/as-lan/ecalli/general/read/read
  local.set $2
  global.get $assembly/dispatch/common/logger
  i32.const 4240
  local.get $2
  call $~lib/number/I64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  local.get $0
  i32.const 0
  local.get $2
  local.get $4
  local.get $1
  call $assembly/dispatch/common/outputLen
  call $~lib/typedarray/Uint8Array#subarray
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/general/dispatchWrite (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i32)
  (local $3 i64)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $1
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $2
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 4288
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  i32.load
  i32.load offset=4
  local.get $1
  i32.load
  i32.load offset=8
  local.get $2
  i32.load
  i32.load offset=4
  local.get $2
  i32.load
  i32.load offset=8
  call $~lib/@fluffylabs/as-lan/ecalli/general/write/write
  local.set $3
  global.get $assembly/dispatch/common/logger
  i32.const 4368
  local.get $3
  call $~lib/number/I64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $3
  i32.const 0
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/general/dispatchInfo (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i64)
  (local $3 i32)
  (local $4 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $1
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 4416
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  call $~lib/typedarray/Uint8Array#constructor
  local.tee $0
  i32.load offset=4
  local.get $3
  local.get $1
  call $~lib/@fluffylabs/as-lan/ecalli/general/info/info
  local.set $2
  global.get $assembly/dispatch/common/logger
  i32.const 4496
  local.get $2
  call $~lib/number/I64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  local.get $0
  i32.const 0
  local.get $2
  local.get $3
  local.get $1
  call $assembly/dispatch/common/outputLen
  call $~lib/typedarray/Uint8Array#subarray
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/dispatch/general/dispatchLog (param $0 i32) (result i64)
  (local $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
  local.set $1
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $2
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $3
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 4544
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  local.get $1
  local.get $2
  i32.load
  i32.load offset=4
  local.get $2
  i32.load
  i32.load offset=8
  local.get $3
  i32.load
  i32.load offset=4
  local.get $3
  i32.load
  i32.load offset=8
  call $~lib/@fluffylabs/as-lan/ecalli/general/log/log
  local.set $0
  global.get $assembly/dispatch/common/logger
  local.get $1
  call $~lib/util/number/utoa32
  local.set $1
  local.get $0
  call $~lib/util/number/utoa32
  local.set $3
  i32.const 4672
  i32.const 1
  local.get $1
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 4672
  i32.const 3
  local.get $3
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 4672
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $0
  i64.extend_i32_u
  i32.const 0
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/accumulate/processOperand (param $0 i32) (param $1 i32) (result i64)
  (local $2 i64)
  (local $3 i32)
  (local $4 i32)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  (local $9 i32)
  (local $10 i64)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
  local.set $5
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
  local.set $6
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
  local.set $7
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
  local.set $8
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
  local.set $2
  block $__inlined_func$~lib/@fluffylabs/as-lan/jam/accumulate/item/WorkExecResult.decode$258 (result i32)
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
   local.tee $3
   i32.eqz
   if
    local.get $3
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    call $~lib/@fluffylabs/as-lan/jam/accumulate/item/WorkExecResult.create
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/accumulate/item/WorkExecResult.decode$258
   end
   local.get $3
   i32.const 6
   i32.gt_u
   if
    local.get $0
    i32.const 1
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#set:_isError
   end
   local.get $3
   call $~lib/@fluffylabs/as-lan/core/bytes/BytesBlob.empty
   call $~lib/@fluffylabs/as-lan/jam/accumulate/item/WorkExecResult.create
  end
  local.set $3
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
  local.set $9
  i32.const 32
  i32.const 12
  call $~lib/rt/stub/__new
  local.tee $4
  local.get $5
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $4
  local.get $6
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $4
  local.get $7
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $4
  local.get $8
  call $~lib/rt/common/OBJECT#set:rtId
  local.get $4
  local.get $2
  i64.store offset=16
  local.get $4
  local.get $3
  i32.store offset=24
  local.get $4
  local.get $9
  i32.store offset=28
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 2880
   local.get $1
   call $~lib/util/number/utoa32
   call $~lib/string/String#concat
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $1
  call $~lib/util/number/utoa32
  local.set $3
  local.get $4
  i32.load
  call $~lib/@fluffylabs/as-lan/core/bytes/Bytes32#toString
  local.set $5
  local.get $4
  i64.load offset=16
  call $~lib/number/U64#toString
  local.set $6
  local.get $4
  i32.load offset=24
  i32.load
  call $~lib/number/I32#toString
  local.set $7
  i32.const 3152
  i32.const 1
  local.get $3
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3152
  i32.const 3
  local.get $5
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3152
  i32.const 5
  local.get $6
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3152
  i32.const 7
  local.get $7
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3152
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $4
  i32.load offset=24
  i32.load
  if
   global.get $assembly/dispatch/common/logger
   local.get $1
   call $~lib/util/number/utoa32
   local.set $1
   local.get $4
   i32.load offset=24
   i32.load
   call $~lib/number/I32#toString
   local.set $3
   i32.const 3344
   i32.const 1
   local.get $1
   call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
   i32.const 3344
   i32.const 3
   local.get $3
   call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
   i32.const 3344
   call $~lib/staticarray/StaticArray<~lib/string/String>#join
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   local.get $4
   i32.load offset=24
   i64.load32_s
   i32.const 0
   call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   return
  end
  local.get $4
  i32.load offset=24
  i32.load offset=4
  local.tee $0
  i32.load
  i32.load offset=8
  i32.eqz
  if
   global.get $assembly/dispatch/common/logger
   i32.const 3488
   i32.const 1
   local.get $1
   call $~lib/util/number/utoa32
   call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
   i32.const 3488
   call $~lib/staticarray/StaticArray<~lib/string/String>#join
   call $~lib/@fluffylabs/as-lan/logger/Logger#info
   i64.const 0
   i32.const 0
   call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   return
  end
  local.get $0
  i32.load
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob
  local.tee $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
  local.set $2
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 3632
   i32.const 1
   local.get $1
   call $~lib/util/number/utoa32
   call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
   i32.const 3632
   call $~lib/staticarray/StaticArray<~lib/string/String>#join
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $1
  call $~lib/util/number/utoa32
  local.set $4
  local.get $2
  call $~lib/number/U64#toString
  local.set $5
  i32.const 3728
  i32.const 1
  local.get $4
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3728
  i32.const 3
  local.get $5
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 3728
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  i64.eqz
  if
   call $assembly/dispatch/general/dispatchGas
   return
  end
  local.get $2
  i64.const 1
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchFetch
   return
  end
  local.get $2
  i64.const 2
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchLookup
   return
  end
  local.get $2
  i64.const 3
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchRead
   return
  end
  local.get $2
  i64.const 4
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchWrite
   return
  end
  local.get $2
  i64.const 5
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchInfo
   return
  end
  local.get $2
  i64.const 100
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchLog
   return
  end
  local.get $2
  i64.const 14
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchBless$265 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $4
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $5
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.set $6
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $7
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 4720
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchBless$265
    end
    i32.load
    i32.load offset=4
    local.get $4
    local.get $5
    local.get $6
    i32.load
    i32.load offset=4
    local.get $7
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/bless/bless
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 4800
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 15
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchAssign$266 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $4
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 4848
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchAssign$266
    end
    i32.load
    i32.load offset=4
    local.get $4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/assign/assign
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 4928
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 16
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchDesignate$267 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 4976
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchDesignate$267
    end
    i32.load
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/designate/designate
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5072
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 17
  i64.eq
  if
   call $~lib/@fluffylabs/as-lan/ecalli/accumulate/checkpoint/checkpoint
   local.set $2
   global.get $assembly/dispatch/common/logger
   i32.const 5120
   local.get $2
   call $~lib/number/I64#toString
   call $~lib/string/String#concat
   call $~lib/@fluffylabs/as-lan/logger/Logger#info
   local.get $2
   i32.const 0
   call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   return
  end
  local.get $2
  i64.const 18
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchNewService$268 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $3
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.set $2
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.set $10
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $4
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $5
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5184
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchNewService$268
    end
    i32.load offset=4
    i32.load offset=4
    local.get $3
    local.get $2
    local.get $10
    local.get $4
    local.get $5
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/new_service/new_service
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5280
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 19
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchUpgrade$269 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.set $2
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.set $10
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5344
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchUpgrade$269
    end
    i32.load offset=4
    i32.load offset=4
    local.get $2
    local.get $10
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/upgrade/upgrade
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5440
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 20
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchTransfer$270 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5488
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchTransfer$270
    end
    i32.const 128
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    i32.load
    local.tee $3
    i32.load offset=8
    local.set $4
    local.get $0
    local.get $3
    i32.const 0
    i32.const 128
    local.get $4
    local.get $4
    i32.const 128
    i32.ge_s
    select
    call $~lib/typedarray/Uint8Array#subarray
    i32.const 0
    call $~lib/typedarray/Uint8Array#set<~lib/typedarray/Uint8Array>
    local.get $0
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/transfer/transfer
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5584
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 21
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchEject$271 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5632
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchEject$271
    end
    i32.load offset=4
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/eject/eject
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5712
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 22
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchQuery$272 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $3
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5760
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchQuery$272
    end
    i32.const 8
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    i32.load offset=4
    i32.load offset=4
    local.get $3
    local.get $0
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/query/query
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5840
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    local.get $0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 23
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchSolicit$273 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $3
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 5888
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchSolicit$273
    end
    i32.load offset=4
    i32.load offset=4
    local.get $3
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/solicit/solicit
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 5984
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 24
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchForget$274 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $3
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 6032
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchForget$274
    end
    i32.load offset=4
    i32.load offset=4
    local.get $3
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/forget/forget
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 6112
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 25
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchYieldResult$275 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 6160
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchYieldResult$275
    end
    i32.load offset=4
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/yield_result/yield_result
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 6256
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 26
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/accumulate/dispatchProvide$276 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.set $3
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 6320
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/accumulate/dispatchProvide$276
    end
    local.get $3
    i32.load
    i32.load offset=4
    local.get $3
    i32.load
    i32.load offset=8
    call $~lib/@fluffylabs/as-lan/ecalli/accumulate/provide/provide
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 6416
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $1
  call $~lib/util/number/utoa32
  local.set $1
  local.get $2
  call $~lib/number/U64#toString
  local.set $3
  i32.const 6528
  i32.const 1
  local.get $1
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6528
  i32.const 3
  local.get $3
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6528
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#warn
  i64.const 0
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u32 (param $0 i32) (result i32)
  (local $1 i32)
  local.get $0
  i32.const 4
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset
  local.tee $1
  i32.const -1
  i32.ne
  if
   local.get $1
   i32.const 31
   i32.shr_u
   local.get $0
   i32.load
   local.tee $0
   i32.load offset=8
   local.get $1
   i32.const 4
   i32.add
   i32.lt_s
   i32.or
   if
    i32.const 1680
    i32.const 1616
    i32.const 87
    i32.const 7
    call $~lib/builtins/abort
    unreachable
   end
   local.get $0
   i32.load offset=4
   local.get $1
   i32.add
   i32.load
   return
  end
  i32.const 0
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u64 (param $0 i32) (result i64)
  (local $1 i32)
  local.get $0
  i32.const 8
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#moveOffset
  local.tee $1
  i32.const -1
  i32.ne
  if
   local.get $0
   i32.load
   local.get $1
   call $~lib/dataview/DataView#getUint64
   return
  end
  i64.const 0
 )
 (func $~lib/@fluffylabs/as-lan/jam/accumulate/item/PendingTransfer#set:memo (param $0 i32) (param $1 i32)
  local.get $0
  local.get $1
  i32.store offset=16
 )
 (func $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u32 (param $0 i32) (param $1 i32)
  (local $2 i32)
  (local $3 i32)
  local.get $0
  i32.const 4
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#ensureCapacity
  i32.eqz
  if
   return
  end
  local.get $0
  i32.load offset=4
  local.tee $2
  i32.const 31
  i32.shr_u
  local.get $0
  i32.load
  local.tee $3
  i32.load offset=8
  local.get $2
  i32.const 4
  i32.add
  i32.lt_s
  i32.or
  if
   i32.const 1680
   i32.const 1616
   i32.const 142
   i32.const 7
   call $~lib/builtins/abort
   unreachable
  end
  local.get $3
  i32.load offset=4
  local.get $2
  i32.add
  local.get $1
  i32.store
  local.get $0
  local.get $0
  i32.load offset=4
  i32.const 4
  i32.add
  call $~lib/rt/common/OBJECT#set:gcInfo
 )
 (func $assembly/accumulate/processTransfer (param $0 i32) (param $1 i32) (result i64)
  (local $2 i32)
  (local $3 i64)
  (local $4 i64)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u32
  local.set $5
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u32
  local.set $6
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u64
  local.set $3
  local.get $0
  i32.const 128
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesFixLen
  local.set $7
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#u64
  local.set $4
  i32.const 32
  i32.const 18
  call $~lib/rt/stub/__new
  local.tee $2
  local.get $5
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $2
  local.get $6
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $2
  local.get $3
  i64.store offset=8
  local.get $2
  local.get $7
  call $~lib/@fluffylabs/as-lan/jam/accumulate/item/PendingTransfer#set:memo
  local.get $2
  local.get $4
  i64.store offset=24
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 6576
   local.get $1
   call $~lib/util/number/utoa32
   call $~lib/string/String#concat
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $1
  call $~lib/util/number/utoa32
  local.set $1
  local.get $2
  i32.load
  call $~lib/util/number/utoa32
  local.set $6
  local.get $2
  i32.load offset=4
  call $~lib/util/number/utoa32
  local.set $7
  local.get $2
  i64.load offset=8
  call $~lib/number/U64#toString
  local.set $0
  local.get $2
  i64.load offset=24
  call $~lib/number/U64#toString
  local.set $8
  i32.const 6848
  i32.const 1
  local.get $1
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6848
  i32.const 3
  local.get $6
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6848
  i32.const 5
  local.get $7
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6848
  i32.const 7
  local.get $0
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6848
  i32.const 9
  local.get $8
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 6848
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  i32.const 0
  global.set $~argumentsLength
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder.create@varargs
  local.tee $0
  local.get $2
  i32.load
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u32
  local.get $0
  local.get $2
  i32.load offset=4
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u32
  local.get $0
  local.get $2
  i64.load offset=8
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u64
  local.get $0
  local.get $2
  i64.load offset=24
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#u64
  i64.const 0
  local.get $0
  call $~lib/@fluffylabs/as-lan/core/codec/encode/Encoder#finish
  call $~lib/@fluffylabs/as-lan/jam/service/Response.with
 )
 (func $assembly/accumulate/accumulate (param $0 i32) (param $1 i32) (result i64)
  (local $2 i64)
  (local $3 i64)
  (local $4 i64)
  (local $5 i32)
  (local $6 i64)
  (local $7 i32)
  (local $8 i32)
  (local $9 i32)
  block $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81 (result i32)
   local.get $0
   local.get $1
   call $~lib/@fluffylabs/as-lan/core/mem/readFromMemory
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob
   local.tee $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $4
   i64.const 4294967295
   i64.gt_u
   if
    i32.const 3
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $6
   i64.const 4294967295
   i64.gt_u
   if
    i32.const 2
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $3
   i64.const 4294967295
   i64.gt_u
   if
    i32.const 4
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81
   end
   local.get $0
   i32.load8_u offset=4
   if
    i32.const 5
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#isFinished
   i32.eqz
   if
    i32.const 6
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs.parse$81
   end
   i32.const 12
   i32.const 6
   call $~lib/rt/stub/__new
   local.tee $0
   local.get $4
   i32.wrap_i64
   call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
   local.get $0
   local.get $6
   i32.wrap_i64
   call $~lib/rt/common/OBJECT#set:gcInfo
   local.get $0
   local.get $3
   i32.wrap_i64
   call $~lib/rt/common/OBJECT#set:gcInfo2
   i32.const 1
   local.get $0
   i32.const 0
   call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor"
  end
  local.tee $0
  i32.load8_u
  if
   global.get $assembly/dispatch/common/logger
   i32.const 2144
   local.get $0
   i32.load offset=8
   call $~lib/number/I32#toString
   call $~lib/string/String#concat
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $0
  i32.load offset=4
  local.tee $1
  i32.load
  call $~lib/util/number/utoa32
  local.set $0
  local.get $1
  i32.load offset=4
  call $~lib/util/number/utoa32
  local.set $7
  local.get $1
  i32.load offset=8
  call $~lib/util/number/utoa32
  local.set $8
  i32.const 2544
  i32.const 1
  local.get $0
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 2544
  i32.const 3
  local.get $7
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 2544
  i32.const 5
  local.get $8
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 2544
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  i32.const 0
  local.set $0
  loop $for-loop|0
   local.get $0
   local.get $1
   i32.load offset=8
   i32.lt_u
   if
    block $for-continue|0
     i32.const 4096
     call $~lib/typedarray/Uint8Array#constructor
     local.tee $5
     i32.load offset=4
     i32.const 0
     i32.const 4096
     i32.const 15
     local.get $0
     i32.const 0
     call $~lib/@fluffylabs/as-lan/ecalli/general/fetch/fetch
     local.tee $3
     i64.const 0
     i64.lt_s
     if
      global.get $assembly/dispatch/common/logger
      i32.const 15
      call $~lib/number/I32#toString
      local.set $8
      local.get $0
      call $~lib/util/number/utoa32
      local.set $5
      local.get $3
      call $~lib/number/I64#toString
      local.set $9
      i32.const 2704
      i32.const 1
      local.get $8
      call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
      i32.const 2704
      i32.const 3
      local.get $5
      call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
      i32.const 2704
      i32.const 5
      local.get $9
      call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
      i32.const 2704
      call $~lib/staticarray/StaticArray<~lib/string/String>#join
      call $~lib/@fluffylabs/as-lan/logger/Logger#warn
      br $for-continue|0
     end
     local.get $5
     i32.const 0
     i64.const 4096
     local.get $3
     local.get $3
     i64.const 4096
     i64.gt_s
     select
     i32.wrap_i64
     call $~lib/typedarray/Uint8Array#subarray
     call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob
     local.tee $5
     call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
     local.set $7
     local.get $5
     i32.load8_u offset=4
     if
      global.get $assembly/dispatch/common/logger
      i32.const 2848
      i32.const 1
      local.get $0
      call $~lib/util/number/utoa32
      call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
      i32.const 2848
      call $~lib/staticarray/StaticArray<~lib/string/String>#join
      call $~lib/@fluffylabs/as-lan/logger/Logger#warn
      br $for-continue|0
     end
     local.get $7
     if
      local.get $7
      i32.const 1
      i32.eq
      if
       local.get $5
       local.get $0
       call $assembly/accumulate/processTransfer
       local.set $2
      else
       global.get $assembly/dispatch/common/logger
       local.get $7
       call $~lib/util/number/utoa32
       local.set $7
       local.get $0
       call $~lib/util/number/utoa32
       local.set $8
       i32.const 7024
       i32.const 1
       local.get $7
       call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
       i32.const 7024
       i32.const 3
       local.get $8
       call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
       i32.const 7024
       call $~lib/staticarray/StaticArray<~lib/string/String>#join
       call $~lib/@fluffylabs/as-lan/logger/Logger#warn
      end
     else
      local.get $5
      local.get $0
      call $assembly/accumulate/processOperand
      local.set $2
     end
    end
    local.get $0
    i32.const 1
    i32.add
    local.set $0
    br $for-loop|0
   end
  end
  local.get $2
 )
 (func $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>#constructor" (param $0 i32) (param $1 i32) (param $2 i32) (result i32)
  local.get $0
  local.get $1
  local.get $2
  i32.const 20
  call $"byn$mgfn-shared$~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor"
 )
 (func $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>" (param $0 i32) (result i32)
  i32.const 0
  i32.const 0
  local.get $0
  call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>#constructor"
 )
 (func $assembly/refine/refine (param $0 i32) (param $1 i32) (result i64)
  (local $2 i64)
  (local $3 i64)
  (local $4 i64)
  (local $5 i32)
  (local $6 i32)
  (local $7 i32)
  (local $8 i32)
  block $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1 (result i32)
   local.get $0
   local.get $1
   call $~lib/@fluffylabs/as-lan/core/mem/readFromMemory
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob
   local.tee $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $3
   i64.const 65535
   i64.gt_u
   if
    i32.const 0
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $4
   i64.const 4294967295
   i64.gt_u
   if
    i32.const 1
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
   local.tee $2
   i64.const 4294967295
   i64.gt_u
   if
    i32.const 2
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
   local.set $1
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
   local.set $5
   local.get $0
   i32.load8_u offset=4
   if
    i32.const 5
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1
   end
   local.get $0
   call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#isFinished
   i32.eqz
   if
    i32.const 6
    call $"~lib/@fluffylabs/as-lan/core/result/Result.err<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>"
    br $__inlined_func$~lib/@fluffylabs/as-lan/jam/service/RefineArgs.parse$1
   end
   i32.const 20
   i32.const 19
   call $~lib/rt/stub/__new
   local.tee $0
   local.get $3
   i64.store16
   local.get $0
   local.get $4
   i32.wrap_i64
   call $~lib/rt/common/OBJECT#set:gcInfo
   local.get $0
   local.get $2
   i32.wrap_i64
   call $~lib/rt/common/OBJECT#set:gcInfo2
   local.get $0
   local.get $1
   call $~lib/rt/common/OBJECT#set:rtId
   local.get $0
   local.get $5
   call $~lib/@fluffylabs/as-lan/jam/accumulate/item/PendingTransfer#set:memo
   i32.const 1
   local.get $0
   i32.const 0
   call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/RefineArgs,i32>#constructor"
  end
  local.tee $0
  i32.load8_u
  if
   global.get $assembly/dispatch/common/logger
   i32.const 7072
   local.get $0
   i32.load offset=8
   call $~lib/number/I32#toString
   call $~lib/string/String#concat
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  local.get $0
  i32.load offset=4
  local.tee $5
  i32.load16_u
  call $~lib/util/number/utoa32
  local.set $6
  local.get $5
  i32.load offset=4
  call $~lib/util/number/utoa32
  local.set $7
  local.get $5
  i32.load offset=8
  call $~lib/util/number/utoa32
  local.set $0
  local.get $5
  i32.load offset=16
  call $~lib/@fluffylabs/as-lan/core/bytes/Bytes32#toString
  local.set $8
  i32.const 7280
  i32.const 1
  local.get $6
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 7280
  i32.const 3
  local.get $7
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 7280
  i32.const 5
  local.get $0
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 7280
  i32.const 7
  local.get $8
  call $~lib/staticarray/StaticArray<~lib/string/String>#__uset
  i32.const 7280
  call $~lib/staticarray/StaticArray<~lib/string/String>#join
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $5
  i32.load offset=12
  i32.load
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder.fromBlob
  local.tee $0
  call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU64
  local.set $2
  local.get $0
  i32.load8_u offset=4
  if
   global.get $assembly/dispatch/common/logger
   i32.const 7344
   call $~lib/@fluffylabs/as-lan/logger/Logger#warn
   i64.const 0
   return
  end
  global.get $assembly/dispatch/common/logger
  i32.const 7440
  local.get $2
  call $~lib/number/U64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#info
  local.get $2
  i64.eqz
  if
   call $assembly/dispatch/general/dispatchGas
   return
  end
  local.get $2
  i64.const 1
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchFetch
   return
  end
  local.get $2
  i64.const 2
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchLookup
   return
  end
  local.get $2
  i64.const 3
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchRead
   return
  end
  local.get $2
  i64.const 4
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchWrite
   return
  end
  local.get $2
  i64.const 5
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchInfo
   return
  end
  local.get $2
  i64.const 100
  i64.eq
  if
   local.get $0
   call $assembly/dispatch/general/dispatchLog
   return
  end
  local.get $2
  i64.const 6
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchHistoricalLookup$281 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytes32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $6
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $7
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 7504
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchHistoricalLookup$281
    end
    local.get $7
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    i32.load offset=4
    i32.load offset=4
    local.get $0
    i32.load offset=4
    local.get $6
    local.get $7
    call $~lib/@fluffylabs/as-lan/ecalli/refine/historical_lookup/historical_lookup
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 7616
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    local.get $0
    i32.const 0
    local.get $2
    local.get $6
    local.get $7
    call $assembly/dispatch/common/outputLen
    call $~lib/typedarray/Uint8Array#subarray
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 7
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchExport$282 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.set $1
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 7680
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchExport$282
    end
    local.get $1
    i32.load
    i32.load offset=4
    local.get $1
    i32.load
    i32.load offset=8
    call $~lib/@fluffylabs/as-lan/ecalli/refine/export/export_
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 7760
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 8
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchMachine$283 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.set $1
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $5
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 7808
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchMachine$283
    end
    local.get $1
    i32.load
    i32.load offset=4
    local.get $1
    i32.load
    i32.load offset=8
    local.get $5
    call $~lib/@fluffylabs/as-lan/ecalli/refine/machine/machine
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 7904
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 9
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchPeek$284 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $5
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $6
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 7952
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchPeek$284
    end
    local.get $6
    call $~lib/typedarray/Uint8Array#constructor
    local.tee $0
    i32.load offset=4
    local.get $5
    local.get $6
    call $~lib/@fluffylabs/as-lan/ecalli/refine/peek/peek
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 8032
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i64.eqz
    if
     local.get $2
     local.get $0
     call $~lib/@fluffylabs/as-lan/jam/service/Response.with
     br $__inlined_func$assembly/dispatch/refine/dispatchPeek$284
    end
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 10
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchPoke$285 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.set $5
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.set $6
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 8080
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchPoke$285
    end
    local.get $5
    i32.load
    i32.load offset=4
    local.get $6
    local.get $5
    i32.load
    i32.load offset=8
    call $~lib/@fluffylabs/as-lan/ecalli/refine/poke/poke
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 8160
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 11
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchPages$245 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 8208
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchPages$245
    end
    call $~lib/@fluffylabs/as-lan/ecalli/refine/pages/pages
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 8288
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 12
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchInvoke$286 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#bytesVarLen
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 8336
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchInvoke$286
    end
    i32.const 8
    call $~lib/typedarray/Uint8Array#constructor
    local.set $0
    i32.load
    i32.load offset=4
    local.get $0
    i32.load offset=4
    call $~lib/@fluffylabs/as-lan/ecalli/refine/invoke/invoke
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 8416
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    local.get $0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  local.get $2
  i64.const 13
  i64.eq
  if
   block $__inlined_func$assembly/dispatch/refine/dispatchExpunge$246 (result i64)
    local.get $0
    call $~lib/@fluffylabs/as-lan/core/codec/decode/Decoder#varU32
    local.get $0
    i32.load8_u offset=4
    if
     global.get $assembly/dispatch/common/logger
     i32.const 8464
     call $~lib/@fluffylabs/as-lan/logger/Logger#warn
     i64.const 0
     br $__inlined_func$assembly/dispatch/refine/dispatchExpunge$246
    end
    call $~lib/@fluffylabs/as-lan/ecalli/refine/expunge/expunge
    local.set $2
    global.get $assembly/dispatch/common/logger
    i32.const 8560
    local.get $2
    call $~lib/number/I64#toString
    call $~lib/string/String#concat
    call $~lib/@fluffylabs/as-lan/logger/Logger#info
    local.get $2
    i32.const 0
    call $~lib/@fluffylabs/as-lan/jam/service/Response.with
   end
   return
  end
  global.get $assembly/dispatch/common/logger
  i32.const 8608
  local.get $2
  call $~lib/number/U64#toString
  call $~lib/string/String#concat
  call $~lib/@fluffylabs/as-lan/logger/Logger#warn
  i64.const 0
 )
 (func $~start
  (local $0 i32)
  i32.const 1056
  call $~lib/string/String#charCodeAt
  global.set $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_0
  i32.const 1088
  call $~lib/string/String#charCodeAt
  drop
  i32.const 1120
  call $~lib/string/String#charCodeAt
  global.set $~lib/@fluffylabs/as-lan/core/bytes/CODE_OF_a
  i32.const 1152
  call $~lib/string/String#charCodeAt
  drop
  i32.const 1184
  call $~lib/string/String#charCodeAt
  drop
  i32.const 1216
  call $~lib/string/String#charCodeAt
  drop
  i32.const 8652
  global.set $~lib/rt/stub/offset
  i32.const 4
  i32.const 5
  call $~lib/rt/stub/__new
  local.tee $0
  i32.const 0
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
  i32.const 1328
  call $~lib/@fluffylabs/as-lan/logger/Logger#set:target
  local.get $0
  global.set $assembly/dispatch/common/logger
 )
 (func $"byn$mgfn-shared$~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#constructor" (param $0 i32) (param $1 i32) (param $2 i32) (param $3 i32) (result i32)
  i32.const 12
  local.get $3
  call $~lib/rt/stub/__new
  local.tee $3
  local.get $0
  i32.store8 offset=1
  local.get $3
  local.get $1
  call $~lib/rt/common/OBJECT#set:gcInfo
  local.get $3
  local.get $2
  call $~lib/rt/common/OBJECT#set:gcInfo2
  local.get $3
  i32.const 0
  call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#set:isError"
  local.get $3
  local.get $0
  i32.eqz
  call $"~lib/@fluffylabs/as-lan/core/result/Result<~lib/@fluffylabs/as-lan/jam/service/AccumulateArgs,i32>#set:isError"
  local.get $3
 )
)
