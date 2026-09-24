#![cfg_attr(all(windows, target_env = "msvc"), windows_subsystem = "windows")]

#[cfg(all(windows, target_env = "msvc"))]
mod host;

#[cfg(all(windows, target_env = "msvc"))]
fn main() {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            app.manage(host::DesktopHost::new(app.handle()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            host::desktop_status,
            host::desktop_default_contract,
            host::desktop_list_campaigns,
            host::desktop_create_campaign,
            host::desktop_open_campaign,
            host::desktop_creation_options,
            host::desktop_host_situation,
            host::desktop_table_action,
            host::desktop_table_text,
        ])
        .run(tauri::generate_context!())
        .expect("DMd could not start its desktop window");
}

#[cfg(not(all(windows, target_env = "msvc")))]
fn main() {
    eprintln!("The DMd desktop package currently requires Windows with the MSVC target.");
    std::process::exit(1);
}
