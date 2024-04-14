use serde::Deserialize;

#[cfg(test)]
mod tests;

/// Information about the assets for a
/// [`version`](crate::MinecraftVersion) of Minecraft.
#[derive(Debug, Clone, Hash, Deserialize)]
pub struct AssetManifest {}
