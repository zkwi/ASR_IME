# Security Policy / 安全策略

VoxType handles microphone audio, clipboard data, API keys, local logs, non-text usage statistics, temporary screen OCR context, and optional local text histories. Security and privacy reports are welcome and should avoid exposing the affected data.

VoxType 会处理麦克风音频、剪贴板、API Key、本地日志、非正文统计、临时屏幕 OCR 上下文和可选本地正文历史。欢迎报告安全与隐私问题，但请勿在公开渠道暴露这些数据。

## Supported Versions / 支持范围

- The latest GitHub Release.
- The current `main` branch.

This is a personal project without a commercial response SLA. Credential disclosure, transcript or clipboard leakage, unsafe update execution, arbitrary code execution, and redaction failures receive priority.

## Report Privately / 私下报告

Use [GitHub private vulnerability reporting](https://github.com/zkwi/VoxType/security/advisories/new). Include the affected version or commit, Windows version, installation method, impact, minimal redacted reproduction steps, and a suggested fix if available.

If private reporting is temporarily unavailable, open a public Issue containing only a high-level impact summary and ask the maintainer for a private channel. Do not publish exploit details or sensitive samples.

Please do not include:

- Real Doubao, Alibaba Cloud, LLM, GitHub, or other credentials.
- Transcript text, screen OCR text, personal hotwords, prompts, or recent context.
- Raw logs, unredacted diagnostic reports, statistics files, or screenshots with personal data.
- Full paths containing a Windows username.

## Security Boundaries / 安全边界

- The desktop app runs locally, but microphone audio is sent to the selected cloud ASR provider.
- Optional LLM editing sends final text and only the explicitly enabled reference context to the configured model provider.
- Transcript, hotword, prompt, recent-context, and screen-OCR bodies must not enter normal logs or diagnostic reports.
- Usage statistics contain duration, character counts, speed, and timestamps—not transcript text.
- Recent context and automatic-hotword histories are disabled by default and stored only in ignored local files when enabled.
- Secret fields are stored in local `config.toml`; the repository contains placeholders only.
- The updater fetches a GitHub Release from the configured repository and starts the selected Windows installer. Artifact signing and provenance remain tracked hardening work; changing this trust model requires a dedicated design.

## Repository Controls / 仓库防护

- GitHub Secret Scanning and Push Protection are enabled.
- Checked-in staged and repository-visible secret scans block known credential patterns and protected local files.
- Dependabot covers npm, Cargo, and GitHub Actions.
- CI runs frontend, Rust, governance, dependency, clippy, and Tauri build checks.
- CodeQL analyzes JavaScript/TypeScript, Rust, and GitHub Actions workflows.
- Reusable Actions are pinned to full commit SHAs and updated by Dependabot.

## Disclosure Process / 处理流程

The maintainer will acknowledge a complete report when practical, reproduce and assess it, prepare a focused fix, run the release checks, publish a release or advisory when appropriate, and credit the reporter unless anonymity is requested. Please allow a reasonable remediation window before public disclosure.
