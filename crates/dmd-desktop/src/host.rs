//! Narrow desktop IPC boundary. Gameplay will delegate to the production table service.
//!
//! This initial infrastructure slice exposes no gameplay actions or authoritative state.
//! The integration slice must construct trusted issuer/session context in this module,
//! never accept it from natural-language text or a renderer-supplied rules command.

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopStatus {
    app_name: &'static str,
    version: &'static str,
    runtime_ready: bool,
    message: &'static str,
}

#[tauri::command]
pub fn desktop_status() -> DesktopStatus {
    DesktopStatus {
        app_name: "DMd",
        version: env!("CARGO_PKG_VERSION"),
        runtime_ready: false,
        message: "The desktop window is ready. Table controls are not connected in this build.",
    }
}
