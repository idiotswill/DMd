use std::{io, path::Path, time::Duration};

/// Call only after closing the pool and dropping its fixture. Windows can retain
/// a transient file-sharing lock during final handle release. Cleanup must still
/// succeed; retry only that specific OS condition for a bounded interval.
pub(super) async fn remove_closed_file(path: &Path) -> io::Result<()> {
    retry_sharing(|| std::fs::remove_file(path)).await
}

pub(super) async fn remove_closed_directory(path: &Path) -> io::Result<()> {
    let directory = path.canonicalize()?;
    let temporary_root = std::env::temp_dir().canonicalize()?;
    // These fixtures allocate unique, direct children of the temporary directory.
    // Never recursively remove the temporary root or a caller-supplied outer path.
    if directory.parent() != Some(temporary_root.as_path())
        || !directory
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("dmd-"))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a DMd temporary fixture directory",
        ));
    }
    retry_sharing(|| std::fs::remove_dir_all(&directory)).await
}

async fn retry_sharing(mut remove: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    for attempt in 0..20 {
        match remove() {
            Err(error)
                if cfg!(windows)
                    && matches!(error.raw_os_error(), Some(32 | 33))
                    && attempt < 19 =>
            {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            result => return result,
        }
    }
    unreachable!("the final attempt always returns its result")
}
