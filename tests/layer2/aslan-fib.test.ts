import { defineSuite } from "../helpers/suite";

defineSuite({
  name: "aslan-fib",
  skipPvmInPvm: true,
  tests: [
    {
      args: "2a0000",
      expected: 0,
      description: "accumulate_ext with args 0x2a0000",
      pc: 5,
    },
  ],
});
