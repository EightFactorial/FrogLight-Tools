use froglight_parse::file::protocol::{ProtocolPackets, ProtocolStatePackets};

use crate::{cli::CliArgs, datamap::DataMap};

mod gen;
use gen::{File, Result, State};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    /// Generate packets from the given [`DataMap`].
    pub async fn generate(datamap: &DataMap, _args: &CliArgs) -> anyhow::Result<()> {
        let dataset = datamap.version_data.iter().next().unwrap().1;
        Self::generate_packets(&dataset.proto.packets).await
    }

    async fn generate_packets(packets: &ProtocolPackets) -> anyhow::Result<()> {
        for (state_name, state_packets) in packets.iter() {
            Self::generate_state_packets(state_name, "toClient", &state_packets.clientbound)
                .await?;
            Self::generate_state_packets(state_name, "toServer", &state_packets.serverbound)
                .await?;
        }
        Ok(())
    }

    async fn generate_state_packets(
        proto_state: &str,
        proto_direction: &str,
        packets: &ProtocolStatePackets,
    ) -> anyhow::Result<()> {
        // Create a new file
        let mut file = File::new();
        let state = State::new().with_item("_");

        // Sort the packets by name
        let mut collection: Vec<_> = packets.iter().collect();
        collection.sort_by_key(|(key, _)| key.as_str());

        // Generate types for each packet
        for (packet_name, packet_type) in
            collection.into_iter().filter(|(name, _)| name.starts_with("packet_"))
        {
            let packet_state = state.with_target(packet_name);
            if let Result::Err(err) = Self::generate_type(&packet_state, packet_type, &mut file) {
                tracing::error!("Error generating type for \"{packet_name}\": {err}");
            }
        }

        // If the file is empty, return early
        let file = file.into_inner();
        if file.items.is_empty() {
            return Ok(());
        }

        // Write the file to disk
        let path = std::path::PathBuf::from(file!());
        let path =
            path.parent().unwrap().join(format!("packets.{proto_state}.{proto_direction}.rs"));

        let unparsed = prettyplease::unparse(&file);
        tokio::fs::write(path, unparsed).await?;
        Ok(())
    }
}

#[allow(dead_code)]
#[allow(clippy::unnecessary_wraps)]
impl PacketGenerator {
    /// Format a field name to prevent conflicts with Rust keywords.
    #[must_use]
    fn format_field_name(field_name: &str) -> &str {
        match field_name {
            "abstract" => "abstract_",
            "as" => "as_",
            "async" => "async_",
            "await" => "await_",
            "become" => "become_",
            "box" => "box_",
            "break" => "break_",
            "const" => "const_",
            "continue" => "continue_",
            "crate" => "crate_",
            "do" => "do_",
            "dyn" => "dyn_",
            "else" => "else_",
            "enum" => "enum_",
            "extern" => "extern_",
            "false" => "false_",
            "final" => "final_",
            "fn" => "fn_",
            "for" => "for_",
            "if" => "if_",
            "impl" => "impl_",
            "in" => "in_",
            "let" => "let_",
            "loop" => "loop_",
            "macro" => "macro_",
            "match" => "match_",
            "mod" => "mod_",
            "move" => "move_",
            "mut" => "mut_",
            "override" => "override_",
            "priv" => "priv_",
            "pub" => "pub_",
            "ref" => "ref_",
            "return" => "return_",
            "self" => "self_",
            "Self" => "Self_",
            "static" => "static_",
            "struct" => "struct_",
            "super" => "super_",
            "trait" => "trait_",
            "true" => "true_",
            "try" => "try_",
            "type" => "type_",
            "typeof" => "typeof_",
            "unsafe" => "unsafe_",
            "unsized" => "unsized_",
            "use" => "use_",
            "virtual" => "virtual_",
            "where" => "where_",
            "while" => "while_",
            "yield" => "yield_",
            other => other,
        }
    }

    /// Format a variant name to allow creating valid Rust [`Idents`](Ident).
    #[must_use]
    fn format_variant_name(variant_name: &str) -> String {
        variant_name.split_once(':').map_or(variant_name, |(_, split)| split).replace(['.'], "_")
    }
}
