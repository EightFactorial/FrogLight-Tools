use froglight_extract::bundle::ExtractBundle;
use froglight_generate::modules::Modules;

#[expect(clippy::unused_async)]
pub(super) async fn generate(
    _bundles: Vec<ExtractBundle>,
    _modules: Vec<Modules>,
) -> anyhow::Result<()> {
    Ok(())
}
