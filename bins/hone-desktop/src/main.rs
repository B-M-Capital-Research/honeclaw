#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod sidecar;
mod tray;

fn main() {
    sidecar::run_desktop_app();
}
