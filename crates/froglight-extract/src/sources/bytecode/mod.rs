//! Data extracted from bytecode parsing.

use cafebabe::{attributes::AttributeData, bytecode::ByteCode, ClassFile};
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

mod modules;
pub use modules::*;

/// Modules that parse Minecraft bytecode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[enum_dispatch(ExtractModule)]
#[serde(untagged)]
pub enum BytecodeModule {
    /// Packet data
    Packets(Packets),
}

/// Get the bytecode for a method in a [`ClassFile`].
pub(crate) fn get_method_code<'a>(
    classfile: &'a ClassFile<'_>,
    method: &str,
) -> Option<&'a ByteCode<'a>> {
    let method = classfile.methods.iter().find(|m| m.name == method)?;

    let attribute = method.attributes.iter().find(|a| a.name == "Code")?;
    let AttributeData::Code(attribute) = &attribute.data else {
        return None;
    };

    attribute.bytecode.as_ref()
}
