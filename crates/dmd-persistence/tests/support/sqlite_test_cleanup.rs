use std::{io, path::Path, time::Duration};

/// Call only after closing the pool and dropping its fixture. Match the app-test
/// cleanup policy for transient Windows file-sharing locks; deletion must succeed.
pub(super) async fn remove_closed_file(path: &Path) -> io::Result<()> {
    for attempt in 0..20 {
        match std::fs::remove_file(path) {
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
