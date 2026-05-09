## Summary

-

## Scope

- [ ] I read `CONTRIBUTING.md`, `SUPPORT.md`, `SECURITY.md`, and `CODE_OF_CONDUCT.md`
- [ ] Does not affect the ASR -> LLM -> clipboard -> paste main workflow
- [ ] Does not change config structure or defaults
- [ ] Does not change privacy, logging, clipboard, hotkey, tray, or update behavior
- [ ] User-facing docs/i18n updated where needed
- [ ] No real keys, transcripts, hotwords, prompts, local context, raw logs, or Windows username paths are included

## Checks

- [ ] `npm run scan:secrets:staged`
- [ ] `npm run check:governance`
- [ ] `npm run test:governance`
- [ ] `npm run test:stats`
- [ ] `npm run ai:check`
- [ ] Manual UI verification, if UI changed

## Risk Areas

Mention any affected areas: ASR, LLM polishing, clipboard/paste, hotkeys, tray, updates, config, logs, stats, privacy, installer, or UI.

## Notes

Mention any skipped checks, known risks, or follow-up work here.
