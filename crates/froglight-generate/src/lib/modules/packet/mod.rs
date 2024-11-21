use std::borrow::Cow;

use convert_case::{Case, Casing};
use froglight_parse::file::protocol::{ProtocolStatePackets, ProtocolTypeMap};

mod gen;
pub use gen::{File, Result, State};

/// A packet generator.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PacketGenerator;

impl PacketGenerator {
    /// Generate packets from the given [`ProtocolStatePackets`].
    #[must_use]
    pub fn generate_packets(packets: &ProtocolStatePackets) -> (syn::File, bool) {
        let mut error = false;

        // Create a new file and state
        let mut file = File::new();
        let state = State::new().with_item("_");

        // Sort the packets by name
        let mut collection: Vec<_> = packets.iter().collect();
        collection.sort_by_key(|(key, _)| key.as_str());

        // Iterate over the packets, generating the types
        for (packet_name, packet_type) in
            collection.into_iter().filter(|(name, _)| name.starts_with("packet_"))
        {
            let packet_state = state.with_target(packet_name);
            if let Result::Err(err) = Self::generate_type(&packet_state, packet_type, &mut file) {
                tracing::error!("Error generating packet \"{packet_name}\": {err}");
                error = true;
            }
        }

        // Return the generated file, and if there was an error
        (file.into_inner(), error)
    }

    /// Generate types from the given [`ProtocolTypeMap`].
    #[must_use]
    pub fn generate_types(types: &ProtocolTypeMap) -> (syn::File, bool) {
        let mut error = false;

        // Create a new file and state
        let mut file = File::new();
        let state = State::new().with_item("_");

        // Sort the types by name
        let mut collection: Vec<_> = types.iter().collect();
        collection.sort_by_key(|(key, _)| key.as_str());

        // Iterate over the types, generating structs and enums
        for (type_name, protocol_type) in collection {
            let packet_state = state.with_target(type_name);
            if let Result::Err(err) = Self::generate_type(&packet_state, protocol_type, &mut file) {
                tracing::error!("Error generating type \"{type_name}\": {err}");
                error = true;
            }
        }

        // Return the generated file, and if there was an error
        (file.into_inner(), error)
    }
}

#[allow(dead_code)]
#[allow(clippy::unnecessary_wraps)]
impl PacketGenerator {
    /// Format a field name to prevent conflicts with Rust keywords.
    #[must_use]
    pub fn format_field_name(field_name: &str) -> Cow<str> {
        match field_name {
            "abstract" => "abstract_".into(),
            "as" => "as_".into(),
            "async" => "async_".into(),
            "await" => "await_".into(),
            "become" => "become_".into(),
            "box" => "box_".into(),
            "break" => "break_".into(),
            "const" => "const_".into(),
            "continue" => "continue_".into(),
            "crate" => "crate_".into(),
            "do" => "do_".into(),
            "dyn" => "dyn_".into(),
            "else" => "else_".into(),
            "enum" => "enum_".into(),
            "extern" => "extern_".into(),
            "false" => "false_".into(),
            "final" => "final_".into(),
            "fn" => "fn_".into(),
            "for" => "for_".into(),
            "if" => "if_".into(),
            "impl" => "impl_".into(),
            "in" => "in_".into(),
            "let" => "let_".into(),
            "loop" => "loop_".into(),
            "macro" => "macro_".into(),
            "match" => "match_".into(),
            "mod" => "mod_".into(),
            "move" => "move_".into(),
            "mut" => "mut_".into(),
            "override" => "override_".into(),
            "priv" => "priv_".into(),
            "pub" => "pub_".into(),
            "ref" => "ref_".into(),
            "return" => "return_".into(),
            "self" => "self_".into(),
            "Self" => "Self_".into(),
            "static" => "static_".into(),
            "struct" => "struct_".into(),
            "super" => "super_".into(),
            "trait" => "trait_".into(),
            "true" => "true_".into(),
            "try" => "try_".into(),
            "type" => "type_".into(),
            "typeof" => "typeof_".into(),
            "unsafe" => "unsafe_".into(),
            "unsized" => "unsized_".into(),
            "use" => "use_".into(),
            "virtual" => "virtual_".into(),
            "where" => "where_".into(),
            "while" => "while_".into(),
            "yield" => "yield_".into(),
            other => other.to_case(Case::Snake).into(),
        }
    }

    /// Format a variant name to allow creating valid Rust [`Idents`](Ident).
    #[must_use]
    pub fn format_variant_name(variant_name: &str) -> String {
        variant_name.split_once(':').map_or(variant_name, |(_, split)| split).replace(['.'], "_")
    }
}
