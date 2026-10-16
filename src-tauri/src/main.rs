#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(all(windows, target_vendor = "win7"))]
mod win7_compat;

fn main() {
    tulsu_pe_timetable_lib::run()
}
