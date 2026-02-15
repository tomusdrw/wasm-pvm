import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 127, description: "AS: store 127 to Uint8Array" },
  { args: "01", expected: 128, description: "AS: store 128 to Uint8Array" },
  { args: "02", expected: 255, description: "AS: store 255 to Uint8Array" },
  { args: "03", expected: 128, description: "AS: direct memory store 128" },
  { args: "04", expected: 287, description: "AS: 128+159 from array" },
  { args: "05", expected: 128, description: "AS: computed 4*32 to array" },
  { args: "06", expected: 159, description: "AS: computed 4*32+31 to array" },
  { args: "07", expected: 159, description: "AS: computed value only" },
  { args: "08", expected: 255, description: "AS: 127+128 from array" },
  { args: "09", expected: 287, description: "AS: direct memory 128+159" },
];

defineSuite({
  name: "as-u8-store-test",
  tests: tests,
});
