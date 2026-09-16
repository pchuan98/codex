import { spawnSync } from "node:child_process";
import process from "node:process";
import { readConfig } from "./config.js";

export function launchCodex(codexPath, launcherArguments) {
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

  const forwardedArguments = [...launcherArguments];
  const separatorIndex = launcherArguments.indexOf("--");
  const optionArguments = separatorIndex === -1
    ? launcherArguments
    : launcherArguments.slice(0, separatorIndex);
  const explicitRemote = optionArguments.some(
    (argument) => argument === "--remote" || argument.startsWith("--remote="),
  );
  const explicitFeatures = new Set();
  for (let index = 0; index < optionArguments.length; index += 1) {
    const argument = optionArguments[index];
    if (argument === "--enable" || argument === "--disable") {
      const feature = optionArguments[index + 1];
      if (feature !== undefined) {
        explicitFeatures.add(feature);
        index += 1;
      }
    } else if (argument.startsWith("--enable=") || argument.startsWith("--disable=")) {
      explicitFeatures.add(argument.slice(argument.indexOf("=") + 1));
    }
  }
  const disableArguments = (config.disabled_features ?? [])
    .filter((feature) => !explicitFeatures.has(feature))
    .flatMap((feature) => ["--disable", feature]);
  if (!explicitRemote && !optionArguments.includes("--no-daemon")) {
    forwardedArguments.unshift("--no-daemon");
  }
  if (config.yolo_enabled !== false && !forwardedArguments.includes("--yolo")) {
    forwardedArguments.unshift("--yolo");
  }

  const contextWindow = config.context_window;

  const contextArguments = contextWindow
    ? ["-c", `model_context_window=${contextWindow}`]
    : [];
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

  const codexArguments = [
    ...providerArguments,
    ...contextArguments,
    ...disableArguments,
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

  return result.status ?? 1;
}
