use anyhow::bail;
use cafebabe::{
    descriptor::{FieldType, Ty},
    ClassFile, FieldInfo,
};
use tracing::warn;

use super::Packets;
use crate::{
    bundle::ExtractBundle,
    bytecode::ClassContainer,
    sources::{bytecode::modules::packets::constructor::CodecConstructor, get_class_field},
};

impl Packets {
    const PACKET_CODEC_NAME: &'static str = "CODEC";

    pub(super) fn packet_fields(
        class: &str,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        // Bundle packets are units with no codec
        if class.contains("Bundle") {
            return Ok(Vec::new());
        }

        if let Some(classfile) = data.jar_container.get(class).map(ClassContainer::parse) {
            Self::codec_fields(&classfile, Self::PACKET_CODEC_NAME, data)
        } else {
            bail!("Packet class \"{class}\" not found in jar");
        }
    }

    pub(super) const PACKET_TYPE: &'static str = "net/minecraft/network/packet/Packet";
    pub(super) const PACKET_CODEC_TYPE: &'static str = "net/minecraft/network/codec/PacketCodec";
    pub(super) const PACKET_CODECS_TYPE: &'static str = "net/minecraft/network/codec/PacketCodecs";

    pub(super) fn codec_fields(
        classfile: &ClassFile<'_>,
        codec_name: &str,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        if let Some(codec_field) = get_class_field(classfile, codec_name) {
            match &codec_field.descriptor {
                FieldType::Ty(Ty::Object(object)) => match &**object {
                    Self::PACKET_CODEC_TYPE => Self::read_codec(classfile, codec_field, data),
                    unk => {
                        warn!("Unknown codec type: {unk}");
                        Self::read_codec(classfile, codec_field, data)
                    }
                },
                unk => bail!(
                    "Class \"{}\" codec \"{codec_name}\" has unknown type: {unk:?}",
                    classfile.this_class
                ),
            }
        } else {
            bail!(
                "Packet class \"{}\" has no codec field named \"{codec_name}\"",
                classfile.this_class,
            );
        }
    }

    fn read_codec(
        classfile: &ClassFile<'_>,
        codec_field: &FieldInfo<'_>,
        data: &ExtractBundle<'_>,
    ) -> anyhow::Result<Vec<String>> {
        match Self::codec_type(classfile, codec_field)? {
            (codec @ CodecConstructor::Create(..), index) => Self::parse_create(codec, index, data),
            (CodecConstructor::Special(member), _) => {
                let Some(classfile) =
                    data.jar_container.get(member.class_name.as_ref()).map(ClassContainer::parse)
                else {
                    bail!("Special codec class not found: {}", member.class_name);
                };

                Self::parse_method(&classfile, &member.name_and_type, data)
            }
            (codec @ CodecConstructor::Tuple(_), index) => {
                Self::parse_tuple(classfile, codec, index, data)
            }
            (CodecConstructor::Unit | CodecConstructor::XMap(..), _) => Ok(Vec::new()),
        }
    }
}
