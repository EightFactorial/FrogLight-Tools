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
#[derive(Debug, Clone, Deref, DerefMut, Serialize, Deserialize)]
pub struct TypesMap(HashMap<CompactString, DataType>);

/// A data type used in the protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Named(CompactString),
    Inline(CompactString, DataTypeArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataTypeArgs {
    Array(ArrayArgs),
    Bitfield(Vec<BitfieldArgs>),
    Buffer(BufferArgs),
    Container(Vec<ContainerArgs>),
    EntityMetadata(EntityMetadataArgs),
    Mapper(MapperArgs),
    Option(Box<DataType>),
    PString(BufferArgs),
    Switch(SwitchArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrayArgs {
    #[serde(rename = "countType")]
    pub count_type: CompactString,
    #[serde(rename = "type")]
    pub kind: Box<DataType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BitfieldArgs {
    pub name: CompactString,
    pub size: u32,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BufferArgs {
    #[serde(default)]
    pub count: Option<u32>,
    #[serde(default, rename = "countType")]
    pub count_type: Option<CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContainerArgs {
    #[serde(default)]
    pub name: Option<CompactString>,
    #[serde(rename = "type")]
    pub kind: DataType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityMetadataArgs {
    #[serde(rename = "endVal")]
    pub end_val: u32,
    #[serde(rename = "type")]
    pub kind: Box<DataType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapperArgs {
    #[serde(rename = "type")]
    pub kind: Box<DataType>,
    pub mappings: HashMap<CompactString, CompactString>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwitchArgs {
    #[serde(rename = "compareTo")]
    pub compare_to: CompactString,
    pub fields: HashMap<CompactString, DataType>,
}

impl Serialize for DataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            DataType::Named(name) => name.serialize(serializer),
            DataType::Inline(kind, args) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element(kind)?;
                seq.serialize_element(args)?;
                seq.end()
            }
        }
    }
}
impl<'de> Deserialize<'de> for DataType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DataTypeVisitor;
        impl<'de> serde::de::Visitor<'de> for DataTypeVisitor {
            type Value = DataType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or array")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(DataType::Named(CompactString::from(value)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let kind: CompactString =
                    seq.next_element()?.ok_or_else(|| A::Error::custom("missing name"))?;

                let args = match kind.as_str() {
                    "array" => DataTypeArgs::Array(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing array args"))?,
                    ),
                    "bitfield" => DataTypeArgs::Bitfield(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing bitfield args"))?,
                    ),
                    "buffer" => DataTypeArgs::Buffer(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing buffer args"))?,
                    ),
                    "container" => DataTypeArgs::Container(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing container args"))?,
                    ),
                    "entityMetadataLoop" => DataTypeArgs::EntityMetadata(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing entityMetadataLoop args"))?,
                    ),
                    "mapper" => DataTypeArgs::Mapper(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing mapper args"))?,
                    ),
                    "option" => DataTypeArgs::Option(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing option args"))?,
                    ),
                    "pstring" => DataTypeArgs::PString(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing pstring args"))?,
                    ),
                    "switch" => DataTypeArgs::Switch(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::custom("missing switch args"))?,
                    ),
                    _ => seq.next_element()?.ok_or_else(|| A::Error::custom("missing args"))?,
                };

                Ok(DataType::Inline(kind, args))
            }
        }

        deserializer.deserialize_any(DataTypeVisitor)
    }
}
