#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import ts from "typescript";

const EXCLUDE_DIRS = new Set([
  ".codex",
  ".git",
  ".idea",
  ".playwright-mcp",
  ".svelte-kit",
  "build",
  "dist",
  "node_modules",
  "output",
  "src-tauri/target",
  "target",
]);

const REQUIRED_WIKI_PAGES = [
  "_Sidebar",
  "Home",
  "Setup-Guide",
  "Setup-Guide-English",
  "Feature-Guide",
  "Feature-Guide-English",
  "Troubleshooting",
  "Troubleshooting-English",
];

const I18N_EXPORTS = [
  { file: "src/lib/i18n/zh-CN.ts", exportName: "zhCN" },
  { file: "src/lib/i18n/zh-TW.ts", exportName: "zhTW" },
  { file: "src/lib/i18n/en.ts", exportName: "en" },
];

function runGit(args, cwd) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  if (result.status !== 0) return null;
  return result.stdout.trim();
}

function repoRoot() {
  const root = runGit(["rev-parse", "--show-toplevel"], process.cwd());
  return root ? path.resolve(root) : process.cwd();
}

function normalize(relativePath) {
  return relativePath.split(path.sep).join("/");
}

function isExcluded(relativePath) {
  const normalized = normalize(relativePath);
  return normalized.split("/").some((part, index, parts) => {
    const prefix = parts.slice(0, index + 1).join("/");
    return EXCLUDE_DIRS.has(part) || EXCLUDE_DIRS.has(prefix);
  });
}

function walkMarkdownFiles(root, dir = root) {
  const files = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    const relative = path.relative(root, fullPath);
    if (isExcluded(relative)) continue;
    if (entry.isDirectory()) files.push(...walkMarkdownFiles(root, fullPath));
    else if (entry.isFile() && entry.name.toLowerCase().endsWith(".md")) files.push(fullPath);
  }
  return files;
}

function existsPathOrMarkdown(targetPath) {
  return fs.existsSync(targetPath) || fs.existsSync(`${targetPath}.md`);
}

function isExternalLink(link) {
  return /^[a-z][a-z0-9+.-]*:/i.test(link) || link.startsWith("mailto:");
}

function stripAnchor(link) {
  return link.split("#", 1)[0];
}

function shouldIgnoreDocReference(file, link) {
  const filename = path.basename(file);
  return filename === "豆包流式语音识别参考文档.md" && (link.startsWith("/") || link === "/span");
}

function checkLocalMarkdownLinks(root, failures) {
  const markdownLinkPattern = /(?<!!)\[[^\]]+\]\(([^)]+)\)|<([^>\s]+)>/g;
  for (const file of walkMarkdownFiles(root)) {
    const text = fs.readFileSync(file, "utf8");
    for (const match of text.matchAll(markdownLinkPattern)) {
      let link = (match[1] || match[2] || "").trim();
      if (!link || link.startsWith("#") || isExternalLink(link)) continue;
      if (shouldIgnoreDocReference(file, link)) continue;

      link = stripAnchor(link);
      if (!link) continue;

      const decoded = decodeURIComponent(link);
      const targetPath = decoded.startsWith("/")
        ? path.join(root, decoded)
        : path.resolve(path.dirname(file), decoded);
      if (!existsPathOrMarkdown(targetPath)) {
        failures.push(`${normalize(path.relative(root, file))}: missing local link target: ${link}`);
      }
    }
  }
}

function checkImageReferences(root, failures) {
  const markdownImagePattern = /!\[[^\]]*\]\(([^)]+)\)/g;
  const htmlImagePattern = /<img\b[^>]*\bsrc=["']([^"']+)["'][^>]*>/gi;

  for (const file of walkMarkdownFiles(root)) {
    const text = fs.readFileSync(file, "utf8");
    const links = [];
    for (const match of text.matchAll(markdownImagePattern)) links.push(match[1].trim());
    for (const match of text.matchAll(htmlImagePattern)) links.push(match[1].trim());

    for (const link of links) {
      if (!link || link.startsWith("data:")) continue;
      let localReference = null;
      let referenceFromRoot = false;
      if (link.startsWith("https://raw.githubusercontent.com/zkwi/VoxType/main/")) {
        localReference = link.replace("https://raw.githubusercontent.com/zkwi/VoxType/main/", "");
        referenceFromRoot = true;
      } else if (!isExternalLink(link) && !link.startsWith("#")) {
        localReference = stripAnchor(link);
      }
      if (!localReference) continue;

      let targetPath;
      if (referenceFromRoot || localReference.startsWith("/")) {
        targetPath = path.join(root, localReference);
      } else {
        targetPath = path.resolve(path.dirname(file), localReference);
      }
      if (!fs.existsSync(targetPath)) {
        failures.push(`${normalize(path.relative(root, file))}: missing image target: ${link}`);
      }
    }
  }
}

function wikiSlugFromUrl(url) {
  const match = url.match(/https:\/\/github\.com\/zkwi\/VoxType\/wiki\/([^)\s>#]+)/);
  return match ? decodeURIComponent(match[1]) : null;
}

function checkWikiMirrors(root, failures) {
  const wikiDir = path.join(root, "docs", "wiki");
  for (const page of REQUIRED_WIKI_PAGES) {
    const filename = page === "_Sidebar" ? "_Sidebar.md" : `${page}.md`;
    if (!fs.existsSync(path.join(wikiDir, filename))) {
      failures.push(`docs/wiki: missing required Wiki mirror ${filename}`);
    }
  }

  for (const file of walkMarkdownFiles(root)) {
    const text = fs.readFileSync(file, "utf8");
    for (const match of text.matchAll(/https:\/\/github\.com\/zkwi\/VoxType\/wiki\/([^)\s>#]+)/g)) {
      const slug = wikiSlugFromUrl(match[0]);
      if (!slug) continue;
      const mirror = path.join(wikiDir, `${slug}.md`);
      if (!fs.existsSync(mirror)) {
        failures.push(`${normalize(path.relative(root, file))}: Wiki link lacks local mirror: ${slug}`);
      }
    }
  }
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, "utf8"));
}

function readCargoPackageVersion(cargoTomlPath) {
  let inPackageSection = false;
  for (const line of fs.readFileSync(cargoTomlPath, "utf8").split(/\r?\n/)) {
    if (/^\s*\[package\]\s*$/.test(line)) {
      inPackageSection = true;
      continue;
    }
    if (inPackageSection && /^\s*\[/.test(line)) break;
    if (!inPackageSection) continue;
    const version = line.match(/^\s*version\s*=\s*"([^"]+)"/);
    if (version) return version[1];
  }
  return null;
}

function readCargoLockPackageVersion(cargoLockPath) {
  const text = fs.readFileSync(cargoLockPath, "utf8");
  for (const match of text.matchAll(/\[\[package\]\]([\s\S]*?)(?=\r?\n\[\[package\]\]|$)/g)) {
    const block = match[1];
    const name = block.match(/^name\s*=\s*"([^"]+)"/m)?.[1];
    if (name !== "voxtype-desktop") continue;
    return block.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? null;
  }
  return null;
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function checkReleaseDocumentation(root, version, failures) {
  const auditsDir = path.join(root, "docs", "audits");
  const releasePattern = new RegExp(`release-${escapeRegExp(version)}(?:-|\\.)`);
  const audits = fs.existsSync(auditsDir)
    ? fs.readdirSync(auditsDir).filter((file) => file.endsWith(".md") && releasePattern.test(file))
    : [];
  if (!audits.length) {
    failures.push(`docs/audits: missing release audit for version ${version}`);
    return;
  }

  // 发布审计只有进入活文档索引才算完成同步，避免文件存在但维护者无法发现。
  const indexPath = path.join(root, "docs", "README.md");
  const index = fs.existsSync(indexPath) ? fs.readFileSync(indexPath, "utf8") : "";
  if (!audits.some((audit) => index.includes(`audits/${audit}`))) {
    failures.push(`docs/README.md: missing release audit link for version ${version}`);
  }
}

function checkVersionConsistency(root, failures) {
  const packageVersion = readJson(path.join(root, "package.json")).version;
  const tauriVersion = readJson(path.join(root, "src-tauri", "tauri.conf.json")).version;
  const cargoVersion = readCargoPackageVersion(path.join(root, "src-tauri", "Cargo.toml"));
  const versions = new Map([
    ["package.json", packageVersion],
    ["src-tauri/tauri.conf.json", tauriVersion],
    ["src-tauri/Cargo.toml", cargoVersion],
    ["src-tauri/Cargo.lock", readCargoLockPackageVersion(path.join(root, "src-tauri", "Cargo.lock"))],
  ]);
  const packageLockPath = path.join(root, "package-lock.json");
  if (fs.existsSync(packageLockPath)) {
    const packageLock = readJson(packageLockPath);
    versions.set("package-lock.json", packageLock.version);
    versions.set('package-lock.json packages[""].version', packageLock.packages?.[""]?.version);
  }

  for (const [file, version] of versions.entries()) {
    if (!version) failures.push(`${file}: missing version`);
  }
  if (new Set([...versions.values()]).size > 1) {
    failures.push(
      `version mismatch: ${[...versions.entries()].map(([file, version]) => `${file}=${version}`).join(", ")}`,
    );
  }

  const changelog = fs.readFileSync(path.join(root, "CHANGELOG.md"), "utf8");
  if (!changelog.includes(`## [${packageVersion}]`)) {
    failures.push(`CHANGELOG.md: missing release section for package version ${packageVersion}`);
  }
  checkReleaseDocumentation(root, packageVersion, failures);
}

function propertyNameText(name) {
  if (ts.isIdentifier(name) || ts.isStringLiteral(name) || ts.isNumericLiteral(name)) return name.text;
  return null;
}

function unwrapObjectLiteralExpression(expression) {
  if (ts.isObjectLiteralExpression(expression)) return expression;
  if (ts.isSatisfiesExpression(expression) || ts.isAsExpression(expression)) {
    return unwrapObjectLiteralExpression(expression.expression);
  }
  return null;
}

function findExportedObjectLiteral(filePath, exportName) {
  const sourceText = fs.readFileSync(filePath, "utf8");
  const sourceFile = ts.createSourceFile(filePath, sourceText, ts.ScriptTarget.Latest, true, ts.ScriptKind.TS);
  let objectLiteral = null;

  function visit(node) {
    if (objectLiteral) return;
    if (ts.isVariableStatement(node)) {
      const isExported = node.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword);
      if (isExported) {
        for (const declaration of node.declarationList.declarations) {
          if (
            ts.isIdentifier(declaration.name) &&
            declaration.name.text === exportName &&
            declaration.initializer
          ) {
            objectLiteral = unwrapObjectLiteralExpression(declaration.initializer);
            return;
          }
        }
      }
    }
    ts.forEachChild(node, visit);
  }

  visit(sourceFile);
  return objectLiteral;
}

function collectObjectKeyPaths(objectLiteral, prefix = []) {
  const paths = [];
  for (const property of objectLiteral.properties) {
    if (!ts.isPropertyAssignment(property)) continue;
    const name = propertyNameText(property.name);
    if (!name) continue;

    const keyPath = [...prefix, name];
    paths.push(keyPath.join("."));
    const childObject = unwrapObjectLiteralExpression(property.initializer);
    if (childObject) {
      paths.push(...collectObjectKeyPaths(childObject, keyPath));
    }
  }
  return paths;
}

function formatKeyList(keys) {
  const visible = keys.slice(0, 12).join(", ");
  return keys.length > 12 ? `${visible}, ... +${keys.length - 12}` : visible;
}

function checkI18nKeyParity(root, failures) {
  const keySets = new Map();
  for (const entry of I18N_EXPORTS) {
    const filePath = path.join(root, entry.file);
    if (!fs.existsSync(filePath)) {
      failures.push(`${entry.file}: missing i18n file`);
      continue;
    }

    const objectLiteral = findExportedObjectLiteral(filePath, entry.exportName);
    if (!objectLiteral) {
      failures.push(`${entry.file}: missing exported object ${entry.exportName}`);
      continue;
    }
    keySets.set(entry, new Set(collectObjectKeyPaths(objectLiteral)));
  }

  const baseEntry = I18N_EXPORTS[0];
  const baseKeys = keySets.get(baseEntry);
  if (!baseKeys) return;

  for (const entry of I18N_EXPORTS.slice(1)) {
    const keys = keySets.get(entry);
    if (!keys) continue;
    const missing = [...baseKeys].filter((key) => !keys.has(key)).sort();
    const extra = [...keys].filter((key) => !baseKeys.has(key)).sort();
    if (missing.length) {
      failures.push(`${entry.file}: i18n keys missing compared with ${baseEntry.file}: ${formatKeyList(missing)}`);
    }
    if (extra.length) {
      failures.push(`${entry.file}: i18n keys extra compared with ${baseEntry.file}: ${formatKeyList(extra)}`);
    }
  }
}

function main() {
  const root = repoRoot();
  const failures = [];

  checkVersionConsistency(root, failures);
  checkI18nKeyParity(root, failures);
  checkLocalMarkdownLinks(root, failures);
  checkImageReferences(root, failures);
  checkWikiMirrors(root, failures);

  if (failures.length) {
    for (const failure of failures) console.log(`[governance] ${failure}`);
    console.log(`[governance] found ${failures.length} issue(s)`);
    process.exit(1);
  }

  console.log("[governance] checks passed");
}

main();
