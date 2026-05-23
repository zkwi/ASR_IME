use crate::{app_log, config, main_window};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

pub const SETUP_GUIDE_URL: &str = "https://github.com/zkwi/VoxType/wiki/Setup-Guide";
pub const DOUBAO_ASR_DOCS_URL: &str = "https://www.volcengine.com/docs/6561/1354869?lang=zh";

pub fn open(app: &AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(SETUP_GUIDE_URL, None::<&str>)
        .map_err(|err| format!("打开配置指南失败: {}", err))
}

pub fn open_doubao_asr_docs(app: &AppHandle) -> Result<(), String> {
    app.opener()
        .open_url(DOUBAO_ASR_DOCS_URL, None::<&str>)
        .map_err(|err| format!("打开豆包帮助文档失败: {}", err))
}

pub fn open_if_config_missing(app: &AppHandle) {
    app_log::info("配置文件检查开始。");
    let Ok(loaded) = config::load_config() else {
        app_log::warn("读取配置文件失败，跳过自动打开配置指南。");
        return;
    };
    if loaded.exists {
        app_log::info(format!("配置文件已存在: {}", loaded.path));
        return;
    }

    main_window::show_existing(app, "缺少配置文件");
    match open(app) {
        Ok(()) => app_log::info("未找到 config.toml，已打开配置指南。"),
        Err(err) => app_log::warn(err),
    }
}
