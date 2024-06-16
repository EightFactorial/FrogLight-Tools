use std::borrow::Cow;

use anyhow::anyhow;
use cafebabe::descriptor::{FieldType, Ty};
use hashbrown::HashMap;
use tracing::trace;

use super::Packets;
use crate::bundle::ExtractBundle;

impl Packets {
    pub(super) const BYTEBUF_TYPE: &'static str = "io/netty/buffer/ByteBuf";
    pub(super) const _PACKET_TYPE: &'static str = "net/minecraft/network/packet/Packet";
    pub(super) const PACKETBYTEBUF_TYPE: &'static str = "net/minecraft/network/PacketByteBuf";
    pub(super) const PACKETCODEC_TYPE: &'static str = "net/minecraft/network/codec/PacketCodec";
    pub(super) const _PACKETCODECS_TYPE: &'static str = "net/minecraft/network/codec/PacketCodecs";
    pub(super) const REGISTRYBYTEBUF_TYPE: &'static str = "net/minecraft/network/RegistryByteBuf";

    /// Parse the packets in the given class map.
    pub(super) fn parse(
        classes: HashMap<String, String>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<HashMap<String, (String, Vec<String>)>> {
        let mut packet_data = HashMap::with_capacity(classes.len());

        for (packet, class) in classes {
            trace!("Packet: {packet}");
            let fields = Self::parse_packet(&class, data)?;
            packet_data.insert(packet, (class, fields));
        }

        Ok(packet_data)
    }

    const PACKET_INIT_METHOD_NAME: &'static str = "<init>";
    const PACKET_INIT_METHOD_DESCRIPTOR: &'static [FieldType<'static>] =
        &[FieldType::Ty(Ty::Object(Cow::Borrowed(Self::PACKETBYTEBUF_TYPE)))];

    const PACKET_CODEC_FIELD_NAME: &'static str = "CODEC";
    const PACKET_CODEC_FIELD_DESCRIPTOR: &'static FieldType<'static> =
        &FieldType::Ty(Ty::Object(Cow::Borrowed(Self::PACKETCODEC_TYPE)));

    /// Parse a packet class to extract its fields.
    fn parse_packet(class: &str, data: &ExtractBundle<'_>) -> anyhow::Result<Vec<String>> {
        // Skip "Bundle" packets, which have no codec or fields
        if class.contains("Bundle") {
            return Ok(Vec::new());
        }

        let classfile = data.jar_container.get_class_err(class)?;

        // Check if the packet has an init method that takes a `PacketByteBuf`
        if let Some(init_method) = classfile.methods.iter().find(|&m| {
            m.name == Self::PACKET_INIT_METHOD_NAME
                && m.descriptor.parameters == Self::PACKET_INIT_METHOD_DESCRIPTOR
        }) {
            trace!("  Reading: {}.{}", classfile.this_class, Self::PACKET_INIT_METHOD_NAME);
            return super::method::parse_method(&classfile, init_method, data);
        }

        // Parse the packet's `CODEC` field
        if let Some(_codec_field) = classfile.fields.iter().find(|&f| {
            f.name == Self::PACKET_CODEC_FIELD_NAME
                && &f.descriptor == Self::PACKET_CODEC_FIELD_DESCRIPTOR
        }) {
            trace!("  Reading: {}.{}", classfile.this_class, Self::PACKET_CODEC_FIELD_NAME);
            return Ok(Vec::new());
        }

        // Error if the packet has no init method or codec field
        Err(anyhow!("Failed to find entrypoint to parse: \"{class}\""))
    }
}
