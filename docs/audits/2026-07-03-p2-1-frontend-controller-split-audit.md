# P2-1 前端主 Controller 拆分审计

日期：2026-07-03

## 改动范围

- 新增 `sessionController.svelte.ts`：承载会话状态应用、录音 toggle 和 ASR final 事件处理。
- 新增 `configController.svelte.ts`：承载配置保存、自动保存定时器、保存状态和保存校验错误处理。
- 新增 `nativeEventController.svelte.ts`：承载 Tauri `listen` 注册和释放。
- 新增 `setupController.svelte.ts`：承载配置健康检查刷新、ASR 测试、LLM 测试和屏幕 OCR 测试流程。
- `VoxTypeController.svelte.ts` 保留状态组装、对外 props 和少量页面协调逻辑，组件接口保持不变。

## 行为审计

- 不修改 UI 组件 props 名称，不改变页面组件调用方式。
- 不修改 ASR、LLM、剪贴板、热键、托盘、统计、隐私和日志的后端逻辑。
- 录音成功/失败状态、ASR final warning 展示、配置保存、自动保存、测试连接和原生事件监听的触发时机保持原行为。
- 拆分采用 getter/setter 注入，避免一次性把所有 `$state` 搬离主 controller 导致大范围响应式行为变化。

## 验证

- `npm run check`：通过，0 errors / 0 warnings。

## 手工回归建议

- 启动主窗口，确认配置加载、健康检查、统计、隐私状态和麦克风列表正常显示。
- 修改普通设置后等待自动保存，确认标题栏保存状态正常变化。
- 测试 ASR、LLM 和屏幕 OCR，确认按钮 loading、成功/失败提示与之前一致。
- 开始/停止录音，确认 session 状态、最终文本、录音质量提示和统计更新正常。
- 触发托盘检查更新、关闭到托盘提示和 overlay 字幕事件，确认原生事件仍能被处理。
