//! TODO

use std::{
    path::{Path, PathBuf},
    pin::Pin,
};

use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Generated data for a specific [`Version`](crate::Version).
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct GeneratedData(HashMap<PathBuf, DataFile>);

impl GeneratedData {
    /// Create a new [`GeneratedData`] from the given data path.
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(data_path: &Path) -> anyhow::Result<Self> {
        let mut data = HashMap::new();
        Self::append_directory(data_path, data_path, &mut data).await?;
        Ok(Self(data))
    }

    /// Recursively append the data files in the given directory to the map.
    fn append_directory<'a>(
        path: &'a Path,
        data_path: &'a Path,
        data: &'a mut HashMap<PathBuf, DataFile>,
    ) -> Pin<Box<dyn 'a + std::future::Future<Output = anyhow::Result<()>>>> {
        Box::pin(async move {
            // Iterate for each data file in the data directory.
            let mut dir = tokio::fs::read_dir(path).await?;
            while let Ok(Some(entry)) = dir.next_entry().await {
                // Recurse into directories.
                if entry.file_type().await?.is_dir() {
                    Self::append_directory(&entry.path(), data_path, data).await?;
                    continue;
                }

                // Skip non-JSON files.
                let filename = entry.file_name();
                let filename = filename.to_string_lossy();
                if !filename.ends_with(".json") {
                    continue;
                }

                // Parse the data file.
                data.insert(
                    entry.path().strip_prefix(data_path).unwrap().to_path_buf(),
                    serde_json::from_str(&tokio::fs::read_to_string(entry.path()).await?)?,
                );
            }
            Ok(())
        })
    }
}

/// A data file.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DataFile(serde_json::Value);
