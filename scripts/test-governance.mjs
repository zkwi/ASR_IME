#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./check-governance.mjs", import.meta.url));

function runGovernance(cwd) {
  return spawnSync(process.execPath, [scriptPath], {
    cwd,
    encoding: "utf8",
  });
}

function writeFile(filePath, content) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
}

function withProject(callback) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "voxtype-governance-"));
  try {
    createValidProject(dir);
    callback(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function createValidProject(dir) {
  writeFile(path.join(dir, "package.json"), JSON.stringify({ version: "1.2.3" }, null, 2));
  writeFile(path.join(dir, "CHANGELOG.md"), "# Changelog\n\n## [1.2.3]\n");
  writeFile(
    path.join(dir, "src-tauri", "Cargo.toml"),
    '[package]\nname = "voxtype-desktop"\nversion = "1.2.3"\n\n[dependencies]\n',
  );
  writeFile(
    path.join(dir, "src-tauri", "tauri.conf.json"),
    JSON.stringify({ version: "1.2.3" }, null, 2),
  );

  writeFile(
    path.join(dir, "README.md"),
    [
      "# Test",
      "",
      "[Setup](https://github.com/zkwi/VoxType/wiki/Setup-Guide)",
      "",
      '<img src="https://raw.githubusercontent.com/zkwi/VoxType/main/screenshots/home.png" alt="home">',
      "",
    ].join("\n"),
  );
  writeFile(path.join(dir, "screenshots", "home.png"), "not a real png");

  for (const page of [
    "_Sidebar",
    "Home",
    "Setup-Guide",
    "Setup-Guide-English",
    "Feature-Guide",
    "Feature-Guide-English",
    "Troubleshooting",
    "Troubleshooting-English",
  ]) {
    writeFile(path.join(dir, "docs", "wiki", `${page}.md`), `# ${page}\n`);
  }
}

withProject((dir) => {
  const result = runGovernance(dir);
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /\[governance\] checks passed/);
});

withProject((dir) => {
  writeFile(path.join(dir, "src-tauri", "tauri.conf.json"), JSON.stringify({ version: "9.9.9" }, null, 2));
  const result = runGovernance(dir);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /version mismatch/);
});

withProject((dir) => {
  fs.rmSync(path.join(dir, "docs", "wiki", "Setup-Guide.md"));
  const result = runGovernance(dir);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /missing required Wiki mirror Setup-Guide\.md/);
  assert.match(result.stdout, /Wiki link lacks local mirror: Setup-Guide/);
});

console.log("[test-governance] all checks passed");
