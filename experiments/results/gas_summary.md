# Optimization Gas + Size Impact Summary

Runnable inputs measured: 19

Positive delta = optimization saves bytes / gas when **on** (turning it off makes the JAM bigger / costs more gas).

**Baseline OOM:** `aslan-ecalli`. These fixtures exhausted the gas budget under the all-opts-on baseline. For most variants `delta_gas` is structurally 0 (variant also OOMs). A few variants make the program halt cleanly with very different gas usage — those represent **behavioral** changes (different halt condition), not per-instruction gas efficiency, so they are excluded from per-flag aggregates and surfaced separately in the "Behavioral cases" section.

## Per-flag aggregate

| Flag | Total ΔJAM | Total ΔGas | JAM regr. | Gas regr. | Sign disagreements | Run failures |
|------|-----------:|-----------:|----------:|----------:|-------------------:|-------------:|
| `--no-fallthrough-jumps` | +1700 | +10326 | 2 | 4 | 5 | 0 |
| `--no-aggressive-regalloc` | -75 | +8471 | 8 | 9 | 5 | 0 |
| `--no-lazy-spill` | +84 | -18702 | 7 | 7 | 3 | 0 |
| `--no-register-alloc` | +1002 | +12861 | 3 | 2 | 2 | 0 |
| `--no-scratch-reg-alloc` | +284 | +1037 | 4 | 6 | 2 | 0 |
| `--no-libcall-recognition` | -7 | +85000 | 2 | 1 | 1 | 0 |
| `--no-caller-saved-alloc` | -6 | -73 | 7 | 5 | 1 | 0 |
| `--no-register-cache` | +14535 | +80562 | 0 | 0 | 0 | 0 |
| `--no-dead-store-elim` | +10922 | +62850 | 0 | 0 | 0 | 0 |
| `--no-icmp-fusion` | +1662 | +17574 | 0 | 0 | 0 | 0 |
| `--no-cross-block-cache` | +873 | +8655 | 0 | 0 | 0 | 0 |
| `--no-inline` | +447 | +3946 | 1 | 0 | 0 | 0 |
| `--no-shrink-wrap` | +235 | +40 | 0 | 0 | 0 | 0 |
| `--no-const-prop` | +94 | +13 | 0 | 0 | 0 | 0 |
| `--no-peephole` | +4 | +1 | 0 | 0 | 0 | 0 |
| `--no-mergefunc` | +0 | +0 | 0 | 0 | 0 | 0 |

## Sign-disagreement cells (optimization helps one axis, hurts the other)

Rows where ΔJAM and ΔGas have opposite signs (both non-zero). These are the cases where 'is this opt worth keeping?' depends on which axis you care about.

| Flag | Fixture | ΔJAM | ΔGas | Baseline JAM | Baseline Gas | Status |
|------|---------|-----:|-----:|-------------:|-------------:|--------|
| `--no-fallthrough-jumps` | regalloc-two-loops | +16 | -497 | 561 | 34073 | ok |
| `--no-fallthrough-jumps` | sha512 | +63 | -545 | 3511 | 14459 | ok |
| `--no-fallthrough-jumps` | sha512-240a | +63 | -1631 | 3511 | 43111 | ok |
| `--no-fallthrough-jumps` | as-decoder-test | -14 | +2 | 6523 | 912 | ok |
| `--no-fallthrough-jumps` | aslan-fib | +456 | -116 | 19831 | 10431 | ok |
| `--no-aggressive-regalloc` | blake2b | +20 | -186 | 3765 | 14819 | ok |
| `--no-aggressive-regalloc` | sha512 | +46 | -82 | 3511 | 14459 | ok |
| `--no-aggressive-regalloc` | sha512-240a | +46 | -244 | 3511 | 43111 | ok |
| `--no-aggressive-regalloc` | u128-div-bench | -9 | +1996 | 765 | 68031 | ok |
| `--no-aggressive-regalloc` | u128-div-bench-slow | -9 | +1996 | 771 | 129031 | ok |
| `--no-lazy-spill` | blake2b | +54 | -317 | 3765 | 14819 | ok |
| `--no-lazy-spill` | u128-div-bench | -27 | +1001 | 765 | 68031 | ok |
| `--no-lazy-spill` | as-array-test | -81 | +1 | 5923 | 779 | ok |
| `--no-register-alloc` | u128-div-bench | -11 | +2998 | 765 | 68031 | ok |
| `--no-register-alloc` | as-array-test | -56 | +1 | 5923 | 779 | ok |
| `--no-scratch-reg-alloc` | u128-mul-bench | +2 | -1000 | 457 | 71031 | ok |
| `--no-scratch-reg-alloc` | aslan-fib | +71 | -76 | 19831 | 10431 | ok |
| `--no-libcall-recognition` | u128-div-bench | -82 | +53000 | 765 | 68031 | ok |
| `--no-caller-saved-alloc` | fibonacci | -2 | +18 | 221 | 429 | ok |

## Behavioral cases (baseline OOM, variant halts)

These rows have huge `ΔGas` magnitudes but are excluded from per-flag aggregates above. The baseline run exhausts the gas budget; the listed variant halts cleanly at a much lower gas count. The delta is real data, but reflects a **different halt condition** under the variant — not per-instruction gas savings. See issue #256 for the underlying investigation.

| Flag | Fixture | ΔJAM | Baseline Gas | Variant Gas | ΔGas |
|------|---------|-----:|-------------:|------------:|-----:|
| `--no-lazy-spill` | aslan-ecalli | 50 | 100000000 | 6604 | -99993396 |
| `--no-register-alloc` | aslan-ecalli | 173 | 100000000 | 6683 | -99993317 |
| `--no-shrink-wrap` | aslan-ecalli | 53 | 100000000 | 6491 | -99993509 |

## Per-fixture gas deltas (positive = opt saves gas)

| Flag | add | fibonacci | factorial | is-prime | regalloc-two-loops | blake2b | sha512 | sha512-240a | u128-mul-bench | u128-div-bench | u128-div-bench-slow | host-call-log | as-fibonacci | as-factorial | as-gcd | as-decoder-test | as-array-test | aslan-fib | aslan-ecalli |
|------|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `--no-fallthrough-jumps` | 0 | 1 | 22 | 1 | -497 | 14 | -545 | -1631 | 5002 | 3001 | 5001 | 0 | 33 | 30 | 8 | 2 | 0 | -116 | 0 |
| `--no-aggressive-regalloc` | 9 | 81 | -19 | 2 | -2003 | -186 | -82 | -244 | 6996 | 1996 | 1996 | -1 | 41 | 15 | 2 | -6 | -9 | -117 | 0 |
| `--no-lazy-spill` | -2 | -2 | -2 | 0 | 498 | -317 | 241 | 723 | -11999 | 1001 | -8999 | 0 | 0 | 0 | 0 | -4 | 1 | 159 | -99993396 |
| `--no-register-alloc` | 7 | 142 | 64 | 9 | 6500 | 360 | 1352 | 4062 | 1998 | 2998 | -5002 | -1 | 43 | 33 | 5 | 3 | 1 | 287 | -99993317 |
| `--no-scratch-reg-alloc` | 0 | 101 | 20 | 0 | 4001 | 0 | 0 | 0 | -1000 | -1000 | -1000 | 0 | 0 | 0 | 0 | -5 | -4 | -76 | 0 |
| `--no-libcall-recognition` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 40000 | 53000 | -8000 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `--no-caller-saved-alloc` | 0 | 18 | -41 | -2 | 997 | 384 | 144 | 432 | -2000 | 0 | 0 | 0 | 43 | -26 | 0 | 6 | 5 | -33 | 0 |
| `--no-register-cache` | 1 | 163 | 64 | 4 | 17008 | 5413 | 7496 | 22421 | 4000 | 6000 | 15000 | 0 | 90 | 62 | 42 | 53 | 67 | 2678 | 0 |
| `--no-dead-store-elim` | 10 | 124 | 54 | 14 | 6506 | 3226 | 4144 | 12394 | 9003 | 8003 | 17003 | 5 | 43 | 41 | 25 | 162 | 131 | 1962 | 0 |
| `--no-icmp-fusion` | 0 | 21 | 11 | 8 | 1002 | 26 | 171 | 500 | 3003 | 6003 | 6003 | 0 | 15 | 12 | 19 | 14 | 17 | 749 | 0 |
| `--no-cross-block-cache` | 0 | 63 | 34 | 3 | 7004 | 42 | 274 | 822 | 0 | 0 | 0 | 0 | 41 | 25 | 10 | 6 | 5 | 326 | 0 |
| `--no-inline` | 0 | 0 | 0 | 0 | 0 | 0 | 694 | 1722 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 815 | 665 | 50 | 0 |
| `--no-shrink-wrap` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 | 8 | 8 | 8 | 8 | 0 | -99993509 |
| `--no-const-prop` | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 8 | 0 |
| `--no-peephole` | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `--no-mergefunc` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

## Per-fixture JAM deltas (positive = opt saves bytes)

| Flag | add | fibonacci | factorial | is-prime | regalloc-two-loops | blake2b | sha512 | sha512-240a | u128-mul-bench | u128-div-bench | u128-div-bench-slow | host-call-log | as-fibonacci | as-factorial | as-gcd | as-decoder-test | as-array-test | aslan-fib | aslan-ecalli |
|------|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `--no-fallthrough-jumps` | 0 | 11 | 18 | 16 | 16 | 60 | 63 | 63 | 24 | 32 | 32 | 0 | 25 | 34 | 21 | -14 | -6 | 456 | 849 |
| `--no-aggressive-regalloc` | 31 | 13 | -8 | 14 | -18 | 20 | 46 | 46 | 14 | -9 | -9 | 0 | 18 | 3 | 5 | -56 | -95 | -7 | -83 |
| `--no-lazy-spill` | -6 | 0 | -3 | 11 | 7 | 54 | 8 | 8 | -37 | -27 | -27 | 0 | 0 | 5 | 4 | -27 | -81 | 145 | 50 |
| `--no-register-alloc` | 25 | 32 | 30 | 37 | 87 | 59 | 122 | 122 | 16 | -11 | -11 | 0 | 32 | 36 | 18 | 20 | -56 | 271 | 173 |
| `--no-scratch-reg-alloc` | 0 | 16 | 8 | 0 | 50 | 0 | 0 | 0 | 2 | -2 | -2 | 0 | 0 | 0 | 0 | -2 | -2 | 71 | 145 |
| `--no-libcall-recognition` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 157 | -82 | -82 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `--no-caller-saved-alloc` | 0 | -2 | -14 | -7 | 23 | 19 | 13 | 13 | -4 | 0 | 0 | 0 | 32 | -18 | 6 | 2 | 2 | -9 | -62 |
| `--no-register-cache` | 4 | 36 | 32 | 17 | 180 | 649 | 706 | 706 | 14 | 79 | 79 | 0 | 135 | 126 | 133 | 1187 | 995 | 3647 | 5810 |
| `--no-dead-store-elim` | 34 | 30 | 26 | 45 | 70 | 437 | 508 | 508 | 45 | 86 | 86 | 17 | 79 | 79 | 78 | 837 | 673 | 2515 | 4769 |
| `--no-icmp-fusion` | 0 | 4 | 5 | 12 | 7 | 16 | 29 | 29 | 9 | 18 | 18 | 0 | 15 | 16 | 21 | 80 | 71 | 438 | 874 |
| `--no-cross-block-cache` | 0 | 14 | 18 | 10 | 70 | 32 | 28 | 28 | 0 | 0 | 0 | 0 | 18 | 23 | 13 | 76 | 40 | 127 | 376 |
| `--no-inline` | 0 | 0 | 0 | 0 | 0 | 0 | 166 | 166 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 173 | 151 | 317 | -526 |
| `--no-shrink-wrap` | 0 | 0 | 0 | 0 | 0 | 2 | 5 | 5 | 2 | 5 | 5 | 0 | 31 | 31 | 30 | 33 | 32 | 1 | 53 |
| `--no-const-prop` | 0 | 0 | 0 | 0 | 0 | 23 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 67 | 0 |
| `--no-peephole` | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 |
| `--no-mergefunc` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
