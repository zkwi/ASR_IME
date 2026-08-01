import { describe, expect, it } from "vitest";
import { createNotificationController } from "$lib/app/notificationController.svelte";

function createController() {
  return createNotificationController({
    t: () => "操作失败",
    setStatusMessage: () => undefined,
    logError: () => undefined,
  });
}

function advanceRounds(
  controller: ReturnType<typeof createNotificationController>,
  rounds: number,
) {
  for (let index = 0; index < rounds; index += 1) {
    controller.advanceSessionRound();
  }
}

describe("notification controller", () => {
  it("clears an error when it falls outside the five-round window", () => {
    const controller = createController();
    controller.show("偶发识别错误", "error");

    advanceRounds(controller, 4);
    expect(controller.message).toBe("偶发识别错误");

    controller.advanceSessionRound();
    expect(controller.message).toBe("");
  });

  it("restarts the five-round window when a newer error replaces the current one", () => {
    const controller = createController();
    controller.show("旧错误", "error");
    advanceRounds(controller, 4);

    controller.show("新错误", "error");
    advanceRounds(controller, 4);
    expect(controller.message).toBe("新错误");

    controller.advanceSessionRound();
    expect(controller.message).toBe("");
  });
});
