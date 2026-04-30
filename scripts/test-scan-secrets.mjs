#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(new URL("./scan-secrets.mjs", import.meta.url));

function runScan(args, cwd) {
  return spawnSync(process.execPath, [scriptPath, ...args], {
    cwd,
    encoding: "utf8",
  });
}

function runGit(args, cwd) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr || result.stdout);
}

function writeSecretFile(filePath) {
  const fakeKey = ["sk", "testsecretvalue123456"].join("-");
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `api_key = "${fakeKey}"\n`, "utf8");
}

function withTempDir(callback) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "voxtype-scan-secrets-"));
  try {
    callback(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

withTempDir((dir) => {
  const secretPath = path.join(dir, "nested", "secret.toml");
  writeSecretFile(secretPath);

  const result = runScan([dir], dir);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /nested\/secret\.toml:1: OpenAI-style key/);
});

withTempDir((dir) => {
  runGit(["init"], dir);
  runGit(["config", "user.email", "voxtype@example.invalid"], dir);
  runGit(["config", "user.name", "VoxType Test"], dir);

  const subdir = path.join(dir, "src-tauri");
  const secretPath = path.join(dir, "nested", "secret.toml");
  fs.mkdirSync(subdir, { recursive: true });
  writeSecretFile(secretPath);

  const result = runScan(["--git-visible"], subdir);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /nested\/secret\.toml:1: OpenAI-style key/);
});

withTempDir((dir) => {
  runGit(["init"], dir);
  const contextPath = path.join(dir, "context", "recent_context.jsonl");
  fs.mkdirSync(path.dirname(contextPath), { recursive: true });
  fs.writeFileSync(contextPath, "{\"text\":\"private transcript\"}\n", "utf8");

  const result = runScan(["--git-visible"], dir);
  assert.equal(result.status, 1, result.stdout + result.stderr);
  assert.match(result.stdout, /context\/recent_context\.jsonl:0: Protected local file/);
});

console.log("[test-scan-secrets] all checks passed");
