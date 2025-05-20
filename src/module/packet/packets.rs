use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::MinecraftCode, version::Version,
};

use super::Packets;

impl Packets {
    pub(super) async fn extract_packet_classes(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<String>> {
        let collections = Self::extract_packet_collections(version, deps).await?;

        Ok(collections)
    }

    async fn extract_packet_collections(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<String>> {
        const PACKET_CLASS_PREFIX: &str = "net/minecraft/network/packet/";

        let mut packets = Vec::with_capacity(64);

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            for class in jar.get_filter(|(n, _)| {
                n.strip_prefix(PACKET_CLASS_PREFIX)
                    .map_or(false, |s| s.ends_with("Packets") && !s.contains('/'))
            }) {
                packets.push(class.this_class.to_string());
            }

            Ok(())
        })
        .await?;

        Ok(packets)
    }
}
