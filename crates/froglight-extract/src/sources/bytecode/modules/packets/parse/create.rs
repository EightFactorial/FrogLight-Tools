use anyhow::bail;
use cafebabe::{
    bytecode::{ByteCode, Opcode},
    constant_pool::{BootstrapArgument, InvokeDynamic, MethodHandle},
    ClassFile,
};
use tracing::trace;

use super::CodecType;
use crate::{bundle::ExtractBundle, sources::helpers::get_bootstrap};

pub(super) fn create_create<'a>(
    classfile: &'a ClassFile<'_>,
    code: &ByteCode<'_>,
    index: usize,
) -> anyhow::Result<CodecType<'a>> {
    let decode = get_method_handle(classfile, code, index - 1)?;
    let encode = get_method_handle(classfile, code, index - 2)?;
    Ok(CodecType::Create { encode, decode })
}

pub(super) fn get_method_handle<'a>(
    classfile: &'a ClassFile<'_>,
    code: &ByteCode<'_>,
    opcode_index: usize,
) -> anyhow::Result<&'a MethodHandle<'a>> {
    let bootstrap_methods = get_bootstrap(&classfile.attributes)?;
    if let Opcode::Invokedynamic(InvokeDynamic { attr_index, .. }) = code.opcodes[opcode_index].1 {
        let Some(method) = bootstrap_methods.get(attr_index as usize) else {
            bail!(
                "Failed to find bootstrap method for `InvokeDynamic` in \"{}\"",
                classfile.this_class
            );
        };

        if let Some(handle) =
            method.arguments.iter().find(|a| matches!(a, BootstrapArgument::MethodHandle(_)))
        {
            let BootstrapArgument::MethodHandle(handle) = handle else {
                unreachable!("Already checked Argument type");
            };
            Ok(handle)
        } else {
            bail!("Failed to find `MethodHandle` in bootstrap method");
        }
    } else {
        trace!("Opcode: {:?}", code.opcodes[opcode_index]);
        bail!("Expected `Create` function to be an `InvokeDynamic`");
    }
}

/// Parse a [`CodecType::Create`] codec
pub(super) fn parse_create(
    _classfile: &ClassFile<'_>,
    decode: &MethodHandle<'_>,
    data: &ExtractBundle<'_>,
) -> anyhow::Result<Vec<String>> {
    super::method::parse_method_handle(decode, data)
}
