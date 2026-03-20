import { defineSuite } from "../helpers/suite";

defineSuite({
  name: "regalloc-call-clobber",
  tests: [
    { args: "00", expected: 100, description: "step=0 after call returns 100" },
    { args: "01", expected: 200, description: "step=1 after call returns 200" },
    { args: "02", expected: 300, description: "step=2 after call returns 300" },
  ],
});
