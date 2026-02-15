import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "e80300000000000000000000040000000200000011223344aabb", expected: 286331153, description: "AS: mini-pvm-runner - marker check" },
];

defineSuite({
  name: "as-mini-pvm-runner",
  tests: tests,
});
