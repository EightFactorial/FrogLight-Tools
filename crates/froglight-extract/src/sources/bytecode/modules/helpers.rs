#![allow(dead_code)]

use std::borrow::Cow;

use anyhow::bail;
use cafebabe::{
    attributes::{AttributeData, AttributeInfo, BootstrapMethodEntry},
    bytecode::ByteCode,
    constant_pool::{MemberRef, MethodHandle, NameAndType},
    ClassFile, FieldInfo, MethodInfo,
};
use tracing::trace;

/// Get the field with the given name in a [`ClassFile`].
pub(crate) fn get_class_field<'a>(
    classfile: &'a ClassFile<'_>,
    field: &str,
) -> anyhow::Result<&'a FieldInfo<'a>> {
    if let Some(field) = classfile.fields.iter().find(|&f| f.name == field) {
        Ok(field)
    } else {
        trace!("Fields: {:?}", classfile.fields.iter().map(|f| &f.name).collect::<Vec<_>>());
        bail!("Class \"{}\" does not have field \"{field}\"", classfile.this_class);
    }
}
/// Get the method with the given name in a [`ClassFile`].
///
/// There may be multiple methods with the same name,
/// so the descriptor can be used to differentiate them.
pub(crate) fn get_class_method<'a>(
    classfile: &'a ClassFile<'_>,
    method: &str,
    descriptor: Option<&str>,
) -> anyhow::Result<&'a MethodInfo<'a>> {
    if let Some(method) = classfile.methods.iter().find(|&m| {
        m.name == method
            && if let Some(descriptor) = descriptor {
                m.descriptor.to_string() == descriptor
            } else {
                true
            }
    }) {
        Ok(method)
    } else {
        trace!("Methods: {:?}", classfile.methods.iter().map(|m| &m.name).collect::<Vec<_>>());
        bail!("Class \"{}\" does not have method \"{method}\"", classfile.this_class);
    }
}

/// Get the [`AttributeData::BootstrapMethods`] for a list of [`AttributeInfo`].
pub(crate) fn get_bootstrap<'a>(
    attributes: &'a [AttributeInfo<'a>],
) -> anyhow::Result<&'a [BootstrapMethodEntry<'a>]> {
    get_bootstrap_silent(attributes).ok_or_else(|| {
        trace!("Attributes: {attributes:?}");
        anyhow::anyhow!("No `BootstrapMethods` in attribute list")
    })
}
pub(crate) fn get_bootstrap_silent<'a>(
    attributes: &'a [AttributeInfo<'a>],
) -> Option<&'a [BootstrapMethodEntry<'a>]> {
    attributes.iter().find_map(|a| {
        if a.name == "BootstrapMethods" {
            if let AttributeData::BootstrapMethods(bootstrap) = &a.data {
                Some(bootstrap.as_ref())
            } else {
                panic!("BootstrapMethods is not `AttributeData::BootstrapMethods`!");
            }
        } else {
            None
        }
    })
}

/// Get the [`AttributeData::Code`] for a list of [`AttributeInfo`].
pub(crate) fn get_code<'a>(
    attributes: &'a [AttributeInfo<'a>],
) -> anyhow::Result<&'a ByteCode<'a>> {
    get_code_silent(attributes).ok_or_else(|| {
        trace!("Attributes: {attributes:?}");
        anyhow::anyhow!("No `Code` in attribute list")
    })
}
pub(crate) fn get_code_silent<'a>(attributes: &'a [AttributeInfo<'a>]) -> Option<&'a ByteCode<'a>> {
    attributes.iter().find_map(|a| {
        if a.name == "Code" {
            if let AttributeData::Code(code) = &a.data {
                code.bytecode.as_ref()
            } else {
                panic!("Code is not `AttributeData::Code`!");
            }
        } else {
            None
        }
    })
}

/// Get the [`AttributeData::Signature`] from a list of [`AttributeInfo`].
pub(crate) fn get_signature<'a>(
    attributes: &'a [AttributeInfo<'a>],
) -> anyhow::Result<&'a Cow<'a, str>> {
    get_signature_silent(attributes).ok_or_else(|| {
        trace!("Attributes: {attributes:?}");
        anyhow::anyhow!("No `Signature` in attribute list")
    })
}
pub(crate) fn get_signature_silent<'a>(
    attributes: &'a [AttributeInfo<'a>],
) -> Option<&'a Cow<'a, str>> {
    attributes.iter().find_map(|a| {
        if a.name == "Signature" {
            if let AttributeData::Signature(signature) = &a.data {
                Some(signature)
            } else {
                panic!("Field signature is not `AttributeData::Signature`!");
            }
        } else {
            None
        }
    })
}

/// Returns `true` if the descriptor points to a function.
pub(crate) fn is_function(descriptor: &str) -> bool { descriptor.starts_with('(') }

/// Creates a [`MemberRef`] from a [`MethodHandle`].
pub(crate) fn handle_to_ref<'a>(handle: &MethodHandle<'a>) -> MemberRef<'a> {
    MemberRef {
        class_name: handle.class_name.clone(),
        name_and_type: NameAndType {
            name: handle.member_ref.name.clone(),
            descriptor: handle.member_ref.descriptor.clone(),
        },
    }
}
pub(crate) fn info_to_ref<'a>(class_name: &Cow<'a, str>, info: &MethodInfo<'a>) -> MemberRef<'a> {
    MemberRef {
        class_name: class_name.clone(),
        name_and_type: NameAndType {
            name: info.name.clone(),
            descriptor: info.descriptor.to_string().into(),
        },
    }
}
