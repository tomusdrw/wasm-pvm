import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0102030405060708", expected: 36, description: "AS: Array.push() from args - sum of 8 bytes" },
  { args: "0a141e28323c4650", expected: 360, description: "AS: Array.push() from args - larger values" },
];

defineSuite({
  name: "as-array-push-args-test",
  tests: tests,
});
