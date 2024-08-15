use std::borrow::Cow;

use anyhow::anyhow;
use cafebabe::descriptor::{FieldType, Ty};
use tracing::{trace, warn};

use super::Packets;
use crate::bundle::ExtractBundle;

impl Packets {
    pub(super) const BYTEBUF_TYPE: &'static str = "io/netty/buffer/ByteBuf";
    pub(super) const PACKET_TYPE: &'static str = "net/minecraft/network/packet/Packet";
    pub(super) const PACKETBYTEBUF_TYPE: &'static str = "net/minecraft/network/PacketByteBuf";
    pub(super) const PACKETCODEC_TYPE: &'static str = "net/minecraft/network/codec/PacketCodec";
    pub(super) const PACKETCODECS_TYPE: &'static str = "net/minecraft/network/codec/PacketCodecs";
    pub(super) const REGISTRYBYTEBUF_TYPE: &'static str = "net/minecraft/network/RegistryByteBuf";

    /// Parse the packets in the given class map.
    pub(super) fn parse(
        classes: Vec<(String, String)>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<(String, String, Vec<String>)>> {
        let mut packet_data = Vec::with_capacity(classes.len());

        for (packet, class) in classes {
            trace!("Packet: {packet}");
            let fields = Self::parse_packet(&class, data)?;
            packet_data.push((packet, class, fields));
        }

        Ok(packet_data)
    }

    const PACKET_INIT_METHOD_NAME: &'static str = "<init>";

    const PACKET_INIT_METHOD_BYTEBUF_PARAMETERS: &'static [FieldType<'static>] =
        &[FieldType::Ty(Ty::Object(Cow::Borrowed(Self::BYTEBUF_TYPE)))];
    const PACKET_INIT_METHOD_PACKETBYTEBUF_PARAMETERS: &'static [FieldType<'static>] =
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
                && (m.descriptor.parameters == Self::PACKET_INIT_METHOD_BYTEBUF_PARAMETERS
                    || m.descriptor.parameters == Self::PACKET_INIT_METHOD_PACKETBYTEBUF_PARAMETERS)
        }) {
            trace!("  Reading: {}.{}", classfile.this_class, Self::PACKET_INIT_METHOD_NAME);
            let result = super::method::parse_method(&classfile, init_method, data)?;
            trace!("  Done: {}.{}", classfile.this_class, Self::PACKET_INIT_METHOD_NAME);
            return Ok(result);
        }

        // Parse the packet's `CODEC` field
        if let Some(codec_field) = classfile.fields.iter().find(|&f| {
            f.name == Self::PACKET_CODEC_FIELD_NAME
                && &f.descriptor == Self::PACKET_CODEC_FIELD_DESCRIPTOR
        }) {
            return super::codec::parse_codec(&classfile, codec_field, data);
        }

        // Override for CustomPayloadS2CPacket
        if class.ends_with("/CustomPayloadS2CPacket") {
            return Ok(vec![
                String::from("VarInt"),
                String::from("ResourceLocation"),
                String::from("UnsizedBuffer"),
            ]);
        }

        // Parse the first `CODEC`-type field in the class
        if let Some(codec_field) =
            classfile.fields.iter().find(|&f| f.descriptor == *Self::PACKET_CODEC_FIELD_DESCRIPTOR)
        {
            warn!("  Reading: Searching for fallback CODEC...");
            return super::codec::parse_codec(&classfile, codec_field, data);
        }

        // Error if the packet has no init method or codec field
        Err(anyhow!("Failed to find entrypoint to parse: \"{class}\""))
    }
}
