# Contributing to VoxType / 参与贡献

Thank you for helping VoxType. This is a personal open-source project optimized for practical Windows use, simplicity, and maintainability. Small, focused changes are welcome. Open an Issue before changing the main voice-input workflow, configuration shape, updater trust model, or privacy boundaries.

感谢参与 VoxType。这是一个重视实用、简洁和可维护性的个人开源项目。欢迎小而清晰的改进；涉及主链路、配置结构、更新信任或隐私边界时，请先开 Issue 讨论。

Read [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md), [SECURITY.md](SECURITY.md), [SUPPORT.md](SUPPORT.md), [ROADMAP.md](ROADMAP.md), and [ARCHITECTURE.md](ARCHITECTURE.md) before contributing. Never publish real credentials, transcripts, personal hotwords, prompts, OCR text, recent context, raw logs, statistics files, or full Windows username paths.

## Good Contributions / 适合贡献

- Reproducible bug fixes with regression coverage.
- Tests for existing behavior and release/governance scripts.
- Setup, troubleshooting, architecture, and translation improvements.
- Small UI or accessibility improvements with all three UI languages kept in sync.
- Dependency and security maintenance with an explained risk assessment.

Large refactors, new providers, and new configuration layers need a concrete user problem and prior Issue discussion.

## Find Work / 寻找任务

- Start with [`good first issue`](https://github.com/zkwi/VoxType/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22good%20first%20issue%22).
- Use [`help wanted`](https://github.com/zkwi/VoxType/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22help%20wanted%22) for work needing extra Windows environments or specialist experience.
- Check [ROADMAP.md](ROADMAP.md) for current priorities and explicit non-goals.
- For an untracked change, open an Issue describing the user problem, scope, affected risk areas, and acceptance criteria.

## Development Setup / 开发环境

Requirements: Windows 10/11, Node.js, npm, the stable Rust toolchain, and WebView2.

```powershell
npm ci
npm run tauri dev
```

Ordinary pre-commit verification:

```powershell
npm run ai:check
```

Dependency and release-grade verification:

```powershell
npm run audit:npm
npm run audit:rust
npm run ai:release-check
```

`audit:rust` requires `cargo-audit`. Install it once with `cargo install cargo-audit --locked`. To enable the repository's local pre-commit secret scan, run `./scripts/enable_git_hooks.ps1`.

## Required Impact Check / 必查影响范围

Before coding and in the pull request, state:

1. What user problem does this solve?
2. Which files and modules change?
3. Does it affect ASR, optional LLM editing, clipboard/paste, final-result gating, or successful statistics?
4. Does it change configuration fields or defaults?
5. Does it affect privacy, logs, diagnostics, local histories, hotkeys, tray, updater, or screenshots?
6. What is explicitly out of scope?
7. What proves the change works?

If a configuration field changes, update `src-tauri/src/config.rs`, validation, `config.example.toml`, the settings UI, `zh-CN`, `zh-TW`, `en`, both README files, and relevant Wiki drafts together.

## Documentation Sync / 文档同步

| Change | Keep in sync |
| --- | --- |
| Install, setup, or defaults | `README.md`, `README.en.md`, `docs/wiki/Setup-Guide*.md` |
| Features or usage | Both README files and `docs/wiki/Feature-Guide*.md` |
| Troubleshooting or diagnostics | `SUPPORT.md`, `docs/wiki/Troubleshooting*.md`, key README entry points |
| Privacy, security, or redaction | `SECURITY.md`, both README files, relevant Wiki sections |
| Architecture or directory rules | `ARCHITECTURE.md`, `docs/README.md`, relevant maintenance docs |
| Release | All version files, `CHANGELOG.md`, `docs/audits/`, and `docs/README.md` |

## Pull Requests

- Keep one PR focused on one problem.
- Explain affected risk areas and why the approach is the smallest sufficient change.
- Include exact checks run and any checks skipped with reasons.
- Include real-window manual verification when UI behavior changes.
- Do not add dependencies, abstractions, or unrelated formatting without need.
- Complete the checked-in PR template and make sure CI plus CodeQL are green.

Use concise commit messages. The repository history commonly uses a short type plus a clear Simplified Chinese summary, for example:

```text
fix: 修复空识别状态回退
docs: 补充安装排障说明
chore: 收紧本地密钥扫描
```
