# VoxType UI / code / docs audit - 2026-05-27

本记录只保存非敏感审计结论，不包含真实密钥、识别正文、屏幕 OCR 正文、热词、prompt、最近上下文正文、日志正文或本机用户名路径。

## Scope

- 主窗口六个页面：首页、热词与提示词、API配置、选项、隐私与本地数据、统计分析。
- 重点检查 UI 排版、导航状态、横向溢出、长文案换行、最近上下文相关交互、默认 LLM 提示词、提示词预览、配置模板、README、英文 README、Wiki 草稿和架构说明。
- 本轮不改 ASR 请求协议、空识别失败态、LLM 触发条件、剪贴板输出、热键默认值、托盘逻辑、统计正文记录或日志脱敏策略。

## UI and interaction findings

- `1280x760` 和 `1100x680` 两个项目目标尺寸下，六个主页面均未发现页面级横向溢出。
- 主导航在六个页面上的 active 状态正确。
- 首页、热词与提示词、API配置、选项、隐私与本地数据、统计分析的主要按钮、标题、说明和表格文本未发现裁切或重叠。
- API配置页仅有输入框内部内容宽度大于可视宽度，这是输入框自身可滚动文本行为，不是页面布局溢出。
- “润色时参考最近上下文”开关已统一为普通 checkbox 样式，不再使用额外副标题或强调色。
- “预览最终提示词”原先使用系统 alert 展示长文本，交互不适合阅读和复制；本轮已改为应用内弹窗，使用只读文本框承载内容，并提供复制按钮。

## Code and docs findings

- `llm_post_edit.use_recent_context` 的 Rust 默认值、前端 fallback、类型定义、配置模板、README、英文 README、Wiki 草稿和架构说明已同步。
- LLM 连接测试仍不会读取或上传本地最近上下文。
- 默认 LLM 提示词已收紧参考信息边界，避免用户词典、场景上下文、最近上下文或屏幕 OCR 把待润色文本没说的内容补进输出。
- 审计发现提示词预览的三语言文案滞后于真实请求拼接规则；已同步 `promptPreviewReferenceRules` 和最近上下文参考信息限制。
- `npm run check:governance` 和 `npm run scan:secrets` 未发现治理或明显密钥问题。

## Follow-up notes

- 后续如果继续调整 LLM prompt，应同步检查 `config.rs`、`config.example.toml`、`src/lib/app/defaults.ts`、提示词预览 i18n、README、英文 README、Wiki 草稿和架构说明。
- 如果继续新增设置项，优先复用已有 `toggle-grid` / `check` 样式，避免单个新增项在同组配置中形成不必要的视觉强调。
