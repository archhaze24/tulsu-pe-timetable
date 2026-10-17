//! Сборка под Windows 7 (`x86_64-win7-windows-msvc`).
//!
//! 1. Импорты. Загрузчик Windows падает на старте, если exe импортирует функцию, которой нет
//!    в системе. Поэтому `advapi32`, `combase` и `api-ms-win-core-winrt-error` подключены через
//!    `/DELAYLOAD` (см. `build.rs`), а хуки ниже подменяют их, когда DLL или функции нет.
//!    Хуки должны лежать в бинарнике, а не в rlib: объекты бинарника линкуются всегда.
//! 2. WebView2 Runtime. `prepare_webview2_runtime` выбирает папку с `EmbeddedBrowserWebView.dll`
//!    до старта Tauri и передаёт её патченному `wry` (`src-tauri/vendor/wry`) через
//!    `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`. Нативный loader на Windows 7 не создаёт environment
//!    (`0x80070032`), поэтому environment создаёт `wry` через внутреннюю функцию этой DLL.
//!    Диагностика и паника пишутся в `~/.tulsu-pe-timetable/webview2.log`.
//!    Подробности: docs/windows7.md.

use std::ffi::{CStr, c_char, c_void};
use std::io::Write;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;

unsafe extern "system" {
    fn LoadLibraryW(file_name: *const u16) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, proc_name: *const c_char) -> *const c_void;
    fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
}

const DLI_NOTE_PRE_LOAD_LIBRARY: u32 = 1;
const DLI_FAIL_GET_PROC: u32 = 4;
const WEBVIEW2_FOLDER_ENV: &str = "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER";
const CREATE_ENVIRONMENT_EXPORT: &CStr = c"CreateWebViewEnvironmentWithOptionsInternal";

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
const REDIRECTS: [(&str, &str); 2] = [
    ("combase.dll", "ole32.dll"),
    ("api-ms-win-core-winrt-error-l1-1-0.dll", "kernel32.dll"),
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
    let requested = unsafe { CStr::from_ptr((*info).sz_dll) }.to_str().unwrap_or_default();
    let Some((name, fallback)) = REDIRECTS
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(requested))
    else {
        return null_mut();
    };
    let module = load_system_dll(name);
    if module.is_null() {
        load_system_dll(fallback)
    } else {
        module
    }
}

/// Грузит DLL по полному пути в системную папку. Поиск по короткому имени заодно просматривает
/// папку exe, и подменить библиотеку можно, положив рядом свою DLL (S-04).
fn load_system_dll(name: &str) -> *mut c_void {
    let mut dir = [0u16; 260];
    let len = unsafe { GetSystemDirectoryW(dir.as_mut_ptr(), dir.len() as u32) } as usize;
    if len == 0 || len >= dir.len() {
        return null_mut();
    }
    let mut path: Vec<u16> = dir[..len].to_vec();
    path.push(u16::from(b'\\'));
    path.extend(name.encode_utf16());
    path.push(0);
    unsafe { LoadLibraryW(path.as_ptr()) }
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

/// Выбирает папку WebView2 Runtime до старта Tauri: первую, в которой загружается
/// `EmbeddedBrowserWebView.dll` с нужной точкой входа. Порядок: `WebView2` рядом с exe, затем
/// `C:\WebView2` (её использует Go-версия). Если переменная уже задана, не трогаем.
pub fn prepare_webview2_runtime() {
    install_panic_log();
    let mut log = Log::default();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf));
    log.line(&format!("exe_dir: {exe_dir:?}\n"));

    if std::env::var_os(WEBVIEW2_FOLDER_ENV).is_some() {
        log.line(&format!("{WEBVIEW2_FOLDER_ENV} уже задана, выбор пропущен\n"));
        return;
    }
    let candidates = exe_dir
        .iter()
        .map(|dir| dir.join("WebView2"))
        .chain(once(PathBuf::from(r"C:\WebView2")));
    for folder in candidates {
        let (usable, report) = check_fixed_runtime(&folder);
        log.line(&report);
        if usable {
            // SAFETY: вызывается из main() до запуска Tauri и его потоков.
            unsafe { std::env::set_var(WEBVIEW2_FOLDER_ENV, &folder) };
            log.line(&format!("выбрана папка: {}\n", folder.display()));
            return;
        }
    }
}

/// Проверяет папку: есть ли `EBWebView\x64\EmbeddedBrowserWebView.dll`, грузится ли он и есть ли
/// в нём точка входа. Загруженный модуль остаётся в процессе, повторная загрузка его не меняет.
fn check_fixed_runtime(folder: &Path) -> (bool, String) {
    let name = folder.display();
    let dll = folder
        .join("EBWebView")
        .join("x64")
        .join("EmbeddedBrowserWebView.dll");
    if !dll.is_file() {
        return (false, format!("{name}: EmbeddedBrowserWebView.dll нет\n"));
    }
    let wide: Vec<u16> = dll.as_os_str().encode_wide().chain(once(0)).collect();
    let module = unsafe { LoadLibraryW(wide.as_ptr()) };
    if module.is_null() {
        let error = std::io::Error::last_os_error();
        return (false, format!("{name}: LoadLibraryW: {error}\n"));
    }
    if unsafe { GetProcAddress(module, CREATE_ENVIRONMENT_EXPORT.as_ptr()) }.is_null() {
        let error = std::io::Error::last_os_error();
        return (false, format!("{name}: GetProcAddress: {error}\n"));
    }
    (
        true,
        format!("{name}: EmbeddedBrowserWebView.dll загружена, точка входа найдена\n"),
    )
}

/// В release-сборке (`windows_subsystem = "windows"`) паника не видна, поэтому пишем её в лог.
fn install_panic_log() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        append_log(&format!("panic: {info}\n"));
        default_hook(info);
    }));
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

fn log_path() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from)?;
    Some(home.join(".tulsu-pe-timetable").join("webview2.log"))
}

fn write_log(text: &str) {
    let Some(path) = log_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

fn append_log(text: &str) {
    let Some(path) = log_path() else {
        return;
    };
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = file.write_all(text.as_bytes());
    }
}
