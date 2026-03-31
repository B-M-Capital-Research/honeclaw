#!/usr/bin/env node

import { execFileSync } from "node:child_process";
import {
  chmodSync,
  cpSync,
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const VALID_PROFILES = new Set(["debug", "release"]);
const COMMON_BINS = ["hone-discord", "hone-feishu", "hone-telegram"];
const MACOS_ONLY_BINS = ["hone-imessage"];
const AUXILIARY_BINS = ["hone-mcp"];
const MACOS_EXTRA_EXTERNAL_BINS = ["opencode"];

function usage() {
  console.error(
    "usage: bun scripts/prepare_tauri_sidecar.mjs [debug|release] [--target-triple <triple>] [--skip-build] [--skip-dev-command] [--skip-build-command] [--json]",
  );
}

function run(command, args, cwd) {
  execFileSync(command, args, {
    cwd,
    stdio: "inherit",
    env: process.env,
  });
}

function runCapture(command, args, cwd) {
  return execFileSync(command, args, {
    cwd,
    stdio: ["ignore", "pipe", "inherit"],
    env: process.env,
    encoding: "utf8",
  }).trim();
}

function detectTargetTriple(rootDir) {
  if (process.env.HONE_TAURI_TARGET_TRIPLE) {
    return process.env.HONE_TAURI_TARGET_TRIPLE;
  }
  if (process.env.TAURI_ENV_TARGET_TRIPLE) {
    return process.env.TAURI_ENV_TARGET_TRIPLE;
  }

  const rustcOutput = execFileSync("rustc", ["-vV"], {
    cwd: rootDir,
    encoding: "utf8",
    env: process.env,
  });
  const match = rustcOutput.match(/^host:\s+(.+)$/m);
  if (!match) {
    throw new Error("failed to detect Rust host target triple from `rustc -vV`");
  }
  return match[1].trim();
}

function isWindowsTarget(targetTriple) {
  return targetTriple.includes("windows");
}

function isMacosTarget(targetTriple) {
  return targetTriple.includes("apple-darwin");
}

function sidecarBinsForTarget(targetTriple) {
  return isMacosTarget(targetTriple)
    ? [...MACOS_ONLY_BINS, ...COMMON_BINS]
    : [...COMMON_BINS];
}

function externalBinsForTarget(targetTriple) {
  const bins = [...sidecarBinsForTarget(targetTriple), ...AUXILIARY_BINS];
  if (isMacosTarget(targetTriple)) {
    bins.push(...MACOS_EXTRA_EXTERNAL_BINS);
  }
  return bins;
}

function binaryName(bin, targetTriple) {
  return isWindowsTarget(targetTriple) ? `${bin}.exe` : bin;
}

function generatedBinaryName(bin, targetTriple) {
  const suffix = isWindowsTarget(targetTriple) ? ".exe" : "";
  return `${bin}-${targetTriple}${suffix}`;
}

function copyExecutable(sourcePath, destinationPath) {
  mkdirSync(path.dirname(destinationPath), { recursive: true });
  cpSync(sourcePath, destinationPath, { force: true });
  try {
    const mode = statSync(sourcePath).mode | 0o755;
    chmodSync(destinationPath, mode);
  } catch {
    chmodSync(destinationPath, 0o755);
  }
}

function buildCargoBin(rootDir, cargoArgs, bin) {
  const output = execFileSync(
    "cargo",
    [...cargoArgs, "--message-format=json-render-diagnostics"],
    {
      cwd: rootDir,
      stdio: ["ignore", "pipe", "inherit"],
      env: process.env,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 20,
    },
  );

  let executable = "";
  for (const line of output.split(/\r?\n/)) {
    if (!line.trim()) continue;
    try {
      const message = JSON.parse(line);
      if (
        message.reason === "compiler-artifact" &&
        message.target?.name === bin &&
        message.executable
      ) {
        executable = message.executable;
      }
    } catch {
      // Ignore non-JSON lines and rely on cargo diagnostics already printed to stderr.
    }
  }
  return executable;
}

function createGeneratedConfig(rootDir, externalBins, options = {}) {
  const { skipDevCommand = false, skipBuildCommand = false } = options;
  const desktopDir = path.join(rootDir, "bins", "hone-desktop");
  const baseConfigPath = path.join(desktopDir, "tauri.conf.json");
  const generatedConfigPath = path.join(desktopDir, "tauri.generated.conf.json");
  const config = JSON.parse(readFileSync(baseConfigPath, "utf8"));

  if (skipDevCommand && config.build) {
    config.build.beforeDevCommand = "true";
  }
  if (skipBuildCommand && config.build) {
    config.build.beforeBuildCommand = "true";
  }

  config.bundle = config.bundle ?? {};
  config.bundle.externalBin = externalBins.map((bin) => `binaries/${bin}`);

  writeFileSync(generatedConfigPath, `${JSON.stringify(config, null, 2)}\n`);
  return generatedConfigPath;
}

function parseArgs(argv) {
  let profile = "debug";
  let targetTriple = "";
  let skipBuild = false;
  let skipDevCommand = false;
  let skipBuildCommand = false;
  let json = false;

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (VALID_PROFILES.has(arg)) {
      profile = arg;
      continue;
    }
    if (arg === "--target-triple") {
      targetTriple = argv[index + 1] ?? "";
      if (!targetTriple) {
        throw new Error("missing value for --target-triple");
      }
      index += 1;
      continue;
    }
    if (arg === "--skip-build") {
      skipBuild = true;
      continue;
    }
    if (arg === "--skip-dev-command") {
      skipDevCommand = true;
      continue;
    }
    if (arg === "--skip-build-command") {
      skipBuildCommand = true;
      continue;
    }
    if (arg === "--json") {
      json = true;
      continue;
    }
    throw new Error(`unknown argument: ${arg}`);
  }

  return { profile, targetTriple, skipBuild, skipDevCommand, skipBuildCommand, json };
}

function commandExists(command) {
  try {
    execFileSync(command, ["--version"], {
      stdio: "ignore",
      env: process.env,
    });
    return true;
  } catch {
    return false;
  }
}

function tryExecCapture(command, args, cwd) {
  try {
    return execFileSync(command, args, {
      cwd,
      stdio: ["ignore", "pipe", "inherit"],
      env: process.env,
      encoding: "utf8",
    }).trim();
  } catch {
    return "";
  }
}

function resolveLocalOpencodeCommandPath() {
  const candidates = [];
  if (process.env.HONE_OPENCODE_WRAPPER) {
    candidates.push(process.env.HONE_OPENCODE_WRAPPER);
  }
  if (process.platform !== "win32") {
    try {
      const resolved = runCapture("/bin/sh", ["-lc", "command -v opencode"], process.cwd());
      if (resolved) {
        candidates.push(resolved);
      }
    } catch {
      // ignore
    }
  }

  for (const candidate of candidates) {
    if (candidate && existsSync(candidate)) {
      return realpathSync(candidate);
    }
  }
  return "";
}

function findOpencodePackageJson(commandPath) {
  if (!commandPath) return "";

  const seen = new Set();
  let current = path.dirname(commandPath);
  while (!seen.has(current)) {
    seen.add(current);
    for (const candidate of [
      path.join(current, "package.json"),
      path.join(current, "node_modules", "opencode-ai", "package.json"),
      path.join(current, "libexec", "lib", "node_modules", "opencode-ai", "package.json"),
    ]) {
      if (!existsSync(candidate)) continue;
      try {
        const pkg = JSON.parse(readFileSync(candidate, "utf8"));
        if (pkg.name === "opencode-ai") {
          return candidate;
        }
      } catch {
        // ignore malformed files
      }
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }

  return "";
}

function detectOpencodeVersion() {
  if (process.env.HONE_OPENCODE_VERSION) {
    return process.env.HONE_OPENCODE_VERSION;
  }

  const commandPath = resolveLocalOpencodeCommandPath();
  const packageJsonPath = findOpencodePackageJson(commandPath);
  if (packageJsonPath) {
    const pkg = JSON.parse(readFileSync(packageJsonPath, "utf8"));
    if (pkg.version) {
      return pkg.version;
    }
  }

  const cellarMatch = commandPath.match(/\/Cellar\/opencode\/([^/]+)\//);
  if (cellarMatch?.[1]) {
    return cellarMatch[1];
  }

  return "";
}

function opencodePackageNameForTarget(targetTriple) {
  switch (targetTriple) {
    case "aarch64-apple-darwin":
      return "opencode-darwin-arm64";
    case "x86_64-apple-darwin":
      return "opencode-darwin-x64-baseline";
    default:
      return "";
  }
}

function detectLocalOpencodeBinary(targetTriple) {
  if (process.env.HONE_OPENCODE_BIN && existsSync(process.env.HONE_OPENCODE_BIN)) {
    return process.env.HONE_OPENCODE_BIN;
  }

  const packageName = opencodePackageNameForTarget(targetTriple);
  if (!packageName) return "";

  const commandPath = resolveLocalOpencodeCommandPath();
  const packageJsonPath = findOpencodePackageJson(commandPath);
  if (!packageJsonPath) {
    return "";
  }

  const opencodeRoot = path.dirname(packageJsonPath);
  const candidate = path.join(opencodeRoot, "node_modules", packageName, "bin", binaryName("opencode", targetTriple));
  return existsSync(candidate) ? candidate : "";
}

function downloadOpencodeBinary(rootDir, targetTriple, destinationPath) {
  const packageName = opencodePackageNameForTarget(targetTriple);
  if (!packageName) {
    throw new Error(`no bundled opencode package mapping for target ${targetTriple}`);
  }

  const version = detectOpencodeVersion();
  if (!version) {
    throw new Error(
      `unable to detect opencode version for ${targetTriple}; set HONE_OPENCODE_VERSION or install opencode locally first`,
    );
  }

  if (!commandExists("tar")) {
    throw new Error("tar is required to unpack bundled opencode binaries");
  }

  const tempRoot = mkdtempSync(path.join(os.tmpdir(), "hone-opencode-"));
  try {
    const extractDir = path.join(tempRoot, "extract");
    mkdirSync(extractDir, { recursive: true });

    let archivePath = "";
    const spec = `${packageName}@${version}`;
    for (const candidate of [
      ["npm", ["pack", spec, "--silent"]],
      ["bunx", ["npm", "pack", spec, "--silent"]],
    ]) {
      const [command, args] = candidate;
      if (!commandExists(command)) continue;
      const packed = tryExecCapture(command, args, tempRoot);
      if (!packed) continue;
      const lastLine = packed.split(/\r?\n/).filter(Boolean).at(-1) ?? "";
      const resolved = path.join(tempRoot, lastLine);
      if (lastLine && existsSync(resolved)) {
        archivePath = resolved;
        break;
      }
    }

    if (!archivePath) {
      if (!commandExists("curl")) {
        throw new Error("curl is required to download bundled opencode binaries");
      }
      archivePath = path.join(tempRoot, `${packageName}-${version}.tgz`);
      run(
        "curl",
        [
          "-fsSL",
          `https://registry.npmjs.org/${packageName}/-/${packageName}-${version}.tgz`,
          "-o",
          archivePath,
        ],
        rootDir,
      );
    }

    run("tar", ["-xzf", archivePath, "-C", extractDir], rootDir);
    const extracted = path.join(
      extractDir,
      "package",
      "bin",
      binaryName("opencode", targetTriple),
    );
    if (!existsSync(extracted)) {
      throw new Error(`downloaded ${packageName}@${version} but missing ${extracted}`);
    }
    copyExecutable(extracted, destinationPath);
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function ensureOpencodeBinary(rootDir, targetTriple, destinationDir) {
  if (!isMacosTarget(targetTriple)) {
    return null;
  }

  const localBinary = detectLocalOpencodeBinary(targetTriple);
  const destinationPath = path.join(
    destinationDir,
    generatedBinaryName("opencode", targetTriple),
  );
  if (localBinary) {
    copyExecutable(localBinary, destinationPath);
  } else {
    downloadOpencodeBinary(rootDir, targetTriple, destinationPath);
  }
  return destinationPath;
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    usage();
    console.error(String(error));
    process.exit(1);
  }

  const {
    profile,
    targetTriple: inputTargetTriple,
    skipBuild,
    skipDevCommand,
    skipBuildCommand,
    json,
  } = options;
  if (!VALID_PROFILES.has(profile)) {
    usage();
    process.exit(1);
  }

  const scriptPath = fileURLToPath(import.meta.url);
  const rootDir = path.resolve(path.dirname(scriptPath), "..");
  const targetTriple = inputTargetTriple || detectTargetTriple(rootDir);
  const sidecarBins = sidecarBinsForTarget(targetTriple);
  const externalBins = externalBinsForTarget(targetTriple);
  const targetDir = path.join(rootDir, "target", targetTriple, profile);
  const destinationDir = path.join(rootDir, "bins", "hone-desktop", "binaries");

  mkdirSync(destinationDir, { recursive: true });

  if (!skipBuild) {
    for (const bin of [...sidecarBins, ...AUXILIARY_BINS]) {
      const cargoArgs = ["build", "--bin", bin, "--target", targetTriple];
      if (profile === "release") {
        cargoArgs.splice(1, 0, "--release");
      }
      const sourcePath = buildCargoBin(rootDir, cargoArgs, bin);
      if (!sourcePath || !existsSync(sourcePath)) {
        throw new Error(
          `expected built sidecar not found for ${bin} under ${targetDir}`,
        );
      }

      const destinationPath = path.join(
        destinationDir,
        generatedBinaryName(bin, targetTriple),
      );
      copyExecutable(sourcePath, destinationPath);
    }

    ensureOpencodeBinary(rootDir, targetTriple, destinationDir);
  }

  const generatedConfigPath = createGeneratedConfig(rootDir, externalBins, {
    skipDevCommand,
    skipBuildCommand,
  });
  if (json) {
    console.log(
      JSON.stringify(
        {
          profile,
          targetTriple,
          skipBuild,
          skipDevCommand,
          skipBuildCommand,
          sidecarBins,
          externalBins,
          generatedConfigPath,
        },
        null,
        2,
      ),
    );
    return;
  }
  console.log(
    `[INFO] prepared ${externalBins.length} desktop bundled binaries for ${targetTriple} (${profile})`,
  );
  if (skipBuild) {
    console.log("[INFO] skipped cargo build/copy and only refreshed generated config");
  }
  if (skipDevCommand) {
    console.log("[INFO] generated config without beforeDevCommand");
  }
  if (skipBuildCommand) {
    console.log("[INFO] generated config without beforeBuildCommand");
  }
  console.log(`[INFO] generated Tauri config: ${generatedConfigPath}`);
}

main();
