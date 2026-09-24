fn main() {
    #[cfg(all(windows, target_env = "msvc"))]
    tauri_build::build();
}
