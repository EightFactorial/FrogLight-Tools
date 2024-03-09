use std::borrow::Cow;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef, NameAndType},
    descriptor::{FieldType, Ty},
    ClassFile,
};
use serde::Serialize;

use crate::modules::code_or_bail;

/// Get all of the possible protocol states.
pub(super) fn get_states<'a>(class: &'a ClassFile) -> Vec<Cow<'a, str>> {
    let mut states = Vec::new();

    for field in &class.fields {
        if let FieldType::Ty(Ty::Object(obj)) = &field.descriptor {
            if *obj == class.this_class {
                states.push(field.name.clone());
            }
        }
    }

    states
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub(super) struct ProtocolState<'a> {
    pub(super) clientbound: Vec<Cow<'a, str>>,
    pub(super) serverbound: Vec<Cow<'a, str>>,
}

/// Get all of the packets for each protocol state.
pub(super) fn get_packets<'a>(
    class: &'a ClassFile,
    states: &[Cow<'_, str>],
) -> anyhow::Result<Vec<(Cow<'a, str>, ProtocolState<'a>)>> {
    let code = code_or_bail(class, "<clinit>")?;

    let mut vec = Vec::new();

    let mut state_name = None;
    let mut state_data = ProtocolState::default();
    let mut is_clientbound = false;

    for (_, op) in &code.opcodes {
        match &op {
            Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(constant)))
            | Opcode::LdcW(Loadable::LiteralConstant(LiteralConstant::String(constant))) => {
                if states.contains(constant) {
                    state_name = Some(constant.clone());
                }
            }
            Opcode::Ldc(Loadable::ClassInfo(class_name))
            | Opcode::LdcW(Loadable::ClassInfo(class_name)) => {
                if is_clientbound {
                    state_data.clientbound.push(class_name.clone());
                } else {
                    state_data.serverbound.push(class_name.clone());
                }
            }
            Opcode::Getstatic(MemberRef {
                class_name,
                name_and_type: NameAndType { name, .. },
                ..
            }) => {
                if class_name == "net/minecraft/network/NetworkSide" {
                    is_clientbound = name == "CLIENTBOUND";
                }
            }
            Opcode::Invokespecial(MemberRef {
                class_name,
                name_and_type: NameAndType { name, .. },
                ..
            }) => {
                if class_name == "net/minecraft/network/NetworkState" && name == "<init>" {
                    let Some(state_name) = state_name.take() else {
                        anyhow::bail!("Found <init> without state name");
                    };

                    vec.push((state_name, std::mem::take(&mut state_data)));
                }
            }
            _ => {}
        }
    }

    Ok(vec)
}
