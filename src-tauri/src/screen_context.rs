use crate::{app_log, config::ScreenContextConfig};
use serde::Serialize;
use std::{
    ffi::c_void,
    mem,
    sync::mpsc::{self, Receiver, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};
use windows::{
    core::{Error as WindowsError, HSTRING},
    Globalization::Language,
    Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::DataWriter,
    Win32::{
        Foundation::{RECT, RPC_E_CHANGED_MODE},
        Graphics::Gdi::{
            BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
            GetDIBits, GetMonitorInfoW, MonitorFromWindow, ReleaseDC, SelectObject, BITMAPINFO,
            BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ,
            MONITORINFO, MONITOR_DEFAULTTONEAREST, SRCCOPY,
        },
        System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect},
    },
};

pub type ScreenContextReceiver = Receiver<Result<ScreenContextSnapshot, String>>;

#[derive(Debug, Clone)]
pub struct ScreenContextSnapshot {
    pub text: String,
    pub elapsed_ms: u128,
    pub selected_language: String,
    pub available_languages: Vec<String>,
    pub image_width: i32,
    pub image_height: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScreenContextTestResult {
    pub available_languages: Vec<String>,
    pub selected_language: Option<String>,
    pub elapsed_ms: u128,
    pub text: String,
    pub text_chars: usize,
    pub image_width: i32,
    pub image_height: i32,
    pub warning: Option<String>,
}

struct CapturedImage {
    pixels: Vec<u8>,
    width: i32,
    height: i32,
}

pub fn spawn_capture(config: &ScreenContextConfig) -> Option<ScreenContextReceiver> {
    if !config.enabled {
        return None;
    }
    let started_at = Instant::now();
    let image = match capture_context_bitmap(config) {
        Ok(image) => image,
        Err(err) => {
            app_log::warn(format!("屏幕 OCR 截图失败，已跳过: {}", err));
            return None;
        }
    };
    let config = config.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = recognize_image_context(image, &config, started_at);
        match &result {
            Ok(snapshot) => app_log::info(format!(
                "屏幕 OCR 上下文已生成: chars={}, elapsed_ms={}, language={}, image={}x{}",
                snapshot.text.chars().count(),
                snapshot.elapsed_ms,
                snapshot.selected_language,
                snapshot.image_width,
                snapshot.image_height
            )),
            Err(err) => app_log::warn(format!("屏幕 OCR 识别失败，已跳过: {}", err)),
        }
        let _ = tx.send(result);
    });
    Some(rx)
}

pub fn wait_for_context(
    receiver: Option<ScreenContextReceiver>,
    timeout_ms: u64,
) -> Option<ScreenContextSnapshot> {
    let receiver = receiver?;
    // 500ms 是近期实测的命中率/首字延迟折中：OCR 超时只跳过上下文，不应阻断录音或最终输出。
    // 维护依据见 docs/asr-quality-latency-guardrails.md。
    match receiver.recv_timeout(Duration::from_millis(timeout_ms.max(1))) {
        Ok(Ok(snapshot)) if !snapshot.text.trim().is_empty() => Some(snapshot),
        Ok(Ok(snapshot)) => {
            app_log::info(format!(
                "屏幕 OCR 上下文为空，已跳过: elapsed_ms={}, language={}, image={}x{}",
                snapshot.elapsed_ms,
                snapshot.selected_language,
                snapshot.image_width,
                snapshot.image_height
            ));
            None
        }
        Ok(Err(err)) => {
            app_log::warn(format!("屏幕 OCR 上下文不可用，已跳过: {}", err));
            None
        }
        Err(RecvTimeoutError::Timeout) => {
            app_log::info(format!("屏幕 OCR 超过 {}ms 未返回，已跳过。", timeout_ms));
            None
        }
        Err(RecvTimeoutError::Disconnected) => {
            app_log::warn("屏幕 OCR 线程提前结束，已跳过。");
            None
        }
    }
}

pub fn test_capture(config: &ScreenContextConfig) -> Result<ScreenContextTestResult, String> {
    let started_at = Instant::now();
    let image = capture_context_bitmap(config)?;
    let recognized = recognize_image_context(image, config, started_at)?;
    let text_chars = recognized.text.chars().count();
    Ok(ScreenContextTestResult {
        available_languages: recognized.available_languages,
        selected_language: Some(recognized.selected_language),
        elapsed_ms: recognized.elapsed_ms,
        warning: if recognized.text.trim().is_empty() {
            Some("未识别到可用文字。".to_string())
        } else {
            None
        },
        text: recognized.text,
        text_chars,
        image_width: recognized.image_width,
        image_height: recognized.image_height,
    })
}

pub fn test_capture_on_worker(
    config: &ScreenContextConfig,
) -> Result<ScreenContextTestResult, String> {
    let config = config.clone();
    thread::spawn(move || test_capture(&config))
        .join()
        .map_err(|_| "屏幕 OCR 测试线程异常结束。".to_string())?
}

fn recognize_image_context(
    image: CapturedImage,
    config: &ScreenContextConfig,
    started_at: Instant,
) -> Result<ScreenContextSnapshot, String> {
    let _apartment = WinRtApartment::init_mta()?;
    let available_languages = available_ocr_language_tags()?;
    let (engine, selected_language) = create_ocr_engine(&available_languages)?;
    let max_dimension = OcrEngine::MaxImageDimension()
        .map(|value| value as i32)
        .unwrap_or(2_600);
    let image = resize_to_ocr_limit(image, max_dimension);
    let bitmap = software_bitmap_from_bgra(&image)?;
    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(|err| windows_error("启动 Windows OCR 识别失败", err))?
        .get()
        .map_err(|err| windows_error("等待 Windows OCR 识别结果失败", err))?;
    let raw_text = result
        .Text()
        .map_err(|err| windows_error("读取 Windows OCR 文本失败", err))?
        .to_string();
    let _ = bitmap.Close();
    let text = normalize_screen_context_text(&raw_text, config.max_chars);
    Ok(ScreenContextSnapshot {
        text,
        elapsed_ms: started_at.elapsed().as_millis(),
        selected_language,
        available_languages,
        image_width: image.width,
        image_height: image.height,
    })
}

fn available_ocr_language_tags() -> Result<Vec<String>, String> {
    let languages = OcrEngine::AvailableRecognizerLanguages()
        .map_err(|err| windows_error("读取 Windows OCR 语言列表失败", err))?;
    let mut tags = Vec::new();
    for index in 0..languages
        .Size()
        .map_err(|err| windows_error("读取 Windows OCR 语言数量失败", err))?
    {
        let language = languages
            .GetAt(index)
            .map_err(|err| windows_error("读取 Windows OCR 语言失败", err))?;
        tags.push(
            language
                .LanguageTag()
                .map_err(|err| windows_error("读取 Windows OCR 语言标签失败", err))?
                .to_string(),
        );
    }
    Ok(tags)
}

fn create_ocr_engine(available_languages: &[String]) -> Result<(OcrEngine, String), String> {
    for preferred in ["zh-Hans-CN", "zh-CN", "zh-Hans", "en-US", "en"] {
        if !available_languages
            .iter()
            .any(|tag| tag.eq_ignore_ascii_case(preferred))
        {
            continue;
        }
        let language = Language::CreateLanguage(&HSTRING::from(preferred))
            .map_err(|err| windows_error("创建 Windows OCR 语言失败", err))?;
        if OcrEngine::IsLanguageSupported(&language)
            .map_err(|err| windows_error("检查 Windows OCR 语言支持失败", err))?
        {
            let engine = OcrEngine::TryCreateFromLanguage(&language)
                .map_err(|err| windows_error("创建 Windows OCR 引擎失败", err))?;
            return Ok((engine, preferred.to_string()));
        }
    }

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|err| windows_error("创建 Windows OCR 用户语言引擎失败", err))?;
    let selected_language = engine
        .RecognizerLanguage()
        .ok()
        .and_then(|language| language.LanguageTag().ok())
        .map(|tag| tag.to_string())
        .unwrap_or_else(|| "user-profile".to_string());
    Ok((engine, selected_language))
}

fn software_bitmap_from_bgra(image: &CapturedImage) -> Result<SoftwareBitmap, String> {
    let writer = DataWriter::new().map_err(|err| windows_error("创建截图缓冲失败", err))?;
    writer
        .WriteBytes(&image.pixels)
        .map_err(|err| windows_error("写入截图缓冲失败", err))?;
    let buffer = writer
        .DetachBuffer()
        .map_err(|err| windows_error("生成截图缓冲失败", err))?;
    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        image.width,
        image.height,
    )
    .map_err(|err| windows_error("创建 Windows OCR 位图失败", err))?;
    let _ = writer.Close();
    Ok(bitmap)
}

fn resize_to_ocr_limit(image: CapturedImage, max_dimension: i32) -> CapturedImage {
    if max_dimension <= 0 || image.width <= max_dimension && image.height <= max_dimension {
        return image;
    }
    let longest = image.width.max(image.height) as f64;
    let scale = max_dimension as f64 / longest;
    let next_width = ((image.width as f64 * scale).round() as i32).max(1);
    let next_height = ((image.height as f64 * scale).round() as i32).max(1);
    resize_bgra_nearest(image, next_width, next_height)
}

fn resize_bgra_nearest(image: CapturedImage, next_width: i32, next_height: i32) -> CapturedImage {
    let mut resized = vec![0u8; next_width as usize * next_height as usize * 4];
    let src_width = image.width as usize;
    let src_height = image.height as usize;
    let dst_width = next_width as usize;
    let dst_height = next_height as usize;

    for y in 0..dst_height {
        let src_y = (y * src_height / dst_height).min(src_height.saturating_sub(1));
        for x in 0..dst_width {
            let src_x = (x * src_width / dst_width).min(src_width.saturating_sub(1));
            let src = (src_y * src_width + src_x) * 4;
            let dst = (y * dst_width + x) * 4;
            resized[dst..dst + 4].copy_from_slice(&image.pixels[src..src + 4]);
        }
    }

    CapturedImage {
        pixels: resized,
        width: next_width,
        height: next_height,
    }
}

fn normalize_screen_context_text(text: &str, max_chars: usize) -> String {
    text.replace('\r', "\n")
        .lines()
        .map(|line| {
            merge_cjk_character_spaces(&line.split_whitespace().collect::<Vec<_>>().join(" "))
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
        .chars()
        .take(max_chars)
        .collect::<String>()
        .trim()
        .to_string()
}

fn merge_cjk_character_spaces(text: &str) -> String {
    let mut merged = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == ' ' {
            let previous = merged.chars().next_back();
            let next = chars.peek().copied();
            if previous.is_some_and(is_cjk_character) && next.is_some_and(is_cjk_character) {
                continue;
            }
        }
        merged.push(ch);
    }
    merged
}

fn is_cjk_character(ch: char) -> bool {
    matches!(
        ch,
        '\u{3400}'..='\u{4DBF}' | '\u{4E00}'..='\u{9FFF}' | '\u{F900}'..='\u{FAFF}'
    )
}

fn capture_context_bitmap(config: &ScreenContextConfig) -> Result<CapturedImage, String> {
    if config.capture_scope == "window" {
        capture_foreground_window_bitmap()
    } else {
        capture_current_monitor_bitmap()
    }
}

fn capture_current_monitor_bitmap() -> Result<CapturedImage, String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return Err("未找到当前前台窗口，无法定位当前显示器。".to_string());
        }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.is_invalid() {
            return Err("未找到当前显示器。".to_string());
        }
        let mut info = MONITORINFO {
            cbSize: mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return Err("读取当前显示器区域失败。".to_string());
        }
        capture_rect_bitmap(info.rcMonitor, "当前显示器")
    }
}

fn capture_foreground_window_bitmap() -> Result<CapturedImage, String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return Err("未找到当前前台窗口。".to_string());
        }
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect).map_err(|err| windows_error("读取当前窗口区域失败", err))?;
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err("当前窗口区域无效。".to_string());
        }

        capture_rect_bitmap(rect, "当前窗口")
    }
}

fn capture_rect_bitmap(rect: RECT, area_label: &str) -> Result<CapturedImage, String> {
    unsafe {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            return Err(format!("{}区域无效。", area_label));
        }
        let screen_dc = ScreenDc::new()?;
        let memory_dc = MemoryDc::new(screen_dc.0)?;
        let bitmap = Bitmap::new(screen_dc.0, width, height)?;
        let _selection = SelectedObject::new(memory_dc.0, HGDIOBJ::from(bitmap.0))?;

        BitBlt(
            memory_dc.0,
            0,
            0,
            width,
            height,
            Some(screen_dc.0),
            rect.left,
            rect.top,
            SRCCOPY | CAPTUREBLT,
        )
        .map_err(|err| windows_error(&format!("复制{}截图失败", area_label), err))?;

        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = width;
        info.bmiHeader.biHeight = -height;
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB.0;

        let mut pixels = vec![0u8; width as usize * height as usize * 4];
        let copied = GetDIBits(
            memory_dc.0,
            bitmap.0,
            0,
            height as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut info,
            DIB_RGB_COLORS,
        );
        if copied == 0 {
            return Err(format!("读取{}截图像素失败。", area_label));
        }
        Ok(CapturedImage {
            pixels,
            width,
            height,
        })
    }
}

struct ScreenDc(HDC);

impl ScreenDc {
    unsafe fn new() -> Result<Self, String> {
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            Err("获取屏幕 DC 失败。".to_string())
        } else {
            Ok(Self(hdc))
        }
    }
}

impl Drop for ScreenDc {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseDC(None, self.0);
        }
    }
}

struct MemoryDc(HDC);

impl MemoryDc {
    unsafe fn new(source: HDC) -> Result<Self, String> {
        let hdc = CreateCompatibleDC(Some(source));
        if hdc.is_invalid() {
            Err("创建截图 DC 失败。".to_string())
        } else {
            Ok(Self(hdc))
        }
    }
}

impl Drop for MemoryDc {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.0);
        }
    }
}

struct Bitmap(HBITMAP);

impl Bitmap {
    unsafe fn new(source: HDC, width: i32, height: i32) -> Result<Self, String> {
        let bitmap = CreateCompatibleBitmap(source, width, height);
        if bitmap.is_invalid() {
            Err("创建截图位图失败。".to_string())
        } else {
            Ok(Self(bitmap))
        }
    }
}

impl Drop for Bitmap {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(HGDIOBJ::from(self.0));
        }
    }
}

struct SelectedObject {
    hdc: HDC,
    previous: HGDIOBJ,
}

impl SelectedObject {
    unsafe fn new(hdc: HDC, object: HGDIOBJ) -> Result<Self, String> {
        let previous = SelectObject(hdc, object);
        if previous.is_invalid() {
            Err("选择截图位图失败。".to_string())
        } else {
            Ok(Self { hdc, previous })
        }
    }
}

impl Drop for SelectedObject {
    fn drop(&mut self) {
        unsafe {
            if !self.previous.is_invalid() {
                let _ = SelectObject(self.hdc, self.previous);
            }
        }
    }
}

struct WinRtApartment {
    should_uninitialize: bool,
}

impl WinRtApartment {
    fn init_mta() -> Result<Self, String> {
        unsafe {
            match RoInitialize(RO_INIT_MULTITHREADED) {
                Ok(()) => Ok(Self {
                    should_uninitialize: true,
                }),
                Err(err) if is_changed_thread_mode(&err) => {
                    // Tauri 命令线程可能已初始化为 STA；WinRT OCR 可以沿用既有 apartment。
                    Ok(Self {
                        should_uninitialize: false,
                    })
                }
                Err(err) => Err(windows_error("初始化 Windows OCR 运行时失败", err)),
            }
        }
    }
}

fn is_changed_thread_mode(err: &WindowsError) -> bool {
    err.code() == RPC_E_CHANGED_MODE
}

impl Drop for WinRtApartment {
    fn drop(&mut self) {
        if !self.should_uninitialize {
            return;
        }
        unsafe {
            RoUninitialize();
        }
    }
}

fn windows_error(context: &str, err: WindowsError) -> String {
    format!("{}: {}", context, err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ocr_text_lightly() {
        let text = normalize_screen_context_text("  豆 包  ASR\r\n\r\n  VoxType   main  ", 1_000);
        assert_eq!(text, "豆包 ASR\nVoxType main");
    }

    #[test]
    fn removes_spaces_between_cjk_characters_only() {
        let text = normalize_screen_context_text(
            "屏 幕 OCR 上 下 文  Ctrl + Q  ASR 和 大 模 型  VoxType 首 页",
            1_000,
        );
        assert_eq!(text, "屏幕 OCR 上下文 Ctrl + Q ASR 和大模型 VoxType 首页");
    }

    #[test]
    fn limits_ocr_text_length() {
        let text = normalize_screen_context_text("abcdef", 3);
        assert_eq!(text, "abc");
    }

    #[test]
    fn treats_existing_com_apartment_as_usable() {
        let err = WindowsError::from_hresult(RPC_E_CHANGED_MODE);
        assert!(is_changed_thread_mode(&err));
    }
}
