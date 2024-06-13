use tracing::trace;

use super::Packets;
use crate::sources::bytecode::modules::packets::constructor::CodecConstructor;

impl Packets {
    /// Parse a special codec.
    ///
    /// Returns a type name if the function is known, otherwise `None`.
    pub(super) fn parse_special(codec: CodecConstructor<'_>) -> Option<String> {
        let CodecConstructor::Special(member) = codec else { panic!("Expected special codec") };

        match &*member.class_name {
            Self::PACKET_TYPE | Self::PACKET_CODEC_TYPE | Self::PACKET_CODECS_TYPE => {
                match &*member.name_and_type.name {
                    "collect" => Some(String::from("Vec")),
                    "entryOf" | "indexed" | "registryValue" => Some(String::from("VarInt")),
                    "map" => Some(String::from("HashMap")),
                    "nbt" | "nbtCompound" => Some(String::from("Nbt")),
                    "registryEntry" => Some(String::from("ResourceLocation")),
                    "string" => Some(String::from("String")),
                    "unlimitedCodec" | "unlimitedRegistryCodec" => Some(String::from("Text")),

                    "dispatch" | "ofStatic" => None,

                    _ => {
                        trace!("Unknown Special Codec: {member:?}");
                        None
                    }
                }
            }
            "net/minecraft/item/ItemStack$1" => Some(String::from("ItemStack")),
            "net/minecraft/util/math/BlockPos$1" => Some(String::from("BlockPos")),
            "net/minecraft/util/Uuids$1" => Some(String::from("Uuid")),

            "net/minecraft/network/codec/PacketCodecs$1" => Some(String::from("bool")),
            "net/minecraft/network/codec/PacketCodecs$2"
            | "net/minecraft/network/codec/PacketCodecs$3" => Some(String::from("Vec<u8>")),
            "net/minecraft/network/codec/PacketCodecs$4" => Some(String::from("String")),
            "net/minecraft/network/codec/PacketCodecs$5" => Some(String::from("Nbt")),
            "net/minecraft/network/codec/PacketCodecs$7" => Some(String::from("Option<Nbt>")),
            "net/minecraft/network/codec/PacketCodecs$8" => Some(String::from("Vec3")),
            "net/minecraft/network/codec/PacketCodecs$9" => Some(String::from("Quat")),
            "net/minecraft/network/codec/PacketCodecs$10" => Some(String::from("Option")),
            "net/minecraft/network/codec/PacketCodecs$12" => Some(String::from("u8")),
            "net/minecraft/network/codec/PacketCodecs$20" => Some(String::from("GameProfile")),
            "net/minecraft/network/codec/PacketCodecs$22" => Some(String::from("i16")),
            "net/minecraft/network/codec/PacketCodecs$25" => Some(String::from("VarInt")),
            "net/minecraft/network/codec/PacketCodecs$26" => Some(String::from("i64")),
            "net/minecraft/network/codec/PacketCodecs$27" => Some(String::from("f32")),
            "net/minecraft/network/codec/PacketCodecs$28" => Some(String::from("f64")),
            _ => {
                trace!("Unknown Special Codec: {member:?}");
                None
            }
        }
    }
}
