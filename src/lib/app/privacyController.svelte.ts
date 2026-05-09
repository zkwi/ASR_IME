import type { CopyKey } from "$lib/i18n";
import type { ConnectionTestResult, LocalDataStatus } from "$lib/types/app";

type SafeInvoke = <T>(
  command: string,
  args?: Record<string, unknown>,
  quiet?: boolean,
) => Promise<T | null>;

type NoticeKind = "success" | "info" | "warning" | "error";

type PrivacyControllerOptions = {
  t: (key: CopyKey, values?: Record<string, string>) => string;
  safeInvoke: SafeInvoke;
  showActionNotice: (message: string, kind: NoticeKind) => void;
  canConfirm: () => boolean;
  refreshStats: () => Promise<void>;
  refreshAutoHotwordStatus: () => Promise<void>;
};

export function createPrivacyController(options: PrivacyControllerOptions) {
  let status = $state<LocalDataStatus | null>(null);
  let clearingRecentContext = $state(false);
  let clearingAutoHotwordHistory = $state(false);
  let clearingUsageStats = $state(false);

  function apply(nextStatus: LocalDataStatus) {
    status = nextStatus;
  }

  async function refreshStatus() {
    const result = await options.safeInvoke<LocalDataStatus>("get_local_data_status", undefined, true);
    if (result) status = result;
  }

  async function clearRecentContext() {
    if (clearingRecentContext) return;
    if (options.canConfirm() && !window.confirm(options.t("privacyClearRecentContextConfirm"))) return;
    clearingRecentContext = true;
    try {
      const result = await options.safeInvoke<ConnectionTestResult>("clear_recent_context", undefined, false);
      if (result) {
        options.showActionNotice(options.t("privacyRecentContextCleared"), "success");
        await refreshStatus();
      }
    } finally {
      clearingRecentContext = false;
    }
  }

  async function clearAutoHotwordHistory() {
    if (clearingAutoHotwordHistory) return;
    if (options.canConfirm() && !window.confirm(options.t("autoHotwordsClearConfirm"))) return;
    clearingAutoHotwordHistory = true;
    try {
      const result = await options.safeInvoke<ConnectionTestResult>("clear_hotword_history", undefined, false);
      if (result) {
        options.showActionNotice(options.t("autoHotwordsHistoryCleared"), "success");
        await Promise.all([options.refreshAutoHotwordStatus(), refreshStatus()]);
      }
    } finally {
      clearingAutoHotwordHistory = false;
    }
  }

  async function clearUsageStats() {
    if (clearingUsageStats) return;
    if (options.canConfirm() && !window.confirm(options.t("privacyClearUsageStatsConfirm"))) return;
    clearingUsageStats = true;
    try {
      const result = await options.safeInvoke<ConnectionTestResult>("clear_usage_stats", undefined, false);
      if (result) {
        options.showActionNotice(options.t("privacyUsageStatsCleared"), "success");
        await Promise.all([options.refreshStats(), refreshStatus()]);
      }
    } finally {
      clearingUsageStats = false;
    }
  }

  return {
    get status() { return status; },
    get clearingRecentContext() { return clearingRecentContext; },
    get clearingAutoHotwordHistory() { return clearingAutoHotwordHistory; },
    get clearingUsageStats() { return clearingUsageStats; },
    apply,
    refreshStatus,
    clearRecentContext,
    clearAutoHotwordHistory,
    clearUsageStats,
  };
}
