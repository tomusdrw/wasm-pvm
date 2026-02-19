/**
 * Comprehensive PVM-in-PVM tests (layer 5).
 *
 * Imports all test suites from layers 1-3 to populate the suite registry,
 * then creates pvm-in-pvm test variants for each registered suite.
 *
 * Note: importing test files also registers their normal test cases,
 * so running `bun test layer5/` will run both normal and pvm-in-pvm tests.
 * To run only pvm-in-pvm tests use:
 *   bun test layer5/ --test-name-pattern "pvm-in-pvm"
 */

// --- Layer 1 imports ---
import "../layer1/add.test";
import "../layer1/as-add.test";
import "../layer1/as-factorial.test";
import "../layer1/as-fibonacci.test";
import "../layer1/as-gcd.test";
import "../layer1/call-indirect.test";
import "../layer1/call.test";
import "../layer1/div.test";
import "../layer1/factorial.test";
import "../layer1/fibonacci.test";
import "../layer1/gcd.test";
import "../layer1/is-prime.test";
import "../layer1/start-section.test";

// --- Layer 2 imports ---
import "../layer2/as-alloc-test-incremental.test";
import "../layer2/as-alloc-test-minimal.test";
import "../layer2/as-alloc-test-stub.test";
import "../layer2/as-array-push-args-test.test";
import "../layer2/as-array-push-test.test";
import "../layer2/as-array-test.test";
import "../layer2/as-complex-alloc-test.test";
import "../layer2/as-decoder-subarray-test.test";
import "../layer2/as-decoder-test.test";
import "../layer2/as-life.test";
import "../layer2/as-memory-args-test.test";
import "../layer2/as-mini-pvm-runner.test";
import "../layer2/as-nested-memory-test.test";
import "../layer2/as-subarray-offset-test.test";
import "../layer2/as-subarray-test.test";
import "../layer2/as-tests-arithmetic.test";
import "../layer2/as-tests-arrays.test";
import "../layer2/as-tests-comparisons.test";
import "../layer2/as-tests-control-flow.test";
import "../layer2/as-tests-fun-ptr.test";
import "../layer2/as-tests-functions.test";
import "../layer2/as-tests-globals.test";
import "../layer2/as-tests-linked-list.test";
import "../layer2/as-tests-memory.test";
import "../layer2/as-tests-structs.test";
import "../layer2/as-varU32-test.test";
import "../layer2/bit-ops.test";
import "../layer2/block-br-test.test";
import "../layer2/block-result.test";
import "../layer2/br-table.test";
import "../layer2/compare-test.test";
import "../layer2/computed-addr-test.test";
import "../layer2/entry-points.test";
import "../layer2/host-call-log.test";
import "../layer2/i64-ops.test";
import "../layer2/loop-offset-store-test.test";
import "../layer2/many-locals-call-test.test";
import "../layer2/many-locals.test";
import "../layer2/memory-copy-overlap.test";
import "../layer2/nested-calls.test";
import "../layer2/recursive.test";
import "../layer2/rotate.test";
import "../layer2/simple-memory-test.test";
import "../layer2/stack-test.test";

// --- Layer 3 imports ---
import "../layer3/as-alloc-loop-debug.test";
import "../layer3/as-array-length-loop-test.test";
import "../layer3/as-array-value-test.test";
import "../layer3/as-array-value-trace-test.test";
import "../layer3/as-complex-alloc-debug.test";
import "../layer3/as-count-vs-sum-test.test";
import "../layer3/as-debug-call-test.test";
import "../layer3/as-flat-ternary-drop.test";
import "../layer3/as-flat-ternary-test.test";
import "../layer3/as-if-result-test.test";
import "../layer3/as-iteration-count-test.test";
import "../layer3/as-largebuf-subarray-test.test";
import "../layer3/as-limit-source-test.test";
import "../layer3/as-local-clobber-test.test";
import "../layer3/as-local-preserve-test.test";
import "../layer3/as-loop-counter-test.test";
import "../layer3/as-lowerBytes-test.test";
import "../layer3/as-memload-condition-test.test";
import "../layer3/as-minimal-fail.test";
import "../layer3/as-minimal-nested-drop-test.test";
import "../layer3/as-minimal-repro.test";
import "../layer3/as-multi-slice-debug.test";
import "../layer3/as-nested-if-test.test";
import "../layer3/as-nested-repro.test";
import "../layer3/as-noinline-call-test.test";
import "../layer3/as-second-loop-test.test";
import "../layer3/as-simple-call-test.test";
import "../layer3/as-simpler-repro.test";
import "../layer3/as-trace-loop-test.test";
import "../layer3/as-two-arg-call-test.test";
import "../layer3/as-u8-store-test.test";
import "../layer3/as-u8-two-elem-test.test";

// --- Generate pvm-in-pvm variants from registry ---
import { getRegisteredSuites, definePvmInPvmSuite } from "../helpers/suite";

const suites = getRegisteredSuites();
for (const suite of suites) {
  definePvmInPvmSuite(suite);
}
