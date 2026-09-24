#![cfg_attr(all(windows, target_env = "msvc"), windows_subsystem = "windows")]

#[cfg(all(windows, target_env = "msvc"))]
mod host;

#[cfg(all(windows, target_env = "msvc"))]
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![host::desktop_status])
        .run(tauri::generate_context!())
        .expect("DMd could not start its desktop window");
}

#[cfg(not(all(windows, target_env = "msvc")))]
fn main() {
    eprintln!("The DMd desktop package currently requires Windows with the MSVC target.");
    std::process::exit(1);
}
