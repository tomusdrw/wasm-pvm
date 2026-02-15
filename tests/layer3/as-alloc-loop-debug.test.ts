import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 31, description: "AS: round 0: 0+31" },
  { args: "01", expected: 95, description: "AS: round 1: 32+63" },
  { args: "02", expected: 159, description: "AS: round 2: 64+95" },
  { args: "03", expected: 223, description: "AS: round 3: 96+127" },
  { args: "04", expected: 287, description: "AS: round 4: 128+159" },
  { args: "05", expected: 795, description: "AS: full loop sum" },
  { args: "06", expected: 2064543, description: "AS: round values encoded" },
  { args: "07", expected: 159, description: "AS: arithmetic check" },
  { args: "08", expected: 287, description: "AS: direct store/load" },
  { args: "09", expected: 2155978399, description: "AS: loop fill encoded" },
];

defineSuite({
  name: "as-alloc-loop-debug",
  tests: tests,
});
