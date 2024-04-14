use serde::Deserialize;

#[cfg(test)]
mod tests;

/// Information about a specific released
/// [`version`](crate::MinecraftVersion) of Minecraft.
#[derive(Debug, Clone, Hash, Deserialize)]
pub struct ReleaseManifest {}
