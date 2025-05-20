use std::borrow::Cow;

use cafebabe::{
    bytecode::Opcode,
    constant_pool::{LiteralConstant, Loadable, MemberRef},
};
use froglight_dependency::{
    container::DependencyContainer, dependency::minecraft::MinecraftCode, version::Version,
};
use tracing::warn;

use super::Packets;
use crate::class_helper::ClassHelper;

impl Packets {
    pub(super) async fn extract_network_states(
        version: &Version,
        deps: &mut DependencyContainer,
    ) -> anyhow::Result<Vec<String>> {
        const NETWORK_PHASE: &str = "net/minecraft/network/NetworkPhase";

        let mut states = Vec::with_capacity(5);

        deps.get_or_retrieve::<MinecraftCode>().await?;
        deps.scoped_fut::<MinecraftCode, anyhow::Result<_>>(async |jars, deps| {
            let jar = jars.get_version(version, deps).await?;
            let class = jar.get(NETWORK_PHASE).ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{NETWORK_PHASE}\" class!")
            })?;

            let code = class.class_code().bytecode.as_ref().ok_or_else(|| {
                anyhow::anyhow!("Packets: Could not find \"{NETWORK_PHASE}\" initialization!")
            })?;

            let mut building = false;
            let (mut constant, mut state) = (None, None);

            for (_, opcode) in &code.opcodes {
                match opcode {
                    Opcode::New(Cow::Borrowed(NETWORK_PHASE)) => building = true,
                    Opcode::Ldc(Loadable::LiteralConstant(LiteralConstant::String(string))) => {
                        match (constant, state) {
                            (None, None) => constant = Some(string),
                            (Some(..), None) => state = Some(string),
                            _ if building => warn!("Extra strings while building NetworkPhase?"),
                            _ => {}
                        }
                    }
                    Opcode::Putstatic(MemberRef { class_name, name_and_type }) => {
                        if class_name == NETWORK_PHASE && Some(&name_and_type.name) == constant {
                            let state = state.take().expect("Building NetworkPhase with no name?");
                            states.push(state.to_string());

                            constant = None;
                            building = false;
                        }
                    }
                    _ => {}
                }
            }

            Ok(())
        })
        .await?;

        Ok(states)
    }
}
