import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0400000002000000deadbeef1234", expected: 4277796864, description: "AS: nested memory test - args_ptr check" },
];

defineSuite({
  name: "as-nested-memory-test",
  tests: tests,
});
