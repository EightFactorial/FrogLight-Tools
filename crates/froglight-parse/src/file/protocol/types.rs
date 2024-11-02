#![allow(missing_docs)]

use compact_str::CompactString;
use derive_more::derive::{Deref, DerefMut};
use hashbrown::HashMap;
use serde::{
    de::{Error, SeqAccess},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize, Serializer,
};

/// A map of types used in the protocol.
#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtocolTypeMap(HashMap<CompactString, ProtocolType>);

/// A data type used in the protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolType {
    Named(CompactString),
    Inline(CompactString, ProtocolTypeArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ProtocolTypeArgs {
    Array(ArrayArgs),
    Bitfield(Vec<BitfieldArg>),
    Buffer(BufferArgs),
    Container(Vec<ContainerArg>),
    EntityMetadata(EntityMetadataArgs),
    Mapper(MapperArgs),
    Option(Box<ProtocolType>),
    PString(BufferArgs),
    Switch(SwitchArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayArgs {
    #[serde(rename = "countType")]
    pub count_type: CompactString,
    #[serde(rename = "type")]
    pub kind: Box<ProtocolType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitfieldArg {
    pub name: CompactString,
    pub size: u32,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferArgs {
    #[serde(rename = "count")]
    Count(u32),
    #[serde(rename = "countType")]
    CountType(CompactString),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerArg {
    #[serde(default)]
    pub name: Option<CompactString>,
    #[serde(rename = "type")]
    pub kind: ProtocolType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMetadataArgs {
    #[serde(rename = "endVal")]
    pub end_val: u32,
    #[serde(rename = "type")]
    pub kind: Box<ProtocolType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapperArgs {
    #[serde(rename = "type")]
    pub kind: Box<ProtocolType>,
    pub mappings: HashMap<CompactString, CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchArgs {
    #[serde(rename = "compareTo")]
    pub compare_to: CompactString,
    pub fields: HashMap<CompactString, ProtocolType>,
}

impl Serialize for ProtocolType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ProtocolType::Named(name) => name.serialize(serializer),
            ProtocolType::Inline(kind, args) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(kind)?;
                seq.serialize_element(args)?;
                seq.end()
            }
        }
    }
}
impl<'de> Deserialize<'de> for ProtocolType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProtocolTypeVisitor;
        impl<'de> serde::de::Visitor<'de> for ProtocolTypeVisitor {
            type Value = ProtocolType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or array")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(ProtocolType::Named(CompactString::from(v)))
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(ProtocolType::Named(CompactString::from(value)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let kind: CompactString =
                    seq.next_element()?.ok_or_else(|| A::Error::custom("missing name"))?;

                let args = match kind.as_str() {
                    "array" => ProtocolTypeArgs::Array(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing array args"))?,
                    ),
                    "bitfield" => ProtocolTypeArgs::Bitfield(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing bitfield args"))?,
                    ),
                    "buffer" => ProtocolTypeArgs::Buffer(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing buffer args"))?,
                    ),
                    "container" => ProtocolTypeArgs::Container(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing container args"))?,
                    ),
                    "entityMetadataLoop" => ProtocolTypeArgs::EntityMetadata(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing entityMetadataLoop args"))?,
                    ),
                    "mapper" => ProtocolTypeArgs::Mapper(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing mapper args"))?,
                    ),
                    "option" => ProtocolTypeArgs::Option(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing option args"))?,
                    ),
                    "pstring" => ProtocolTypeArgs::PString(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing pstring args"))?,
                    ),
                    "switch" => ProtocolTypeArgs::Switch(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing switch args"))?,
                    ),
                    unknown => {
                        return Err(A::Error::custom(format!("unknown data type, \"{unknown}\"")))
                    }
                };

                Ok(ProtocolType::Inline(kind, args))
            }
        }

        deserializer.deserialize_any(ProtocolTypeVisitor)
    }
}
