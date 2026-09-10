import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  renameSync,
  statSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";

function resolveCodexHome() {
  const configuredHome = process.env.CODEX_HOME?.trim();
  if (!configuredHome) {
    return path.join(os.homedir(), ".codex");
  }

  let metadata;
  try {
    metadata = statSync(configuredHome);
  } catch (error) {
    throw new Error(`Failed to resolve CODEX_HOME ${configuredHome}: ${error.message}`);
  }
  if (!metadata.isDirectory()) {
    throw new Error(`CODEX_HOME ${configuredHome} is not a directory.`);
  }
  return realpathSync(configuredHome);
}

export const codexHome = resolveCodexHome();
export const configPath = path.join(codexHome, "cpa.toml");
export const maxContextWindowTokens = 872_000;

function parseTomlString(value, key) {
  const literalMatch = /^'([^']*)'\s*(?:#.*)?$/u.exec(value);
  if (literalMatch) {
    return literalMatch[1];
  }
  const match = /^("(?:\\.|[^"\\])*")\s*(?:#.*)?$/u.exec(value);
  if (!match) {
    throw new Error(`${key} in ${configPath} must be a quoted string.`);
  }
  try {
    const parsed = JSON.parse(match[1]);
    if (typeof parsed === "string") {
      return parsed;
    }
  } catch {
    // Report a CPA-specific error below.
  }
  throw new Error(`${key} in ${configPath} must be a quoted string.`);
}

export function readConfig() {
  if (!existsSync(configPath)) {
    return {};
  }

  let contents;
  try {
    contents = readFileSync(configPath, "utf8").replace(/^\uFEFF/u, "");
  } catch (error) {
    throw new Error(`Failed to read CPA config at ${configPath}: ${error.message}`);
  }

  const config = {};
  for (const [index, rawLine] of contents.split(/\r?\n/u).entries()) {
    const line = rawLine.trim();
    if (!line || line.startsWith("#")) {
      continue;
    }

    const match = /^([A-Za-z0-9_-]+)\s*=\s*(.*)$/u.exec(line);
    if (!match) {
      throw new Error(`Invalid CPA config at ${configPath}:${index + 1}.`);
    }

    const [, key, value] = match;
    if (Object.hasOwn(config, key)) {
      throw new Error(`Duplicate ${key} in ${configPath}.`);
    }
    if (key === "api_key") {
      config.api_key = parseTomlString(value, key);
    } else if (key === "context_mode") {
      const mode = parseTomlString(value, key);
      if (mode !== "exp" && mode !== "default") {
        throw new Error(`${key} in ${configPath} must be 'exp' or 'default'.`);
      }
      config.context_mode = mode;
    } else if (key === "context_window") {
      const integerMatch = /^(\d(?:_?\d)*)\s*(?:#.*)?$/u.exec(value);
      if (!integerMatch) {
        throw new Error(`${key} in ${configPath} must be a positive integer.`);
      }
      const contextWindow = Number(integerMatch[1].replaceAll("_", ""));
      if (!Number.isSafeInteger(contextWindow) || contextWindow <= 0) {
        throw new Error(`${key} in ${configPath} must be a positive integer.`);
      }
      if (contextWindow > maxContextWindowTokens) {
        throw new Error(
          `${key} in ${configPath} cannot exceed ${maxContextWindowTokens}.`,
        );
      }
      config.context_window = contextWindow;
    } else if (key === "provider_enabled") {
      const booleanMatch = /^(true|false)\s*(?:#.*)?$/u.exec(value);
      if (!booleanMatch) {
        throw new Error(`${key} in ${configPath} must be true or false.`);
      }
      config.provider_enabled = booleanMatch[1] === "true";
    } else if (key === "yolo_enabled") {
      const booleanMatch = /^(true|false)\s*(?:#.*)?$/u.exec(value);
      if (!booleanMatch) {
        throw new Error(`${key} in ${configPath} must be true or false.`);
      }
      config.yolo_enabled = booleanMatch[1] === "true";
    } else {
      throw new Error(`Unknown CPA config key ${key} in ${configPath}.`);
    }
  }
  return config;
}

function writeConfig(config) {
  const lines = [];
  if (config.api_key) {
    lines.push(`api_key = ${JSON.stringify(config.api_key)}`);
  }
  if (config.context_window !== undefined) {
    lines.push(`context_window = ${config.context_window}`);
  }
  if (config.context_mode === "exp") {
    lines.push('context_mode = "exp"');
  }
  lines.push(`provider_enabled = ${config.provider_enabled !== false}`);
  lines.push(`yolo_enabled = ${config.yolo_enabled !== false}`);

  mkdirSync(codexHome, { recursive: true });
  let existingMode;
  if (existsSync(configPath)) {
    const metadata = lstatSync(configPath);
    if (metadata.isSymbolicLink()) {
      throw new Error(`Refusing to replace symbolic link at ${configPath}.`);
    }
    existingMode = metadata.mode;
  }

  const temporaryPath = path.join(
    codexHome,
    `.cpa.toml.${process.pid}.${Date.now()}.tmp`,
  );
  try {
    writeFileSync(temporaryPath, `${lines.join("\n")}\n`, {
      encoding: "utf8",
      flag: "wx",
      ...(existingMode === undefined ? {} : { mode: existingMode }),
    });
    renameSync(temporaryPath, configPath);
  } finally {
    if (existsSync(temporaryPath)) {
      unlinkSync(temporaryPath);
    }
  }
}

export function readStoredApiKey() {
  const value = readConfig().api_key?.trim();
  return value || undefined;
}

export function writeStoredApiKey(apiKey) {
  const config = readConfig();
  config.api_key = apiKey;
  writeConfig(config);
}

export function readContextWindow() {
  return readConfig().context_window;
}

export function writeContextWindow(contextWindow) {
  const config = readConfig();
  if (contextWindow === undefined) {
    delete config.context_window;
    delete config.context_mode;
  } else {
    config.context_window = contextWindow;
  }
  writeConfig(config);
}

export function writeContextMode(mode) {
  const config = readConfig();
  if (mode === "exp") {
    config.context_mode = mode;
  } else {
    delete config.context_mode;
  }
  writeConfig(config);
}

export function writeProviderEnabled(enabled) {
  const config = readConfig();
  config.provider_enabled = enabled;
  writeConfig(config);
}

export function writeYoloEnabled(enabled) {
  const config = readConfig();
  config.yolo_enabled = enabled;
  writeConfig(config);
}

export async function readSecret(prompt) {
  if (!process.stdin.isTTY || !process.stdout.isTTY) {
    throw new Error("CPA_API_KEY must be provided in a terminal.");
  }

  process.stdout.write(prompt);
  process.stdin.setRawMode(true);
  process.stdin.resume();
  process.stdin.setEncoding("utf8");

  return await new Promise((resolve, reject) => {
    let value = "";

    const finish = (error) => {
      process.stdin.off("data", onData);
      process.stdin.setRawMode(false);
      process.stdin.pause();
      process.stdout.write("\n");
      if (error) {
        reject(error);
      } else {
        resolve(value);
      }
    };

    const onData = (chunk) => {
      for (const character of chunk) {
        if (character === "\u0003") {
          finish(new Error("Input cancelled."));
          return;
        }
        if (character === "\r" || character === "\n") {
          finish();
          return;
        }
        if (character === "\u007f" || character === "\b") {
          value = value.slice(0, -1);
          continue;
        }
        value += character;
      }
    };

    process.stdin.on("data", onData);
  });
}
