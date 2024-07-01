use std::path::Path;

use froglight_extract::bundle::ExtractBundle;

use crate::{bundle::GenerateBundle, helpers::update_tag};

mod block;
mod registry;

pub(super) async fn create_versioned(
    ver_path: &Path,
    generate: &GenerateBundle<'_>,
    extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    let blck_path = ver_path.join("blocks.rs");
    block::generate_blocks(&blck_path, generate, extract).await?;

    let reg_path = ver_path.join("registries");
    registry::generate_registries(&reg_path, generate, extract).await?;

    update_tag(&ver_path.join("mod.rs")).await
}
