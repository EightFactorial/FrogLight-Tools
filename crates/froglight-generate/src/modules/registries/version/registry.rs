use std::path::Path;

use froglight_extract::bundle::ExtractBundle;

use crate::bundle::GenerateBundle;

#[allow(clippy::unused_async)]
pub(super) async fn generate_registries(
    _reg_path: &Path,
    _generate: &GenerateBundle<'_>,
    _extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    Ok(())
}
