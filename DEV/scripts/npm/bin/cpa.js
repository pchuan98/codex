#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import process from "node:process";
import { readConfig } from "./config.js";

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

let config;
try {
  config = readConfig();
} catch (error) {
  console.error(error.message);
  process.exit(1);
}

const providerEnabled = config.provider_enabled !== false;
const apiKey = providerEnabled
  ? process.env.CPA_API_KEY?.trim() || config.api_key?.trim()
  : undefined;
if (providerEnabled && !apiKey) {
  console.error("CPA_API_KEY is not configured.");
  console.error("Run: cpa-set api-key <API_KEY>");
  process.exit(1);
}

const launcherArguments = process.argv.slice(2);
const forwardedArguments = [...launcherArguments];
if (config.yolo_enabled !== false && !forwardedArguments.includes("--yolo")) {
  forwardedArguments.unshift("--yolo");
}

const contextWindow = config.context_window;

const contextArguments = contextWindow
  ? ["-c", `model_context_window=${contextWindow}`]
  : [];
if (config.context_mode === "exp") {
  contextArguments.push("-c", "features.context_management.experimental_mode=true");
}

const providerArguments = providerEnabled
  ? [
      "-c",
      "model_provider='cpa'",
      "-c",
      "model_providers.cpa.name='OpenAI'",
      "-c",
      "model_providers.cpa.base_url='https://codex.pchuan.top/v1'",
      "-c",
      "model_providers.cpa.env_key='CPA_API_KEY'",
      "-c",
      "model_providers.cpa.wire_api='responses'",
    ]
  : [];

const telemetryArguments = [
  "-c",
  "analytics.enabled=false",
  "-c",
  "feedback.enabled=false",
  "-c",
  "otel.metrics_exporter='none'",
];

const codexArguments = [
  ...providerArguments,
  ...telemetryArguments,
  ...contextArguments,
  ...forwardedArguments,
];

const result = spawnSync(codexPath, codexArguments, {
  env: providerEnabled
    ? { ...process.env, CPA_API_KEY: apiKey }
    : process.env,
  stdio: "inherit",
});

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
