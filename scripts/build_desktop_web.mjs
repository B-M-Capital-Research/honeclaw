#!/usr/bin/env node

import { spawnSync } from "node:child_process";

const result = spawnSync("bun", ["--filter", "@hone-financial/app", "build"], {
  stdio: "inherit",
  env: {
    ...process.env,
    HONE_APP_RELATIVE_BASE: "1",
  },
  shell: process.platform === "win32",
});

if (result.error) {
  throw result.error;
}

process.exit(result.status ?? 1);
