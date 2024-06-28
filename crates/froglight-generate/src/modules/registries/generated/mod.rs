use std::path::Path;

use froglight_extract::bundle::ExtractBundle;

use crate::bundle::GenerateBundle;

mod attribute;
mod block;
mod registry;

#[allow(clippy::unused_async)]
pub(super) async fn create_generated(
    _gen_path: &Path,
    _generate: &GenerateBundle<'_>,
    _extract: &ExtractBundle<'_>,
) -> anyhow::Result<()> {
    todo!()
}
