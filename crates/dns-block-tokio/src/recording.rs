use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};

use tokio::sync::RwLock;

/// For debugging purposes or generating DNS request datasets, it might be interesting to record the handled DNS queries.
/// This module holds some functions that help with setting that up.

/// Sets up a file at the given path and returns a `tokio::fs::File` handle
async fn _setup_query_recorder(file_path: &Option<String>) -> Arc<Option<RwLock<tokio::fs::File>>> {
    // which writes out the data
    if let Some(path) = file_path {
        tokio::fs::create_dir_all(Path::new(path)).await.unwrap();
        println!("Recording queries to {path}");
    }

    Arc::new({
        if let Some(ref path) = file_path {
            let filename = format!(
                "{}.bin",
                std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            );
            Some(RwLock::new(
                tokio::fs::File::create(PathBuf::new().join(path).join(filename))
                    .await
                    .unwrap(),
            ))
        } else {
            None
        }
    })
}
