//! TODO

use std::path::Path;

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Assets generated for a specific [`Version`](crate::Version).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedAssets {
    /// Generated blockstates.
    pub blockstates: GeneratedBlockstates,
    /// Generated models.
    pub models: GeneratedModels,
}

impl GeneratedAssets {
    /// Create a new [`GeneratedAssets`] from the given assets path.
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(assets_path: &Path) -> anyhow::Result<Self> {
        let mc_dir = assets_path.join("minecraft");
        Ok(Self {
            blockstates: GeneratedBlockstates::new(&mc_dir.join("blockstates")).await?,
            models: GeneratedModels::new(&mc_dir.join("models")).await?,
        })
    }
}

/// Generated blockstates.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct GeneratedBlockstates(HashMap<CompactString, BlockstateAsset>);

impl GeneratedBlockstates {
    /// Create a new [`GeneratedBlockstates`] from the given blockstates path.
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        let mut blockstates = HashMap::new();
        // Iterate for each blockstate file in the blockstates directory.
        let mut dir = tokio::fs::read_dir(path).await?;
        while let Ok(Some(entry)) = dir.next_entry().await {
            // Skip directories.
            if entry.file_type().await?.is_dir() {
                continue;
            }

            // Skip non-JSON files.
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();
            if !filename.ends_with(".json") {
                continue;
            }

            // Parse the blockstate file.
            let contents = tokio::fs::read_to_string(entry.path()).await?;
            blockstates.insert(filename.into(), serde_json::from_str(&contents)?);
        }
        Ok(Self(blockstates))
    }
}

/// A blockstate asset.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BlockstateAsset(serde_json::Value);

/// Generated models.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedModels {
    /// Block models.
    pub block: HashMap<CompactString, ModelAsset>,
    /// Item models.
    pub item: HashMap<CompactString, ModelAsset>,
}

impl GeneratedModels {
    /// Create a new [`GeneratedModels`] from the given models path.
    #[allow(clippy::missing_errors_doc)]
    pub async fn new(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            block: Self::new_map(&path.join("block")).await?,
            item: Self::new_map(&path.join("item")).await?,
        })
    }

    async fn new_map(path: &Path) -> anyhow::Result<HashMap<CompactString, ModelAsset>> {
        let mut models = HashMap::new();
        // Iterate for each model file in the models directory.
        let mut dir = tokio::fs::read_dir(path).await?;
        while let Ok(Some(entry)) = dir.next_entry().await {
            // Skip directories.
            if entry.file_type().await?.is_dir() {
                continue;
            }

            // Skip non-JSON files.
            let filename = entry.file_name();
            let filename = filename.to_string_lossy();
            if !filename.ends_with(".json") {
                continue;
            }

            // Parse the model file.
            let contents = tokio::fs::read_to_string(entry.path()).await?;
            models.insert(filename.into(), serde_json::from_str(&contents)?);
        }
        Ok(models)
    }
}

/// A model asset.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelAsset(serde_json::Value);
