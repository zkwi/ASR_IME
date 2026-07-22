use crate::{app_log, config::UpdateConfig};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Duration;

const USER_AGENT: &str = concat!("VoxType/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatus {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub asset_name: Option<String>,
    pub asset_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallUpdateResult {
    pub version: String,
    pub asset_name: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

#[derive(Debug, Clone)]
struct ReleaseAsset {
    name: String,
    url: String,
    size: u64,
}

pub async fn check_for_update(config: &UpdateConfig) -> Result<UpdateStatus, String> {
    let release = fetch_latest_release(config).await?;
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version = clean_version(&release.tag_name);
    let asset = choose_windows_installer(&release.assets);
    let update_available = compare_versions(&latest_version, &current_version) == Ordering::Greater;
    app_log::info(format!(
        "更新检查完成: current={} latest={} available={} asset={}",
        current_version,
        latest_version,
        update_available,
        asset
            .as_ref()
            .map(|item| item.name.as_str())
            .unwrap_or("none")
    ));

    Ok(UpdateStatus {
        current_version,
        latest_version,
        update_available,
        asset_name: asset.as_ref().map(|item| item.name.clone()),
        asset_size: asset.as_ref().map(|item| item.size),
    })
}

pub async fn download_and_install(config: &UpdateConfig) -> Result<InstallUpdateResult, String> {
    let release = fetch_latest_release(config).await?;
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_version = clean_version(&release.tag_name);
    if compare_versions(&latest_version, current_version) != Ordering::Greater {
        return Err("当前已是最新版本，无需安装。".to_string());
    }
    let asset = choose_windows_installer(&release.assets).ok_or_else(|| {
        "最新 Release 未找到 Windows 安装包，请打开 GitHub Release 页面手动下载。".to_string()
    })?;

    app_log::info(format!(
        "开始下载更新安装包: version={} asset={} size={}",
        latest_version, asset.name, asset.size
    ));
    let installer_path = download_asset(&asset).await?;
    launch_installer(&installer_path)?;
    app_log::info(format!(
        "更新安装程序已静默启动: version={} asset={} path={}",
        latest_version,
        asset.name,
        installer_path.display()
    ));

    Ok(InstallUpdateResult {
        version: latest_version,
        asset_name: asset.name,
    })
}

fn launch_installer(installer_path: &Path) -> Result<(), String> {
    std::process::Command::new(installer_path)
        .args(silent_installer_args())
        .spawn()
        .map_err(|err| format!("启动安装程序失败: {}", err))?;
    Ok(())
}

fn silent_installer_args() -> [&'static str; 2] {
    ["/S", "/R"]
}

async fn fetch_latest_release(config: &UpdateConfig) -> Result<GitHubRelease, String> {
    let repo = config.github_repo.trim();
    if repo.is_empty() || !repo.contains('/') {
        return Err("更新检查配置不完整，请检查 GitHub 仓库地址。".to_string());
    }

    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| format!("创建更新检查客户端失败: {}", err))?;
    let response = client
        .get(url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|err| friendly_network_error("连接 GitHub 更新服务失败", &err.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        app_log::warn(format!(
            "GitHub Release 检查失败: status={} body={}",
            status, body
        ));
        if status.as_u16() == 404 {
            return Err("没有找到 GitHub Release，请确认项目已发布安装包。".to_string());
        }
        return Err(format!("GitHub Release 检查失败，HTTP 状态码 {}。", status));
    }
    response
        .json::<GitHubRelease>()
        .await
        .map_err(|err| format!("解析 GitHub Release 信息失败: {}", err))
}

async fn download_asset(asset: &ReleaseAsset) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|err| format!("创建下载客户端失败: {}", err))?;
    let response = client
        .get(&asset.url)
        .header(reqwest::header::USER_AGENT, USER_AGENT)
        .send()
        .await
        .map_err(|err| friendly_network_error("下载安装包失败", &err.to_string()))?;
    if !response.status().is_success() {
        return Err(format!(
            "下载安装包失败，HTTP 状态码 {}。",
            response.status()
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("读取安装包内容失败: {}", err))?;
    let dir = std::env::temp_dir().join("VoxType-update");
    std::fs::create_dir_all(&dir).map_err(|err| format!("创建更新下载目录失败: {}", err))?;
    let path = dir.join(safe_file_name(&asset.name));
    std::fs::write(&path, &bytes).map_err(|err| format!("保存安装包失败: {}", err))?;
    Ok(path)
}

fn choose_windows_installer(assets: &[GitHubAsset]) -> Option<ReleaseAsset> {
    assets
        .iter()
        .filter(|asset| {
            let lower = asset.name.to_ascii_lowercase();
            lower.ends_with(".exe") && !lower.contains("portable")
        })
        .max_by_key(|asset| installer_score(&asset.name))
        .map(|asset| ReleaseAsset {
            name: asset.name.clone(),
            url: asset.browser_download_url.clone(),
            size: asset.size,
        })
}

fn installer_score(name: &str) -> i32 {
    let lower = name.to_ascii_lowercase();
    let mut score = 0;
    if lower.contains("setup") {
        score += 8;
    }
    if lower.contains("windows") || lower.contains("win") {
        score += 4;
    }
    if lower.contains("x64") || lower.contains("amd64") {
        score += 2;
    }
    if lower.starts_with("voxtype") {
        score += 1;
    }
    score
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = version_parts(left);
    let right_parts = version_parts(right);
    let count = left_parts.len().max(right_parts.len()).max(1);
    for index in 0..count {
        let left_value = *left_parts.get(index).unwrap_or(&0);
        let right_value = *right_parts.get(index).unwrap_or(&0);
        match left_value.cmp(&right_value) {
            Ordering::Equal => {}
            other => return other,
        }
    }
    Ordering::Equal
}

fn version_parts(value: &str) -> Vec<u64> {
    clean_version(value)
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0)
        })
        .collect()
}

fn clean_version(value: &str) -> String {
    value.trim().trim_start_matches(['v', 'V']).to_string()
}

fn safe_file_name(name: &str) -> String {
    let cleaned = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '(' | ')' | ' ') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    if cleaned.trim().is_empty() {
        "VoxType-update-setup.exe".to_string()
    } else {
        cleaned
    }
}

fn friendly_network_error(prefix: &str, error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") {
        format!("{}，请求超时。请检查网络或稍后重试。", prefix)
    } else if lower.contains("dns")
        || lower.contains("connect")
        || lower.contains("connection")
        || lower.contains("proxy")
        || lower.contains("tls")
    {
        format!("{}。请检查网络、代理或防火墙设置。", prefix)
    } else {
        format!("{}: {}", prefix, error)
    }
}

#[cfg(test)]
mod tests {
    use super::{choose_windows_installer, compare_versions, silent_installer_args, GitHubAsset};
    use std::cmp::Ordering;

    #[test]
    fn compares_semver_like_tags() {
        assert_eq!(compare_versions("v0.1.10", "0.1.9"), Ordering::Greater);
        assert_eq!(compare_versions("0.2.0", "0.10.0"), Ordering::Less);
        assert_eq!(compare_versions("v1.0.0", "1.0"), Ordering::Equal);
    }

    #[test]
    fn prefers_nsis_windows_setup_asset() {
        let assets = vec![
            asset("VoxType-v0.2.0-windows-x64-setup.exe"),
            asset("VoxType-v0.2.0-portable.exe"),
            asset("source.zip"),
        ];
        let selected = choose_windows_installer(&assets).expect("installer asset");
        assert_eq!(selected.name, "VoxType-v0.2.0-windows-x64-setup.exe");
    }

    #[test]
    fn launches_nsis_installer_in_silent_mode_and_reopens_app() {
        assert_eq!(silent_installer_args(), ["/S", "/R"]);
    }

    fn asset(name: &str) -> GitHubAsset {
        GitHubAsset {
            name: name.to_string(),
            browser_download_url: "https://example.com/download".to_string(),
            size: 1,
        }
    }
}
