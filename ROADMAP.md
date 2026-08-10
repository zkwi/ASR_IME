# VoxType Roadmap

VoxType is an actively maintained personal open-source project. This roadmap communicates current priorities and contribution boundaries; it is not a delivery-date promise.

中文说明：本路线图用于公开当前维护重点和适合贡献的方向，不承诺固定发布时间。项目优先级始终是实用、简洁、可维护和真实 Windows 使用体验。

## Principles

- Protect the core path: trigger → microphone → final ASR result → optional LLM edit → clipboard → paste → clipboard restore.
- Keep privacy choices explicit. Cloud ASR and optional LLM data boundaries must be visible to users and contributors.
- Prefer focused fixes, tests, documentation, and small UX improvements over new frameworks or speculative features.
- Keep Windows 10/11 daily use reliable before expanding platform scope.

## Now

- Keep CI, CodeQL, dependency audits, secret scanning, and release checks green and understandable.
- Improve English contributor onboarding and keep README, Wiki drafts, and maintenance docs aligned.
- Review Dependabot updates in small, verifiable batches.
- Continue regression coverage for final-result gating, empty recognition, log redaction, clipboard recovery, and conservative trigger defaults.

## Next

- Add a repeatable clean-Windows-VM installer and first-launch smoke-test checklist.
- Expand English maintenance documentation for contributors who do not read Chinese.
- Design release artifact verification, signing, and provenance before changing the in-app updater trust model.
- Evaluate Windows ARM64 demand and build feasibility using real hardware or CI evidence before promising support.

## Later

- Improve keyboard and screen-reader accessibility with focused, testable issues.
- Add provider-conformance fixtures when a new ASR provider or protocol revision has a real user need.
- Reduce release toil where automation can preserve the existing audit and privacy gates.

## Contribution Lanes

- [`good first issue`](https://github.com/zkwi/VoxType/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22good%20first%20issue%22): bounded documentation, tests, and low-risk maintenance work.
- [`help wanted`](https://github.com/zkwi/VoxType/issues?q=is%3Aissue%20state%3Aopen%20label%3A%22help%20wanted%22): work that benefits from additional Windows environments or specialist experience.
- [`security`](https://github.com/zkwi/VoxType/issues?q=is%3Aissue%20state%3Aopen%20label%3Asecurity): public hardening work only. Report exploitable or sensitive details privately through [SECURITY.md](SECURITY.md).

Before starting a main-workflow, configuration, updater, privacy, logging, clipboard, hotkey, or tray change, open an Issue and read [AGENTS.md](AGENTS.md), [ARCHITECTURE.md](ARCHITECTURE.md), and [CONTRIBUTING.md](CONTRIBUTING.md).

## Explicit Non-goals

- Becoming a cross-platform framework before the Windows workflow is proven stable.
- Bundling provider credentials or hiding third-party ASR/LLM costs and data policies.
- Storing transcript text in logs or usage statistics.
- Enabling recent context, right Alt, middle mouse, or system-audio muting by default.
- Replacing focused modules with a large plugin architecture without demonstrated need.
