import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 30, description: "AS: 10+20 (both<128)" },
  { args: "01", expected: 128, description: "AS: 127+1" },
  { args: "02", expected: 129, description: "AS: 1+128 (second>=128)" },
  { args: "03", expected: 129, description: "AS: 128+1 (first>=128)" },
  { args: "04", expected: 256, description: "AS: 128+128" },
  { args: "05", expected: 128, description: "AS: arr[0] only" },
  { args: "06", expected: 159, description: "AS: arr[1] only" },
  { args: "07", expected: 32927, description: "AS: arr[0]<<8|arr[1]" },
  { args: "08", expected: 487, description: "AS: 3-elem sum" },
  { args: "09", expected: 287, description: "AS: reverse store order" },
];

defineSuite({
  name: "as-u8-two-elem-test",
  tests: tests,
});
