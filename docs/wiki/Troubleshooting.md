# 声写 VoxType 常见问题与排障

本页面是 GitHub Wiki `Troubleshooting` 的仓库内草稿镜像，用于避免线上 Wiki 与仓库文档长期漂移。

## 启动白屏或卡在启动页

### 现象

- 打开 VoxType 后主窗口一直白屏。
- 一直停留在“声写正在启动”页面。
- 重装 VoxType 后仍然无法进入主界面。
- 其他依赖 WebView2 的 Windows 桌面应用也可能异常。

### 原因

VoxType 是 Tauri 桌面应用，主窗口依赖 Microsoft Edge WebView2 Runtime 渲染。如果系统里的 WebView2 Runtime 损坏、缺失、版本过旧，或被系统策略阻断，应用前端还没加载起来，VoxType 自己就无法展示完整的排障界面。

因此不建议 VoxType 在应用内自动下载、提权或静默修复系统组件。更稳妥的做法是让安装包负责缺失时安装 Runtime；如果系统 Runtime 已损坏，再由用户按下面步骤手动覆盖修复。

### 推荐修复步骤

1. 关闭 VoxType。
2. 打开微软官方 WebView2 下载页：<https://developer.microsoft.com/microsoft-edge/webview2/>
3. 下载 **Evergreen Standalone Installer**，Windows 10/11 普通 64 位电脑选择 `x64`。
4. 将安装器保存到本地，例如 `C:\Temp\MicrosoftEdgeWebView2RuntimeInstallerX64.exe`。
5. 右键安装器，选择“以管理员身份运行”，执行覆盖安装。
6. 安装完成后重启 VoxType。
7. 如果仍然白屏，重启 Windows 后再打开 VoxType。

也可以在管理员 PowerShell 中运行：

```powershell
Start-Process -Wait -Verb RunAs "C:\Temp\MicrosoftEdgeWebView2RuntimeInstallerX64.exe" -ArgumentList "/silent", "/install"
```

### 验证是否修复

- VoxType 主窗口能正常显示首页。
- 任务管理器中能看到 WebView2 相关进程随 VoxType 启动。
- Microsoft Edge 或其他 WebView2 应用也能正常打开页面。

### 仍然失败时反馈这些信息

- Windows 版本。
- VoxType 版本。
- 是否安装过系统精简、卸载 Edge/WebView2、组策略屏蔽浏览器组件或安全软件拦截。
- WebView2 覆盖安装是否成功，安装器是否报错。
- 是否能正常打开 Microsoft Edge。

不要发送真实 API Key、热词、prompt、识别正文、未脱敏日志或包含 Windows 用户名路径的截图。

## 麦克风无法录音

先检查 Windows 权限：

```text
Windows 设置 -> 隐私和安全性 -> 麦克风 -> 允许桌面应用访问麦克风
```

然后进入 VoxType 选项页选择正确的输入设备。若使用蓝牙耳机，建议确认 Windows 当前默认输入设备不是免提通话残留设备。

## 快捷键没有反应

- 默认触发方式是 `Ctrl + Q`。
- 右 Alt 和鼠标中键默认关闭，需要在选项页手动启用。
- 如果其他软件占用了 `Ctrl + Q`，先换一个全局快捷键。
- 以管理员身份运行的目标软件可能拦截普通权限程序的模拟输入，可尝试同样以管理员身份启动 VoxType。

## 识别成功但没有粘贴

先按 `Ctrl + V` 手动粘贴。如果文字能粘贴，说明识别和剪贴板写入成功，问题通常在目标软件拦截模拟粘贴。

可在选项页尝试：

- 在“自动粘贴”和“仅复制到剪贴板”之间切换。
- 调整粘贴延迟。
- 改用另一个目标输入框验证。

## 更新异常

如果应用内更新失败，可以到 GitHub Releases 手动下载最新安装包：

<https://github.com/zkwi/VoxType/releases>

安装新版本前通常不需要卸载旧版本。若安装器提示文件占用，退出托盘中的 VoxType 后重试。
