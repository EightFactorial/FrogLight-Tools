use std::path::Path;

use froglight_extract::bundle::ExtractBundle;
use serde_json::Value;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use tracing::trace;

use super::Packets;
use crate::{
    bundle::GenerateBundle,
    consts::GENERATE_NOTICE,
    helpers::{class_to_module_name, class_to_struct_name, format_file},
};

impl Packets {
    pub(super) async fn create_packet(
        packet_name: &str,
        _packet_data: &Value,
        path: &Path,
        _generate: &GenerateBundle<'_>,
        _extract: &ExtractBundle<'_>,
    ) -> anyhow::Result<()> {
        let module_name = class_to_module_name(packet_name);
        let struct_name = class_to_struct_name(packet_name);

        let mut packet_path = path.join(&module_name);
        packet_path.set_extension("rs");

        trace!("Creating packet at \"{}\"", packet_path.display());

        let mut packet_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&packet_path)
            .await?;

        let (imports, derives) = Self::imports_and_derives();
        let attributes = Self::attributes();

        let output = format!(
            r#"{GENERATE_NOTICE}

{imports}

{derives}
{attributes}
pub struct {struct_name};"#
        );
        packet_file.write_all(output.as_bytes()).await?;
        format_file(&mut packet_file).await?;

        Ok(())
    }

    /// The imports and derives for the packet struct.
    fn imports_and_derives() -> (String, String) {
        let imports = String::from("use froglight_macros::FrogReadWrite;");
        let derives = String::from("#[derive(Debug, Clone, PartialEq, Hash, FrogReadWrite)]");

        (imports, derives)
    }

    /// The attributes for the packet struct.
    fn attributes() -> String {
        String::from(r#"#[cfg_attr(feature = "bevy", derive(bevy_reflect::Reflect))]"#)
    }
}
