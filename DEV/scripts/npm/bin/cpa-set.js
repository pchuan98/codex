#!/usr/bin/env node

import { readFileSync } from "node:fs";
import process from "node:process";
import {
  configPath,
  maxContextWindowTokens,
  readConfig,
  readSecret,
  writeStoredApiKey,
  writeContextWindow,
  writeContextMode,
  writeProviderEnabled,
  writeYoloEnabled,
} from "./config.js";

const argumentsAfterCommand = process.argv.slice(2);
const packageJson = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf8").replace(
    /^\uFEFF/u,
    "",
  ),
);

function printHelp() {
  console.log(`cpa-set ${packageJson.version}

Configure the CPA launcher in <CODEX_HOME>/cpa.toml.

Usage:
  cpa-set <command> [arguments]

Commands:
  api-key [API_KEY]       Save an API key; prompts securely when omitted
  context [exp|default|SIZE]  Set context mode or size (256k); omit to reset both
  provider <true|false>   Enable or disable the CPA model provider
  yolo <true|false>       Enable or disable automatic --yolo
  show                    Show the current configuration with the key masked
  path                    Print the CPA configuration file path

Options:
  -h, --help              Show help
  -V, --version           Show version`);
}

function usageError(message) {
  throw new Error(`${message}\nRun 'cpa-set --help' for usage.`);
}

function maskedApiKey(apiKey) {
  if (!apiKey) {
    return "not set";
  }
  return apiKey.length <= 8
    ? "********"
    : `${apiKey.slice(0, 4)}...${apiKey.slice(-4)}`;
}

async function main() {
  const [command, ...commandArguments] = argumentsAfterCommand;

  if (!command || command === "-h" || command === "--help") {
    printHelp();
    return;
  }

  if (command === "-V" || command === "--version") {
    console.log(packageJson.version);
    return;
  }

  if (command === "api-key") {
    if (commandArguments.length > 1) {
      usageError("The api-key command accepts at most one API key.");
    }

    let apiKey = commandArguments[0]?.trim();
    if (!apiKey) {
      apiKey = (await readSecret("CPA_API_KEY: ")).trim();
    }
    if (!apiKey) {
      usageError("CPA_API_KEY cannot be empty.");
    }

    writeStoredApiKey(apiKey);
    console.log(`CPA API key saved to ${configPath}`);
    return;
  }

  if (command === "context") {
    if (commandArguments.length > 1) {
      usageError("The context command accepts at most one mode or size.");
    }

    const size = commandArguments[0];
    if (size === "exp" || size === "default") {
      writeContextMode(size);
      console.log(`CPA context mode set to ${size}.`);
      console.log(`CPA config saved to ${configPath}`);
      return;
    }
    if (size === undefined) {
      writeContextWindow(undefined);
      console.log("CPA context restored to the Codex default.");
      console.log(`CPA config saved to ${configPath}`);
      return;
    }

    const match = /^(\d+)k$/iu.exec(size);
    if (!match) {
      usageError("Context must be 'exp', 'default', or a size such as '256k'.");
    }
    const contextWindow = Number(match[1]) * 1000;
    if (!Number.isSafeInteger(contextWindow) || contextWindow <= 0) {
      usageError("Context size must be greater than 0k.");
    }
    if (contextWindow > maxContextWindowTokens) {
      usageError(
        `Context size cannot exceed ${maxContextWindowTokens / 1000}k.`,
      );
    }

    writeContextWindow(contextWindow);
    console.log(`CPA context set to ${match[1]}k (${contextWindow} tokens).`);
    console.log(`CPA config saved to ${configPath}`);
    return;
  }

  if (command === "provider") {
    if (commandArguments.length !== 1) {
      usageError("The provider command requires 'true' or 'false'.");
    }
    const value = commandArguments[0].toLowerCase();
    if (value !== "true" && value !== "false") {
      usageError("Provider must be 'true' or 'false'.");
    }

    const enabled = value === "true";
    writeProviderEnabled(enabled);
    console.log(`CPA model provider ${enabled ? "enabled" : "disabled"}.`);
    console.log(`CPA config saved to ${configPath}`);
    return;
  }

  if (command === "yolo") {
    if (commandArguments.length !== 1) {
      usageError("The yolo command requires 'true' or 'false'.");
    }
    const value = commandArguments[0].toLowerCase();
    if (value !== "true" && value !== "false") {
      usageError("Yolo must be 'true' or 'false'.");
    }

    const enabled = value === "true";
    writeYoloEnabled(enabled);
    console.log(`Automatic --yolo ${enabled ? "enabled" : "disabled"}.`);
    console.log(`CPA config saved to ${configPath}`);
    return;
  }

  if (command === "show") {
    if (commandArguments.length !== 0) {
      usageError("The show command does not accept arguments.");
    }
    const config = readConfig();
    console.log(`Config: ${configPath}`);
    console.log(`API key: ${maskedApiKey(config.api_key)}`);
    console.log(`Context mode: ${config.context_mode ?? "default"}`);
    console.log(
      `Context: ${config.context_window === undefined ? "default" : `${config.context_window / 1000}k`}`,
    );
    console.log(`CPA provider: ${config.provider_enabled !== false ? "enabled" : "disabled"}`);
    console.log(`Automatic --yolo: ${config.yolo_enabled !== false ? "enabled" : "disabled"}`);
    return;
  }

  if (command === "path") {
    if (commandArguments.length !== 0) {
      usageError("The path command does not accept arguments.");
    }
    console.log(configPath);
    return;
  }

  usageError(`Unknown command '${command}'.`);
}

try {
  await main();
} catch (error) {
  console.error(`cpa-set: ${error.message}`);
  process.exitCode = 1;
}
