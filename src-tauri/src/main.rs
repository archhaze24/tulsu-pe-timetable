#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(windows, target_vendor = "win7"))]
mod win7_compat;

fn main() {
    #[cfg(all(windows, target_vendor = "win7"))]
    win7_compat::prepare_webview2_runtime();

    tulsu_pe_timetable_lib::run()
}
