//! Data extracted from bytecode parsing.

use std::borrow::Cow;

use cafebabe::{
    attributes::{AttributeData, BootstrapMethodEntry},
    bytecode::ByteCode,
    ClassFile, FieldInfo, MethodInfo,
};
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

/// Get the field with the given name in a [`ClassFile`].
pub(crate) fn get_class_field<'a>(
    classfile: &'a ClassFile<'_>,
    field: &str,
) -> Option<&'a FieldInfo<'a>> {
    classfile.fields.iter().find(|&f| f.name == field)
}
/// Get the method with the given name in a [`ClassFile`].
pub(crate) fn get_class_method<'a>(
    classfile: &'a ClassFile<'_>,
    method: &str,
) -> Option<&'a MethodInfo<'a>> {
    classfile.methods.iter().find(|&m| m.name == method)
}
/// Get the [`AttributeData::BootstrapMethods`] of a [`ClassFile`].
pub(crate) fn get_class_bootstrap<'a>(
    classfile: &'a ClassFile<'_>,
) -> Option<&'a [BootstrapMethodEntry<'a>]> {
    classfile.attributes.iter().find(|&a| a.name == "BootstrapMethods").and_then(|a| {
        if let AttributeData::BootstrapMethods(bootstrap) = &a.data {
            Some(bootstrap.as_ref())
        } else {
            None
        }
    })
}

/// Get the [`ByteCode`] for a method in a [`ClassFile`].
pub(crate) fn get_class_method_code<'a>(
    classfile: &'a ClassFile<'_>,
    method: &str,
) -> Option<&'a ByteCode<'a>> {
    get_class_method(classfile, method).and_then(get_method_code)
}
/// Get the [`AttributeData::Code`] for a [`MethodInfo`].
pub(crate) fn get_method_code<'a>(method: &'a MethodInfo<'_>) -> Option<&'a ByteCode<'a>> {
    method.attributes.iter().find(|&a| a.name == "Code").and_then(|a| {
        if let AttributeData::Code(code) = &a.data {
            code.bytecode.as_ref()
        } else {
            None
        }
    })
}
/// Get the [`AttributeData::Signature`] of a [`MethodInfo`].
#[allow(dead_code)]
pub(crate) fn get_method_signature<'a>(method: &'a MethodInfo<'_>) -> Option<&'a Cow<'a, str>> {
    method.attributes.iter().find(|&a| a.name == "Signature").and_then(|a| {
        if let AttributeData::Signature(signature) = &a.data {
            Some(signature)
        } else {
            None
        }
    })
}

/// Get the [`AttributeData::Signature`] of a [`FieldInfo`].
pub(crate) fn get_field_signature<'a>(field: &'a FieldInfo<'_>) -> Option<&'a Cow<'a, str>> {
    field.attributes.iter().find(|&a| a.name == "Signature").and_then(|a| {
        if let AttributeData::Signature(signature) = &a.data {
            Some(signature)
        } else {
            None
        }
    })
}

/// Returns `true` if the descriptor points to a function.
pub(crate) fn is_descriptor_function(descriptor: &str) -> bool { descriptor.starts_with('(') }
