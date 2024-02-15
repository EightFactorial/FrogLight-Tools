use std::{borrow::Cow, collections::BTreeMap};

use cafebabe::{
    attributes::AttributeData,
    bytecode::Opcode,
    constant_pool::MemberRef,
    descriptor::{BaseType, FieldType, Ty},
    ClassFile,
};
use serde::Serialize;
use tracing::{error, warn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Output<'a> {
    Unnamed(Vec<OutputType>),
    Named(BTreeMap<Cow<'a, str>, OutputType>),
}

/// An output type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[allow(dead_code)]
pub(crate) enum OutputType {
    Bool,
    U8,
    U16,
    U32,
    U64,
    U128,

    VarU16,
    VarU32,
    VarU64,

    I8,
    I16,
    I32,
    I64,
    I128,

    F32,
    F64,

    Enum,

    String,
    Uuid,
    Text,
    ResourceLocation,

    Item,
    ItemStack,
    BlockPos,

    Nbt,
    GameProfile,

    Option(Box<Resolvable>),
    Vec(Box<Resolvable>),
    Map(Box<Resolvable>, Box<Resolvable>),

    Obj(String),
}

/// A resolvable type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[allow(dead_code)]
pub(crate) enum Resolvable {
    Unknown,
    Type(OutputType),
}

impl OutputType {
    /// Try to convert a read method name to an output type.
    ///
    /// This does not handle generic types.
    pub(super) fn try_from_read_method(method: &str) -> Option<Self> {
        match method {
            "readBoolean" => Some(Self::Bool),
            "readUnsignedByte" | "readByte" => Some(Self::U8),
            "readUnsignedShort" => Some(Self::U16),
            "readUnsignedInt" => Some(Self::U32),
            "readUnsignedLong" | "readInstant" => Some(Self::U64),

            "readVarShort" => Some(Self::VarU16),
            "readVarInt" | "readEnumConstant" => Some(Self::VarU32),
            "readVarLong" => Some(Self::VarU64),

            "readShort" => Some(Self::I16),
            "readInt" => Some(Self::I32),
            "readLong" => Some(Self::I64),

            "readFloat" => Some(Self::F32),
            "readDouble" => Some(Self::F64),

            "readEnumSet" => Some(Self::Enum),

            "readUuid" => Some(Self::Uuid),
            "readString" => Some(Self::String),
            "readText" => Some(Self::Text),
            "readIdentifier" | "readRegistryValue" | "readRegistryEntry" => {
                Some(Self::ResourceLocation)
            }

            "readByteArray" => Some(Self::Vec(Box::new(Resolvable::Type(OutputType::U8)))),
            "readIntList" | "readIntArray" => {
                Some(Self::Vec(Box::new(Resolvable::Type(OutputType::I32))))
            }

            "readBlockPos" => Some(Self::BlockPos),

            "readItem" => Some(Self::Item),
            "readItemStack" => Some(Self::ItemStack),

            "readNbt" => Some(Self::Nbt),
            "readGameProfile" => Some(Self::GameProfile),

            "readCollection" | "readList" | "readSet" | "readMap" | "readNullable"
            | "readOptional" => {
                // These types are generic and are resolved with a fallback method
                None
            }
            unk => {
                warn!("Unknown PacketByteBuf method: `{unk}`");
                None
            }
        }
    }

    /// Try to convert a write method name to an output type.
    ///
    /// This does not handle generic types.
    pub(super) fn try_from_write_method(method: &str) -> Option<Self> {
        match method {
            "writeBoolean" => Some(Self::Bool),
            "writeByte" | "writeUnsignedByte" => Some(Self::U8),
            "writeUnsignedShort" => Some(Self::U16),
            "writeUnsignedInt" => Some(Self::U32),
            "writeUnsignedLong" | "writeInstant" => Some(Self::U64),

            "writeVarShort" => Some(Self::VarU16),
            "writeVarInt" | "writeEnumConstant" => Some(Self::VarU32),
            "writeVarLong" => Some(Self::VarU64),

            "writeShort" => Some(Self::I16),
            "writeInt" => Some(Self::I32),
            "writeLong" => Some(Self::I64),

            "writeFloat" => Some(Self::F32),
            "writeDouble" => Some(Self::F64),

            "writeEnumSet" => Some(Self::Enum),

            "writeUuid" => Some(Self::Uuid),
            "writeString" => Some(Self::String),
            "writeText" => Some(Self::Text),
            "writeIdentifier" | "writeRegistryValue" | "writeRegistryEntry" => {
                Some(Self::ResourceLocation)
            }

            "writeByteArray" => Some(Self::Vec(Box::new(Resolvable::Type(OutputType::U8)))),
            "writeIntList" | "writeIntArray" => {
                Some(Self::Vec(Box::new(Resolvable::Type(OutputType::I32))))
            }

            "writeBlockPos" => Some(Self::BlockPos),

            "writeItem" => Some(Self::Item),
            "writeItemStack" => Some(Self::ItemStack),

            "writeNbt" => Some(Self::Nbt),
            "writeGameProfile" => Some(Self::GameProfile),

            "writeCollection" | "writeList" | "writeSet" | "writeMap" | "writeNullable"
            | "writeOptional" => {
                // These types are generic and are resolved with a fallback method
                None
            }
            unk => {
                warn!("Unknown PacketByteBuf method: `{unk}`");
                None
            }
        }
    }

    /// Try to convert a generic type to an output type.
    pub(super) fn try_from_generic(class: &ClassFile, opcodes: &[(usize, Opcode)]) -> Option<Self> {
        // Find the field name for the generic type
        let mut field_name = None;
        for (_, op) in opcodes {
            match op {
                Opcode::Getfield(MemberRef { class_name, name_and_type })
                | Opcode::Putfield(MemberRef { class_name, name_and_type }) => {
                    if class.this_class == *class_name {
                        field_name = Some(name_and_type.name.clone());
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(field_name) = field_name else {
            error!("Could not find a field name for generic type in `{}`", class.this_class);
            return None;
        };

        // Find the field in the class
        let Some(field) = class.fields.iter().find(|field| field.name == field_name) else {
            error!("Could not find field `{field_name}` in `{}`", class.this_class);
            return None;
        };

        // Try to get the type from the Signature attribute
        if let Some(attr) = field.attributes.iter().find(|attr| attr.name == "Signature") {
            let AttributeData::Signature(sig) = &attr.data else {
                unreachable!("Attribute is not a signature")
            };

            // Try to get the type from the signature
            OutputType::try_from_signature(sig)
        } else if let FieldType::Ty(Ty::Object(obj)) = &field.descriptor {
            // Try to get the type from the descriptor
            Some(OutputType::from_descriptor(obj))
        } else {
            error!("Could not find a type for field `{field_name}` in `{}`", class.this_class);

            None
        }
    }

    /// Try to convert a signature to an output type.
    ///
    /// # Example
    /// - `Ljava/util/List<Ljava/lang/String;>;` ->
    ///   `Vec(Box<Resolvable::Type(OutputType::String))`
    /// - `Ljava/util/Optional<Ljava/lang/String;>;` ->
    ///   `Option(Box<Resolvable::Type(OutputType::String))`
    fn try_from_signature(signature: &str) -> Option<Self> {
        let mut chars = signature.chars().peekable();
        let ty = chars.next()?;

        match ty {
            'Z' => Some(Self::Bool),
            'B' => Some(Self::U8),
            'C' => Some(Self::U16),
            'S' => Some(Self::I16),
            'I' => Some(Self::I32),
            'J' => Some(Self::I64),
            'F' => Some(Self::F32),
            'D' => Some(Self::F64),
            'L' => {
                let mut name = String::new();
                let mut skip_count = 0;

                for c in chars.by_ref() {
                    if c == '<' {
                        skip_count += 1;
                    } else if c == '>' {
                        skip_count -= 1;
                    } else if skip_count == 0 && c == ';' {
                        break;
                    }
                    name.push(c);
                }

                // Support for generic util types
                if let Some(suffix) = name.strip_prefix("java/util/") {
                    if suffix.starts_with("List<") {
                        let inner = suffix.strip_prefix("List<")?;
                        let inner = inner.strip_suffix('>')?;
                        let inner = Self::try_from_signature(inner);
                        return Some(Self::Vec(Box::new(Resolvable::from(inner))));
                    } else if suffix.starts_with("Set<") {
                        let inner = suffix.strip_prefix("Set<")?;
                        let inner = inner.strip_suffix('>')?;
                        let inner = Self::try_from_signature(inner);
                        return Some(Self::Vec(Box::new(Resolvable::from(inner))));
                    } else if suffix.starts_with("Collection<") {
                        let inner = suffix.strip_prefix("Collection<")?;
                        let inner = inner.strip_suffix('>')?;
                        let inner = Self::try_from_signature(inner);
                        return Some(Self::Vec(Box::new(Resolvable::from(inner))));
                    } else if suffix.starts_with("Optional<") {
                        let inner = suffix.strip_prefix("Optional<")?;
                        let inner = inner.strip_suffix('>')?;
                        let inner = Self::try_from_signature(inner);
                        return Some(Self::Option(Box::new(Resolvable::from(inner))));
                    } else if suffix.starts_with("Map<") {
                        let inner = suffix.strip_prefix("Map<")?;
                        let inner = inner.strip_suffix('>')?;
                        let mut inner = inner.split(';');
                        let key = Self::try_from_signature(inner.next().unwrap());
                        let value = Self::try_from_signature(inner.next().unwrap());
                        return Some(Self::Map(
                            Box::new(Resolvable::from(key)),
                            Box::new(Resolvable::from(value)),
                        ));
                    }
                }

                // Resolve generic fastutil types
                if let Some(suffix) = name.strip_prefix("it/unimi/dsi/fastutil/objects/") {
                    if suffix.starts_with("Object2IntMap<") {
                        let inner = suffix.strip_prefix("Object2IntMap<")?;
                        let inner = inner.strip_suffix('>')?;
                        let inner = Self::try_from_signature(inner);
                        return Some(Self::Map(
                            Box::new(Resolvable::new(Self::I32)),
                            Box::new(Resolvable::from(inner)),
                        ));
                    }
                }

                Some(Self::from_descriptor(&name))
            }
            '[' => {
                let inner = Self::try_from_signature(&chars.collect::<String>());
                Some(Self::Vec(Box::new(Resolvable::from(inner))))
            }
            _ => {
                warn!("Unable to decode signature: `{signature}`");
                None
            }
        }
    }

    fn from_descriptor(desc: &str) -> Self {
        match desc {
            "java/lang/String" => Self::String,
            "java/util/UUID" => Self::Uuid,
            "net/minecraft/text/Text" => Self::Text,
            "net/minecraft/util/Identifier" => Self::ResourceLocation,
            "net/minecraft/item/Item" => Self::Item,
            "net/minecraft/item/ItemStack" => Self::ItemStack,
            "net/minecraft/util/math/BlockPos" => Self::BlockPos,
            "net/minecraft/nbt/CompoundTag" => Self::Nbt,
            _ => {
                warn!("Signature Object: `{desc}`");
                Self::Obj(desc.to_string())
            }
        }
    }
}

impl From<BaseType> for OutputType {
    fn from(value: BaseType) -> Self {
        match value {
            BaseType::Boolean => Self::Bool,
            BaseType::Byte => Self::I8,
            BaseType::Char => Self::U16,
            BaseType::Short => Self::I16,
            BaseType::Int => Self::I32,
            BaseType::Long => Self::I64,
            BaseType::Float => Self::F32,
            BaseType::Double => Self::F64,
        }
    }
}

impl Resolvable {
    fn new(ty: OutputType) -> Self { Self::Type(ty) }
}

impl From<Option<OutputType>> for Resolvable {
    fn from(value: Option<OutputType>) -> Self {
        match value {
            Some(ty) => Self::Type(ty),
            None => Self::Unknown,
        }
    }
}
