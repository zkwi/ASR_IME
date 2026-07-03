use crate::{app_log, config, update};
use tauri::AppHandle;

#[tauri::command]
pub(crate) async fn check_for_update() -> Result<update::UpdateStatus, String> {
    app_log::info("开始检查软件更新。");
    let loaded = config::load_config()?;
    update::check_for_update(&loaded.data.update)
        .await
        .map_err(|err| {
            app_log::warn(format!("软件更新检查失败: {}", err));
            err
        })
}

#[tauri::command]
pub(crate) async fn download_and_install_update(
    app: AppHandle,
) -> Result<update::InstallUpdateResult, String> {
    app_log::info("开始下载并安装软件更新。");
    let loaded = config::load_config()?;
    match update::download_and_install(&loaded.data.update).await {
        Ok(result) => {
            exit_after_update_installer_starts(app);
            Ok(result)
        }
        Err(err) => {
            app_log::warn(format!("下载并安装软件更新失败: {}", err));
            Err(err)
        }
    }
}

fn exit_after_update_installer_starts(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(800));
        app_log::info("更新安装程序已启动，退出当前版本以释放安装文件。");
        app.exit(0);
    });
}
