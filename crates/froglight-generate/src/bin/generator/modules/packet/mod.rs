use froglight_generate::{CliArgs, DataMap, PacketGenerator};
use hashbrown::HashSet;

use super::ModuleGenerator;

impl ModuleGenerator for PacketGenerator {
    /// Generate packets from the given [`DataMap`].
    async fn generate(_datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> {
        // if datamap.version_data.is_empty() {
        //     tracing::warn!("PacketGenerator: No data to generate packets from!");
        //     return Ok(());
        // }

        // let universal = universal_types(datamap);
        // for type_name in universal {
        //     tracing::info!(
        //         "PacketGenerator: Type \"{type_name}\" is identical across all
        // versions."     );
        // }

        Ok(())
    }
}

/// Get the universal types, which are supported by all versions.
///
/// Only need to check one version,
/// if it's supported by all versions it'll be here.
fn _universal_types(datamap: &DataMap) -> HashSet<&str> {
    let mut universal_types = HashSet::new();
    // For all types in the first version
    if let Some(data) = datamap.version_data.values().next() {
        for (proto_name, proto_data) in
            data.proto.types.iter().filter(|(_, data)| !data.is_native())
        {
            // If all versions contain it *and* it's identical
            if datamap.version_data.values().all(|data| {
                if let Some(data) = data.proto.types.get(proto_name) {
                    data == proto_data
                } else {
                    false
                }
            }) {
                universal_types.insert(proto_name.as_str());
            }
        }
    }
    universal_types
}
