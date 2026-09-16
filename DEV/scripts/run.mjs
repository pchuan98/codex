import process from "node:process";
import { launchCodex } from "./npm/bin/launch.js";

process.exit(launchCodex(process.argv[2], process.argv.slice(3)));
