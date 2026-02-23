import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 0, description: "AS: arr[0] = 0" },
  { args: "01", expected: 1, description: "AS: arr[1] = 1" },
  { args: "02", expected: 2, description: "AS: arr[2] = 2" },
  { args: "03", expected: 10, description: "AS: arr[0] + arr[1]*10 = 10" },
  { args: "04", expected: 1234, description: "AS: loop concat digits = 1234" },
  { args: "05", expected: 1, description: "AS: v0*10 + v1 = 1" },
  { args: "06", expected: 3, description: "AS: sum(0,1,2) = 3" },
  { args: "07", expected: 3, description: "AS: loop sum(0,1,2) = 3" },
  { args: "08", expected: 1, description: "AS: direct memory arr[1] = 1" },
];

defineSuite({
  name: "as-array-value-test",
  // pvm-in-pvm currently returns an empty outer result buffer (r7 == r8)
  // for this suite when executed through anan-as-compiler.jam.
  skipPvmInPvm: true,
  tests: tests,
});
