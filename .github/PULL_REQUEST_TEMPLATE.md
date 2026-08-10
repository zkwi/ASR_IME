## Summary

-

## User Problem and Scope

- User problem:
- Files/modules changed:
- Explicitly out of scope:

## Impact Check

- [ ] I read `CONTRIBUTING.md`, `ARCHITECTURE.md`, `SECURITY.md`, and the relevant guardrails.
- [ ] I described any impact on ASR, final-result gating, optional LLM editing, clipboard/paste, or successful statistics.
- [ ] I described any config/default, privacy, logging, local-history, hotkey, tray, updater, installer, or screenshot impact.
- [ ] User-visible behavior is synchronized across both README files, Wiki drafts, and all three UI languages where applicable.
- [ ] No real keys, transcripts, hotwords, prompts, OCR/context bodies, raw logs, statistics files, or Windows username paths are included.

## Checks

- [ ] `npm run scan:secrets:staged`
- [ ] `npm run ai:check`
- [ ] `npm run audit:npm`
- [ ] `npm run audit:rust`
- [ ] `npm run ai:release-check` when release-facing
- [ ] Manual UI verification, if UI changed

## Risk Areas

Mention any affected areas: ASR, LLM polishing, clipboard/paste, hotkeys, tray, updates, config, logs, stats, privacy, installer, or UI.

## Evidence and Notes

Include regression evidence, skipped checks with reasons, known risks, and follow-up work.
