// src-tauri/src/main.rs
// Ponto de entrada do processo nativo Tauri

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sag_desktop_lib::run();
}
