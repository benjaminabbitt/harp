#!/usr/bin/env node
/**
 * CLI example using harp-wasm bindings.
 */

import { parseArgs } from "node:util";
import { generate_name, version } from "../../pkg/harp_wasm.js";

const { values } = parseArgs({
  options: {
    count: {
      type: "string",
      short: "n",
      default: "1",
    },
    version: {
      type: "boolean",
      short: "v",
      default: false,
    },
    help: {
      type: "boolean",
      short: "h",
      default: false,
    },
  },
});

if (values.help) {
  console.log(`Usage: harp-cli [options]

Options:
  -n, --count <number>  Number of names to generate (default: 1)
  -v, --version         Show version and exit
  -h, --help            Show this help message`);
  process.exit(0);
}

if (values.version) {
  console.log(`harp ${version()}`);
  process.exit(0);
}

const count = parseInt(values.count, 10);
for (let i = 0; i < count; i++) {
  console.log(generate_name());
}
