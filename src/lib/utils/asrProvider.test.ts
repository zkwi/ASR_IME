import { describe, expect, it } from "vitest";
import { fallbackConfig } from "$lib/app/defaults";
import { activeAsrConfigFingerprint, hasAsrProviderConfig } from "./asrProvider";

function agentPlanConfig() {
  const config = structuredClone(fallbackConfig);
  Object.assign(config.auth, {
    mode: "agent_plan",
    api_key: "plan-test-key",
    app_key: "",
    access_key: "",
  });
  return config;
}

describe("Doubao authentication modes", () => {
  it("treats an Agent Plan API key as complete Doubao authentication", () => {
    expect(hasAsrProviderConfig(agentPlanConfig())).toBe(true);
  });

  it("invalidates the ASR test fingerprint when the authentication mode changes", () => {
    const agentPlan = agentPlanConfig();
    const appAccess = structuredClone(agentPlan);
    Object.assign(appAccess.auth, { mode: "app_access" });

    expect(activeAsrConfigFingerprint(agentPlan)).not.toBe(activeAsrConfigFingerprint(appAccess));
  });

  it("requires the fixed Seed ASR 2.0 resource in Agent Plan mode", () => {
    const config = agentPlanConfig();
    config.auth.resource_id = "volc.seedasr.sauc.concurrent";

    expect(hasAsrProviderConfig(config)).toBe(false);
  });
});
