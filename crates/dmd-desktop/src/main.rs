#![cfg_attr(all(windows, target_env = "msvc"), windows_subsystem = "windows")]

#[cfg(all(windows, target_env = "msvc"))]
mod host;

#[cfg(all(windows, target_env = "msvc"))]
fn main() {
    use tauri::Manager;

    tauri::Builder::default()
        // Register first: a second process must exit before creating another retry writer.
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
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
            host::desktop_submit_table,
        ])
        .run(tauri::generate_context!())
        .expect("DMd could not start its desktop window");
}

#[cfg(not(all(windows, target_env = "msvc")))]
fn main() {
    eprintln!("The DMd desktop package currently requires Windows with the MSVC target.");
    std::process::exit(1);
}
