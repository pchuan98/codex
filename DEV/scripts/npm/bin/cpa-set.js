#!/usr/bin/env node

import { readFileSync } from "node:fs";
import process from "node:process";
import {
  configPath,
  maxContextWindowTokens,
  normalizeDisabledFeatures,
  readConfig,
  readSecret,
  writeStoredApiKey,
  writeContextWindow,
  writeProviderEnabled,
  writeYoloEnabled,
  writeDisabledFeatures,
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
  context [SIZE]         Set context size (256k); omit to restore the default
  provider <true|false>   Enable or disable the CPA model provider
  yolo <true|false>       Enable or disable automatic --yolo
  disable add <FEATURE...>     Persistently disable one or more Codex features
  disable remove <FEATURE...>  Remove features from the persistent disable list
  disable list                Show persistently disabled features
  disable clear               Clear the persistent disable list
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
      usageError("The context command accepts at most one size.");
    }

    const size = commandArguments[0];
    if (size === undefined) {
      writeContextWindow(undefined);
      console.log("CPA context restored to the Codex default.");
      console.log(`CPA config saved to ${configPath}`);
      return;
    }

    const match = /^(\d+)k$/iu.exec(size);
    if (!match) {
      usageError("Context must be a size such as '256k', or omitted to restore the default.");
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
    console.log(
      `Context: ${config.context_window === undefined ? "default" : `${config.context_window / 1000}k`}`,
    );
    console.log(`CPA provider: ${config.provider_enabled !== false ? "enabled" : "disabled"}`);
    console.log(`Automatic --yolo: ${config.yolo_enabled !== false ? "enabled" : "disabled"}`);
    console.log(`Disabled features: ${(config.disabled_features ?? []).join(", ") || "none"}`);
    return;
  }

  if (command === "disable") {
    const [action, ...features] = commandArguments;
    if (action === "list" || action === "clear") {
      if (features.length !== 0) {
        usageError(`The disable ${action} command does not accept feature names.`);
      }
      if (action === "list") {
        const disabled = readConfig().disabled_features ?? [];
        console.log(disabled.length ? disabled.join("\n") : "No persistently disabled features.");
        return;
      }
      writeDisabledFeatures([]);
      console.log(`Disabled features cleared. CPA config saved to ${configPath}`);
      return;
    }
    if (action !== "add" && action !== "remove") {
      usageError("Use disable add, remove, list or clear.");
    }
    if (features.length === 0) {
      usageError(`The disable ${action} command requires at least one feature name.`);
    }
    const requested = normalizeDisabledFeatures(features);
    const disabled = readConfig().disabled_features ?? [];
    const updated = action === "add"
      ? [...disabled, ...requested]
      : disabled.filter((feature) => !requested.includes(feature));
    writeDisabledFeatures(updated);
    console.log(`Disabled features: ${normalizeDisabledFeatures(updated).join(", ") || "none"}`);
    console.log(`CPA config saved to ${configPath}`);
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
