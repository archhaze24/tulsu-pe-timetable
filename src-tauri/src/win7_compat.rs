//! Сборка под Windows 7 (`x86_64-win7-windows-msvc`).
//!
//! Загрузчик Windows падает на старте, если exe импортирует функцию, которой нет в системе.
//! Поэтому в win7-сборке `advapi32`, `combase` и `api-ms-win-core-winrt-error` подключены через
//! `/DELAYLOAD` (см. `build.rs`), и загрузчик обращается к ним только при первом вызове.
//! Хуки ниже вызываются, когда DLL или функции нет: они подменяют DLL и дают заглушки.
//! Хуки должны лежать в бинарнике, а не в rlib: объекты бинарника линкуются всегда.
//! Подробности: docs/windows7.md.

use std::ffi::{CStr, c_char, c_void};
use std::ptr::null_mut;

unsafe extern "system" {
    fn LoadLibraryA(file_name: *const c_char) -> *mut c_void;
}

const DLI_NOTE_PRE_LOAD_LIBRARY: u32 = 1;
const DLI_FAIL_GET_PROC: u32 = 4;

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
