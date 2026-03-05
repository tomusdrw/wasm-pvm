import { defineSuite } from "../helpers/suite";

defineSuite({
  name: "aslan-fib",
  skipPvmInPvm: true,
  tests: [
    {
      // accumulate entry point: args=0x2a (service 42), computes fib(10)=55
      // Result: [status=0x01, fib_result=0x37=55, ...padding]
      // First 4 bytes LE: 0x00003701 = 14081
      args: "2a0000",
      expected: 14081,
      description: "accumulate with service 42 -> fib(10)=55",
      pc: 5,
    },
  ],
});
