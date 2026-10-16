import { spawnSync } from 'node:child_process'
import { existsSync } from 'node:fs'

const runtime = 'src-tauri/WebView2/msedgewebview2.exe'
if (!existsSync(runtime)) {
  console.error(`Не найден ${runtime}: положите туда распакованный WebView2 Fixed Version 109 (см. README).`)
  process.exit(1)
}

// Target Windows 7 в Rust (tier 3) поставляется без готовой std: собираем её из исходников (build-std, нужен rust-src).
// На nightly (сборка 2026-10-15) std для этого таргета не собирается, поэтому stable с RUSTC_BOOTSTRAP=1.
// WIN7_TOOLCHAIN фиксирует версию Rust (так делает CI).
// CARGO_BUILD_TARGET, а не --target: Tauri CLI проверяет --target по `rustup target list`, где этого таргета нет.
const result = spawnSync(
  'pnpm',
  ['tauri', 'build', '--config', 'src-tauri/tauri.win7.conf.json', ...process.argv.slice(2)],
  {
    stdio: 'inherit',
    shell: process.platform === 'win32',
    env: {
      ...process.env,
      RUSTUP_TOOLCHAIN: process.env.WIN7_TOOLCHAIN ?? 'stable',
      RUSTC_BOOTSTRAP: '1',
      CARGO_UNSTABLE_BUILD_STD: 'std',
      CARGO_BUILD_TARGET: 'x86_64-win7-windows-msvc'
    }
  }
)
process.exit(result.status ?? 1)
