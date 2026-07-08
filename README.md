# 声写 VoxType - 基于 Rust/Tauri 的 Windows AI 语音输入工具

[English](README.en.md) | 简体中文

声写（VoxType）是一个基于 Rust/Tauri 的 Windows 10/11 桌面 AI 语音输入、语音转文字和听写工具。把光标放到任意输入框后，按下全局热键开始说话，程序会录制麦克风音频，通过所选 ASR 服务实时识别语音（默认豆包流式 ASR，可切换阿里云 FunASR Realtime），可选调用 OpenAI 兼容大模型润色文本，并将最终结果写入剪贴板后自动粘贴到当前输入位置。

当前代码已迁移为根目录 Tauri 项目：Rust 负责全局热键、输入钩子、音频采集、ASR 会话、剪贴板、系统托盘、悬浮字幕窗和系统音量；Svelte 负责主窗口 GUI。

> 这是个人项目，目标是实用、轻量、易修改。请勿把真实密钥、个人热词、上下文或本地日志提交到仓库。

## 适合场景

- 在 Windows 任意输入框中进行中文语音输入、英文听写或多语言语音转文字。
- 用豆包流式 ASR 或阿里云 FunASR 获取实时字幕和最终转写结果，再自动粘贴到微信、浏览器、编辑器、表单或办公软件。
- 对长句口述内容做轻度 LLM 润色，减少错别字、口语冗余和格式混乱。
- 需要一个本地运行、可审计、默认不记录识别正文统计的开源语音输入工具。

## 文档

- 仓库内文档索引：[docs/README.md](docs/README.md)
- Wiki 首页：<https://github.com/zkwi/VoxType/wiki>
- 用户配置指南：<https://github.com/zkwi/VoxType/wiki/Setup-Guide>
- 功能特性与使用优化：<https://github.com/zkwi/VoxType/wiki/Feature-Guide>
- 常见问题与排障：<https://github.com/zkwi/VoxType/wiki/Troubleshooting>
- 贡献指南：[CONTRIBUTING.md](CONTRIBUTING.md)
- 安全策略：[SECURITY.md](SECURITY.md)
- 支持说明：[SUPPORT.md](SUPPORT.md)
- 行为准则：[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- 许可证：[MIT](LICENSE)

## 界面预览

主界面采用蓝白配色和紧凑侧边栏，首页顶部语音卡片集中展示当前输入状态，并用单行紧凑标签展示三种启动方式（主快捷键、鼠标中键、右 Alt）。识别完成后，首页会显示本次输入已复制并尝试粘贴，用户可临时复制或查看识别文本；文本只保留在当前窗口，关闭窗口或开始下一次录音后清除。下方展示最近 24 小时、最近 7 日、输入速度和节省时间等输入表现统计；节省时间按“手打等效时间 - 实际语音时长”估算。

<img src="screenshots/ScreenShot_2026-05-09_130744_793.png" alt="VoxType 中文首页：语音输入状态、启动方式和输入表现" width="820">

左侧导航按使用任务拆分为首页、热词与提示词、API配置、选项、隐私与本地数据和统计分析：热词与提示词页优先展示常用热词、场景上下文、最近上下文和自动热词候选，低频 Prompt 参数折叠在高级区；API配置页管理 ASR 服务商、豆包/阿里云必要认证字段与可选大模型 API，地域、模型、语言和 thinking 等兼容项折叠在高级区；选项页按常用设置、体验增强、应用维护组织快捷键、粘贴方式、麦克风、字幕外观、开机启动、关闭行为、更新和诊断，备用触发和录音排障默认折叠；隐私与本地数据页说明配置文件、日志、最近上下文、候选生成历史、统计、运行时数据的保存位置、上传边界和清理入口，并通过“管理设置”跳转回对应设置页。热词与提示词、API配置和选项页直接从正文分组开始，不再重复显示通用配置状态头部；可见设置按页面分组直接展示，协议地址、剪贴板快照、重试次数等底层参数仅支持通过 `config.toml` 修改。统计页展示最近 24 小时、最近 7 日、平均输入速度和按日使用情况，新识别结果写入后会刷新。

API配置页顶部提供配置健康检查，只把真正阻断主流程的问题放到显眼位置。当前 ASR 服务认证、麦克风、粘贴方式、触发方式和隐私设置会分别展示状态，豆包/阿里云 ASR 与可选大模型配置区域提供测试入口。截图中的密钥已做模糊处理，公开截图时也应保持脱敏。

<img src="screenshots/ScreenShot_2026-05-09_130838_673.png" alt="VoxType 中文 API配置与健康检查界面" width="820">

录音和处理过程中会在当前屏幕居下显示悬浮字幕，用于实时查看转写内容与必要状态提示；字幕会先清理 ASR 中间态产生的中文间多余空格，短文本保持单行显示，长文本、实际换行或 ASR 返回换行时最多显示两行，并在不裁切的前提下尽量放大字号；尺寸不足时自动退回单行滚动。字幕配色和透明度可在选项页调整，尺寸、位置和自定义颜色保留在 `config.toml`。

<img src="screenshots/ScreenShot_2026-04-24_150427_629.png" alt="VoxType 实时字幕悬浮窗" width="560">

## Windows 语音输入功能

- 全局触发：默认只启用 `Ctrl + Q`；右 Alt 和鼠标中键可在选项页手动开启，避免误触或与其他软件冲突。
- 麦克风采集：使用 Rust `cpal` 采集 PCM 音频，可选择输入设备。
- 实时语音识别：默认对接豆包 `bigmodel_async` WebSocket，也可在 API配置页切换到阿里云 FunASR Realtime；字幕只作反馈，最终粘贴必须等待所选 ASR 服务返回最终完成事件。豆包链路继续使用二遍识别和 `full` 全量结果，阿里云链路等待 `task-finished` 后再进入润色/粘贴。
- 无反馈兜底：ASR 连续 30 秒没有返回有效文本反馈时，会按正常停录流程收尾；不再依赖本地音量阈值。
- 悬浮字幕：录音时在屏幕居下显示实时识别文本，不抢焦点；短文本保持一行，长文本、实际换行或 ASR 返回换行时最多显示两行。选项页展示字幕预览、预设配色和透明度预设，自定义颜色、宽高和位置保留在 `config.toml`。
- 自动输入：最终文本写入剪贴板，并用带扫描码和短间隔的 `Ctrl+V` 或 `Shift+Insert` 粘贴到当前焦点输入框；首页可临时展开查看并一键复制最近一次识别文本，关闭窗口或开始下一次录音后清除。选项页直接提供 `Ctrl+V`、`Shift+Insert` 和“仅复制到剪贴板”，剪贴板恢复延迟等底层参数保留在 `config.toml`。
- 标点处理：默认会自动移除最终文本末尾的中文句号或英文句点；如需保留句末标点，可在选项页关闭该开关。
- 可选润色：可调用 OpenAI 兼容接口做轻度后处理；API配置页可直接测试当前 ASR 服务和大模型 Key 是否可用，热词与提示词页管理大模型提示词和“润色时参考最近上下文”开关。
- 屏幕 OCR 上下文：默认开启。开始录音时默认截取当前显示器，也可在选项页切换为仅当前窗口；用 Windows OCR 提取文字后，轻量合并中文字符之间的多余空格，并作为临时上下文发送给当前 ASR 服务和可选大模型，帮助识别人名、文件名、代码标识符和界面词；发给大模型前会按预算压缩，超时或失败会自动跳过。
- 自动热词候选：可在热词页开启本地采集 VoxType 最终语音输入文本，并手动调用已配置的大模型生成候选热词；候选必须用户勾选确认后才会加入热词列表。
- 系统音量：可在 `config.toml` 中配置录音期间临时静音系统音量，结束后恢复；默认关闭，避免影响会议、视频或系统提示音。
- 托盘常驻：关闭主窗口默认隐藏到托盘，输入和处理期间托盘图标会切换为输入中样式；左键单击托盘图标打开主窗口，右键托盘可打开配置、查看日志、问题反馈、检查更新、重启程序或退出，也可在选项页改为直接退出或每次询问。
- 开机启动：可在选项页开启随 Windows 登录自动启动。
- 检查更新：可在选项页通过 GitHub Release 检查最新版；发现新版本时提示中会提供“立即更新”按钮，下载后自动启动 Windows 安装包，应用内更新会尽量静默安装并退出当前版本释放文件。
- 诊断日志：选项页和托盘均可打开本地日志，也可一键复制脱敏诊断报告，便于排查识别、粘贴、网络和更新问题。
- 隐私与本地数据：左侧导航提供入口，可查看配置文件与密钥、日志与诊断报告、最近上下文、候选生成历史、统计、ASR 音频、屏幕 OCR、LLM 润色文本和剪贴板的保存/上传边界，并清空本地上下文、候选生成历史和统计数据。
- 配置健康检查：API配置页顶部显示 ASR 密钥填写/未测试/测试结果、麦克风、粘贴方式、触发方式和隐私设置状态，帮助新用户快速知道还差哪一步。
- 多语言界面：简体中文、繁体中文、英语，默认简体中文。

## 主链路保护

这些行为直接影响普通用户对语音输入结果的信任，维护时应保持：

- 空识别会进入失败态并提示“没有识别到文字”，不会显示“已粘贴”，也不会触发润色、粘贴或成功统计。
- 只有在大模型润色已启用、润色触发长度达到 `min_chars`，且 Base URL、API Key、模型名都填写完整时，界面才显示“正在润色文本”。
- 默认自动粘贴后恢复原剪贴板；纯文本恢复最稳定。原剪贴板包含大块表格、图片、文件、位图句柄或部分私有格式时，可能无法完整恢复并给出 warning；快照大小上限用于避免大剪贴板导致卡顿。
- 健康检查的“已准备好”只判断 ASR 密钥、麦克风和至少一种触发方式。ASR 连接测试只在用户手动测试成功后显示“测试通过”；未测试不阻断主流程可用性。

## 环境

仅面向 Windows 10/11。

普通用户请下载并运行 `VoxType-*-setup.exe` 安装包。安装包会内置 Microsoft Edge WebView2 Bootstrapper，在系统缺少 WebView2 Runtime 时自动安装运行时。

如果主窗口一直白屏或长时间停留在启动页，通常是系统的 Microsoft Edge WebView2 Runtime 损坏、缺失或被策略阻断。请先按 [常见问题与排障](https://github.com/zkwi/VoxType/wiki/Troubleshooting) 中的“启动白屏或卡在启动页”处理，不建议在 VoxType 内自动修复系统组件。

项目不再发布绿色版 ZIP。绿色版不会安装系统运行时，容易在干净电脑上出现缺少 WebView2 Runtime 的问题。

运行时还需要 Windows 允许桌面应用访问麦克风。若录音失败，请在“设置 → 隐私和安全性 → 麦克风”中开启麦克风访问权限。

开发构建需要安装：

- Node.js 和 npm
- Rust 工具链

如果 Rust 已安装但当前终端找不到 `cargo`，先执行：

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;$env:PATH"
```

## 配置

首次使用可以参考配置指南：[Setup Guide](https://github.com/zkwi/VoxType/wiki/Setup-Guide)。如果安装版启动时找不到 `config.toml`，程序会自动打开该指南；主窗口首页也会显示配置健康检查，提示还缺少哪些配置。

VoxType 默认使用豆包 ASR，也可以在 API配置页切换到阿里云 FunASR Realtime。当前 ASR 服务需要的认证信息是主流程必填项；未填写时，主窗口会优先引导到 API配置页，录音、识别、粘贴等后续入口会被锁定。API配置页会显示“填写密钥、测试连接、回首页开始输入”的三步引导；填写后会自动保存并生效。

配置速查：

| 场景 | 必填 | 可先不填 | 测试入口 |
| --- | --- | --- | --- |
| 只想语音转文字 | 当前 ASR 服务认证信息（豆包 App Key/Access Key，或阿里云 API Key + Workspace ID） | 大模型 API、热词、屏幕 OCR | API配置页的 ASR 测试 |
| 想让文本更自然 | 当前 ASR 服务 + 大模型 Base URL、API Key、模型名，并开启润色 | 自动热词候选 | API配置页的大模型测试 |
| 测试失败 | 先看红色提示、Key 是否填错、网络/代理是否可用 | 不要先改高级参数 | 复制脱敏诊断报告排查 |

复制配置模板：

```powershell
Copy-Item .\config.example.toml .\config.toml
```

默认豆包配置示例：

```toml
[asr]
provider = "doubao"

[auth]
app_key = ""
access_key = ""
resource_id = "volc.seedasr.sauc.duration"
```

VoxType 当前按豆包流式语音识别文档发送 `X-Api-App-Key`、`X-Api-Access-Key` 和 `X-Api-Resource-Id` 三个请求头。`resource_id` 默认使用豆包语音识别大模型 2.0 小时版的 `volc.seedasr.sauc.duration`；如果你的账号开通的是并发版或旧模型，才需要按火山引擎控制台/文档改成对应值。不要把大模型平台的 API Key、GitHub Token、火山引擎 IAM Secret 等填到这里。API配置页的豆包认证区域提供豆包帮助文档入口，方便首次配置时对照官方字段说明。

API配置页默认只展示服务商和认证字段。豆包 ASR 的输入语言收在“高级连接与语言设置”里，默认选择自动/服务默认，会省略请求里的 `language` 参数；当前主链路使用 `bigmodel_async + enable_nonstream` 二遍识别，豆包文档说明二遍不支持 `language`，留空更适合中文、英文、方言和混合输入。中文普通话无需设置，旧配置里的 `zh-CN` 会自动迁移为空；只有明确要排查非默认语种时，再切换为 `en-US`、`ja-JP`、`yue-CN` 等代码。

阿里云 FunASR Realtime 配置示例：

```toml
[asr]
provider = "aliyun_fun"

[aliyun_asr]
api_key = ""
workspace_id = ""
region = "cn-beijing"
websocket_url = ""
model = "fun-asr-realtime"
language_hint = ""
semantic_punctuation_enabled = false
max_sentence_silence = 1300
vocabulary_id = ""
```

阿里云模式下，VoxType 使用 Bearer API Key 建立 WebSocket 连接，发送 `run-task`、PCM 音频帧和 `finish-task`，并等待服务端 `task-finished` 后才把最终文本交给润色/粘贴链路。API配置页日常只需要填写 API Key 和 Workspace ID；地域、模型、自定义 WebSocket 地址和语言提示收在高级区。`workspace_id` 会自动拼接为 `wss://{WorkspaceId}.{region}.maas.aliyuncs.com/api-ws/v1/inference`；只有阿里云文档或控制台要求自定义 endpoint 时才填写 `websocket_url`。`language_hint` 默认留空使用自动识别；固定语种可填 `zh`、`en`、`ja`、`ko`、`yue`。

ASR 测试常见失败原因：

| 提示 | 优先检查 |
| --- | --- |
| 认证或权限失败 | 当前服务商的 Key、资源、Workspace、地域和模型是否属于同一账号并已开通 |
| 连接失败或超时 | 网络、代理、防火墙是否能访问当前 ASR 服务 endpoint |
| 语言参数相关失败 | 先把识别语言/语言提示改成自动/服务默认，再重新测试 |
| 测试通过但录音无字 | Windows 麦克风权限、输入设备、音量和环境噪声 |

正式录音链路同样会区分连接超时、连接失败、等待最终结果超时和连接提前关闭。本轮失败后会进入失败态并短暂提示错误，下一次按快捷键会重新开始一轮识别，不会一直停在“等待最终结果”。

如果启用大模型润色，还需要填写：

```toml
[llm_post_edit]
enabled = true
min_chars = 40
use_recent_context = false
screen_context_max_chars = 400
screen_context_max_lines = 12
recent_context_max_chars = 200
reference_hotwords_limit = 50
api_key = ""
base_url = "https://dashscope.aliyuncs.com/compatible-mode/v1"
model = "qwen3.5-plus"
enable_thinking = false
thinking_strategy = "auto"
```

大模型配置走 OpenAI 兼容接口，API配置页日常只展示启用润色、Base URL、API Key、模型和测试按钮；`thinking_strategy` 收在高级兼容性设置里，通常保持自动即可。默认示例使用阿里云百炼/DashScope 的北京地域地址 `https://dashscope.aliyuncs.com/compatible-mode/v1`。Base URL 可填写服务根地址、`/v1` 地址或完整 `/chat/completions` 地址；例如 `https://api.deepseek.com`、`https://api.deepseek.com/v1/`、`https://api.deepseek.com/v1/chat/completions` 会作为等价地址处理。`api_key` 必须来自同一个大模型服务商，`model` 必须是该账号和地域可用的模型名。只需要语音识别时可以完全不配置大模型；开启润色后，短文本默认低于 `min_chars` 不会调用大模型，以减少延迟；已保存的 `min_chars` 会按用户配置原样保留，不再按旧默认值猜测迁移。`min_chars` 使用润色触发长度：中文等 CJK 字符按单字计，英文和数字按连续词片段计，空格和标点不计；因此默认 `40` 约等于 40 个汉字或 40 个英文/数字片段。达到最小触发长度的最终识别文本会发送到你配置的大模型服务；用户词典、场景与产品偏好、按预算压缩后的屏幕 OCR 和可选最近上下文会作为参考信息追加。最近上下文进入大模型默认关闭，只有 `[context].enable_recent_context` 和 `[llm_post_edit].use_recent_context` 同时开启时才会发送，且默认限制为最近几段中的约 200 字。屏幕 OCR 发给大模型前会按行去空、去重，并默认限制为 12 行 / 400 字；热词参考默认最多 50 条，这些 LLM 参考预算保留在 `config.toml`，不作为普通用户日常设置。真实润色请求会按输入长度设置输出上限，减少模型生成过长导致的等待。`thinking_strategy = "auto"` 会按服务商选择关闭思考/推理的兼容写法，例如 DashScope 使用 `enable_thinking=false`，DeepSeek 和 MiMo 使用 `thinking.type=disabled`，OpenRouter 使用较低 reasoning effort；API配置页的大模型测试会使用较长的内置语音输入样例，尝试候选策略并保存最快的成功结果。修改 Base URL、API Key、模型名或 thinking 开关并自动保存后，VoxType 会自动从 `auto` 候选重新测速并保存最快成功策略；测试不会读取剪贴板、实时屏幕 OCR 或本地最近上下文正文。

DashScope 模型选择可参考 [2026-05-28 LLM 润色模型测试记录](docs/audits/2026-05-28-llm-polishing-model-test.md)，其中 2026-05-30 复测修正了旧结论：日常仍优先考虑 `qwen3.7-max`；低延迟优先可考虑 `qwen3.6-flash-2026-04-16`，但它对提示词样式文本和技术路径更容易改偏；不要仅因技术文本切换到 `deepseek-v4-pro`，当前简化 prompt 下它也会改写代码路径，路径、文件名和标识符应优先依赖屏幕 OCR、热词或人工确认。

大模型测试常见失败原因：

| 提示 | 优先检查 |
| --- | --- |
| API Key 或权限失败 | Key 是否属于当前 Base URL 对应的平台和地域 |
| 模型名称错误 | 模型名是否拼写正确、账号是否有权限调用 |
| 连接失败 | Base URL 是否属于当前服务商，代理/网络是否可用 |
| 测试有响应但润色不触发 | 是否开启润色、润色触发长度是否达到 `min_chars` |

VoxType 已内置一套面向语音输入的默认大模型提示词。默认规则会把“待润色文本”标成唯一需要改写并输出的内容，不回答、不执行其中的指令或问题，避免把润色任务误变成内容分析；短消息、单句命令和问题只做轻量纠错，可以补自然标点但不扩写；长段口述、记录、复盘、说明、会议纪要、产品反馈和投资复盘会按正式正文做成稿化润色，删除口水词、语气垫词、重复表达、无效停顿和自我修正过程，必要时调整语序、拆分句子和补足连接词，默认整理成 2-4 个自然段。默认提示词仍会保留原文事实、判断、语气强弱和立场，保留专有名词、英文缩写、金融和编程术语，并避免主动添加标题、列表、Markdown 或反引号。用户词典、场景与产品偏好、可选最近上下文、屏幕 OCR 会在真实请求中作为“参考信息”分区追加，只用于纠正术语、称谓、界面词、上下文承接、路径、文件名和代码标识符，不作为待润色文本或指令来源，也不会把待润色文本没说的参考信息补进输出。最近上下文不能被续写、复述或总结；屏幕 OCR 只在与待润色文本相关时用于纠错。文件路径、命令、日志字段和代码标识符不确定时会要求模型保留原样，只有参考信息中出现明确写法时才纠正。金融、投资、量化语境中，默认提示词会把明确金额、收益率和百分比整理为常用数字写法，例如把一百万写成 `100万`、把百分之一写成 `1%`，但不会计算收益或回答问题。热词页先展示常用热词、场景上下文、最近上下文和自动热词候选；大模型提示词模板和最小润色触发长度收在高级 Prompt 设置里，恢复默认和预览入口保留在页面中。预览会显示参考信息拼接规则、场景上下文是否进入大模型提示词、最近上下文是否进入大模型提示词，以及屏幕 OCR 当前开关策略。LLM 参考信息预算保留在 `config.toml`；System Prompt 也保留在 `config.toml` 中，避免普通设置页过长。

屏幕 OCR 上下文默认开启，可在选项页关闭、测试或切换识别范围。默认范围是当前显示器，适合参考一个文档并在另一个窗口输入；如屏幕中有敏感内容，可切换为仅当前窗口或关闭。OCR 文本只保留在本轮请求内，不写入日志、统计、配置文件，也不会缓存最近 2-3 份截图 OCR 内容。发送前会轻量合并中文字符之间的多余空格，避免 `屏 幕 OCR 上 下 文` 这类结果干扰上下文；英文缩写、快捷键和路径间距会尽量保留。ASR 建连前默认最多等待 500ms 获取 OCR 上下文，失败或超时会跳过，不影响录音、最终识别和粘贴主链路。发给 ASR 的 OCR 仍受 `[screen_context].max_chars` 控制；发给大模型润色的 OCR 会另按 `[llm_post_edit]` 预算压缩，默认最多 12 行 / 400 字。

```toml
[screen_context]
enabled = true
capture_scope = "screen"  # screen = 当前显示器，window = 仅当前窗口
max_chars = 1200
timeout_ms = 500
```

热词页还提供“自动生成热词候选”。该功能默认关闭，开启后只保存 VoxType 自己生成的最终语音输入文本，不记录键盘输入，不读取剪贴板历史。只有用户点击“生成候选”时，才会把本地历史摘要发送到已配置的大模型服务；生成结果只是候选，必须勾选确认后才会合并到 `context.hotwords`。历史文本默认上限为 5000 字；已保存的上限会按用户配置保留，不再按旧默认值自动改写。热词生成会使用比普通润色更高的输出长度和超时预算；如果完整历史生成返回不完整或超时，会自动缩小历史范围并减少候选数量重试一次。若仍失败，可在 `config.toml` 中降低历史文本上限或候选数量后重试。发送给 ASR 的直传/上下文热词会做数量限制，手动热词优先于自动确认热词，避免请求过大影响实时识别。

```toml
[auto_hotwords]
enabled = false
max_history_chars = 5000
max_candidates = 30
ignored_hotwords = []
```

填写当前 ASR 服务或大模型 Key 后，可在 API配置页点击对应区域的“测试”按钮，先确认 Key、Base URL、模型名称和网络环境是否可用，再开始正式录音。

配置修改后会自动保存，标题栏会短暂显示“更改将自动保存 / 正在保存设置 / 设置已保存”。自动保存前会做基础字段校验，明显非法的采样率、声道数、录音时长、粘贴延迟、剪贴板恢复延迟、快照大小、枚举值、URL scheme、GitHub 仓库格式、LLM 必填项、超时时间、悬浮窗尺寸和字幕颜色不会写入配置文件。

录音相关配置：

```toml
[asr]
no_feedback_auto_stop_seconds = 30

[audio]
max_record_seconds = 300
stop_grace_ms = 250
input_gain_db = 0.0
mute_system_volume_while_recording = false
```

VoxType 会把麦克风实际采集到的 PCM 音频统一转换为豆包大模型流式 ASR 支持的 `16000Hz`、单声道、16-bit PCM 后再发送；`sample_rate` 和 `channels` 只作为底层采集偏好，普通用户不需要修改。ASR 实际发送分片会限制在豆包建议的 `100-200ms`，默认 `200ms`。

在选项页选择麦克风后，VoxType 会同时保存设备名称和旧版数字 index；下次启动录音时优先按名称匹配，蓝牙耳机重连或设备枚举顺序变化时更不容易选错。若保存的麦克风已不可用，会自动回退系统默认输入设备并显示非阻塞提示。

豆包文档没有要求客户端必须开启自动增益，也没有单独的增益请求参数；VoxType 默认保持 `input_gain_db = 0.0`，不额外放大麦克风音频。只有当录音质量卡片反复提示音量偏小、且系统麦克风音量和距离已确认正常时，才建议在录音排障中手动小幅提高输入增益，例如先试 `+3 dB` 或 `+6 dB`；避免过高增益造成削波或放大环境噪声。

录音结束后，首页会按需显示一张轻量的录音质量卡片，展示最近一轮的 RMS、峰值、有效语音比例和建议；如果本轮已经成功识别并输出文本，低有效语音占比提示会被隐藏，避免把可用结果误报为问题。这些指标不包含正文，也不会写入主统计表。

VoxType 会把约 `50ms` 头部静音并入第一包真实音频，第一包总时长仍按当前分片配置发送，默认约 `200ms`，帮助豆包稳定起始识别，同时避免发送独立 50ms 小包；如果没有采集到真实音频，则不会额外发送静音包。豆包返回的中间识别文本会用更短的本地节流尽快显示在悬浮字幕中，过快的新字幕会合并最新文本并按时补发；同一响应里如果分句累计文本比 `result.text` 更完整，字幕优先显示更完整的累计文本，但最终粘贴仍必须等待豆包最终包。

近期实测较稳定的组合是：保留默认 `200ms` ASR 分片，把体感速度优化放在 `20ms` 响应轮询、`50ms` 字幕节流和 `500ms` OCR 上下文等待上；首字返回加速默认关闭，优先保障开头准确率。最终文本仍只接受豆包最终包，并优先使用最终包里的 `result.text` 整段文本。`definite=true` 分句用于稳定最终结果，但当最终包与分句高度重合且补齐了开头或尾字时，即使最终包对前文有轻微缩写，也应优先保留最终包整段文本，避免回退成缺字输出。

首字返回加速默认关闭，`enable_accelerate_text = false` 且 `accelerate_score = 0`；已保存的显式取值会保留。如果特别看重实时字幕起步速度，可在 `config.toml` 中手动开启，但首字准确率可能下降。

语义顺滑 `enable_ddc` 默认开启，用于短文本和中等文本的 ASR 侧轻量顺滑，减少短文本依赖 LLM 润色带来的等待；已保存的显式取值会保留，如果更看重专有词、短命令、路径或标点敏感内容的原样识别，可以手动关闭。

录音过程中如果麦克风输入流报错，VoxType 会立即让本轮进入失败态，避免把缺帧或不完整音频识别出的低质量文本继续润色、粘贴或统计。

停录时，`stop_grace_ms` 是固定保留的真实尾音等待时间，默认约 `250ms`，不再依赖本地音量检测判断是否延长。这样更换低音量麦克风后，也不会因为音量阈值误判而提前切断尾音。不足一片的尾部音频会在关闭麦克风前补发给 ASR，不再额外追加尾部静音，最后一个音频包会直接作为负包发送，帮助豆包触发二遍最终判停。

ASR 无反馈自动停止默认开启，`no_feedback_auto_stop_seconds = 30`。如果 ASR 服务在 30 秒内没有返回任何有效文本反馈，VoxType 会按手动停录同样的尾音收尾流程结束录音，然后继续等待服务端最终包；填 `0` 可关闭。它不再依赖本地音量阈值，因此低音量麦克风不会因为静音误判而被提前截断。

文本后处理相关配置：

```toml
[typing]
remove_trailing_period = true
```

开启后，最终文本以中文句号或英文句点结尾时会自动去掉；关闭后会保留 ASR 或大模型输出的句末标点。

剪贴板恢复相关配置：

```toml
[typing]
clipboard_restore_delay_ms = 800
clipboard_snapshot_max_bytes = 8388608
```

恢复延迟越长，目标应用越有时间读取语音文本；但原剪贴板恢复也会更晚。为避免慢应用粘贴到旧剪贴板内容，自动粘贴时会使用不低于 500ms 的安全恢复等待。快照大小上限用于跳过过大的格式，降低大剪贴板卡顿风险。

如需随 Windows 登录自动启动，可在选项页开启，或在 `config.toml` 中设置：

```toml
[startup]
launch_on_startup = true
```

更新检查默认读取 `zkwi/VoxType` 的 GitHub Release。需要关闭启动自动检查时，可在选项页关闭，或在 `config.toml` 中设置：

```toml
[update]
auto_check_on_startup = false
github_repo = "zkwi/VoxType"
```

`config.toml`、本地日志和统计文件已被 `.gitignore` 忽略。示例配置和文档只保留占位值，不应写入真实密钥、个人热词或自定义上下文。

## 配置和日志在哪里

开发运行时，VoxType 继续使用仓库根目录的 `config.toml` 和 `voice_input.log`，方便调试和版本管理。

安装版默认使用 Windows 用户数据目录：

- 配置文件：`%APPDATA%\VoxType\config.toml`
- 日志文件：`%LOCALAPPDATA%\VoxType\logs\voice_input.log`

如果安装版启动时发现旧位置已有 VoxType `config.toml`，且新默认位置还没有配置文件，主窗口会提示是否迁移。确认后只复制一次旧配置到新位置，不做复杂备份，也不会删除旧文件。

## 开发运行

在仓库根目录执行：

```powershell
npm install
npm run tauri dev
```

开发服务固定使用：

```text
http://127.0.0.1:18080
```

没有继续使用 Tauri 模板默认的 `1420` 端口，因为部分 Windows 环境会把相邻端口段保留给系统，导致 Vite 报 `listen EACCES`。

## 构建

调试构建：

```powershell
npx tauri build --debug --no-bundle
```

正式构建：

```powershell
npx tauri build
```

NSIS 安装包会嵌入 WebView2 Bootstrapper。首次安装到缺少 WebView2 Runtime 的干净电脑时，安装程序会联网安装该运行时。

安装包内置简体中文、繁体中文和英语。安装时默认根据 Windows 系统语言自动选择安装器语言；不额外弹出语言选择窗口。

正式可执行文件通常位于：

```text
src-tauri\target\release\voxtype-desktop.exe
```

不要直接用 `cargo build --release` 作为桌面端发布产物；那样不会先构建前端资源，可能导致窗口打开后访问开发地址失败。

## 使用

1. 启动 `VoxType`。
2. 先在 API配置页选择 ASR 服务并填写认证信息，填写后会自动保存；未填写时主流程入口会保持锁定。
3. 把光标放到目标输入框。
4. 按 `Ctrl + Q` 开始录音；如已在选项页开启，也可使用右 Alt 或鼠标中键。
5. 录音时查看屏幕居下悬浮字幕。
6. 再按一次触发键停止录音。
7. 程序等待当前 ASR 服务返回最终结果，可选润色，然后自动粘贴到当前焦点输入框。若 ASR 连接提前结束或未返回完整最终结果，本轮会失败而不会粘贴中间结果；若粘贴快捷键发送失败，识别文本会保留在剪贴板，可手动 `Ctrl + V`。

默认隐私与误触策略：

- 最近上下文默认关闭，不保存最近识别片段；需要连续识别增强时可在热词页或隐私与本地数据页手动开启。开启后识别片段写入单独的本地 `context/recent_context.jsonl`，不会写回 `config.toml`；可在隐私与本地数据页清空。最近上下文默认只进入当前 ASR 服务；如需辅助大模型润色，需额外开启“润色时参考最近上下文”。
- 屏幕 OCR 上下文默认开启，默认识别当前显示器，识别结果不落盘、不跨轮缓存；如屏幕可能包含敏感信息，可在选项页切换为仅当前窗口或关闭。
- 自动热词候选默认关闭。开启后本地采集文本写入 `context/hotword_history.jsonl`，可在热词页或隐私与本地数据页清空；诊断报告和日志不会输出历史正文、候选词或 prompt。
- 统计只保存字数、时长、速度等非正文数据，可在隐私与本地数据页清空。
- 右 Alt 和鼠标中键默认关闭，确认不与其他软件冲突后再开启。
- ASR 无反馈自动停止默认 `30` 秒，填 `0` 可关闭；停录尾音窗口会保留用户设置，录音期间静音系统声音默认关闭。
- 最终识别正文默认不打印到控制台。

托盘行为：

- 单击托盘图标：打开主窗口。
- 输入、等待结果、润色和粘贴处理期间：托盘图标显示输入中样式，完成或失败后恢复普通图标。
- 关闭主窗口：默认隐藏到托盘，首次会提示“仍可按快捷键使用，完全退出请右键托盘图标选择退出”。
- 托盘菜单“打开配置”：用系统默认编辑器打开 `config.toml`。
- 托盘菜单“查看日志”：用系统默认程序打开本地日志。
- 托盘菜单“问题反馈”：打开 GitHub Issue 反馈页面。
- 托盘菜单“检查更新”：打开主窗口并执行一次手动更新检查。
- 托盘菜单“重启程序”：停止当前会话并重启 VoxType，适合配置更新后生效或快速恢复偶发异常。
- 托盘菜单“退出”：停止会话并退出程序。

选项页中的“诊断与日志”可以直接打开本地日志或复制诊断报告。日志会记录关键启动阶段、配置保存、ASR/LLM 错误、更新检查和前端异常；密钥形态会自动脱敏。诊断报告默认不包含识别正文、屏幕 OCR 正文、密钥、热词、prompt、最近上下文、候选生成历史正文、候选词或 Windows 用户名路径。

## 常见问题

### VoxType 是什么？

VoxType 是 Windows 桌面语音输入工具，用豆包流式 ASR 或阿里云 FunASR 将麦克风语音转成文字，再复制并自动粘贴到当前输入框。它更像系统级听写助手，不是聊天机器人。

### VoxType 支持哪些输入场景？

只要目标软件能接收剪贴板粘贴，通常就能使用 VoxType，包括浏览器输入框、聊天软件、Markdown 编辑器、IDE、办公文档和内部管理系统。少数拦截 `Ctrl + V` 的软件可以改用 `Shift + Insert` 或仅复制到剪贴板。

### VoxType 会保存我的语音识别正文吗？

默认不会。统计只记录时长、字数、速度等非正文数据；最近上下文和自动热词候选默认关闭。最近上下文开启后会发送给当前 ASR 服务；只有额外开启“润色时参考最近上下文”并实际触发大模型润色时，才会把受限前文作为参考信息发送给大模型。屏幕 OCR 上下文默认开启但不落盘，也不保留最近几次截图 OCR，只在本轮请求中临时发送给 ASR/LLM；其中发给 LLM 的 OCR 会按预算压缩，可在选项页关闭，或从隐私与本地数据页点击“管理设置”跳转调整；本地上下文、候选生成历史和统计可在隐私与本地数据页清空。清空按钮只删除 VoxType 本地文件，第三方 ASR/LLM 服务是否留存取决于对应服务商。

### 为什么需要配置 ASR 服务？

VoxType 的主链路依赖语音识别服务。没有当前服务商需要的认证信息时，录音、识别和自动粘贴入口会保持锁定，避免用户以为已经输入成功。

## 常用命令

```powershell
# 前端类型检查
npm run check

# 前端构建
npm run build

# Rust 检查
Set-Location .\src-tauri
cargo check

# Rust 测试
cargo test

# Rust lint
cargo clippy --all-targets -- -D warnings

# 本地密钥扫描
Set-Location ..
npm run scan:secrets

# 暂存区密钥扫描
npm run scan:secrets:staged

# npm 依赖审计
npm run audit:npm

# Rust 依赖审计（需要先安装 cargo-audit）
npm run audit:rust
```

启用 Git pre-commit 钩子：

```powershell
.\scripts\enable_git_hooks.ps1
```

钩子会调用 `scripts/scan-secrets.mjs` 扫描暂存文件，避免误提交本地配置、密钥、热词和上下文。

## AI 维护与本地检查

本项目允许使用 AI 辅助维护代码，但所有 AI 改动必须遵守根目录 `AGENTS.md`。

日常改动后，在仓库根目录运行：

```powershell
npm run ai:check
```

也可以直接运行：

```powershell
.\scripts\ai-check.ps1
```

该检查会依次执行：

```text
npm run check
npm run build
npm run scan:secrets
npm run test:secrets
cargo fmt --check
cargo check
cargo test
```

如果本次改动涉及 UI，还需要手工检查：

- 首页空闲状态
- 配置缺失状态
- 录音中状态
- 空识别提示
- LLM 关闭和开启两种状态
- 剪贴板纯文本恢复
- 托盘关闭提示
- 简体中文、繁体中文、英文三语言
- 1100 × 680 和 1280 × 760 两类窗口尺寸

更多规则见：

- `AGENTS.md`
- `CHANGELOG.md`
- `docs/code-style.md`
- `docs/directory-structure.md`

仓库包含一个最小 GitHub Actions CI：`.github/workflows/ci.yml`。CI 在 Windows runner 上执行前端类型检查、前端构建、密钥扫描、Rust 格式检查、clippy 和测试；本地仍以 `npm run ai:check` 作为日常提交前入口。

发布前可运行：

```powershell
npm run ai:release-check
```

Rust 依赖审计依赖 `cargo-audit`。本机未安装时先运行：

```powershell
cargo install cargo-audit --locked
```

发布或合并前应同步更新 `CHANGELOG.md` 的 `[Unreleased]` 或对应版本段，记录用户可见变化、主链路保护和维护性调整。

## 开源协作

提交 Issue 或 PR 前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)、[SUPPORT.md](SUPPORT.md) 和 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。本项目欢迎小而明确的 bug 修复、文档改进、配置样例优化和测试补充；涉及 ASR、LLM、剪贴板、热键、托盘、日志、统计或配置结构的改动，请在 PR 描述中明确说明影响范围和验证方式。

安全和隐私问题请参考 [SECURITY.md](SECURITY.md)。公开 Issue、PR、截图和日志中不要包含真实密钥、识别正文、个人热词、prompt、最近上下文或 Windows 用户名路径。

## 界面与适配

主窗口按 1280 × 760 设计，最小窗口为 1100 × 680。首页会根据窗口高度和宽度进入紧凑模式，并将语音输入和启动方式放在同一顶部卡片；启动方式使用单行紧凑标签，最近输入和输入表现分层展示，避免在高 DPI 或较小窗口中出现文字遮挡、卡片裁切和不必要的滚动条。

界面维护时重点检查这些状态：

- 空闲、录音中、配置缺失三种首页状态。
- 简体中文、繁体中文、英文三种语言。
- 1100 × 680、1280 × 760 以及高缩放显示器。
- 侧边栏长麦克风设备名、长热键文本和统计数字较大的情况。

首页只展示正式用户信息。不要加入调试路径、协议细节、内部状态码或占位图表。

## 目录

```text
VoxType/
├── src/                         # Svelte 主窗口界面
├── src-tauri/                   # Tauri/Rust 桌面端
│   ├── src/
│   │   ├── audio.rs             # 麦克风采集
│   │   ├── asr.rs               # ASR 请求组装与结果解析
│   │   ├── asr_ws/              # 豆包 WebSocket 会话模块
│   │   ├── autostart.rs         # Windows 开机自启动
│   │   ├── config.rs            # TOML 配置加载
│   │   ├── hotkey.rs            # 全局热键与输入钩子
│   │   ├── llm_post_edit.rs     # LLM 后处理
│   │   ├── overlay.rs           # 悬浮字幕窗
│   │   ├── session.rs           # 录音会话状态机
│   │   ├── stats.rs             # 使用统计
│   │   ├── system_audio.rs      # 系统音量控制
│   │   ├── text_output.rs       # 剪贴板与粘贴
│   │   ├── tray.rs              # 系统托盘
│   │   └── update.rs            # GitHub Release 更新检查
│   ├── capabilities/
│   ├── icons/
│   └── tauri.conf.json
├── static/                      # 静态资源
├── docs/                        # 接口参考文档
├── scripts/
│   ├── enable_git_hooks.ps1
│   └── scan-secrets.mjs
├── config.example.toml          # 配置模板，不含真实密钥
├── package.json
├── svelte.config.js
├── tsconfig.json
└── vite.config.js
```

## 本地文件

以下文件只用于本机运行，不提交：

- `config.toml`
- `*.local.toml`
- `context/recent_context.jsonl`
- `context/hotword_history.jsonl`
- `voice_input.log`
- `voice_input_stats.jsonl`
- `src-tauri/target/`
- `node_modules/`
- `build/`
