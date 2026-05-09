export function isSafeShortUserMessage(value: string) {
  const message = value.trim();
  if (!message || message.length > 180) return false;
  if (/[\r\n]/.test(message)) return false;
  if (/[A-Za-z]:\\|\/Users\/|\/home\//.test(message)) return false;
  if (/[{}[\]]/.test(message)) return false;
  if (/websocket|header|stack|panic|trace/i.test(message)) return false;
  if (/token|secret|access[_ -]?key|bearer|authorization/i.test(message)) return false;
  return true;
}

export function userFacingInvokeFailure(command: string, error: unknown, genericMessage: string) {
  const raw = typeof error === "string" ? error.trim() : "";
  if (
    (command === "test_asr_config" ||
      command === "test_llm_config" ||
      command === "test_screen_context" ||
      command === "generate_hotword_candidates") &&
    isSafeShortUserMessage(raw)
  ) {
    return raw;
  }
  return genericMessage;
}
