/**
 * Differential testing: PVM vs native WASM.
 *
 * Imports all suites from layers 1-2 to populate the registry, then runs
 * each test case through both the PVM (via anan-as) and Bun's native
 * WebAssembly engine (JavaScriptCore). Asserts that both produce the same
 * result (value match or both trap).
 *
 * Modules with function imports (e.g. host-call-log, AS modules with
 * env.abort) are automatically skipped because they cannot be instantiated
 * without providing host stubs.
 *
 * Run with:
 *   cd tests && bun test differential/ --test-name-pattern "differential"
 */

// --- Layer 1 imports ---
import "../layer1/add.test";
import "../layer1/call-indirect.test";
import "../layer1/call.test";
import "../layer1/div.test";
import "../layer1/factorial.test";
import "../layer1/fibonacci.test";
import "../layer1/gcd.test";
import "../layer1/is-prime.test";
import "../layer1/start-section.test";

// --- Layer 2 imports (WAT-based suites; AS suites auto-skipped via import check) ---
import "../layer2/bit-ops.test";
import "../layer2/block-br-test.test";
import "../layer2/block-result.test";
import "../layer2/br-table.test";
import "../layer2/compare-test.test";
import "../layer2/computed-addr-test.test";
import "../layer2/entry-points.test";
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
import "../layer2/memory-copy-word.test";
import "../layer2/phi-cycles.test";

// --- Generate differential test variants from registry ---
import {
  getRegisteredSuites,
  defineDifferentialSuite,
} from "../helpers/suite";

const suites = getRegisteredSuites();
for (const suite of suites) {
  defineDifferentialSuite(suite);
}
