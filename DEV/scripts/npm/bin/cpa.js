#!/usr/bin/env node

import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import process from "node:process";
import { launchCodex } from "./launch.js";

const platformPackages = {
  "win32-x64": "@pchuan98/cpa-win32-x64",
  "linux-x64": "@pchuan98/cpa-linux-x64",
  "linux-arm64": "@pchuan98/cpa-linux-arm64",
};
const platformKey = `${process.platform}-${process.arch}`;
const platformPackage = platformPackages[platformKey];
if (!platformPackage) {
  console.error(`Unsupported platform: ${platformKey}`);
  process.exit(1);
}

const require = createRequire(import.meta.url);
let packageRoot;
try {
  packageRoot = path.dirname(require.resolve(`${platformPackage}/package.json`));
} catch {
  console.error(`Missing CPA platform package: ${platformPackage}`);
  console.error("Reinstall @pchuan98/cpa so npm can install the package for this platform.");
  process.exit(1);
}

const executableSuffix = process.platform === "win32" ? ".exe" : "";
const codexPath = path.join(packageRoot, "native", `codex${executableSuffix}`);
const hostPath = path.join(
  packageRoot,
  "native",
  `codex-code-mode-host${executableSuffix}`,
);

for (const executable of [codexPath, hostPath]) {
  if (!existsSync(executable)) {
    console.error(`Missing packaged executable: ${executable}`);
    process.exit(1);
  }
}

process.exit(launchCodex(codexPath, process.argv.slice(2)));
