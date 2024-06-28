use std::path::Path;

use froglight_extract::bundle::ExtractBundle;

use crate::bundle::GenerateBundle;

#[allow(clippy::unused_async)]
pub(super) async fn create_versioned(
    _ver_path: &Path,
    _generate: &GenerateBundle<'_>,
    _extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    Ok(())
}
