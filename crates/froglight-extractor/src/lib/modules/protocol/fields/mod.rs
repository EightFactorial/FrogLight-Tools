//! Ok, this gets a *bit* complicated, but I'll try to explain it as best as I
//! can here.
//!
//! Here, we have our list of states and packets. We want to get the fields for
//! each packet. That's not too hard, right? We just need to find the class for
//! the packet and then get the fields from that class.
//!
//! But here's the thing: the field of the packet are not one-to-one with the
//! data sent over the network. For example, a packet might have multiple ints
//! that are used as masks, but the actual data sent over the network is a
//! single byte.
//!
//! The solution used here is to get the constructor of the packet class that
//! takes a `PacketByteBuf`. From there, we can read every function call that
//! the packet makes to the `PacketByteBuf` and use that to determine the fields
//! of the packet.

use std::borrow::Cow;

use tracing::error;

use self::output::{Output, OutputType};
use super::packets::ProtocolState;
use crate::classmap::ClassMap;

mod find;

mod resolve;

mod output;

pub(super) fn get_fields<'a>(
    states: &'a [(Cow<'_, str>, ProtocolState<'_>)],
    classmap: &'a ClassMap,
) -> serde_json::Value {
    let mut value = serde_json::Value::default();

    for (_name, packets) in states {
        for packet in packets.clientbound.iter().chain(packets.serverbound.iter()) {
            // Skip the `BundleS2CPacket` as it's a special case
            if packet == "net/minecraft/network/packet/s2c/play/BundleS2CPacket" {
                continue;
            }

            // Find the class for the packet
            let Some(class) = classmap.get(packet) else {
                unreachable!("Could not find class for packet {packet}");
            };

            match (find::bytebuf_reader(&class), find::bytebuf_writer(&class)) {
                // If we have both a reader and a writer, we can compare them
                (Some(reader), Some(writer)) => {
                    let result = resolve::compare_methods(&class, reader, writer, classmap);

                    match result {
                        Output::Unnamed(fields) => {
                            add_unnamed(packet, &fields, &mut value);
                        }
                        Output::Named(fields) => {
                            add_named(packet, &fields, &mut value);
                        }
                    }
                }
                // If we only have a reader, resolve it
                (Some(reader), None) => {
                    let result = resolve::resolve_reader(&class, reader);
                    add_unnamed(
                        packet,
                        &result.into_iter().map(|(_, ty)| ty).collect::<Vec<_>>(),
                        &mut value,
                    );
                }
                // If we only have a writer, resolve it
                (None, Some(writer)) => {
                    let result = resolve::resolve_writer(&class, writer);
                    add_unnamed(
                        packet,
                        &result.into_iter().map(|(_, ty)| ty).collect::<Vec<_>>(),
                        &mut value,
                    );
                }
                // If we have neither, log an error
                (None, None) => {
                    error!("Could not find reader or writer for packet {packet}");
                }
            }
        }
    }

    value
}

fn add_unnamed(name: &str, fields: &[OutputType], value: &mut serde_json::Value) {
    value[name.to_string()] = serde_json::to_value(fields).unwrap();
}

fn add_named<'a>(
    name: &'a str,
    fields: &[(Cow<'a, str>, OutputType)],
    value: &mut serde_json::Value,
) {
    for (field_name, field) in fields {
        value[name.to_string()][field_name.to_string()] = serde_json::to_value(field).unwrap();
    }
}
