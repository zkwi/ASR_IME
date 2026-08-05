use chrono::Local;
use std::io::Write;
use std::path::{Path, PathBuf};

const MAX_LOG_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ARCHIVE_FILES: usize = 3;
const MAX_MESSAGE_CHARS: usize = 2000;
const REDACTED: &str = "[redacted]";
const APP_DATA_DIR_NAME: &str = "VoxType";
const SENSITIVE_KEYS: &[&str] = &[
    "access_key",
    "app_key",
    "api_key",
    "authorization",
    "bearer",
    "client_secret",
    "password",
    "secret",
    "secret_key",
    "token",
];

pub fn log_path() -> PathBuf {
    if is_development_layout() {
        return resolve_development_log_path();
    }
    installed_log_path().unwrap_or_else(resolve_development_log_path)
}

fn resolve_development_log_path() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("voice_input.log"));
        candidates.push(cwd.join("..").join("voice_input.log"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("voice_input.log"));
            candidates.push(dir.join("..").join("..").join("..").join("voice_input.log"));
        }
    }
    candidates
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|dir| dir.join("voice_input.log")))
        })
        .unwrap_or_else(|| PathBuf::from("voice_input.log"))
}

fn installed_log_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|base| installed_log_path_from_localappdata(&base))
}

fn installed_log_path_from_localappdata(base: &Path) -> PathBuf {
    dunce::simplified(
        &base
            .join(APP_DATA_DIR_NAME)
            .join("logs")
            .join("voice_input.log"),
    )
    .to_path_buf()
}

fn is_development_layout() -> bool {
    if let Ok(cwd) = std::env::current_dir() {
        if cwd.ancestors().any(looks_like_project_root) {
            return true;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.ancestors().any(looks_like_project_root);
        }
    }
    false
}

fn looks_like_project_root(path: &Path) -> bool {
    path.join("package.json").exists() && path.join("src-tauri").is_dir()
}

pub fn info(message: impl AsRef<str>) {
    write_line("INFO", message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    write_line("WARNING", message.as_ref());
}

fn write_line(level: &str, message: &str) {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    rotate_if_needed(&path);
    let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    else {
        return;
    };
    let message = sanitize_message(message);
    let _ = writeln!(
        file,
        "{} {} {}",
        Local::now().format("%Y-%m-%d %H:%M:%S"),
        level,
        message
    );
}

fn rotate_if_needed(path: &PathBuf) {
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.len() < MAX_LOG_BYTES {
        return;
    }

    let oldest = archive_path(path, MAX_ARCHIVE_FILES);
    let _ = std::fs::remove_file(oldest);
    for index in (1..MAX_ARCHIVE_FILES).rev() {
        let from = archive_path(path, index);
        let to = archive_path(path, index + 1);
        if from.exists() {
            let _ = std::fs::rename(from, to);
        }
    }
    let _ = std::fs::rename(path, archive_path(path, 1));
}

fn archive_path(path: &Path, index: usize) -> PathBuf {
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return path.with_extension(format!("log.{}", index));
    };
    path.with_file_name(format!("{}.{}", file_name, index))
}

fn sanitize_message(message: &str) -> String {
    let flattened = message
        .chars()
        .map(|ch| if ch == '\r' || ch == '\n' { ' ' } else { ch })
        .collect::<String>();
    let mut redacted = redact_prefixed_token(&flattened, "bearer ");
    for key in SENSITIVE_KEYS {
        redacted = redact_key_values(&redacted, key);
    }
    redacted = redact_openai_style_keys(&redacted);
    redacted = redact_prefixed_token(&redacted, "ark-");
    redacted = redact_current_user_profile_path(&redacted);

    let mut limited = redacted.chars().take(MAX_MESSAGE_CHARS).collect::<String>();
    if redacted.chars().count() > MAX_MESSAGE_CHARS {
        limited.push_str("...");
    }
    limited
}

fn redact_key_values(message: &str, key: &str) -> String {
    let mut output = message.to_string();
    let mut search_start = 0;
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(relative_pos) = lower[search_start..].find(key) else {
            break;
        };
        let pos = search_start + relative_pos;
        let bytes = output.as_bytes();
        let mut cursor = pos + key.len();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() || !matches!(bytes[cursor], b':' | b'=') {
            search_start = pos + key.len();
            continue;
        }
        cursor += 1;
        while cursor < output.len() && output.as_bytes()[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        let quote = output
            .as_bytes()
            .get(cursor)
            .copied()
            .filter(|&byte| byte == b'"' || byte == b'\'');
        if quote.is_some() {
            cursor += 1;
        }
        let value_start = cursor;
        while cursor < output.len() {
            let byte = output.as_bytes()[cursor];
            if let Some(quote) = quote {
                if byte == quote {
                    break;
                }
            } else if byte.is_ascii_whitespace() || matches!(byte, b',' | b';' | b'}' | b']') {
                break;
            }
            cursor += 1;
        }
        if cursor > value_start {
            output.replace_range(value_start..cursor, REDACTED);
            search_start = value_start + REDACTED.len();
        } else {
            search_start = cursor;
        }
    }
    output
}

fn redact_prefixed_token(message: &str, prefix: &str) -> String {
    let mut output = message.to_string();
    let mut search_start = 0;
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(relative_pos) = lower[search_start..].find(prefix) else {
            break;
        };
        let value_start = search_start + relative_pos + prefix.len();
        let mut cursor = value_start;
        while cursor < output.len()
            && !output.as_bytes()[cursor].is_ascii_whitespace()
            && !matches!(output.as_bytes()[cursor], b',' | b';' | b'}' | b']')
        {
            cursor += 1;
        }
        if cursor > value_start {
            output.replace_range(value_start..cursor, REDACTED);
            search_start = value_start + REDACTED.len();
        } else {
            search_start = cursor;
        }
    }
    output
}

fn redact_openai_style_keys(message: &str) -> String {
    let mut output = message.to_string();
    let mut search_start = 0;
    while let Some(relative_pos) = output[search_start..].find("sk-") {
        let value_start = search_start + relative_pos;
        let mut cursor = value_start;
        while cursor < output.len()
            && !output.as_bytes()[cursor].is_ascii_whitespace()
            && !matches!(
                output.as_bytes()[cursor],
                b',' | b';' | b'}' | b']' | b'"' | b'\''
            )
        {
            cursor += 1;
        }
        if cursor.saturating_sub(value_start) >= 12 {
            output.replace_range(value_start..cursor, REDACTED);
            search_start = value_start + REDACTED.len();
        } else {
            search_start = cursor;
        }
    }
    output
}

fn redact_current_user_profile_path(message: &str) -> String {
    let Ok(profile) = std::env::var("USERPROFILE") else {
        return message.to_string();
    };
    redact_path_with_profile(message, &profile)
}

fn redact_path_with_profile(message: &str, profile: &str) -> String {
    if profile.is_empty() {
        return message.to_string();
    }
    let profile_lower = profile.to_ascii_lowercase();
    let mut output = message.to_string();
    let mut search_start = 0;
    loop {
        let lower = output.to_ascii_lowercase();
        let Some(relative_pos) = lower[search_start..].find(&profile_lower) else {
            break;
        };
        let value_start = search_start + relative_pos;
        let value_end = value_start + profile.len();
        if !is_path_boundary(output[value_end..].chars().next()) {
            search_start = value_end;
            continue;
        }
        output.replace_range(value_start..value_end, "%USERPROFILE%");
        search_start = value_start + "%USERPROFILE%".len();
    }
    output
}

fn is_path_boundary(next: Option<char>) -> bool {
    match next {
        None => true,
        Some('\\' | '/' | '"' | '\'' | ',' | ';' | ')' | ']' | '}') => true,
        Some(ch) => ch.is_whitespace(),
    }
}

#[cfg(test)]
mod tests {
    use super::{installed_log_path_from_localappdata, redact_path_with_profile, sanitize_message};
    use std::path::{Path, PathBuf};

    #[test]
    fn redacts_common_secret_shapes() {
        let openai_like_key = ["sk", "1234567890abcdef"].join("-");
        let message = sanitize_message(&format!(
            "access_key=\"abc123456789\" api_key={} Authorization: Bearer tokenvalue",
            openai_like_key
        ));
        assert!(!message.contains("abc123456789"));
        assert!(!message.contains(&openai_like_key),);
        assert!(!message.contains("tokenvalue"));
    }

    #[test]
    fn redacts_unlabelled_agent_plan_keys() {
        let agent_plan_key = ["ark", "testagentplancredential"].join("-");
        let message = sanitize_message(&format!("request failed near {}", agent_plan_key));

        assert!(!message.contains(&agent_plan_key));
    }

    #[test]
    fn flattens_multiline_messages() {
        let message = sanitize_message("first\nsecond\r\nthird");
        assert_eq!(message, "first second  third");
    }

    #[test]
    fn redacts_user_profile_paths_without_matching_similar_prefixes() {
        assert_eq!(
            redact_path_with_profile(
                "config at C:\\Users\\Alice\\AppData\\VoxType\\config.toml",
                "c:\\users\\alice",
            ),
            "config at %USERPROFILE%\\AppData\\VoxType\\config.toml"
        );
        assert_eq!(
            redact_path_with_profile("C:\\Users\\AliceBackup\\config.toml", "C:\\Users\\Alice"),
            "C:\\Users\\AliceBackup\\config.toml"
        );
    }

    #[test]
    fn installed_log_path_uses_localappdata_logs_dir() {
        let path =
            installed_log_path_from_localappdata(Path::new("C:\\Users\\Alice\\AppData\\Local"));
        assert_eq!(
            path,
            PathBuf::from("C:\\Users\\Alice\\AppData\\Local")
                .join("VoxType")
                .join("logs")
                .join("voice_input.log")
        );
    }
}
