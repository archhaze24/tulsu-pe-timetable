// SPDX-License-Identifier: Apache-2.0 OR MIT
//
// Патч для Windows 7, не часть upstream wry. Описание: src-tauri/vendor/README.md.
// На Windows 7 нативные `CreateCoreWebView2EnvironmentWithOptions` и
// `GetAvailableCoreWebView2BrowserVersionString` возвращают `0x80070032`. Внутренняя функция
// `EmbeddedBrowserWebView.dll` работает, тем же путём идут go-webview2 v1.0.10 и
// `scripts/win7/test-env.c`.

use std::{
  ffi::c_void,
  iter::once,
  os::windows::ffi::OsStrExt,
  path::{Path, PathBuf},
  sync::OnceLock,
};

use webview2_com::Microsoft::Web::WebView2::Win32::{
  ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler, ICoreWebView2EnvironmentOptions,
};
use windows::{
  core::{s, w, Error, Interface, Result, HRESULT, HSTRING, PCWSTR},
  Win32::{
    Foundation::E_FAIL,
    Storage::FileSystem::{
      GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    },
    System::LibraryLoader::{GetProcAddress, LoadLibraryW},
  },
};

/// Версия, которую принимает внутренняя функция (как `kMinimumCompatibleVersion` в go-webview2).
/// Значение по умолчанию в `webview2-com` (`145.0.3800.47`) runtime 109 не принимает.
pub(super) const TARGET_COMPATIBLE_BROWSER_VERSION: &str = "86.0.616.0";

const FOLDER_ENV: &str = "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER";

/// Очищаются перед вызовом, как в go-webview2 (`preventEnvAndRegistryOverrides`): иначе
/// переменные переопределяют выбор runtime. Папку мы уже запомнили в `FixedRuntime`.
const OVERRIDE_ENVS: [(&str, &str); 5] = [
  ("WEBVIEW2_PIPE_FOR_SCRIPT_DEBUGGER", ""),
  ("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", ""),
  ("WEBVIEW2_RELEASE_CHANNEL_PREFERENCE", "0"),
  (FOLDER_ENV, ""),
  ("WEBVIEW2_USER_DATA_FOLDER", ""),
];

/// Сигнатура как в `scripts/win7/test-env.c`: `(TRUE, 1, userData, options, handler)`.
type CreateEnvironmentInternal = unsafe extern "system" fn(
  unknown: i32,
  runtime_type: i32,
  user_data_folder: PCWSTR,
  options: *mut c_void,
  handler: *mut c_void,
) -> HRESULT;

pub(super) struct FixedRuntime {
  client_dll: PathBuf,
  create_environment_internal: CreateEnvironmentInternal,
}

static FIXED_RUNTIME: OnceLock<Option<FixedRuntime>> = OnceLock::new();

/// Папка берётся из `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` один раз. После первого вызова
/// переменные очищаются, поэтому прочитать её повторно из окружения нельзя.
pub(super) fn get() -> Option<&'static FixedRuntime> {
  FIXED_RUNTIME.get_or_init(load).as_ref()
}

fn load() -> Option<FixedRuntime> {
  let folder = std::env::var_os(FOLDER_ENV).filter(|folder| !folder.is_empty())?;
  let client_dll = Path::new(&folder)
    .join("EBWebView")
    .join("x64")
    .join("EmbeddedBrowserWebView.dll");
  let create_environment_internal = load_create_environment(&client_dll)?;
  for (name, value) in OVERRIDE_ENVS {
    std::env::set_var(name, value);
  }
  Some(FixedRuntime {
    client_dll,
    create_environment_internal,
  })
}

fn load_create_environment(client_dll: &Path) -> Option<CreateEnvironmentInternal> {
  let wide = wide_path(client_dll);
  let module = unsafe { LoadLibraryW(PCWSTR(wide.as_ptr())) }.ok()?;
  let proc = unsafe { GetProcAddress(module, s!("CreateWebViewEnvironmentWithOptionsInternal")) }?;
  // SAFETY: экспорт имеет сигнатуру `CreateEnvironmentInternal`, см. test-env.c.
  Some(unsafe {
    std::mem::transmute::<unsafe extern "system" fn() -> isize, CreateEnvironmentInternal>(proc)
  })
}

impl FixedRuntime {
  /// Вызов асинхронный: результат придёт в `handler`. Аргументы живут до возврата из функции.
  pub(super) fn create_environment(
    &self,
    user_data_folder: &HSTRING,
    options: &ICoreWebView2EnvironmentOptions,
    handler: &ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
  ) -> Result<()> {
    let hr = unsafe {
      (self.create_environment_internal)(
        1,
        1,
        PCWSTR(user_data_folder.as_ptr()),
        options.as_raw(),
        handler.as_raw(),
      )
    };
    hr.ok()
  }

  pub(super) fn version(&self) -> Result<String> {
    product_version(&self.client_dll)
  }
}

/// `VS_FIXEDFILEINFO` даёт версию в формате `109.0.1518.78`, как `get_BrowserVersionString`.
fn product_version(path: &Path) -> Result<String> {
  let wide = wide_path(path);
  let file = PCWSTR(wide.as_ptr());
  unsafe {
    let size = GetFileVersionInfoSizeW(file, None);
    if size == 0 {
      return Err(Error::from_thread());
    }
    let mut block = vec![0u8; size as usize];
    GetFileVersionInfoW(file, None, size, block.as_mut_ptr().cast())?;
    let mut info = std::ptr::null_mut::<c_void>();
    let mut len = 0;
    if !VerQueryValueW(block.as_ptr().cast(), w!("\\"), &mut info, &mut len).as_bool()
      || (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>()
    {
      return Err(Error::new(E_FAIL, "VS_FIXEDFILEINFO not found"));
    }
    let fixed = std::ptr::read_unaligned(info as *const VS_FIXEDFILEINFO);
    let (ms, ls) = (fixed.dwProductVersionMS, fixed.dwProductVersionLS);
    Ok(format!(
      "{}.{}.{}.{}",
      ms >> 16,
      ms & 0xFFFF,
      ls >> 16,
      ls & 0xFFFF
    ))
  }
}

fn wide_path(path: &Path) -> Vec<u16> {
  path.as_os_str().encode_wide().chain(once(0)).collect()
}
