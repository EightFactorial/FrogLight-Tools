use std::path::Path;

use froglight_extract::bundle::ExtractBundle;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::warn;

use super::Packets;
use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{update_file_modules, version_module_name, version_struct_name},
};

#[allow(clippy::unused_async)]
impl Packets {
    pub(super) async fn create_version(
        path: &Path,
        generate: &GenerateBundle<'_>,
        extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let version_module = version_module_name(&generate.version.base).to_string();

        let version_path = path.join(&version_module);
        if !version_path.exists() {
            warn!("Creating version at \"{}\"", version_path.display());
            tokio::fs::create_dir(&version_path).await?;
        }

        // TODO: Create the connection state modules

        // Update the version module
        Self::update_mod(&version_path.join("mod.rs"), generate, extract).await
    }

    const VERSION_DOCS: &'static str = "//! Protocol `{PROTOCOL}`
//!
//! Used by Minecraft `{BASE}` - `{JAR}`";

    async fn update_mod(
        path: &Path,
        generate: &GenerateBundle<'_>,
        extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        // Create the `mod.rs` file
        let mut mod_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .await?;

        // Get the protocol version from the extracted data
        let protocol = extract.output["version"]["protocol_version"].as_i64().unwrap();

        // Create the module docs
        let mod_docs = Self::VERSION_DOCS
            .replace("{PROTOCOL}", &protocol.to_string())
            .replace("{BASE}", &generate.version.base.to_long_string())
            .replace("{JAR}", &generate.version.jar.to_long_string());

        // Create the struct docs
        let struct_docs = mod_docs.replace("//!", "///");

        // Create the struct and impl
        let output_contents = format!(
            r#"{mod_docs}
//!
{GENERATE_NOTICE}

{struct_docs}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]
pub struct {0};

impl crate::traits::Version for {0} {{
    const ID: i32 = {protocol};
}}
"#,
            version_struct_name(&generate.version.base)
        );
        mod_file.write_all(output_contents.as_bytes()).await?;

        // Update the tag and modules
        update_file_modules(&mut mod_file, path, true, false).await
    }
}
