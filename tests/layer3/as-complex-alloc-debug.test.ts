import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 45, description: "AS: Step 0 - Decoder.u8() sum 0-9" },
  { args: "01", expected: 190, description: "AS: Step 1 - Decoder.bytes() sum 0-19" },
  { args: "02", expected: 45, description: "AS: Step 2 - lowerBytes sum 0-9" },
  { args: "03", expected: 15, description: "AS: Step 3 - pre-alloc array 1+2+3+4+5" },
  { args: "04", expected: 795, description: "AS: Step 4 - multiple allocs sum" },
  { args: "05", expected: 1090, description: "AS: Step 5+ - total sum" },
];

defineSuite({
  name: "as-complex-alloc-debug",
  tests: tests,
});
