//! Сборка под Windows 7 (`x86_64-win7-windows-msvc`).
//!
//! 1. Импорты. Загрузчик Windows падает на старте, если exe импортирует функцию, которой нет
//!    в системе. Поэтому `advapi32`, `combase` и `api-ms-win-core-winrt-error` подключены через
//!    `/DELAYLOAD` (см. `build.rs`), а хуки ниже подменяют их, когда DLL или функции нет.
//!    Хуки должны лежать в бинарнике, а не в rlib: объекты бинарника линкуются всегда.
//! 2. WebView2 Runtime. Tauri выставляет `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` в `<exe>\./WebView2/`,
//!    и на Windows 7 такой runtime не был принят. `prepare_webview2_runtime` выбирает папку сам
//!    до старта Tauri и пишет диагностику в `~/.tulsu-pe-timetable/webview2.log`.
//!    Подробности: docs/windows7.md.

use std::ffi::{CStr, c_char, c_void};
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};

unsafe extern "system" {
    fn LoadLibraryA(file_name: *const c_char) -> *mut c_void;
    fn GetAvailableCoreWebView2BrowserVersionString(
        browser_executable_folder: *const u16,
        version_info: *mut *mut u16,
    ) -> i32;
}

const DLI_NOTE_PRE_LOAD_LIBRARY: u32 = 1;
const DLI_FAIL_GET_PROC: u32 = 4;
const WEBVIEW2_FOLDER_ENV: &str = "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER";

/// `DelayLoadProc` и `DelayLoadInfo` из `delayimp.h` (x64).
#[repr(C)]
struct DelayLoadProc {
    import_by_name: i32,
    name_or_ordinal: *const c_char,
}

#[repr(C)]
struct DelayLoadInfo {
    cb: u32,
    pidd: *const c_void,
    ppfn: *mut *const c_void,
    sz_dll: *const c_char,
    dlp: DelayLoadProc,
    hmod_cur: *mut c_void,
    pfn_cur: *const c_void,
    dw_last_error: u32,
}

type DelayLoadHook = unsafe extern "system" fn(u32, *mut DelayLoadInfo) -> *mut c_void;

/// DLL Windows 8+ и DLL, где те же функции есть на Windows 7. Сначала пробуем настоящую DLL.
/// У `kernel32` нет `RoOriginateErrorW`, поэтому дальше сработает заглушка из `dli_fail`.
const REDIRECTS: [(&CStr, &CStr); 2] = [
    (c"combase.dll", c"ole32.dll"),
    (c"api-ms-win-core-winrt-error-l1-1-0.dll", c"kernel32.dll"),
];

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[used]
static __pfnDliNotifyHook2: DelayLoadHook = dli_notify;

#[allow(non_upper_case_globals)]
#[unsafe(no_mangle)]
#[used]
static __pfnDliFailureHook2: DelayLoadHook = dli_fail;

unsafe extern "system" fn dli_notify(reason: u32, info: *mut DelayLoadInfo) -> *mut c_void {
    if reason != DLI_NOTE_PRE_LOAD_LIBRARY {
        return null_mut();
    }
    let dll = unsafe { CStr::from_ptr((*info).sz_dll) };
    let Some((_, fallback)) = REDIRECTS
        .iter()
        .find(|(name, _)| name.to_bytes().eq_ignore_ascii_case(dll.to_bytes()))
    else {
        return null_mut();
    };
    let module = unsafe { LoadLibraryA(dll.as_ptr()) };
    if module.is_null() {
        unsafe { LoadLibraryA(fallback.as_ptr()) }
    } else {
        module
    }
}

unsafe extern "system" fn dli_fail(reason: u32, info: *mut DelayLoadInfo) -> *mut c_void {
    if reason != DLI_FAIL_GET_PROC {
        return null_mut();
    }
    let proc = unsafe { &(*info).dlp };
    if proc.import_by_name == 0 {
        return null_mut();
    }
    let name = unsafe { CStr::from_ptr(proc.name_or_ordinal) };
    match name.to_bytes() {
        b"EventSetInformation" => event_set_information as *mut c_void,
        b"RoOriginateErrorW" => ro_originate_error_w as *mut c_void,
        _ => null_mut(),
    }
}

// Сигнатуры как у настоящих функций. Ничего не делают: на Windows 7 этих функций нет.
// Успех, а не ошибка: вызывающий код (трассировка WebView2) не должен уходить в ветку ошибки.
unsafe extern "system" fn event_set_information(
    _reg_handle: u64,
    _info_class: i32,
    _info: *const c_void,
    _info_len: u32,
) -> u32 {
    0
}

unsafe extern "system" fn ro_originate_error_w(_error: i32, _max_chars: u32, _message: *const u16) -> i32 {
    0
}

/// Выбирает папку WebView2 Runtime до старта Tauri: первую, которую загрузчик принимает.
/// Порядок: `WebView2` рядом с exe, затем `C:\WebView2` (её использует Go-версия).
/// Если переменная уже задана, не трогаем. Все проверки пишем в `webview2.log`.
pub fn prepare_webview2_runtime() {
    let mut log = Log::default();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    log.line(&format!("exe_dir: {exe_dir:?}\n"));

    if std::env::var_os(WEBVIEW2_FOLDER_ENV).is_some() {
        log.line(&format!("{WEBVIEW2_FOLDER_ENV} уже задана, выбор пропущен\n"));
    } else {
        let candidates = exe_dir
            .iter()
            .map(|dir| dir.join("WebView2"))
            .chain(once(PathBuf::from(r"C:\WebView2")));
        for folder in candidates {
            let (hr, version) = available_version(Some(&folder));
            log.line(&describe(&folder.display().to_string(), Some(&folder), hr, &version));
            if hr >= 0 {
                // SAFETY: вызывается из main() до запуска Tauri и его потоков.
                unsafe { std::env::set_var(WEBVIEW2_FOLDER_ENV, &folder) };
                break;
            }
        }
    }

    let (hr, version) = available_version(None);
    let label = format!("как проверяет Tauri (env={:?})", std::env::var_os(WEBVIEW2_FOLDER_ENV));
    log.line(&describe(&label, None, hr, &version));
}

/// Лог пишется после каждого шага: если загрузчик упадёт аварийно, причина всё равно останется в файле.
#[derive(Default)]
struct Log(String);

impl Log {
    fn line(&mut self, text: &str) {
        self.0.push_str(text);
        write_log(&self.0);
    }
}

/// Вызывает `GetAvailableCoreWebView2BrowserVersionString`. Без папки загрузчик смотрит переменную
/// окружения, как и при проверке в Tauri. Строку версии не освобождаем: это разовая диагностика.
fn available_version(folder: Option<&Path>) -> (i32, String) {
    let wide: Option<Vec<u16>> = folder.map(|f| f.as_os_str().encode_wide().chain(once(0)).collect());
    let mut version: *mut u16 = null_mut();
    let hr = unsafe {
        GetAvailableCoreWebView2BrowserVersionString(
            wide.as_ref().map_or(null(), |w| w.as_ptr()),
            &mut version,
        )
    };
    if version.is_null() {
        return (hr, String::new());
    }
    let mut len = 0;
    unsafe {
        while *version.add(len) != 0 {
            len += 1;
        }
        (hr, String::from_utf16_lossy(std::slice::from_raw_parts(version, len)))
    }
}

fn describe(label: &str, folder: Option<&Path>, hr: i32, version: &str) -> String {
    let files = folder.map_or(String::new(), |f| {
        let client = f.join("EBWebView").join("x64").join("EmbeddedBrowserWebView.dll");
        format!(
            "msedgewebview2.exe={}, EmbeddedBrowserWebView.dll={}, ",
            yes_no(f.join("msedgewebview2.exe").is_file()),
            yes_no(client.is_file()),
        )
    });
    format!("{label}: {files}hr=0x{:08X}, версия={version:?}\n", hr as u32)
}

fn yes_no(flag: bool) -> &'static str {
    if flag { "есть" } else { "нет" }
}

fn write_log(text: &str) {
    let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) else {
        return;
    };
    let dir = home.join(".tulsu-pe-timetable");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("webview2.log"), text);
}
