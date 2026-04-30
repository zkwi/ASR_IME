#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

const EXCLUDE_DIRS = new Set([
  ".codex",
  ".git",
  ".idea",
  ".playwright-mcp",
  ".svelte-kit",
  ".venv",
  "build",
  "dist",
  "env",
  "logs",
  "node_modules",
  "output",
  "package",
  "target",
  "venv",
]);

const BINARY_SUFFIXES = new Set([
  ".7z",
  ".dll",
  ".exe",
  ".gif",
  ".ico",
  ".icns",
  ".jpg",
  ".jpeg",
  ".mp3",
  ".mp4",
  ".pdf",
  ".png",
  ".pyd",
  ".rar",
  ".so",
  ".wav",
  ".webp",
  ".zip",
]);

const PROTECTED_LOCAL_FILES = new Set([
  "config.toml",
  "config.local.toml",
  "hotword_history.jsonl",
  "recent_context.jsonl",
  "voice_input.log",
  "voice_input_stats.jsonl",
]);

const PLACEHOLDER_VALUES = new Set([
  "",
  "changeme",
  "null",
  "none",
  "your key",
  "your-key",
  "your_api_key",
  "your-api-key",
  "your-access-key",
  "your-secret-key",
  "your token",
  "your access key",
  "your secret key",
  "your app key",
]);

const SECRET_PATTERNS = [
  ["OpenAI-style key", /sk-[A-Za-z0-9_-]{16,}/],
  ["GitHub PAT", /github_pat_[A-Za-z0-9_]{20,}/],
  ["GitHub token", /ghp_[A-Za-z0-9]{20,}/],
  ["Slack token", /xox[baprs]-[A-Za-z0-9-]{10,}/],
  ["AWS access key", /AKIA[0-9A-Z]{16}/],
  ["Private key block", /-----BEGIN (?:RSA|DSA|EC|OPENSSH|PGP) PRIVATE KEY-----/],
];

const ASSIGNMENT_PATTERN =
  /^\s*(api[_-]?key|access[_-]?key|app[_-]?key|secret|secret[_-]?key|client[_-]?secret|token|password|passwd|pwd)\s*(?:[:=]|=>)\s*(?:(["'])(.*?)\2|([A-Za-z0-9_./:+-]{8,}))\s*$/i;
const USER_ARRAY_PATTERN = /^\s*(hotwords|recent_context)\s*=\s*\[(.*)$/i;
const CONTEXT_BLOCK_PATTERN = /^\s*\[\[context\.(?:prompt_context|recent_context)\]\]\s*$/i;
const IMAGE_URL_PATTERN = /^\s*image_url\s*=\s*(["'])(.*?)\1\s*$/i;

function runGit(args, cwd) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  if (result.status !== 0) return null;
  return result.stdout.trim();
}

function repoRoot() {
  const root = runGit(["rev-parse", "--show-toplevel"], process.cwd());
  return root ? path.resolve(root) : null;
}

function normalize(relativePath) {
  return relativePath.split(path.sep).join("/").toLowerCase();
}

function isProtectedLocalFile(relativePath) {
  const normalized = normalize(relativePath);
  const name = path.basename(normalized);
  if (PROTECTED_LOCAL_FILES.has(name)) return true;
  if (normalized.startsWith("context/")) return true;
  if (name.endsWith(".local.toml")) return true;
  if (normalized.endsWith("/config.toml")) return true;
  if (normalized.endsWith("/hotword_history.jsonl")) return true;
  if (normalized.endsWith("/recent_context.jsonl")) return true;
  if (normalized.endsWith("/voice_input_stats.jsonl")) return true;
  if (name === ".env") return true;
  if (name.startsWith(".env.") && ![".env.example", ".env.sample", ".env.template"].includes(name)) {
    return true;
  }
  return false;
}

function isProbablyBinary(filePath) {
  if (BINARY_SUFFIXES.has(path.extname(filePath).toLowerCase())) return true;
  try {
    const chunk = fs.readFileSync(filePath).subarray(0, 2048);
    return chunk.includes(0);
  } catch {
    return true;
  }
}

function shouldSkip(relativePath, includeProtected = false, absolutePath = path.resolve(relativePath)) {
  if (!includeProtected && isProtectedLocalFile(relativePath)) return true;
  const parts = normalize(relativePath).split("/");
  if (parts.some((part) => EXCLUDE_DIRS.has(part))) return true;
  return isProbablyBinary(absolutePath);
}

function looksLikePlaceholder(value) {
  const normalized = value.trim().replace(/^['"]|['"]$/g, "").toLowerCase();
  if (PLACEHOLDER_VALUES.has(normalized)) return true;
  if (/^\d+$/.test(normalized)) return true;
  if (["true", "false"].includes(normalized)) return true;
  return ["example", "placeholder", "示例", "填写", "这里填"].some((word) => normalized.includes(word));
}

function maskSecret(value) {
  if (value.length <= 8) return "*".repeat(value.length);
  return `${value.slice(0, 4)}...${value.slice(-4)}`;
}

function scanFile(filePath) {
  const findings = [];
  let text;
  try {
    text = fs.readFileSync(filePath, "utf8");
  } catch {
    return findings;
  }

  const scanUserContent = [".toml", ".md", ".markdown"].includes(path.extname(filePath).toLowerCase());
  let userArrayKey = null;
  let userArrayReported = false;

  text.split(/\r?\n/).forEach((line, index) => {
    const lineNo = index + 1;
    const stripped = line.trim();

    if (scanUserContent && userArrayKey) {
      if (stripped.startsWith("]")) {
        userArrayKey = null;
        userArrayReported = false;
        return;
      }
      if (!stripped || stripped.startsWith("#") || stripped.startsWith("//")) return;
      if (!userArrayReported) {
        findings.push([lineNo, `Custom ${userArrayKey}`, "non-empty array"]);
        userArrayReported = true;
      }
      return;
    }

    if (!stripped || stripped.startsWith("#") || stripped.startsWith("//")) return;
    if ((stripped.match(/\|/g) || []).length >= 2) return;

    if (scanUserContent && CONTEXT_BLOCK_PATTERN.test(stripped)) {
      findings.push([lineNo, "Custom context block", "context.*"]);
      return;
    }

    const userArray = scanUserContent ? stripped.match(USER_ARRAY_PATTERN) : null;
    if (userArray) {
      const key = userArray[1];
      let value = userArray[2].split("#", 1)[0].trim();
      const closedInline = value.includes("]");
      value = value.split("]", 1)[0].trim();
      if (value) findings.push([lineNo, `Custom ${key}`, "non-empty array"]);
      else if (!closedInline) {
        userArrayKey = key;
        userArrayReported = false;
      }
      return;
    }

    const imageUrl = scanUserContent ? stripped.match(IMAGE_URL_PATTERN) : null;
    if (imageUrl) {
      const value = imageUrl[2].trim();
      if (value && !looksLikePlaceholder(value)) findings.push([lineNo, "Custom image_url", maskSecret(value)]);
      return;
    }

    for (const [name, pattern] of SECRET_PATTERNS) {
      const match = line.match(pattern);
      if (match) findings.push([lineNo, name, maskSecret(match[0])]);
    }

    const assignment = stripped.match(ASSIGNMENT_PATTERN);
    if (assignment) {
      const key = assignment[1];
      const value = (assignment[3] || assignment[4] || "").trim();
      if (value.length >= 8 && !looksLikePlaceholder(value) && !/^[A-Za-z_][A-Za-z0-9_]*$/.test(value)) {
        findings.push([lineNo, `Suspicious ${key}`, maskSecret(value)]);
      }
    }
  });

  return findings;
}

function walkFiles(root, base = root) {
  const files = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const fullPath = path.join(root, entry.name);
    const relative = path.relative(base, fullPath);
    if (entry.isDirectory()) {
      if (!EXCLUDE_DIRS.has(entry.name)) files.push(...walkFiles(fullPath, base));
    } else if (!shouldSkip(relative, false, fullPath)) {
      files.push(fullPath);
    }
  }
  return files;
}

function filesFromGit(root, mode) {
  const output =
    mode === "staged"
      ? runGit(["diff", "--cached", "--name-only", "--diff-filter=ACMR"], root)
      : runGit(["ls-files", "--cached", "--others", "--exclude-standard"], root);
  if (output === null) return null;
  return output
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((relative) => path.join(root, relative))
    .filter((filePath) => fs.existsSync(filePath) && fs.statSync(filePath).isFile());
}

function main() {
  const args = process.argv.slice(2);
  const staged = args.includes("--staged");
  const gitVisible = args.includes("--git-visible");
  const positional = args.filter((arg) => !arg.startsWith("--"));
  if (staged && gitVisible) {
    console.error("[scan-secrets] choose only one of --staged or --git-visible");
    process.exit(2);
  }

  let root = process.cwd();
  let files = [];
  const protectedFilesAreErrors = staged || gitVisible;
  if (staged || gitVisible) {
    const foundRoot = repoRoot();
    if (!foundRoot) {
      console.error("[scan-secrets] not inside a git repository");
      process.exit(2);
    }
    root = foundRoot;
    files = filesFromGit(root, staged ? "staged" : "git-visible");
    if (!files) {
      console.error("[scan-secrets] failed to query git files");
      process.exit(2);
    }
    if (staged && files.length === 0) {
      console.log("[scan-secrets] no staged files to scan");
      return;
    }
  } else {
    const target = path.resolve(positional[0] || ".");
    if (!fs.existsSync(target)) {
      console.error(`[scan-secrets] path not found: ${target}`);
      process.exit(2);
    }
    root = fs.statSync(target).isDirectory() ? target : path.dirname(target);
    files = fs.statSync(target).isDirectory() ? walkFiles(target) : [target];
  }

  let findingsCount = 0;
  for (const filePath of files) {
    const relative = path.relative(root, filePath);
    if (protectedFilesAreErrors && isProtectedLocalFile(relative)) {
      findingsCount += 1;
      console.log(`${normalize(relative)}:0: Protected local file: do not commit local config/log/state`);
      continue;
    }
    if (shouldSkip(relative, protectedFilesAreErrors)) continue;
    for (const [lineNo, kind, masked] of scanFile(filePath)) {
      findingsCount += 1;
      console.log(`${normalize(relative)}:${lineNo}: ${kind}: ${masked}`);
    }
  }

  if (findingsCount) {
    console.log(`[scan-secrets] found ${findingsCount} potential secret(s)`);
    process.exit(1);
  }
  console.log("[scan-secrets] no obvious secrets found");
}

main();
