import { defineSuite } from "../helpers/suite";

// --- phi-swap: classic 2-variable swap loop ---
// Each iteration swaps a and b. After n swaps:
//   even n -> a unchanged, odd n -> a gets original b
// Input: a (i32 LE), b (i32 LE), n (i32 LE)
defineSuite({
  name: "phi-swap",
  tests: [
    // 0 swaps: a=5, b=3, n=0 -> result = 5
    { args: "050000000300000000000000", expected: 5, description: "swap 0 times: a stays 5" },
    // 1 swap: a=5, b=3, n=1 -> result = 3
    { args: "050000000300000001000000", expected: 3, description: "swap 1 time: a becomes 3" },
    // 2 swaps: a=5, b=3, n=2 -> result = 5
    { args: "050000000300000002000000", expected: 5, description: "swap 2 times: a back to 5" },
    // 3 swaps: a=5, b=3, n=3 -> result = 3
    { args: "050000000300000003000000", expected: 3, description: "swap 3 times: a becomes 3" },
    // 10 swaps (even): a=42, b=99, n=10 -> result = 42
    { args: "2a000000630000000a000000", expected: 42, description: "swap 10 times (even): a stays 42" },
    // 7 swaps (odd): a=42, b=99, n=7 -> result = 99
    { args: "2a0000006300000007000000", expected: 99, description: "swap 7 times (odd): a becomes 99" },
  ],
});

// --- phi-rotate3: three-variable rotation loop ---
// Each iteration: (a, b, c) = (b, c, a)
// After n rotations: n%3==0 -> a, n%3==1 -> b, n%3==2 -> c
// Input: a (i32 LE), b (i32 LE), c (i32 LE), n (i32 LE)
defineSuite({
  name: "phi-rotate3",
  tests: [
    // 0 rotations: a=10, b=20, c=30 -> a=10
    { args: "0a000000140000001e00000000000000", expected: 10, description: "rotate 0: a=10" },
    // 1 rotation: (10,20,30) -> (20,30,10) -> a=20
    { args: "0a000000140000001e00000001000000", expected: 20, description: "rotate 1: a=20 (was b)" },
    // 2 rotations: -> (30,10,20) -> a=30
    { args: "0a000000140000001e00000002000000", expected: 30, description: "rotate 2: a=30 (was c)" },
    // 3 rotations: full cycle -> a=10
    { args: "0a000000140000001e00000003000000", expected: 10, description: "rotate 3: full cycle a=10" },
    // 4 rotations: same as 1 -> a=20
    { args: "0a000000140000001e00000004000000", expected: 20, description: "rotate 4: same as 1, a=20" },
    // 6 rotations: two full cycles -> a=10
    { args: "0a000000140000001e00000006000000", expected: 10, description: "rotate 6: two full cycles a=10" },
    // Different values: a=100, b=200, c=300, n=2 -> a=300
    { args: "64000000c80000002c01000002000000", expected: 300, description: "rotate 2 with 100,200,300: a=300" },
  ],
});

// --- phi-multi-pred: multi-predecessor merge + swap loop ---
// selector chooses initial (a,b), then swap_count swaps.
// sel=0: a=10, b=20 | sel=1: a=100, b=200 | else: a=1000, b=2000
// After n swaps: even -> a, odd -> b
defineSuite({
  name: "phi-multi-pred",
  tests: [
    // sel=0, 0 swaps: a=10
    { args: "0000000000000000", expected: 10, description: "sel=0, 0 swaps: a=10" },
    // sel=0, 1 swap: a=20
    { args: "0000000001000000", expected: 20, description: "sel=0, 1 swap: a=20" },
    // sel=1, 0 swaps: a=100
    { args: "0100000000000000", expected: 100, description: "sel=1, 0 swaps: a=100" },
    // sel=1, 1 swap: a=200
    { args: "0100000001000000", expected: 200, description: "sel=1, 1 swap: a=200" },
    // sel=2, 0 swaps: a=1000
    { args: "0200000000000000", expected: 1000, description: "sel=2, 0 swaps: a=1000" },
    // sel=2, 1 swap: a=2000
    { args: "0200000001000000", expected: 2000, description: "sel=2, 1 swap: a=2000" },
    // sel=0, 2 swaps (even): a=10
    { args: "0000000002000000", expected: 10, description: "sel=0, 2 swaps: a back to 10" },
    // sel=1, 3 swaps (odd): a=200
    { args: "0100000003000000", expected: 200, description: "sel=1, 3 swaps: a=200" },
  ],
});

// --- phi-dependent: mutually dependent variables in a loop ---
// Each iteration: new_a = a + b, new_b = a * 2
// Both depend on old a, creating an interdependent phi cycle.
// Trace for a=1, b=2:
//   iter 0: a=1, b=2
//   iter 1: a=3, b=2
//   iter 2: a=5, b=6
//   iter 3: a=11, b=10
//   iter 4: a=21, b=22
defineSuite({
  name: "phi-dependent",
  tests: [
    // 0 iterations: a=1
    { args: "010000000200000000000000", expected: 1, description: "0 iters: a=1" },
    // 1 iteration: a=1+2=3
    { args: "010000000200000001000000", expected: 3, description: "1 iter: a=3" },
    // 2 iterations: a=3+2=5
    { args: "010000000200000002000000", expected: 5, description: "2 iters: a=5" },
    // 3 iterations: a=5+6=11
    { args: "010000000200000003000000", expected: 11, description: "3 iters: a=11" },
    // 4 iterations: a=11+10=21
    { args: "010000000200000004000000", expected: 21, description: "4 iters: a=21" },
    // Start with a=0, b=1, 5 iters:
    // iter1: a=0+1=1, b=0*2=0
    // iter2: a=1+0=1, b=1*2=2
    // iter3: a=1+2=3, b=1*2=2
    // iter4: a=3+2=5, b=3*2=6
    // iter5: a=5+6=11, b=5*2=10
    { args: "000000000100000005000000", expected: 11, description: "a=0,b=1, 5 iters: a=11" },
    // Start with a=3, b=0, 3 iters:
    // iter1: a=3+0=3, b=3*2=6
    // iter2: a=3+6=9, b=3*2=6
    // iter3: a=9+6=15, b=9*2=18
    { args: "030000000000000003000000", expected: 15, description: "a=3,b=0, 3 iters: a=15" },
  ],
});
