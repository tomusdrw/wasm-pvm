import { defineSuite } from "../helpers/suite";

// Regression test for issue #256: the div-by-zero trap-bypass label
// (used by i32.div_u/rem_u) used to call `define_label`, which clears
// the emitter's `alloc_reg_slot` state. With lazy spill on, the
// loop-carried `value` phi was held in an allocated register; the
// back-edge spill was elided, so when the cleared alloc state forced a
// reload from the (never-written) stack slot, the loop read garbage
// and either spun forever or returned a wrong sum.
//
// The fixture uses `value` TWICE per iteration around two trap
// bypasses (rem_u then div_u), plus extra live phis to raise register
// pressure so the compiler keeps `value` in a callee-saved register.
//
// args layout (5 i32, little-endian): value, ext_a, ext_b, ext_c, ext_d.
// Result = digit_sum(value). The extras are rotated through xor/add per
// loop iteration and parked to memory but are not part of the returned
// value (the WAT only stores `$sum` at address 0).
//
// For value=12345: digit sum = 5+4+3+2 = 14.
// For value=10: digit sum = 0 (10%10 = 0).
const tests = [
  {
    args: "01000000" + "00000000".repeat(4),
    expected: 0,
    description: "value=1: loop never runs, sum=0",
  },
  {
    args: "09000000" + "00000000".repeat(4),
    expected: 0,
    description: "value=9: boundary, sum=0",
  },
  {
    args: "0a000000" + "00000000".repeat(4),
    expected: 0,
    description: "value=10: one iter; 10%10=0",
  },
  {
    args: "39300000" + "00000000".repeat(4),
    expected: 14,
    description: "value=12345: 5+4+3+2=14",
  },
  // value=4294967295 → digits 4,2,9,4,9,6,7,2,9,5; loop visits the bottom 9
  // (4 < 10 exits) and sums 5+9+2+7+6+9+4+9+2 = 53.
  {
    args: "ffffffff" + "00000000".repeat(4),
    expected: 53,
    description: "value=u32::MAX: bottom 9 digits sum",
  },
];

defineSuite({
  name: "div-zero-trap-clobber",
  tests: tests,
});
