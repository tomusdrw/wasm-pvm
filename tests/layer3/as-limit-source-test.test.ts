import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 10, description: "AS: hardcoded limit=5 (2 arrays)" },
  { args: "0105", expected: 10, description: "AS: limit=5 from args (2 arrays)" },
  { args: "02", expected: 10, description: "AS: limit=5 from local var (2 arrays)" },
  { args: "0305", expected: 10, description: "AS: limit=5 from args (1 array)" },
  { args: "04", expected: 10, description: "AS: explicit val check (2 arrays)" },
  { args: "05", expected: 1, description: "AS: arr1[1] = 1 (2 arrays)" },
  { args: "06", expected: 1, description: "AS: arr1[0]+arr1[1] (2 arrays)" },
  { args: "07", expected: 1, description: "AS: 2 iters hardcoded (2 arrays)" },
  { args: "0802", expected: 1, description: "AS: 2 iters from args (2 arrays)" },
];

defineSuite({
  name: "as-limit-source-test",
  tests: tests,
});
