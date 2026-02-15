import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 45, description: "AS: first slice only" },
  { args: "01", expected: 145, description: "AS: second slice only" },
  { args: "02", expected: 45145, description: "AS: both slices sum1*1000+sum2" },
  { args: "03", expected: 101110, description: "AS: arr2[0]*10k+arr2[1]*100+len" },
  { args: "04", expected: 101110, description: "AS: slice2[0]*10k+slice2[1]*100+len" },
  { args: "05", expected: 910, description: "AS: slice1 after slice2 created" },
  { args: "06", expected: 910, description: "AS: arr1 after both lowerBytes" },
];

defineSuite({
  name: "as-multi-slice-debug",
  tests: tests,
});
