use std::{borrow::Cow, path::Path};

use cafebabe::{
    attributes::AttributeData,
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef, NameAndType},
    descriptor::{FieldType, Ty},
    ClassFile,
};
use froglight_data::Version;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Extract;
use crate::classmap::ClassMap;

/// A module that extracts protocol information.
///
/// This includes things like the possible states and packets.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ProtocolModule;

impl Extract for ProtocolModule {
    async fn extract(
        &self,
        _: &Version,
        classmap: &ClassMap,
        _: &Path,
        output: &mut Value,
    ) -> anyhow::Result<()> {
        let Some(class) = classmap.get("net/minecraft/network/NetworkState") else {
            anyhow::bail!("Could not find NetworkState");
        };
        let class = class.parse();

        // Get protocol information
        let states = get_states(&class);
        let packets = get_packets(&class, &states)?;

        // Add protocol information to the output
        output["protocol"]["states"] = serde_json::to_value(&packets)?;

        Ok(())
    }
}

/// Get all of the possible protocol states.
fn get_states<'a>(class: &'a ClassFile) -> Vec<Cow<'a, str>> {
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
struct ProtocolState<'a> {
    clientbound: Vec<Cow<'a, str>>,
    serverbound: Vec<Cow<'a, str>>,
}

/// Get all of the packets for each protocol state.
fn get_packets<'a>(
    class: &'a ClassFile,
    states: &[Cow<'_, str>],
) -> anyhow::Result<HashMap<Cow<'a, str>, ProtocolState<'a>>> {
    let Some(clinit) = class.methods.iter().find(|m| m.name == "<clinit>") else {
        anyhow::bail!("Could not find <clinit>");
    };
    let Some(code) = clinit.attributes.iter().find(|a| a.name == "Code") else {
        anyhow::bail!("Could not find Code attribute");
    };
    let AttributeData::Code(code) = &code.data else {
        unreachable!("Code attribute is not a Code attribute")
    };
    let Some(code) = code.bytecode.as_ref() else {
        anyhow::bail!("Code attribute has no bytecode");
    };

    let mut hashmap = HashMap::new();

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

                    hashmap.insert(state_name, std::mem::take(&mut state_data));
                }
            }
            _ => {}
        }
    }

    Ok(hashmap)
}
